use core::str;
use std::{borrow::Cow, cmp, mem::transmute, num::NonZero, str::Utf8Error};

use crate::{
    common::ResultCompErrExt as _, core::values, headers::{fmt, ChainedHeader, EncodedHeader, Header, HeaderError, TextEnc}, macros::all_multi_tuples, sys, types::{self, Fmt, MessageFormat, MessageId}, Buffer, Completion, Conn, ConsumeValue, Error, ExtractValue, MqMask, MqStruct, MqValue, MqiOption, ResultCompErr, StrCcsidCow
};

use super::Object;

#[derive(Clone, Debug)]
pub struct Headers<'a> {
    message_length: usize,
    init_format: MessageFormat,
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
#[derive(thiserror::Error, Debug)]
pub enum GetStringError {
    #[error("Message parsing error: {}", .0)]
    Utf8Parse(Utf8Error, Option<types::Warning>),
    #[error("Unexpected format or CCSID. Message format = '{}', CCSID = {}", .0, .1)]
    UnexpectedFormat(TextEnc<Fmt>, sys::MQLONG, Option<types::Warning>),
    #[error(transparent)]
    MQ(#[from] Error),
}

#[derive(thiserror::Error, Debug)]
pub enum GetStringCcsidError {
    #[error("Unexpected format. Message format = '{}'", .0)]
    UnexpectedFormat(TextEnc<Fmt>, Option<types::Warning>),
    #[error(transparent)]
    MQ(#[from] Error),
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
    ConvertTo(sys::MQLONG, MqMask<values::MQENC>),
}

pub type GetParam = (MqStruct<'static, sys::MQMD2>, MqStruct<'static, sys::MQGMO>);

pub struct GetState<B> {
    pub buffer: B,
    pub data_length: usize,
    pub message_length: usize,
    pub format: MessageFormat,
}

pub trait GetExtract<B>: ExtractValue<GetParam, GetState<B>> {}
impl<B, T: ExtractValue<GetParam, GetState<B>>> GetExtract<B> for T {}

pub trait GetConsume<B>: ConsumeValue<GetParam, GetState<B>> {
    #[must_use]
    fn max_data_size() -> Option<NonZero<usize>> {
        None
    }
}

impl<B> GetConsume<B> for () {}
impl<'a, B: Buffer<'a>> GetConsume<B> for StrCcsidCow<'a> {}
impl<'a, B: Buffer<'a>> GetConsume<B> for Cow<'a, str> {}
impl<'a, B: Buffer<'a>> GetConsume<B> for Cow<'a, [u8]> {}

macro_rules! impl_getconsume {
    ($first:ident, [$($ty:ident),*]) => {
        impl<B, $first, $($ty),*> GetConsume<B> for ($first, $($ty),*)
        where
            $first: GetConsume<B>,
            $($ty: GetExtract<B>),*
        {
            fn max_data_size() -> Option<NonZero<usize>> {
                $first::max_data_size()
            }
        }
    };
}
all_multi_tuples!(impl_getconsume);

impl<'a, P, B: Buffer<'a>> ConsumeValue<P, GetState<B>> for StrCcsidCow<'a> {
    type Error = GetStringCcsidError;

    fn consume_from(state: GetState<B>, _param: &P, warning: Option<types::Warning>) -> Result<Self, Self::Error> {
        if state.format.fmt != TextEnc::Ascii(fmt::MQFMT_STRING) {
            return Err(GetStringCcsidError::UnexpectedFormat(state.format.fmt, warning));
        }

        Ok(Self {
            ccsid: NonZero::new(state.format.ccsid),
            data: state.buffer.truncate(state.data_length).into_cow(),
            le: (state.format.encoding & sys::MQENC_INTEGER_REVERSED) != 0,
        })
    }
}

impl<'a, P, B: Buffer<'a>> ConsumeValue<P, GetState<B>> for Cow<'a, str> {
    type Error = GetStringError;

    fn consume_from(state: GetState<B>, _param: &P, warning: Option<types::Warning>) -> Result<Self, Self::Error> {
        if state.format.fmt != TextEnc::Ascii(fmt::MQFMT_STRING) || state.format.ccsid != 1208 {
            return Err(GetStringError::UnexpectedFormat(
                state.format.fmt,
                state.format.ccsid,
                warning,
            ));
        }

        match (warning, state.buffer.truncate(state.data_length).into_cow()) {
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

impl<'a, P, B: Buffer<'a>> ConsumeValue<P, GetState<B>> for Cow<'a, [u8]> {
    type Error = Error;

    fn consume_from(state: GetState<B>, _param: &P, _warning: Option<types::Warning>) -> Result<Self, Self::Error> {
        Ok(state.buffer.truncate(state.data_length).into_cow())
    }
}

impl<'a, P, B: Buffer<'a>> ExtractValue<P, GetState<B>> for Headers<'a> {
    fn extract_from(state: GetState<B>, _param: &P, _warning: Option<types::Warning>) -> Result<(Self, GetState<B>), Error> {
        let data = &state.buffer.as_ref()[..state.data_length];
        let mut header_length = 0;
        let mut final_format = state.format;
        let mut error = None;
        for result in Header::iter(data, state.format) {
            match result {
                Ok((.., header_size, message_format)) => {
                    header_length += header_size;
                    final_format = message_format;
                }
                Err(e) => error = Some(e),
            }
        }

        let (headers, tail) = state.buffer.split_at(header_length);

        Ok((
            Self {
                init_format: state.format,
                data: headers.into_cow(),
                error,
                message_length: state.message_length,
            },
            GetState {
                buffer: tail,
                data_length: state.data_length - header_length,
                message_length: state.message_length - header_length,
                format: final_format,
            },
        ))
    }
}

impl<P, B> ExtractValue<P, GetState<B>> for MessageFormat {
    fn extract_from(state: GetState<B>, _param: &P, _warning: Option<types::Warning>) -> Result<(Self, GetState<B>), Error> {
        Ok((state.format, state))
    }
}

impl<S> ExtractValue<GetParam, S> for MqStruct<'static, sys::MQMD2> {
    fn extract_from(state: S, (md, ..): &GetParam, _warning: Option<types::Warning>) -> Result<(Self, S), Error> {
        Ok((md.clone(), state))
    }
}

impl<S> ExtractValue<GetParam, S> for MessageId {
    fn extract_from(state: S, (md, ..): &GetParam, _warning: Option<types::Warning>) -> Result<(Self, S), Error> {
        Ok((Self(md.MsgId), state))
    }
}

pub trait GetOption: MqiOption<GetParam> {}
impl<T: MqiOption<GetParam>> GetOption for T {}

impl<C: Conn> Object<C> {
    pub fn get_message<'b, R, B>(&self, options: impl GetOption, buffer: B) -> ResultCompErr<Option<R>, R::Error>
    where
        R: GetConsume<B>,
        B: Buffer<'b>,
    {
        let mut buffer = buffer;
        let write_area = match R::max_data_size() {
            Some(max_len) => &mut buffer.as_mut()[..max_len.into()],
            None => buffer.as_mut(),
        };

        let mut param = (
            MqStruct::default(),
            MqStruct::new(sys::MQGMO {
                Version: sys::MQGMO_VERSION_4,
                ..sys::MQGMO::default()
            }),
        );

        options.apply_param(&mut param);

        let get_result = match self
            .connection()
            .mq()
            .mqget(
                self.connection().handle(),
                self.handle(),
                Some(&mut *param.0),
                &mut param.1,
                write_area,
            )
            .map_completion(|length| {
                (
                    length,
                    match param.1.ReturnedLength {
                        sys::MQRL_UNDEFINED => cmp::min(
                            write_area
                                .len()
                                .try_into()
                                .expect("length of buffer must fit in positive i32"),
                            length,
                        ),
                        returned_length => returned_length,
                    },
                )
            }) {
            Err(Error(cc, _, rc)) if cc == sys::MQCC_FAILED && rc == sys::MQRC_NO_MSG_AVAILABLE => Ok(Completion::new(None)),
            other => other.map_completion(Some),
        }?;

        Ok(match get_result {
            Completion(Some((message_length, data_length)), warning) => Completion(
                Some(R::consume_from(
                    GetState {
                        buffer,
                        data_length: data_length.try_into().expect("length within positive usize range"),
                        message_length: message_length.try_into().expect("length within positive usize range"),
                        format: MessageFormat {
                            ccsid: param.0.CodedCharSetId,
                            encoding: MqMask::from(param.0.Encoding),
                            fmt: TextEnc::Ascii(unsafe { transmute::<[i8; 8], Fmt>(param.0.Format) }),
                        },
                    },
                    &param,
                    warning,
                )?),
                warning,
            ),
            comp => comp.map(|_| None),
        })
    }
}
