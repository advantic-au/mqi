use core::str;
use std::{borrow::Cow, cmp, mem::transmute, num::NonZero, str::Utf8Error};

use crate::{
    headers::{fmt, ChainedHeader, EncodedHeader, Header, HeaderError, TextEnc},
    macros::all_multi_tuples,
    prelude::*,
    sys,
    types::{self, Fmt, MessageFormat, MessageId},
    values::{self, CCSID},
    Buffer, Completion, Conn, Error, MqStruct, MqiAttr, MqiValue, Object, ResultComp, ResultCompErr, StrCcsidCow,
};

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
#[derive(derive_more::Error, derive_more::From, derive_more::Display, Debug)]
pub enum GetStringError {
    #[display("Message parsing error: {_0}")]
    Utf8Parse(Utf8Error, Option<types::Warning>),
    #[display("Unexpected format or CCSID. Message format = '{_0}', CCSID = {_1}")]
    UnexpectedFormat(TextEnc<Fmt>, CCSID, Option<types::Warning>),
    #[from]
    MQ(Error),
}

#[derive(derive_more::Error, derive_more::Display, derive_more::From, Debug)]
pub enum GetStringCcsidError {
    #[display("Unexpected format. Message format = '{_0}'")]
    UnexpectedFormat(TextEnc<Fmt>, Option<types::Warning>),
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
    pub buffer: B,
    pub data_length: usize,
    pub message_length: usize,
    pub format: MessageFormat,
}

pub trait GetAttr<B>: MqiAttr<GetParam, GetState<B>> {}
impl<B, T: MqiAttr<GetParam, GetState<B>>> GetAttr<B> for T {}

pub trait GetValue<B>: MqiValue<GetParam, GetState<B>> {
    #[must_use]
    fn max_data_size() -> Option<NonZero<usize>> {
        None
    }
}

impl<B> GetValue<B> for () {}
impl<'a, B: Buffer<'a>> GetValue<B> for StrCcsidCow<'a> {}
impl<'a, B: Buffer<'a>> GetValue<B> for Cow<'a, str> {}
impl<'a, B: Buffer<'a>> GetValue<B> for Cow<'a, [u8]> {}
impl<'a, B: Buffer<'a>> GetValue<B> for Vec<u8> {}

macro_rules! impl_getvalue {
    ([$first:ident, $($ty:ident),*]) => {
        impl<B, $first, $($ty),*> GetValue<B> for ($first, $($ty),*)
        where
            $first: GetValue<B>,
            ($first, $($ty),*): MqiValue<GetParam, GetState<B>>
        {
            fn max_data_size() -> Option<NonZero<usize>> {
                $first::max_data_size()
            }
        }
    };
}

all_multi_tuples!(impl_getvalue);

impl<'a, P, B: Buffer<'a>> MqiValue<P, GetState<B>> for StrCcsidCow<'a> {
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
            ccsid: state.format.ccsid,
            data: state.buffer.truncate(state.data_length).into_cow(),
            le: (state.format.encoding & sys::MQENC_INTEGER_REVERSED) != 0,
        }))
    }
}

impl<'buffer, P, B> MqiValue<P, GetState<B>> for Cow<'buffer, str>
where
    B: Buffer<'buffer>,
{
    type Error = GetStringError;

    fn consume<F>(param: &mut P, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut P) -> ResultComp<GetState<B>>,
    {
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
            Completion(_, Some((rc @ values::MQRC(sys::MQRC_NOT_CONVERTED), verb))) => {
                Err(Error(values::MQCC(sys::MQCC_WARNING), verb, rc).into())
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

impl<'buffer, P, B: Buffer<'buffer>> MqiValue<P, GetState<B>> for Cow<'buffer, [u8]> {
    type Error = Error;

    fn consume<F>(param: &mut P, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut P) -> ResultComp<GetState<B>>,
    {
        get(param).map_completion(|state| state.buffer.truncate(state.data_length).into_cow())
    }
}

impl<'buffer, P, B: Buffer<'buffer>> MqiValue<P, GetState<B>> for Vec<u8> {
    type Error = Error;

    fn consume<F>(param: &mut P, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut P) -> ResultComp<GetState<B>>,
    {
        get(param).map_completion(|state| state.buffer.truncate(state.data_length).into_cow().into_owned())
    }
}

impl<'a, P, B: Buffer<'a>> MqiAttr<P, GetState<B>> for Headers<'a> {
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

impl<P, B> MqiAttr<P, GetState<B>> for MessageFormat {
    fn extract<F>(param: &mut P, get: F) -> ResultComp<(Self, GetState<B>)>
    where
        F: FnOnce(&mut P) -> ResultComp<GetState<B>>,
    {
        get(param).map_completion(|state| (state.format, state))
    }
}

impl<S> MqiAttr<GetParam, S> for MqStruct<'static, sys::MQMD2> {
    fn extract<F>(param: &mut GetParam, get: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<S>,
    {
        get(param).map_completion(|state| (param.md.clone(), state))
    }
}

impl<S> MqiAttr<GetParam, S> for MessageId {
    fn extract<F>(param: &mut GetParam, get: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<S>,
    {
        get(param).map_completion(|state| (Self(param.md.MsgId.into()), state))
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
        B: Buffer<'b>,
    {
        self.get_as(options, buffer)
    }

    pub fn get_data_with<'b, A, B>(&self, options: impl GetOption, buffer: B) -> ResultComp<Option<(Cow<'b, [u8]>, A)>>
    where
        A: GetAttr<B>,
        B: Buffer<'b>,
    {
        self.get_as(options, buffer)
    }

    pub fn get_string<'b, B>(
        &self,
        options: impl GetOption,
        buffer: B,
    ) -> ResultCompErr<Option<StrCcsidCow<'b>>, GetStringCcsidError>
    where
        B: Buffer<'b>,
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
        B: Buffer<'b>,
    {
        self.get_as(options, buffer)
    }

    pub fn get_as<'b, R, B>(&self, options: impl GetOption, buffer: B) -> ResultCompErr<Option<R>, R::Error>
    where
        R: GetValue<B>,
        B: Buffer<'b>,
    {
        let mut param = GetParam {
            md: MqStruct::default(),
            gmo: MqStruct::new(sys::MQGMO {
                Version: sys::MQGMO_VERSION_4,
                ..sys::MQGMO::default()
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
                    format: MessageFormat {
                        ccsid: CCSID(param.md.CodedCharSetId),
                        encoding: values::MQENC(param.md.Encoding),
                        fmt: TextEnc::Ascii(unsafe { transmute::<[i8; 8], Fmt>(param.md.Format) }),
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
