use std::borrow::Cow;

use libmqm_sys as mq;

use crate::{
    Buffer, CCSID, Completion, Conn, Error, Properties, ResultComp, ResultCompErr, StrCcsidCow, constants, conversion, headers,
    prelude::*, structs, types,
};

use super::option;

#[derive(Debug, Clone, Default)]
pub struct MatchOptions<'a> {
    pub msg_id: Option<&'a types::MessageId>,
    pub correl_id: Option<&'a types::CorrelationId>,
    pub group_id: Option<&'a types::GroupId>,
    pub seq_number: Option<types::MQLONG>,
    pub offset: Option<types::MQLONG>,
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

#[derive(Default)]
pub enum GetWait {
    #[default]
    NoWait,
    Wait(types::MQLONG),
}

pub enum GetConvert {
    NoConvert,
    Convert,
    ConvertTo(CCSID, types::MQENC),
}

#[derive(Clone, Debug, derive_more::Constructor)]
pub struct Headers<'a> {
    message_length: usize,
    init_format: types::MessageFormat,
    data: Cow<'a, [u8]>,
    error: Option<headers::HeaderError>,
}

impl<'a> Headers<'a> {
    pub fn all_headers(&'a self) -> impl Iterator<Item = headers::Header<'a>> {
        headers::Header::iter(&self.data, self.init_format).filter_map(|result| match result {
            Ok((header, ..)) => Some(header),
            Err(_) => None,
        })
    }

    pub fn header<C: headers::ChainedHeader + 'a>(&'a self) -> impl Iterator<Item = headers::EncodedHeader<'a, C>> {
        self.all_headers().filter_map(C::from_header)
    }

    #[must_use]
    pub const fn error(&self) -> Option<&headers::HeaderError> {
        self.error.as_ref()
    }

    #[must_use]
    pub const fn message_length(&self) -> usize {
        self.message_length
    }
}

// TODO: add MQ warnings to error messages
#[derive(derive_more::Error, derive_more::From, derive_more::Display, Debug)]
pub enum GetStringError {
    #[display("Message parsing error: {_0}")]
    Utf8Parse(std::str::Utf8Error, Option<types::Warning>),
    #[display("Unexpected format or CCSID. Message format = '{_0}', CCSID = {_1}")]
    UnexpectedFormat(headers::TextEnc<types::Fmt>, CCSID, Option<types::Warning>),
    #[from]
    MQ(Error),
}

#[derive(derive_more::Error, derive_more::Display, derive_more::From, Debug)]
pub enum GetStringCcsidError {
    #[display("Unexpected format. Message format = '{_0}'")]
    UnexpectedFormat(headers::TextEnc<types::Fmt>, Option<types::Warning>),
    #[from]
    MQ(Error),
}

impl option::GetOption for types::MQGMO {
    fn apply_param(&self, param: &mut option::GetParam) {
        let gmo_options: &mut Self = param.gmo.Options.as_mut();
        gmo_options.insert(*self);
    }
}

impl option::GetOption for GetWait {
    fn apply_param(&self, param: &mut option::GetParam) {
        let gmo_options: &mut types::MQGMO = param.gmo.Options.as_mut();
        match self {
            Self::NoWait => {
                gmo_options.remove(constants::MQGMO_WAIT);
                gmo_options.insert(constants::MQGMO_NO_WAIT);
            }
            Self::Wait(interval) => {
                gmo_options.remove(constants::MQGMO_NO_WAIT);
                gmo_options.insert(constants::MQGMO_WAIT);
                param.gmo.WaitInterval = *interval;
            }
        }
    }
}

impl option::GetOption for GetConvert {
    fn apply_param(&self, param: &mut option::GetParam) {
        let gmo_options: &mut types::MQGMO = param.gmo.Options.as_mut();
        match self {
            Self::NoConvert => gmo_options.remove(constants::MQGMO_CONVERT),
            Self::Convert => gmo_options.insert(constants::MQGMO_CONVERT),
            Self::ConvertTo(ccsid, encoding) => {
                gmo_options.insert(constants::MQGMO_CONVERT);
                *param.md.CodedCharSetId.as_mut() = *ccsid;
                *param.md.Encoding.as_mut() = *encoding;
            }
        }
    }
}

impl<C: Conn> option::GetOption for &mut Properties<C> {
    fn apply_param(&self, param: &mut option::GetParam) {
        param.gmo.set_min_version(mq::MQGMO_VERSION_4);
        let gmo_options: &mut types::MQGMO = param.gmo.Options.as_mut();
        gmo_options.insert(constants::MQGMO_PROPERTIES_IN_HANDLE);
        param.gmo.MsgHandle = unsafe { self.handle().raw_handle() }
    }
}

impl option::GetOption for MatchOptions<'_> {
    fn apply_param(&self, param: &mut option::GetParam) {
        // Set up the MQMD
        if let Some(msg_id) = self.msg_id {
            param.md.MsgId = msg_id.0;
        }
        if let Some(correl_id) = self.correl_id {
            param.md.CorrelId = correl_id.0;
        }
        if let Some(group_id) = self.group_id {
            param.md.GroupId = group_id.0;
        }
        param.md.MsgSeqNumber = self.seq_number.unwrap_or(0);
        param.md.Offset = self.offset.unwrap_or(0);

        // Set up the GMO
        if let Some(token) = self.token {
            param.gmo.set_min_version(mq::MQGMO_VERSION_3);
            param.gmo.MsgToken = token.0;
        }
        param.gmo.set_min_version(mq::MQGMO_VERSION_2);
        *param.gmo.MatchOptions.as_mut() = self
            .correl_id
            .map_or(constants::MQMO_NONE, |_| constants::MQMO_MATCH_CORREL_ID)
            | self.msg_id.map_or(constants::MQMO_NONE, |_| constants::MQMO_MATCH_MSG_ID)
            | self.group_id.map_or(constants::MQMO_NONE, |_| constants::MQMO_MATCH_GROUP_ID)
            | self
                .seq_number
                .map_or(constants::MQMO_NONE, |_| constants::MQMO_MATCH_MSG_SEQ_NUMBER)
            | self.offset.map_or(constants::MQMO_NONE, |_| constants::MQMO_MATCH_OFFSET)
            | self.token.map_or(constants::MQMO_NONE, |_| constants::MQMO_MATCH_MSG_TOKEN);
    }
}

impl option::GetOption for types::CorrelationId {
    fn apply_param(&self, param: &mut option::GetParam) {
        param.md.CorrelId = self.0;
        let match_options: &mut types::MQMO = param.gmo.MatchOptions.as_mut();
        match_options.insert(constants::MQMO_MATCH_CORREL_ID);
    }
}

impl option::GetOption for types::MessageId {
    fn apply_param(&self, param: &mut option::GetParam) {
        param.md.MsgId = self.0;
        let match_options: &mut types::MQMO = param.gmo.MatchOptions.as_mut();
        match_options.insert(constants::MQMO_MATCH_MSG_ID);
    }
}

impl option::GetOption for types::GroupId {
    fn apply_param(&self, param: &mut option::GetParam) {
        param.md.GroupId = self.0;
        let match_options: &mut types::MQMO = param.gmo.MatchOptions.as_mut();
        match_options.insert(constants::MQMO_MATCH_GROUP_ID);
    }
}

impl option::GetOption for types::MsgToken {
    fn apply_param(&self, param: &mut option::GetParam) {
        param.gmo.MsgToken = self.0;
        let match_options: &mut types::MQMO = param.gmo.MatchOptions.as_mut();
        match_options.insert(constants::MQMO_MATCH_MSG_TOKEN);
    }
}

#[cfg(feature = "mqai")]
#[expect(unused_parens)]
mod get_bag_impl {
    use super::option;

    use crate::{ResultComp, macros::all_multi_tuples, prelude::*};

    macro_rules! impl_getbagattr {
        ([$first:ident, $($ty:ident),*]) => {
            #[expect(non_snake_case)]
            #[diagnostic::do_not_recommend]
            impl<'b, $first, $($ty),*> option::GetBagAttr for ($first, $($ty),*)
            where
                $first: option::GetBagAttr,
                $($ty: option::GetBagAttr),*
            {
                #[inline]
                fn get_bag_extract<F>(param: &mut option::GetParam, get_bag: F) -> ResultComp<Self>
                where
                    F: FnOnce(&mut option::GetParam) -> ResultComp<()>,
                {
                    let mut rest_outer = None;
                    $first::get_bag_extract(param, |param| {
                        <($($ty),*) as option::GetBagAttr>::get_bag_extract(param, get_bag).map_completion(|rest| {
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

    impl option::GetBagAttr for () {
        fn get_bag_extract<F>(param: &mut option::GetParam, get_bag: F) -> ResultComp<Self>
        where
            F: FnOnce(&mut option::GetParam) -> ResultComp<()>,
        {
            get_bag(param) // No extra data to retrieve
        }
    }
}

#[expect(unused_parens)]
mod get_impl {
    use super::option;
    use crate::{
        Buffer, ResultComp, ResultCompErr,
        get::{GetAttr, GetState, GetValue},
        macros::all_multi_tuples,
        prelude::*,
    };

    macro_rules! impl_getvalue {
        ([$first:ident, $($ty:ident),*]) => {
            #[expect(non_snake_case)]
            #[diagnostic::do_not_recommend]
            impl<'b, R, B, $first, $($ty),*> GetValue<'b, R, B> for ($first, $($ty),*)
            where
                $first: GetValue<'b, R, B>,
                $($ty: GetAttr<'b, R>),*
            {
                type Error = $first::Error;

                #[inline]
                fn get_consume<F>(param: &mut option::GetParam, get: F) -> ResultCompErr<Self, Self::Error>
                where
                    F: FnOnce(&mut option::GetParam) -> ResultComp<GetState<B>>,
                    B: Buffer<'b, R>,
                {
                    let mut rest_outer = None;
                    $first::get_consume(param, |param| {
                        <($($ty),*) as GetAttr<R>>::get_extract(param, get).map_completion(|(rest, state)| {
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
            #[diagnostic::do_not_recommend]
            impl<'b, R, $first, $($ty),*> GetAttr<'b, R> for ($first, $($ty),*)
            where
                $first: GetAttr<'b, R>,
                $($ty: GetAttr<'b, R>),*
            {
                #[inline]
                fn get_extract<F, B>(param: &mut option::GetParam, get: F) -> ResultComp<(Self, GetState<B>)>
                where
                    F: FnOnce(&mut option::GetParam) -> ResultComp<GetState<B>>,
                    B: Buffer<'b, R>,
                {
                    let mut rest_outer = None;
                    $first::get_extract(param, |param| {
                        <($($ty),*) as GetAttr<R>>::get_extract(param, get).map_completion(|(rest, state)| {
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

impl<'b, B> option::GetValue<'b, u8, B> for StrCcsidCow<'b> {
    type Error = GetStringCcsidError;

    fn get_consume<F>(param: &mut option::GetParam, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut option::GetParam) -> ResultComp<super::GetState<B>>,
        B: Buffer<'b, u8>,
    {
        let state = get(param)?;
        if state.format.fmt != headers::TextEnc::Ascii(headers::fmt::MQFMT_STRING) {
            return Err(GetStringCcsidError::UnexpectedFormat(state.format.fmt, state.warning()));
        }

        Ok(state.map(|state| Self {
            ccsid: state.format.ccsid,
            le: state.format.encoding.contains(constants::MQENC_INTEGER_REVERSED),
            data: conversion::bytes_to_cow_mqchar(state.into_truncated_buffer().into_cow()),
        }))
    }
}

impl<'b, B> option::GetValue<'b, u8, B> for Cow<'b, str> {
    type Error = GetStringError;

    fn get_consume<F>(param: &mut option::GetParam, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut option::GetParam) -> ResultComp<super::GetState<B>>,
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

        match get_result.map(|state| state.into_truncated_buffer().into_cow()) {
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

impl<'b, R, B> option::GetValue<'b, R, B> for Cow<'b, [R]>
where
    R: Clone,
{
    type Error = Error;

    #[inline]
    fn get_consume<F>(param: &mut option::GetParam, get: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut option::GetParam) -> ResultComp<super::GetState<B>>,
        B: Buffer<'b, R>,
    {
        get(param).map_completion(|state| state.into_truncated_buffer().into_cow())
    }
}

impl<'b, R, B> option::GetValue<'b, R, B> for &'b mut [R]
where
    B: Into<Self>,
{
    type Error = Error;

    fn get_consume<F>(param: &mut option::GetParam, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut option::GetParam) -> ResultComp<super::GetState<B>>,
        B: Buffer<'b, R>,
    {
        get(param).map_completion(|state| state.into_truncated_buffer().into())
    }
}

impl<'b, R, B> option::GetValue<'b, R, B> for Vec<R>
where
    B: Into<Self>,
{
    type Error = Error;

    #[inline]
    fn get_consume<F>(param: &mut option::GetParam, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut option::GetParam) -> ResultComp<super::GetState<B>>,
        B: Buffer<'b, R>,
    {
        get(param).map_completion(|state| state.into_truncated_buffer().into())
    }
}

impl<'b> super::GetAttr<'b, u8> for Headers<'b> {
    fn get_extract<F, B>(param: &mut option::GetParam, get: F) -> ResultComp<(Self, super::GetState<B>)>
    where
        F: FnOnce(&mut option::GetParam) -> ResultComp<super::GetState<B>>,
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
                super::GetState {
                    buffer: tail,
                    data_length: state.data_length - header_length,
                    message_length: state.message_length - header_length,
                    format: final_format,
                },
            )
        }))
    }
}

impl<'b, R> super::GetAttr<'b, R> for types::MessageFormat {
    #[inline]
    fn get_extract<F, B>(param: &mut option::GetParam, get: F) -> ResultComp<(Self, super::GetState<B>)>
    where
        F: FnOnce(&mut option::GetParam) -> ResultComp<super::GetState<B>>,
        B: Buffer<'b, R>,
    {
        get(param).map_completion(|state| (state.format, state))
    }
}

impl<'b, R> super::GetAttr<'b, R> for structs::MQMD2 {
    #[inline]
    fn get_extract<F, B>(param: &mut option::GetParam, get: F) -> ResultComp<(Self, super::GetState<B>)>
    where
        F: FnOnce(&mut option::GetParam) -> ResultComp<super::GetState<B>>,
        B: Buffer<'b, R>,
    {
        get(param).map_completion(|state| (param.md.clone(), state))
    }
}

impl<'b, R> super::GetAttr<'b, R> for types::MessageId {
    #[inline]
    fn get_extract<F, B>(param: &mut option::GetParam, get: F) -> ResultComp<(Self, super::GetState<B>)>
    where
        F: FnOnce(&mut option::GetParam) -> ResultComp<super::GetState<B>>,
        B: Buffer<'b, R>,
    {
        get(param).map_completion(|state| (Self(param.md.MsgId), state))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use libmqm_default as default;
    use types::{CorrelationId, Identifier, MessageFormat};

    use super::*;
    use crate::{CCSID, constants};

    const FMT_STRING: types::MessageFormat = types::MessageFormat {
        ccsid: CCSID(1208),
        encoding: constants::MQENC_NATIVE,
        fmt: headers::TextEnc::Ascii(headers::fmt::MQFMT_STRING),
    };

    const FMT_BYTES: types::MessageFormat = types::MessageFormat {
        ccsid: CCSID(1208),
        encoding: constants::MQENC_NATIVE,
        fmt: headers::TextEnc::Ascii(headers::fmt::MQFMT_NONE),
    };

    fn mock_get_failure<T>(rc: types::MQRC) -> impl FnOnce(&mut option::GetParam) -> ResultComp<T> {
        move |_| Err(Error(constants::MQCC_FAILED, "MQGET", rc))
    }

    fn mock_get_message<'b, B: Buffer<'b, u8>>(
        buffer: B,
        fmt: types::MessageFormat,
    ) -> impl FnOnce(&mut option::GetParam) -> ResultComp<option::GetState<B>> + use<B> {
        let len = buffer.len();
        move |_| {
            Ok(Completion::new(option::GetState {
                buffer,
                data_length: len,
                message_length: len,
                format: fmt,
            }))
        }
    }

    const fn default_getparam() -> option::GetParam {
        option::GetParam {
            md: structs::MQMD2::new(default::MQMD2_DEFAULT),
            gmo: structs::MQGMO::new(default::MQGMO_DEFAULT),
        }
    }

    fn test_get_option<F>(params: &mut option::GetParam, option: &impl option::GetOption, f: F)
    where
        F: FnOnce(&option::GetParam),
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
            option::GetValue::get_consume(&mut params, mock_get_message(empty, FMT_STRING)).discard_warning()?;
        assert_eq!(empty_result, Cow::from(&[]));

        // Empty bytes
        let empty: &mut [u8] = &mut [];
        let mut params = default_getparam();
        let empty_result: Cow<[u8]> =
            option::GetValue::get_consume(&mut params, mock_get_message(empty, FMT_BYTES)).discard_warning()?;
        assert_eq!(empty_result, Cow::from(&[]));

        // Failure should be passed through
        let mut params = default_getparam();
        let failure: ResultCompErr<Cow<[u8]>, _> =
            option::GetValue::<u8, &mut [u8]>::get_consume(&mut params, mock_get_failure(constants::MQRC_NOT_AUTHORIZED));
        assert!(matches!(failure, Err(Error(_, _, constants::MQRC_NOT_AUTHORIZED))));

        Ok(())
    }

    #[test]
    pub fn get_value_strccsdcow() -> Result<(), Box<dyn std::error::Error>> {
        let mut empty: [u8; 0] = [];
        // Empty string
        let mut params = default_getparam();
        let empty_result: StrCcsidCow =
            option::GetValue::get_consume(&mut params, mock_get_message(empty.as_mut_slice(), FMT_STRING)).discard_warning()?;
        assert_eq!(empty_result.ccsid, CCSID(1208));
        assert_eq!(empty_result.data, Cow::from(&[]));

        // Empty bytes message should fail
        let mut params = default_getparam();
        let empty_bytes: ResultCompErr<StrCcsidCow, _> =
            option::GetValue::get_consume(&mut params, mock_get_message(empty.as_mut_slice(), FMT_BYTES));
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
            option::GetValue::<_, &mut [u8]>::get_consume(&mut params, mock_get_failure(constants::MQRC_NOT_AUTHORIZED));
        assert!(matches!(
            failure,
            Err(GetStringCcsidError::MQ(Error(_, _, constants::MQRC_NOT_AUTHORIZED)))
        ));

        Ok(())
    }

    #[test]
    pub fn get_value_cow_str() -> Result<(), Box<dyn std::error::Error>> {
        let mut empty: [u8; 0] = [];
        // Empty string
        let mut params = default_getparam();
        let empty_result: Cow<str> =
            option::GetValue::get_consume(&mut params, mock_get_message(empty.as_mut_slice(), FMT_STRING)).discard_warning()?;
        assert_eq!(empty_result, Cow::from(""));

        // Empty bytes message should fail
        let mut params = default_getparam();
        let empty_bytes: ResultCompErr<Cow<str>, _> =
            option::GetValue::get_consume(&mut params, mock_get_message(empty.as_mut_slice(), FMT_BYTES));
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
            option::GetValue::get_consume(&mut params, mock_get_message(Vec::new(), FMT_STRING)).discard_warning()?;
        assert_eq!(empty_result, Cow::from(""));

        // invalid UTF-8
        let mut invalid: [u8; 2] = [0xa0, 0xa1];
        let mut params = default_getparam();
        let failure: ResultCompErr<Cow<str>, _> =
            option::GetValue::get_consume(&mut params, mock_get_message(invalid.as_mut_slice(), FMT_STRING));
        assert!(matches!(failure, Err(GetStringError::Utf8Parse(_, None))));

        // Failure should be passed through
        let mut params = default_getparam();
        let failure: ResultCompErr<Cow<str>, _> =
            option::GetValue::<_, &mut [u8]>::get_consume(&mut params, mock_get_failure(constants::MQRC_NOT_AUTHORIZED));
        assert!(matches!(
            failure,
            Err(GetStringError::MQ(Error(_, _, constants::MQRC_NOT_AUTHORIZED)))
        ));

        Ok(())
    }

    #[test]
    pub fn get_value_tuple() -> Result<(), Box<dyn std::error::Error>> {
        let mut empty: [u8; 0] = [];
        let mut params = default_getparam();
        let (data, fmt): (Cow<[u8]>, MessageFormat) =
            option::GetValue::get_consume(&mut params, mock_get_message(empty.as_mut_slice(), FMT_BYTES)).discard_warning()?;
        assert_eq!(data, Cow::from(&[]));
        assert_eq!(fmt, FMT_BYTES);

        Ok(())
    }

    #[test]
    pub fn get_option_correlationid() {
        const ID: Identifier<24> = [0xC; 24];
        test_get_option(&mut default_getparam(), &CorrelationId(ID), |p| {
            assert_eq!(p.md.CorrelId, ID);
            assert!(types::MQMO(p.gmo.MatchOptions).contains(constants::MQMO_MATCH_CORREL_ID));
        });
        let mut get_param = default_getparam();
        get_param.gmo.MatchOptions = !0;
        test_get_option(&mut get_param, &CorrelationId(ID), |p| assert_eq!(p.gmo.MatchOptions, !0));
    }

    #[test]
    pub fn get_option_messageid() {
        const ID: Identifier<24> = [0xC; 24];
        test_get_option(&mut default_getparam(), &types::MessageId(ID), |p| {
            assert_eq!(p.md.MsgId, ID);
            assert!(types::MQMO(p.gmo.MatchOptions).contains(constants::MQMO_MATCH_MSG_ID));
        });
        let mut get_param = default_getparam();
        get_param.gmo.MatchOptions = !0;
        test_get_option(&mut get_param, &types::MessageId(ID), |p| assert_eq!(p.gmo.MatchOptions, !0));
    }
    #[test]
    pub fn get_option_groupid() {
        const ID: Identifier<24> = [0xC; 24];
        test_get_option(&mut default_getparam(), &types::GroupId(ID), |p| {
            assert_eq!(p.md.GroupId, ID);
            assert!(types::MQMO(p.gmo.MatchOptions).contains(constants::MQMO_MATCH_GROUP_ID));
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
            assert!(types::MQMO(p.gmo.MatchOptions).contains(constants::MQMO_MATCH_MSG_TOKEN));
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
            assert!(!types::MQGMO(p.gmo.Options).contains(constants::MQGMO_WAIT));
        });
        get_param.gmo.MatchOptions = !0;
        test_get_option(&mut get_param, &GetWait::Wait(50), |p| {
            assert!(types::MQGMO(p.gmo.Options).contains(constants::MQGMO_WAIT));
            assert_eq!(p.gmo.WaitInterval, 50);
        });
    }
}
