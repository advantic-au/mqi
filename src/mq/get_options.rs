use std::borrow::Cow;

use crate::{
    conversion, macros::all_option_tuples, sys, types, values, Completion, Conn, Error, Properties, ResultComp, ResultCompErr,
    prelude::*,
};

use super::{
    get::{
        GetAttr, GetConvert, GetOption, GetParam, GetState, GetStringCcsidError, GetStringError, GetValue, GetWait,
        Headers, MatchOptions,
    },
    headers, impl_mqstruct_min_version, Buffer, MqStruct, StrCcsidCow,
};

all_option_tuples!(GetOption, GetParam);

impl_mqstruct_min_version!(sys::MQGMO);

impl GetOption for values::MQGMO {
    fn apply_param(&self, param: &mut GetParam) {
        param.gmo.Options |= self.value();
    }
}

impl GetOption for GetWait {
    fn apply_param(&self, param: &mut GetParam) {
        match self {
            Self::NoWait => {
                param.gmo.Options &= !sys::MQGMO_WAIT;
                param.gmo.Options |= sys::MQGMO_NO_WAIT;
            }
            Self::Wait(interval) => {
                param.gmo.Options &= !sys::MQGMO_NO_WAIT;
                param.gmo.Options |= sys::MQGMO_WAIT;
                param.gmo.WaitInterval = *interval;
            }
        }
    }
}

impl GetOption for GetConvert {
    fn apply_param(&self, param: &mut GetParam) {
        match self {
            Self::NoConvert => param.gmo.Options &= !sys::MQGMO_CONVERT,
            Self::Convert => param.gmo.Options |= sys::MQGMO_CONVERT,
            Self::ConvertTo(ccsid, encoding) => {
                param.gmo.Options |= sys::MQGMO_CONVERT;
                param.md.CodedCharSetId = ccsid.0 as sys::MQLONG;
                param.md.Encoding = encoding.value();
            }
        }
    }
}

impl<C: Conn> GetOption for &mut Properties<C> {
    fn apply_param(&self, param: &mut GetParam) {
        param.gmo.set_min_version(sys::MQGMO_VERSION_4);
        param.gmo.Options |= sys::MQGMO_PROPERTIES_IN_HANDLE;
        param.gmo.MsgHandle = unsafe { self.handle().raw_handle() }
    }
}

impl GetOption for MatchOptions<'_> {
    fn apply_param(&self, param: &mut GetParam) {
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
    fn apply_param(&self, param: &mut GetParam) {
        param.md.CorrelId = *self.0;
        param.gmo.MatchOptions |= sys::MQMO_MATCH_CORREL_ID;
    }
}

impl GetOption for types::MessageId {
    fn apply_param(&self, param: &mut GetParam) {
        param.md.MsgId = *self.0;
        param.gmo.MatchOptions |= sys::MQMO_MATCH_MSG_ID;
    }
}

impl GetOption for types::GroupId {
    fn apply_param(&self, param: &mut GetParam) {
        param.md.GroupId = *self.0;
        param.gmo.MatchOptions |= sys::MQMO_MATCH_GROUP_ID;
    }
}

impl GetOption for types::MsgToken {
    fn apply_param(&self, param: &mut GetParam) {
        param.gmo.MsgToken = self.0;
        param.gmo.MatchOptions |= sys::MQMO_MATCH_MSG_TOKEN;
    }
}

#[cfg(feature = "mqai")]
#[expect(unused_parens)]
mod get_bag_impl {

    use crate::get::{GetBagAttr, GetParam};
    use crate::macros::all_multi_tuples;
    use crate::prelude::*;
    use crate::ResultComp;

    macro_rules! impl_getbagattr {
        ([$first:ident, $($ty:ident),*]) => {
            #[expect(non_snake_case)]
            impl<'b, $first, $($ty),*> GetBagAttr for ($first, $($ty),*)
            where
                $first: GetBagAttr,
                $($ty: GetBagAttr),*
            {
                #[inline]
                fn get_bag_extract<F>(param: &mut GetParam, get_bag: F) -> ResultComp<Self>
                where
                    F: FnOnce(&mut GetParam) -> ResultComp<()>,
                {
                    let mut rest_outer = None;
                    $first::get_bag_extract(param, |param| {
                        <($($ty),*) as GetBagAttr>::get_bag_extract(param, get_bag).map_completion(|rest| {
                            rest_outer = Some(rest);
                        })
                    })
                    .map_completion(|a| {
                        let ($($ty),*) = rest_outer.expect("rest_outer should be set by extract closure");
                        (a, $($ty),*)
                    })
                }
            }
        }
    }

    all_multi_tuples!(impl_getbagattr);
}

#[expect(unused_parens)]
mod get_impl {
    use crate::get::{GetAttr, GetValue, GetParam, GetState};
    use crate::Buffer;
    use crate::macros::all_multi_tuples;
    use crate::prelude::*;
    use crate::{ResultCompErr, ResultComp};

    macro_rules! impl_getvalue {
        ([$first:ident, $($ty:ident),*]) => {
            #[expect(non_snake_case)]
            impl<'b, $first, $($ty),*> GetValue<'b> for ($first, $($ty),*)
            where
                $first: GetValue<'b>,
                $($ty: GetAttr<'b>),*
            {
                type Error = $first::Error;

                #[inline]
                fn get_consume<F,B>(param: &mut GetParam, get: F) -> ResultCompErr<Self, Self::Error>
                where
                    F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
                    B: Buffer<'b, u8>,
                {
                    let mut rest_outer = None;
                    $first::get_consume(param, |param| {
                        <($($ty),*) as GetAttr>::get_extract(param, get).map_completion(|(rest, state)| {
                            rest_outer = Some(rest);
                            state
                        })
                    })
                    .map_completion(|a| {
                        let ($($ty),*) = rest_outer.expect("rest_outer should be set by the extract closure");
                        (a, $($ty),*)
                    })
                }

                fn get_max_data_size() -> Option<std::num::NonZero<usize>> {
                    $first::get_max_data_size()
                }
            }
        };
    }

    macro_rules! impl_getattr {
        ([$first:ident, $($ty:ident),*]) => {
            #[expect(non_snake_case)]
            impl<'b, $first, $($ty),*> GetAttr<'b> for ($first, $($ty),*)
            where
                $first: GetAttr<'b>,
                $($ty: GetAttr<'b>),*
            {
                #[inline]
                fn get_extract<F, B>(param: &mut GetParam, get: F) -> ResultComp<(Self, GetState<B>)>
                where
                    F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
                    B: Buffer<'b, u8>,
                {
                    let mut rest_outer = None;
                    $first::get_extract(param, |param| {
                        <($($ty),*) as GetAttr>::get_extract(param, get).map_completion(|(rest, state)| {
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

impl<'b> GetValue<'b> for StrCcsidCow<'b> {
    type Error = GetStringCcsidError;

    fn get_consume<F, B>(param: &mut GetParam, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        B: Buffer<'b, u8>,
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

impl<'b> GetValue<'b> for Cow<'b, str> {
    type Error = GetStringError;

    fn get_consume<F, B>(param: &mut GetParam, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        B: Buffer<'b, u8>,
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

impl<'b> GetValue<'b> for Cow<'b, [u8]> {
    type Error = Error;

    #[inline]
    fn get_consume<F, B>(param: &mut GetParam, get: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        B: Buffer<'b, u8>,
    {
        get(param).map_completion(|state| state.buffer.truncate(state.data_length).into_cow())
    }
}

impl<'b> GetValue<'b> for Vec<u8> {
    type Error = Error;

    #[inline]
    fn get_consume<F, B>(param: &mut GetParam, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        B: Buffer<'b, u8>,
    {
        Cow::<[u8]>::get_consume(param, get).map_completion(Cow::into_owned)
    }
}

impl<'b> GetAttr<'b> for Headers<'b> {
    fn get_extract<F, B>(param: &mut GetParam, get: F) -> ResultComp<(Self, GetState<B>)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        B: Buffer<'b, u8>,
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

impl<'b> GetAttr<'b> for types::MessageFormat {
    #[inline]
    fn get_extract<F, B>(param: &mut GetParam, get: F) -> ResultComp<(Self, GetState<B>)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        B: Buffer<'b, u8>,
    {
        get(param).map_completion(|state| (state.format, state))
    }
}

impl<'b> GetAttr<'b> for MqStruct<'static, sys::MQMD2> {
    #[inline]
    fn get_extract<F, B>(param: &mut GetParam, get: F) -> ResultComp<(Self, GetState<B>)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        B: Buffer<'b, u8>,
    {
        get(param).map_completion(|state| (param.md.clone(), state))
    }
}

impl<'b> GetAttr<'b> for types::MessageId {
    #[inline]
    fn get_extract<F, B>(param: &mut GetParam, get: F) -> ResultComp<(Self, GetState<B>)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        B: Buffer<'b, u8>,
    {
        get(param).map_completion(|state| (Self(param.md.MsgId.into()), state))
    }
}

impl super::get::GetBagAttr for () {
    fn get_bag_extract<F>(param: &mut GetParam, get_bag: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<()>,
    {
        get_bag(param) // No extra data to retrieve
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use super::*;
    use libmqm_default as default;
    use types::{CorrelationId, Identifier, MessageFormat};

    const FMT_STRING: types::MessageFormat = types::MessageFormat {
        ccsid: values::CCSID(1208),
        encoding: values::MQENC(sys::MQENC_NATIVE),
        fmt: headers::TextEnc::Ascii(headers::fmt::MQFMT_STRING),
    };

    const FMT_BYTES: types::MessageFormat = types::MessageFormat {
        ccsid: values::CCSID(1208),
        encoding: values::MQENC(sys::MQENC_NATIVE),
        fmt: headers::TextEnc::Ascii(headers::fmt::MQFMT_NONE),
    };

    fn mock_get_failure<T>(rc: values::MQRC) -> impl FnOnce(&mut GetParam) -> ResultComp<T> {
        move |_| Err(Error(values::MQCC(sys::MQCC_FAILED), "MQGET", rc))
    }

    fn mock_get_message<'b, B: Buffer<'b, u8>>(
        buffer: B,
        fmt: types::MessageFormat,
    ) -> impl FnOnce(&mut GetParam) -> ResultComp<GetState<B>> {
        let len = buffer.len();
        move |_| {
            Ok(Completion::new(GetState {
                buffer,
                data_length: len,
                message_length: len,
                format: fmt,
            }))
        }
    }

    fn default_getparam() -> GetParam {
        GetParam {
            md: MqStruct::new(default::MQMD2_DEFAULT),
            gmo: MqStruct::new(default::MQGMO_DEFAULT),
        }
    }

    fn test_get_option<F>(params: &mut GetParam, option: &impl GetOption, f: F)
    where
        F: FnOnce(&GetParam),
    {
        option.apply_param(params);
        f(params);
    }

    #[test]
    pub fn get_value_cow_bytes() -> Result<(), Box<dyn std::error::Error>> {
        // Empty string
        let empty: &mut [u8] = &mut [];
        let mut params = default_getparam();
        let empty_result: Cow<[u8]> =
            GetValue::get_consume(&mut params, mock_get_message(empty, FMT_STRING)).discard_warning()?;
        assert_eq!(empty_result, Cow::from(&[]));

        // Empty bytes
        let empty: &mut [u8] = &mut [];
        let mut params = default_getparam();
        let empty_result: Cow<[u8]> = GetValue::get_consume(&mut params, mock_get_message(empty, FMT_BYTES)).discard_warning()?;
        assert_eq!(empty_result, Cow::from(&[]));

        // Failure should be passed through
        let mut params = default_getparam();
        let failure: ResultCompErr<Cow<[u8]>, _> =
            GetValue::get_consume::<_, &mut [u8]>(&mut params, mock_get_failure(values::MQRC(sys::MQRC_NOT_AUTHORIZED)));
        assert!(matches!(failure, Err(Error(_, _, values::MQRC(sys::MQRC_NOT_AUTHORIZED)))));

        Ok(())
    }

    #[test]
    pub fn get_value_strccsdcow() -> Result<(), Box<dyn std::error::Error>> {
        let mut empty: [u8; 0] = [];
        // Empty string
        let mut params = default_getparam();
        let empty_result: StrCcsidCow =
            GetValue::get_consume(&mut params, mock_get_message(empty.as_mut_slice(), FMT_STRING)).discard_warning()?;
        assert_eq!(empty_result.ccsid, values::CCSID(1208));
        assert_eq!(empty_result.data, Cow::from(&[]));

        // Empty bytes message should fail
        let mut params = default_getparam();
        let empty_bytes: ResultCompErr<StrCcsidCow, _> =
            GetValue::get_consume(&mut params, mock_get_message(empty.as_mut_slice(), FMT_BYTES));
        assert!(matches!(
            empty_bytes,
            Err(GetStringCcsidError::UnexpectedFormat(
                headers::TextEnc::Ascii(headers::fmt::MQFMT_NONE),
                None
            ))
        ));

        // Failure should be passed through
        let mut params = default_getparam();
        let failure: ResultCompErr<StrCcsidCow, _> =
            GetValue::get_consume::<_, &mut [u8]>(&mut params, mock_get_failure(values::MQRC(sys::MQRC_NOT_AUTHORIZED)));
        assert!(matches!(
            failure,
            Err(GetStringCcsidError::MQ(Error(_, _, values::MQRC(sys::MQRC_NOT_AUTHORIZED))))
        ));

        Ok(())
    }

    #[test]
    pub fn get_value_cow_str() -> Result<(), Box<dyn std::error::Error>> {
        let mut empty: [u8; 0] = [];
        // Empty string
        let mut params = default_getparam();
        let empty_result: Cow<str> =
            GetValue::get_consume(&mut params, mock_get_message(empty.as_mut_slice(), FMT_STRING)).discard_warning()?;
        assert_eq!(empty_result, Cow::from(""));

        // Empty bytes message should fail
        let mut params = default_getparam();
        let empty_bytes: ResultCompErr<Cow<str>, _> =
            GetValue::get_consume(&mut params, mock_get_message(empty.as_mut_slice(), FMT_BYTES));
        assert!(matches!(
            empty_bytes,
            Err(GetStringError::UnexpectedFormat(
                headers::TextEnc::Ascii(headers::fmt::MQFMT_NONE),
                _,
                None
            ))
        ));

        // Empty string with owned (Vec) origin
        let mut params = default_getparam();
        let empty_result: Cow<str> =
            GetValue::get_consume(&mut params, mock_get_message(Vec::new(), FMT_STRING)).discard_warning()?;
        assert_eq!(empty_result, Cow::from(""));

        // invalid UTF-8
        let mut invalid: [u8; 2] = [0xa0, 0xa1];
        let mut params = default_getparam();
        let failure: ResultCompErr<Cow<str>, _> =
            GetValue::get_consume(&mut params, mock_get_message(invalid.as_mut_slice(), FMT_STRING));
        assert!(matches!(failure, Err(GetStringError::Utf8Parse(_, None))));

        // Failure should be passed through
        let mut params = default_getparam();
        let failure: ResultCompErr<Cow<str>, _> =
            GetValue::get_consume::<_, &mut [u8]>(&mut params, mock_get_failure(values::MQRC(sys::MQRC_NOT_AUTHORIZED)));
        assert!(matches!(
            failure,
            Err(GetStringError::MQ(Error(_, _, values::MQRC(sys::MQRC_NOT_AUTHORIZED))))
        ));

        Ok(())
    }

    #[test]
    pub fn get_value_tuple() -> Result<(), Box<dyn std::error::Error>> {
        let mut empty: [u8; 0] = [];
        let mut params = default_getparam();
        let (data, fmt): (Cow<[u8]>, MessageFormat) =
            GetValue::get_consume(&mut params, mock_get_message(empty.as_mut_slice(), FMT_BYTES)).discard_warning()?;
        assert_eq!(data, Cow::from(&[]));
        assert_eq!(fmt, FMT_BYTES);

        Ok(())
    }

    #[test]
    pub fn get_option_correlationid() {
        const ID: Identifier<24> = Identifier([0xC; 24]);
        test_get_option(&mut default_getparam(), &CorrelationId(ID), |p| {
            assert_eq!(p.md.CorrelId, ID.0);
            assert_ne!(p.gmo.MatchOptions & sys::MQMO_MATCH_CORREL_ID, 0);
        });
        let mut get_param = default_getparam();
        get_param.gmo.MatchOptions = !0;
        test_get_option(&mut get_param, &CorrelationId(ID), |p| assert_eq!(p.gmo.MatchOptions, !0));
    }

    #[test]
    pub fn get_option_messageid() {
        const ID: Identifier<24> = Identifier([0xC; 24]);
        test_get_option(&mut default_getparam(), &types::MessageId(ID), |p| {
            assert_eq!(p.md.MsgId, ID.0);
            assert_ne!(p.gmo.MatchOptions & sys::MQMO_MATCH_MSG_ID, 0);
        });
        let mut get_param = default_getparam();
        get_param.gmo.MatchOptions = !0;
        test_get_option(&mut get_param, &types::MessageId(ID), |p| assert_eq!(p.gmo.MatchOptions, !0));
    }
    #[test]
    pub fn get_option_groupid() {
        const ID: Identifier<24> = Identifier([0xC; 24]);
        test_get_option(&mut default_getparam(), &types::GroupId(ID), |p| {
            assert_eq!(p.md.GroupId, ID.0);
            assert_ne!(p.gmo.MatchOptions & sys::MQMO_MATCH_GROUP_ID, 0);
        });
        let mut get_param = default_getparam();
        get_param.gmo.MatchOptions = !0;
        test_get_option(&mut get_param, &types::GroupId(ID), |p| assert_eq!(p.gmo.MatchOptions, !0));
    }

    #[test]
    pub fn get_option_msgtoken() {
        const TOKEN: [u8; 16] = [0xa; 16];
        test_get_option(&mut default_getparam(), &types::MsgToken(TOKEN), |p| {
            assert_eq!(p.gmo.MsgToken, TOKEN);
            assert_ne!(p.gmo.MatchOptions & sys::MQMO_MATCH_MSG_TOKEN, 0);
        });
        let mut get_param = default_getparam();
        get_param.gmo.MatchOptions = !0;
        test_get_option(&mut get_param, &types::MsgToken(TOKEN), |p| {
            assert_eq!(p.gmo.MatchOptions, !0);
        });
    }

    #[test]
    pub fn get_option_get_wait() {
        let mut get_param = default_getparam();
        get_param.gmo.MatchOptions = !0;
        test_get_option(&mut get_param, &GetWait::NoWait, |p| {
            assert_eq!(p.gmo.Options & sys::MQGMO_WAIT, 0);
        });
        get_param.gmo.MatchOptions = !0;
        test_get_option(&mut get_param, &GetWait::Wait(50), |p| {
            assert_ne!(p.gmo.Options & sys::MQGMO_WAIT, 0);
            assert_eq!(p.gmo.WaitInterval, 50);
        });
    }
}
