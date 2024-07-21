use std::{borrow::Cow, cmp, num::NonZero, ops::Deref, str::Utf8Error};

use crate::{
    common::ResultCompErrExt as _, core::values, mqstr, sys, types, Buffer, Completion, Conn, Error, Message, MqMask, MqStr, MqStruct, MqValue, ResultCompErr, StrCcsidCow, MQMD as _
};

use super::Object;

// TODO: Move this
#[derive(Clone, Debug)]
pub struct MessageFormat {
    pub ccsid: sys::MQLONG,
    pub encoding: MqMask<values::MQENC>,
    pub format: MqStr<8>,
}

#[derive(Clone, Debug)]
pub struct Mqmd<T> {
    pub mqmd: MqStruct<'static, sys::MQMD2>,
    pub body: T,
}

#[derive(Debug, Clone, Default)]
pub struct MatchOptions<'a> {
    pub msg_id: Option<&'a types::MessageId>,
    pub correl_id: Option<&'a types::CorrelationId>,
    pub group_id: Option<&'a types::GroupId>,
    pub seq_number: Option<sys::MQLONG>,
    pub offset: Option<sys::MQLONG>,
    pub token: Option<&'a types::MsgToken>,
}

pub const ANY_MESSAGE: &MatchOptions = &MatchOptions {
    msg_id: None,
    correl_id: None,
    group_id: None,
    seq_number: None,
    offset: None,
    token: None,
};

// TODO: add MQ warnings to error messages
#[derive(thiserror::Error, Debug)]
pub enum GetStringError {
    #[error("Message parsing error: {}", .0)]
    Utf8Parse(Utf8Error, Option<types::Warning>),
    #[error("Unexpected format or CCSID. Message format = '{}', CCSID = {}", .0, .1)]
    UnexpectedFormat(MqStr<8>, sys::MQLONG, Option<types::Warning>),
    #[error(transparent)]
    MQ(#[from] Error),
}

#[derive(thiserror::Error, Debug)]
pub enum GetStringCcsidError {
    #[error("Unexpected format. Message format = '{}'", .0)]
    UnexpectedFormat(MqStr<8>, Option<types::Warning>),
    #[error(transparent)]
    MQ(#[from] Error),
}

pub trait GetMessage<'a>: Sized {
    type Error: std::fmt::Debug + From<Error>;

    fn create_from<C: Conn>(
        object: &Object<C>,
        data: Cow<'a, [u8]>,
        md: &MqStruct<'static, sys::MQMD2>,
        gmo: &MqStruct<sys::MQGMO>,
        format: &MessageFormat,
        warning: Option<types::Warning>,
    ) -> Result<Self, Self::Error>;

    #[must_use]
    fn max_data_size() -> Option<NonZero<usize>> {
        None
    }

    #[allow(unused_variables)]
    fn apply_mqget(md: &mut MqStruct<sys::MQMD2>, gmo: &mut MqStruct<sys::MQGMO>) {}
}

const MQSTR: MqStr<8> = mqstr!("MQSTR");

impl<'a> GetMessage<'a> for StrCcsidCow<'a> {
    type Error = GetStringCcsidError;

    fn create_from<C: Conn>(
        _object: &Object<C>,
        data: Cow<'a, [u8]>,
        _md: &MqStruct<'static, sys::MQMD2>,
        _gmo: &MqStruct<sys::MQGMO>,
        format: &MessageFormat,
        warning: Option<types::Warning>,
    ) -> Result<Self, Self::Error> {
        if format.format != MQSTR {
            return Err(GetStringCcsidError::UnexpectedFormat(format.format, warning));
        }

        Ok(Self {
            ccsid: NonZero::new(format.ccsid),
            data
        })
    }
}

impl<'a> GetMessage<'a> for Cow<'a, str> {
    type Error = GetStringError;

    fn apply_mqget(md: &mut MqStruct<sys::MQMD2>, gmo: &mut MqStruct<sys::MQGMO>) {
        gmo.Options |= sys::MQGMO_CONVERT;
        md.CodedCharSetId = 1208;
    }

    fn create_from<C: Conn>(
        _object: &Object<C>,
        data: Cow<'a, [u8]>,
        _md: &MqStruct<'static, sys::MQMD2>,
        _gmo: &MqStruct<sys::MQGMO>,
        format: &MessageFormat,
        warning: Option<types::Warning>,
    ) -> Result<Self, Self::Error> {
        if format.format != MQSTR || format.ccsid != 1208 {
            return Err(GetStringError::UnexpectedFormat(format.format, format.ccsid, warning));
        }

        match (warning, data) {
            (Some((rc, verb)), _) if rc == sys::MQRC_NOT_CONVERTED => {
                Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc).into())
            }
            (warning, Cow::Borrowed(bytes)) => Ok(Cow::Borrowed(
                std::str::from_utf8(bytes).map_err(|e| GetStringError::Utf8Parse(e, warning))?,
            )),
            (warning, Cow::Owned(bytes)) => Ok(Cow::Owned(
                String::from_utf8(bytes).map_err(|e| GetStringError::Utf8Parse(e.utf8_error(), warning))?,
            )),
        }
    }
}

impl<'a> GetMessage<'a> for Cow<'a, [u8]> {
    type Error = Error;

    fn create_from<C: Conn>(
        _object: &Object<C>,
        data: Self,
        _md: &MqStruct<'static, sys::MQMD2>,
        _gmo: &MqStruct<sys::MQGMO>,
        _format: &MessageFormat,
        _warning: Option<types::Warning>,
    ) -> Result<Self, Self::Error> {
        Ok(data)
    }
}

// impl GetMessage<'_> for Vec<u8> {
//     type Error = Error;

//     fn create_from<C: Conn>(
//         _object: &Object<C>,
//         data: Cow<[u8]>,
//         _md: &MqStruct<sys::MQMD2>,
//         _gmo: &MqStruct<sys::MQGMO>,
//         _format: &MessageFormat,
//         _warning: Option<types::Warning>,
//     ) -> Result<Self, Self::Error> {
//         Ok(data.into_owned())
//     }
// }

impl<'a, T: GetMessage<'a>> GetMessage<'a> for Mqmd<T> {
    type Error = T::Error;

    fn create_from<C: Conn>(
        object: &Object<C>,
        data: Cow<'a, [u8]>,
        md: &MqStruct<'static, sys::MQMD2>,
        gmo: &MqStruct<sys::MQGMO>,
        format: &MessageFormat,
        warning: Option<types::Warning>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            mqmd: md.clone(),
            body: T::create_from(object, data, md, gmo, format, warning)?,
        })
    }

    fn max_data_size() -> Option<NonZero<usize>> {
        T::max_data_size()
    }

    fn apply_mqget(md: &mut MqStruct<sys::MQMD2>, gmo: &mut MqStruct<sys::MQGMO>) {
        T::apply_mqget(md, gmo);
    }
}

impl<T> Deref for Mqmd<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl<C: Conn> Object<C> {
    pub fn get_message<'a, T: GetMessage<'a>>(
        &self,
        gmo: MqMask<values::MQGMO>,
        mo: &MatchOptions,
        wait: Option<sys::MQLONG>,
        properties: Option<&mut Message<C>>,
        buffer: impl Buffer<'a>,
    ) -> ResultCompErr<Option<T>, T::Error> {
        let mut buffer = match T::max_data_size() {
            Some(max_len) => buffer.truncate(max_len.into()),
            None => buffer,
        };

        let mut md = MqStruct::new(sys::MQMD2::match_by(mo));
        let mut mqgmo = MqStruct::new(sys::MQGMO {
            Version: sys::MQGMO_VERSION_4,
            Options: gmo.value(),
            ..sys::MQGMO::default()
        });
        if let Some(token) = mo.token {
            mqgmo.MsgToken = *token;
        }
        mqgmo.MatchOptions = mo.correl_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_CORREL_ID)
            | mo.msg_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_ID)
            | mo.group_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_GROUP_ID)
            | mo.seq_number.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_SEQ_NUMBER)
            | mo.offset.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_OFFSET)
            | mo.token.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_TOKEN);

        // Set up the wait
        if let Some(interval) = wait {
            mqgmo.Options |= sys::MQGMO_WAIT;
            mqgmo.WaitInterval = interval;
        }

        if let Some(props) = properties {
            mqgmo.Options |= sys::MQGMO_PROPERTIES_IN_HANDLE;
            mqgmo.MsgHandle = unsafe { props.handle().raw_handle() }
        }

        T::apply_mqget(&mut md, &mut mqgmo);

        let get_result = match self
            .connection()
            .mq()
            .mqget(
                self.connection().handle(),
                self.handle(),
                Some(&mut *md),
                &mut mqgmo,
                buffer.as_mut(),
            )
            .map_completion(|length| match mqgmo.ReturnedLength {
                sys::MQRL_UNDEFINED => cmp::min(
                    buffer.len().try_into().expect("length of buffer must fit in positive i32"),
                    length,
                ),
                returned_length => returned_length,
            }) {
            Err(Error(cc, _, rc)) if cc == sys::MQCC_FAILED && rc == sys::MQRC_NO_MSG_AVAILABLE => Ok(Completion::new(None)),
            other => other.map_completion(Some),
        }?;

        Ok(match get_result {
            Completion(Some(length), warning) => {
                buffer = buffer.truncate(length.try_into().expect("length within positive usize range"));
                Completion(
                    Some(T::create_from(
                        self,
                        buffer.into_cow(),
                        &md,
                        &mqgmo,
                        &MessageFormat {
                            ccsid: md.CodedCharSetId,
                            encoding: MqMask::from(md.Encoding),
                            format: md.Format.into(),
                        },
                        warning,
                    )?),
                    warning,
                )
            }
            comp => comp.map(|_| None),
        })
    }
}
