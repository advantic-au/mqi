use core::str;
use std::{borrow::Cow, cmp, mem::transmute, num::NonZero, str::Utf8Error};

use crate::{
    common::ResultCompErrExt as _,
    core::values,
    headers::{fmt, ChainedHeader, EncodedHeader, Header, HeaderError, TextEnc},
    macros::all_multi_tuples,
    sys,
    types::{self, Fmt, MessageFormat, MessageId},
    Buffer, Completion, Conn, ConsumeValue2, Error, ExtractValue2, MqMask, MqStruct, MqValue, MqiOption,
    ResultComp, ResultCompErr, StrCcsidCow,
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

pub trait GetExtract<B>: ExtractValue2<GetParam, GetState<B>> {}
impl<B, T: ExtractValue2<GetParam, GetState<B>>> GetExtract<B> for T {}

pub trait GetConsume<B>: ConsumeValue2<GetParam, GetState<B>> {
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

impl<'a, P, B: Buffer<'a>> ConsumeValue2<P, GetState<B>> for StrCcsidCow<'a> {
    type Error = GetStringCcsidError;

    fn consume<F>(param: &mut P, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut P) -> ResultComp<GetState<B>>,
    {
        let state = get(param)?;
        if state.format.fmt != TextEnc::Ascii(fmt::MQFMT_STRING) {
            return Err(GetStringCcsidError::UnexpectedFormat(state.format.fmt, state.warning()));
        }

        Ok(state.map(|state| Self {
            ccsid: NonZero::new(state.format.ccsid),
            data: state.buffer.truncate(state.data_length).into_cow(),
            le: (state.format.encoding & sys::MQENC_INTEGER_REVERSED) != 0,
        }))
    }
}

impl<'buffer, P, B> ConsumeValue2<P, GetState<B>> for Cow<'buffer, str>
where
    B: Buffer<'buffer>,
{
    type Error = GetStringError;

    fn consume<F: FnOnce(&mut P) -> ResultComp<GetState<B>>>(param: &mut P, get: F) -> ResultCompErr<Self, Self::Error> {
        // TODO: set 1208 in MQMD?
        let get_result = get(param)?;

        if get_result.format.fmt != TextEnc::Ascii(fmt::MQFMT_STRING) || get_result.format.ccsid != 1208 {
            return Err(GetStringError::UnexpectedFormat(
                get_result.format.fmt,
                get_result.format.ccsid,
                get_result.warning(),
            ));
        }

        match get_result.map(|state| state.buffer.truncate(state.data_length).into_cow()) {
            Completion(_, Some((rc, verb))) if rc == sys::MQRC_NOT_CONVERTED => {
                Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc).into())
            }
            Completion(Cow::Borrowed(bytes), warning) => Ok(Completion(
                Cow::Borrowed(std::str::from_utf8(bytes).map_err(|e| GetStringError::Utf8Parse(e, warning))?),
                warning,
            )),
            Completion(Cow::Owned(bytes), warning) => Ok(Completion(
                Cow::Owned(String::from_utf8(bytes).map_err(|e| GetStringError::Utf8Parse(e.utf8_error(), warning))?),
                warning,
            )),
        }
    }
}

impl<'a, P, B: Buffer<'a>> ConsumeValue2<P, GetState<B>> for Cow<'a, [u8]> {
    type Error = Error;

    fn consume<F>(param: &mut P, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut P) -> ResultComp<GetState<B>>,
    {
        get(param).map_completion(|state| state.buffer.truncate(state.data_length).into_cow())
    }
}

impl<'a, P, B: Buffer<'a>> ExtractValue2<P, GetState<B>> for Headers<'a> {
    fn extract<F>(param: &mut P, get: F) -> ResultComp<(Self, GetState<B>)>
    where
        F: FnOnce(&mut P) -> ResultComp<GetState<B>>,
    {
        let state = get(param)?;

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

        Ok(state.map(|state| {
            let (headers, tail) = state.buffer.split_at(header_length);
            (
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
            )
        }))
    }
}

impl<P, B> ExtractValue2<P, GetState<B>> for MessageFormat {
    fn extract<F: FnOnce(&mut P) -> ResultComp<GetState<B>>>(param: &mut P, get: F) -> ResultComp<(Self, GetState<B>)> {
        get(param).map_completion(|state| (state.format, state))
    }
}

impl<S> ExtractValue2<GetParam, S> for MqStruct<'static, sys::MQMD2> {
    fn extract<F: FnOnce(&mut GetParam) -> ResultComp<S>>(param: &mut GetParam, get: F) -> ResultComp<(Self, S)> {
        let state = get(param)?;
        let (md, ..) = param;
        Ok(state.map(|state| (md.clone(), state)))
    }
}

impl<S> ExtractValue2<GetParam, S> for MessageId {
    fn extract<F>(param: &mut GetParam, get: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<S>,
    {
        get(param).map_completion(|state| (Self(param.0.MsgId), state))
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
        let mut param = (
            MqStruct::default(),
            MqStruct::new(sys::MQGMO {
                Version: sys::MQGMO_VERSION_4,
                ..sys::MQGMO::default()
            }),
        );
        let mut no_msg_available = false;

        options.apply_param(&mut param);

        let result = R::consume(&mut param, |(md, gmo)| {
            let mut buffer = buffer;
            let write_area = match R::max_data_size() {
                Some(max_len) => &mut buffer.as_mut()[..max_len.into()],
                None => buffer.as_mut(),
            };

            let mqi_get = self
                .connection()
                .mq()
                .mqget(self.connection().handle(), self.handle(), Some(&mut **md), gmo, write_area)
                .map_completion(|length| {
                    (
                        length,
                        match gmo.ReturnedLength {
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
                })
                .map_completion(|(message_length, data_length)| GetState {
                    buffer,
                    data_length: data_length.try_into().expect("length within positive usize range"),
                    message_length: message_length.try_into().expect("length within positive usize range"),
                    format: MessageFormat {
                        ccsid: md.CodedCharSetId,
                        encoding: MqMask::from(md.Encoding),
                        fmt: TextEnc::Ascii(unsafe { transmute::<[i8; 8], Fmt>(md.Format) }),
                    },
                });
            no_msg_available = mqi_get
                .as_ref()
                .is_err_and(|&Error(cc, _, rc)| cc == sys::MQCC_FAILED && rc == sys::MQRC_NO_MSG_AVAILABLE);

            mqi_get
        });

        if no_msg_available {
            Ok(Completion::new(None))
        } else {
            result.map_completion(Some)
        }
    }
}
