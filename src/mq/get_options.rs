use std::borrow::Cow;

use crate::{
    conversion, macros::all_option_tuples, sys, types, values, Completion, Conn, Error, Properties, ResultComp, ResultCompErr,
    prelude::*,
};

use super::{
    get::{
        GetAttr, GetConvert, GetOption, GetParam, GetState, GetStringCcsidError, GetStringError, GetValue, GetWait, Headers,
        MatchOptions,
    },
    headers, impl_mqstruct_min_version, Buffer, MqStruct, StrCcsidCow,
};

all_option_tuples!(GetOption, GetParam);

impl_mqstruct_min_version!(sys::MQGMO);

impl GetOption for values::MQGMO {
    fn apply_param(self, param: &mut GetParam) {
        param.gmo.Options |= self.value();
    }
}

impl GetOption for GetWait {
    fn apply_param(self, param: &mut GetParam) {
        match self {
            Self::NoWait => param.gmo.Options |= sys::MQGMO_NO_WAIT,
            Self::Wait(interval) => {
                param.gmo.Options |= sys::MQGMO_WAIT;
                param.gmo.WaitInterval = interval;
            }
        }
    }
}

impl GetOption for GetConvert {
    fn apply_param(self, param: &mut GetParam) {
        match self {
            Self::NoConvert => {}
            Self::Convert => param.gmo.Options |= sys::MQGMO_CONVERT,
            Self::ConvertTo(ccsid, encoding) => {
                param.gmo.Options |= sys::MQGMO_CONVERT;
                param.md.CodedCharSetId = ccsid.0;
                param.md.Encoding = encoding.value();
            }
        }
    }
}

impl<C: Conn> GetOption for &mut Properties<C> {
    fn apply_param(self, param: &mut GetParam) {
        param.gmo.set_min_version(sys::MQGMO_VERSION_4);
        param.gmo.Options |= sys::MQGMO_PROPERTIES_IN_HANDLE;
        param.gmo.MsgHandle = unsafe { self.handle().raw_handle() }
    }
}

impl GetOption for MatchOptions<'_> {
    fn apply_param(self, param: &mut GetParam) {
        // Set up the MQMD
        if let Some(msg_id) = self.msg_id {
            param.md.MsgId = *msg_id.0;
        }
        if let Some(correl_id) = self.correl_id {
            param.md.CorrelId = *correl_id.0;
        }
        if let Some(group_id) = self.group_id {
            param.md.GroupId = *group_id.0;
        }
        param.md.MsgSeqNumber = self.seq_number.unwrap_or(0);
        param.md.Offset = self.offset.unwrap_or(0);

        // Set up the GMO
        if let Some(token) = self.token {
            param.gmo.set_min_version(sys::MQGMO_VERSION_3);
            param.gmo.MsgToken = token.0;
        }
        param.gmo.set_min_version(sys::MQGMO_VERSION_2);
        param.gmo.MatchOptions = self.correl_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_CORREL_ID)
            | self.msg_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_ID)
            | self.group_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_GROUP_ID)
            | self.seq_number.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_SEQ_NUMBER)
            | self.offset.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_OFFSET)
            | self.token.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_TOKEN);
    }
}

impl GetOption for types::CorrelationId {
    fn apply_param(self, param: &mut GetParam) {
        param.md.CorrelId = *self.0;
        param.gmo.MatchOptions |= sys::MQMO_MATCH_CORREL_ID;
    }
}

impl GetOption for types::MessageId {
    fn apply_param(self, param: &mut GetParam) {
        param.md.MsgId = *self.0;
        param.gmo.MatchOptions |= sys::MQMO_MATCH_MSG_ID;
    }
}

impl GetOption for types::GroupId {
    fn apply_param(self, param: &mut GetParam) {
        param.md.GroupId = *self.0;
        param.gmo.MatchOptions |= sys::MQMO_MATCH_GROUP_ID;
    }
}

impl GetOption for types::MsgToken {
    fn apply_param(self, param: &mut GetParam) {
        param.gmo.MsgToken = self.0;
        param.gmo.MatchOptions |= sys::MQMO_MATCH_MSG_TOKEN;
    }
}

#[expect(unused_parens)]
mod get_impl {
    use crate::get::{GetAttr, GetValue, GetParam, GetState};
    use crate::macros::all_multi_tuples;
    use crate::prelude::*;
    use crate::{ResultCompErr, ResultComp};

    macro_rules! impl_getvalue {
        ([$first:ident, $($ty:ident),*]) => {
            #[expect(non_snake_case)]
            impl<B, $first, $($ty),*> GetValue<B> for ($first, $($ty),*)
            where
                $first: GetValue<B>,
                $($ty: GetAttr<B>),*
            {
                type Error = $first::Error;

                #[inline]
                fn consume<F>(param: &mut GetParam, mqi: F) -> ResultCompErr<Self, Self::Error>
                where
                    F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
                {
                    let mut rest_outer = None;
                    $first::consume(param, |param| {
                        <($($ty),*) as GetAttr<B>>::extract(param, mqi).map_completion(|(rest, state)| {
                            rest_outer = Some(rest);
                            state
                        })
                    })
                    .map_completion(|a| {
                        let ($($ty),*) = rest_outer.expect("rest_outer should be set by the extract closure");
                        (a, $($ty),*)
                    })
                }

                fn max_data_size() -> Option<std::num::NonZero<usize>> {
                    $first::max_data_size()
                }
            }
        };
    }

    macro_rules! impl_getattr {
        ([$first:ident, $($ty:ident),*]) => {
            #[expect(non_snake_case)]
            impl<B, $first, $($ty),*> GetAttr<B> for ($first, $($ty),*)
            where
                $first: GetAttr<B>,
                $($ty: GetAttr<B>),*
            {
                #[inline]
                fn extract<F>(param: &mut GetParam, mqi: F) -> ResultComp<(Self, GetState<B>)>
                where
                    F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>
                {
                    let mut rest_outer = None;
                    $first::extract(param, |param| {
                        <($($ty),*) as GetAttr<B>>::extract(param, mqi).map_completion(|(rest, state)| {
                            rest_outer = Some(rest);
                            state
                        })
                    })
                    .map_completion(|(a, s)| {
                        let ($($ty),*) = rest_outer.expect("rest_outer should be set by extract closure");
                        ((a, $($ty),*), s)
                    })
                }
            }
        }
    }

    all_multi_tuples!(impl_getvalue);
    all_multi_tuples!(impl_getattr);
}

impl<'a, B> GetValue<B> for StrCcsidCow<'a>
where
    B: Buffer<'a, u8>,
{
    type Error = GetStringCcsidError;

    fn consume<F>(param: &mut GetParam, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
    {
        let state = get(param)?;
        if state.format.fmt != headers::TextEnc::Ascii(headers::fmt::MQFMT_STRING) {
            return Err(GetStringCcsidError::UnexpectedFormat(state.format.fmt, state.warning()));
        }

        Ok(state.map(|state| Self {
            ccsid: state.format.ccsid,
            data: conversion::bytes_to_cow_mqchar(state.buffer.truncate(state.data_length).into_cow()),
            le: (state.format.encoding & sys::MQENC_INTEGER_REVERSED) != 0,
        }))
    }
}

impl<'buffer, B> GetValue<B> for Cow<'buffer, str>
where
    B: Buffer<'buffer, u8>,
{
    type Error = GetStringError;

    fn consume<F>(param: &mut GetParam, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
    {
        // TODO: set 1208 in MQMD?
        let get_result = get(param)?;

        if get_result.format.fmt != headers::TextEnc::Ascii(headers::fmt::MQFMT_STRING) || get_result.format.ccsid != 1208 {
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

impl<'buffer, B, T> GetValue<B> for Cow<'buffer, [T]>
where
    B: Buffer<'buffer, T>,
    [T]: ToOwned,
{
    type Error = Error;

    #[inline]
    fn consume<F>(param: &mut GetParam, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
    {
        get(param).map_completion(|state| state.buffer.truncate(state.data_length).into_cow())
    }
}

impl<'buffer, B> GetValue<B> for Vec<sys::MQBYTE>
where
    B: Buffer<'buffer, u8>,
{
    type Error = Error;

    #[inline]
    fn consume<F>(param: &mut GetParam, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
    {
        get(param).map_completion(|state| state.buffer.truncate(state.data_length).into_cow().into_owned())
    }
}

impl<'a, B> GetAttr<B> for Headers<'a>
where
    B: Buffer<'a, u8>,
{
    fn extract<F>(param: &mut GetParam, get: F) -> ResultComp<(Self, GetState<B>)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
    {
        let state = get(param)?;

        let data = &state.buffer.as_ref()[..state.data_length];
        let mut header_length = 0;
        let mut final_format = state.format;
        let mut error = None;
        for result in headers::Header::iter(data, state.format) {
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
                Self::new(state.message_length, state.format, headers.into_cow(), error),
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

impl<B> GetAttr<B> for types::MessageFormat {
    #[inline]
    fn extract<F>(param: &mut GetParam, get: F) -> ResultComp<(Self, GetState<B>)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
    {
        get(param).map_completion(|state| (state.format, state))
    }
}

impl<B> GetAttr<B> for MqStruct<'static, sys::MQMD2> {
    #[inline]
    fn extract<F>(param: &mut GetParam, get: F) -> ResultComp<(Self, GetState<B>)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
    {
        get(param).map_completion(|state| (param.md.clone(), state))
    }
}

impl<B> GetAttr<B> for types::MessageId {
    #[inline]
    fn extract<F>(param: &mut GetParam, get: F) -> ResultComp<(Self, GetState<B>)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
    {
        get(param).map_completion(|state| (Self(param.md.MsgId.into()), state))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use libmqm_default as default;

    #[expect(clippy::unnecessary_wraps)]
    fn empty_string(_: &mut GetParam) -> ResultComp<GetState<&'static mut [u8]>> {
        Ok(Completion::new(GetState {
            buffer: &mut [],
            data_length: 0,
            message_length: 0,
            format: types::MessageFormat {
                ccsid: values::CCSID(1208),
                encoding: values::MQENC(sys::MQENC_NATIVE),
                fmt: headers::TextEnc::Ascii(headers::fmt::MQFMT_STRING),
            },
        }))
    }

    fn default_getparam() -> GetParam {
        GetParam {
            md: MqStruct::new(default::MQMD2_DEFAULT),
            gmo: MqStruct::new(default::MQGMO_DEFAULT),
        }
    }

    #[test]
    pub fn get_value_strccsdcow() -> Result<(), Box<dyn std::error::Error>> {
        let mut params = default_getparam();

        let result: StrCcsidCow = GetValue::consume(&mut params, empty_string).discard_warning()?;
        assert_eq!(result.ccsid, values::CCSID(1208));
        assert_eq!(result.data, Cow::from(&[]));

        Ok(())
    }
}
