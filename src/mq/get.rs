use std::{borrow::Cow, cmp, num::NonZero, str::Utf8Error};

use libmqm_default as default;

use crate::{
    headers::{ChainedHeader, EncodedHeader, Header, HeaderError, TextEnc},
    prelude::*,
    sys,
    types,
    values, Buffer, Completion, Conn, Error, MqStruct, Object, ResultComp, ResultCompErr, StrCcsidCow,
};

#[derive(Clone, Debug, derive_more::Constructor)]
pub struct Headers<'a> {
    message_length: usize,
    init_format: types::MessageFormat,
    data: Cow<'a, [u8]>,
    error: Option<HeaderError>,
}

impl<'a> Headers<'a> {

    pub fn all_headers(&'a self) -> impl Iterator<Item = Header<'a>> {
        Header::iter(&self.data, self.init_format).filter_map(|result| match result {
            Ok((header, ..)) => Some(header),
            Err(_) => None,
        })
    }

    pub fn header<C: ChainedHeader + 'a>(&'a self) -> impl Iterator<Item = EncodedHeader<'a, C>> {
        self.all_headers().filter_map(C::from_header)
    }

    #[must_use]
    pub const fn error(&self) -> Option<&HeaderError> {
        self.error.as_ref()
    }

    #[must_use]
    pub const fn message_length(&self) -> usize {
        self.message_length
    }
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

pub const ANY_MESSAGE: MatchOptions = MatchOptions {
    msg_id: None,
    correl_id: None,
    group_id: None,
    seq_number: None,
    offset: None,
    token: None,
};

// TODO: add MQ warnings to error messages
#[derive(derive_more::Error, derive_more::From, derive_more::Display, Debug)]
pub enum GetStringError {
    #[display("Message parsing error: {_0}")]
    Utf8Parse(Utf8Error, Option<types::Warning>),
    #[display("Unexpected format or CCSID. Message format = '{_0}', CCSID = {_1}")]
    UnexpectedFormat(TextEnc<types::Fmt>, values::CCSID, Option<types::Warning>),
    #[from]
    MQ(Error),
}

#[derive(derive_more::Error, derive_more::Display, derive_more::From, Debug)]
pub enum GetStringCcsidError {
    #[display("Unexpected format. Message format = '{_0}'")]
    UnexpectedFormat(TextEnc<types::Fmt>, Option<types::Warning>),
    #[from]
    MQ(Error),
}

#[derive(Default)]
pub enum GetWait {
    #[default]
    NoWait,
    Wait(sys::MQLONG),
}

pub enum GetConvert {
    NoConvert,
    Convert,
    ConvertTo(values::CCSID, values::MQENC),
}

pub struct GetParam {
    pub md: MqStruct<'static, sys::MQMD2>,
    pub gmo: MqStruct<'static, sys::MQGMO>,
}

pub struct GetState<B> {
    /// The buffer holding the message data from the `MQGET` call.
    pub buffer: B,
    /// The length of the message data returned by the `MQGET` call, confined by buffer size
    pub data_length: usize,
    /// The full length of the message data unconfined by buffer size
    pub message_length: usize,
    /// The format of the returned message
    pub format: types::MessageFormat,
}

pub trait GetAttr<B> {
    fn extract<F>(param: &mut GetParam, mqi: F) -> ResultComp<(Self, GetState<B>)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        Self: Sized;
}

pub trait GetValue<B> {
    type Error: std::fmt::Debug;

    fn consume<F>(param: &mut GetParam, mqi: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        Self: std::marker::Sized;

    #[must_use]
    fn max_data_size() -> Option<NonZero<usize>> {
        None
    }
}

/// A trait that manipulates the parameters to the [`mqget`](`crate::core::MqFunctions::mqget`) function
#[diagnostic::on_unimplemented(message = "{Self} does not implement `GetOption` so it can't be used as an argument for MQI get")]
pub trait GetOption {
    fn apply_param(self, param: &mut GetParam);
}

impl<C: Conn> Object<C> {
    pub fn get_data<'b, B>(&self, options: impl GetOption, buffer: B) -> ResultComp<Option<Cow<'b, [u8]>>>
    where
        B: Buffer<'b, u8>,
    {
        self.get_as(options, buffer)
    }

    pub fn get_data_with<'b, A, B>(&self, options: impl GetOption, buffer: B) -> ResultComp<Option<(Cow<'b, [u8]>, A)>>
    where
        A: GetAttr<B>,
        B: Buffer<'b, u8>,
    {
        self.get_as(options, buffer)
    }

    pub fn get_string<'b, B>(
        &self,
        options: impl GetOption,
        buffer: B,
    ) -> ResultCompErr<Option<StrCcsidCow<'b>>, GetStringCcsidError>
    where
        B: Buffer<'b, u8>,
    {
        self.get_as(options, buffer)
    }

    pub fn get_string_with<'b, A, B>(
        &self,
        options: impl GetOption,
        buffer: B,
    ) -> ResultCompErr<Option<(StrCcsidCow<'b>, A)>, GetStringCcsidError>
    where
        A: GetAttr<B>,
        B: Buffer<'b, u8>,
    {
        self.get_as(options, buffer)
    }

    pub fn get_as<'b, R, B>(&self, options: impl GetOption, buffer: B) -> ResultCompErr<Option<R>, R::Error>
    where
        R: GetValue<B>,
        B: Buffer<'b, u8>,
    {
        let mut param = GetParam {
            md: MqStruct::new(default::MQMD2_DEFAULT),
            gmo: MqStruct::new(sys::MQGMO {
                Version: sys::MQGMO_VERSION_3, // Version 3 for ReturnedLength
                ..default::MQGMO_DEFAULT
            }),
        };
        let mut no_msg_available = false;

        options.apply_param(&mut param);

        let result = R::consume(&mut param, |param| {
            let mut buffer = buffer;
            let write_area = match R::max_data_size() {
                Some(max_len) => &mut buffer.as_mut()[..max_len.into()],
                None => buffer.as_mut(),
            };

            let mqi_get = self
                .connection()
                .mq()
                .mqget(
                    self.connection().handle(),
                    self.handle(),
                    Some(&mut *param.md),
                    &mut param.gmo,
                    write_area,
                )
                .map_completion(|length| {
                    (
                        length,
                        match param.gmo.ReturnedLength {
                            sys::MQRL_UNDEFINED => cmp::min(
                                write_area
                                    .len()
                                    .try_into()
                                    .expect("length of buffer should be within positive i32 range"),
                                length,
                            ),
                            returned_length => returned_length,
                        },
                    )
                })
                .map_completion(|(message_length, data_length)| GetState {
                    buffer,
                    data_length: data_length
                        .try_into()
                        .expect("data length should be within positive usize range"),
                    message_length: message_length
                        .try_into()
                        .expect("message length should be within positive usize range"),
                    format: types::MessageFormat {
                        ccsid: values::CCSID(param.md.CodedCharSetId),
                        encoding: values::MQENC(param.md.Encoding),
                        fmt: TextEnc::Ascii(param.md.Format),
                    },
                });
            no_msg_available = mqi_get.as_ref().is_err_and(|e| {
                matches!(
                    e,
                    &Error(values::MQCC(sys::MQCC_FAILED), _, values::MQRC(sys::MQRC_NO_MSG_AVAILABLE))
                )
            });

            mqi_get
        });

        if no_msg_available {
            Ok(Completion::new(None))
        } else {
            result.map_completion(Some)
        }
    }
}
