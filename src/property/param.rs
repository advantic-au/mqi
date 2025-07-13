#![expect(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]

use core::str;
use std::{mem, num::NonZero, ptr, slice};

use super::option;
use crate::{
    CCSID, Completion, Error, MqStr, ReadRaw, ResultComp, StrCcsidOwned, StringCcsid, constants, conversion,
    macros::{all_multi_tuples, reverse_ident},
    prelude::*,
    structs,
    types::{MQBYTE, MQCHAR, MQCOPY, MQENC, MQFLOAT32, MQFLOAT64, MQIMPO, MQINT8, MQINT16, MQINT64, MQLONG, MQPD, MQTYPE},
};

pub const INQUIRE_ALL: &str = "%";
pub const INQUIRE_ALL_USR: &str = "usr.%";

macro_rules! impl_setproperty_tuple {
    ([$first:ident, $($ty:ident),*]) => {
        #[diagnostic::do_not_recommend]
        impl<$first, $($ty),*> option::SetProperty for ($first, $($ty),*)
        where
            $first: option::SetProperty,
            $($ty: option::SetPropertyAttr),*
        {
            type Data = $first::Data;

            #[allow(non_snake_case,unused_parens)]
            fn apply_mqsetmp(&self, pd: &mut structs::MQPD, smpo: &mut structs::MQSMPO) -> (&Self::Data, MQTYPE) {
                let reverse_ident!($first, $($ty),*) = self;
                $first.apply_mqsetmp(pd, smpo);
                $($ty.apply_mqsetmp(pd, smpo));*
            }
        }
    };
}

impl option::SetPropertyAttr for Attributes {
    fn apply_mqsetmp(&self, pd: &mut structs::MQPD, _smpo: &mut structs::MQSMPO) {
        self.mqpd.clone_into(pd);
    }
}

all_multi_tuples!(impl_setproperty_tuple);

#[derive(Debug, Clone)]
pub struct Attributes {
    mqpd: structs::MQPD,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub length: usize,
    pub ccsid: CCSID,
    pub encoding: MQENC,
    pub value_type: MQTYPE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Null;

#[derive(Debug, Clone, derive_more::Deref, derive_more::DerefMut, derive_more::Constructor)]
pub struct Raw<T> {
    #[deref]
    #[deref_mut]
    data: T,
    metadata: Metadata,
}

impl<T> Raw<T> {
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Int8(MQINT8),
    Int16(MQINT16),
    Int32(MQLONG),
    Int64(MQINT64),
    Float32(MQFLOAT32),
    Float64(MQFLOAT64),
    ByteString(Vec<MQBYTE>),
    String(StrCcsidOwned),
    Null,
}

impl Metadata {
    #[must_use]
    #[allow(clippy::missing_const_for_fn, reason = "false positive - non-const deref")]
    pub fn new(length: usize, impo: &structs::MQIMPO, value_type: MQTYPE) -> Self {
        Self {
            length,
            ccsid: CCSID(impo.ReturnedCCSID),
            encoding: MQENC(impo.ReturnedEncoding),
            value_type,
        }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.length
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    #[must_use]
    pub const fn ccsid(&self) -> CCSID {
        self.ccsid
    }

    #[must_use]
    pub const fn encoding(&self) -> MQENC {
        self.encoding
    }

    #[must_use]
    pub const fn value_type(&self) -> MQTYPE {
        self.value_type
    }
}

unsafe impl option::PropertyAttr for Metadata {
    #[inline]
    fn property_extract<'p, 's, F>(
        param: &mut option::PropertyParam<'p>,
        mqinqmp: F,
    ) -> ResultComp<(Self, option::PropertyState<'s>)>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        mqinqmp(param).map_completion(|state| (Self::new(state.value.len(), &param.impo, param.value_type), state))
    }
}

unsafe impl option::PropertyAttr for Attributes {
    #[inline]
    fn property_extract<'p, 's, F>(
        param: &mut option::PropertyParam<'p>,
        mqinqmp: F,
    ) -> ResultComp<(Self, option::PropertyState<'s>)>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        mqinqmp(param).map_completion(|state| {
            (
                Self {
                    mqpd: param.mqpd.clone(),
                },
                state,
            )
        })
    }
}

impl Attributes {
    #[allow(clippy::missing_const_for_fn, reason = "false positive")]
    pub fn set_support(&mut self, support: MQPD) {
        self.mqpd.Support = support.0;
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn, reason = "false positive")]
    pub fn support(&self) -> MQPD {
        MQPD(self.mqpd.Support)
    }

    #[allow(clippy::missing_const_for_fn, reason = "false positive")]
    pub fn set_context(&mut self, context: MQPD) {
        self.mqpd.Context = context.0;
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn, reason = "false positive")]
    pub fn context(&self) -> MQPD {
        MQPD(self.mqpd.Context)
    }

    #[allow(clippy::missing_const_for_fn, reason = "false positive")]
    pub fn set_copy_options(&mut self, copy_options: MQPD) {
        *self.mqpd.CopyOptions.as_mut() = copy_options;
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn, reason = "false positive")]
    pub fn copy_options(&self) -> MQCOPY {
        MQCOPY(self.mqpd.CopyOptions)
    }
}

macro_rules! impl_primitive_setproptype {
    ($type:ty, $mqtype:path) => {
        impl option::SetProperty for $type {
            type Data = Self;
            fn apply_mqsetmp(&self, _pd: &mut structs::MQPD, smpo: &mut structs::MQSMPO) -> (&Self::Data, MQTYPE) {
                *smpo.ValueEncoding.as_mut() = constants::MQENC_NATIVE;
                (self, $mqtype)
            }
        }
    };
}

impl option::SetProperty for bool {
    type Data = MQLONG;
    fn apply_mqsetmp(&self, _pd: &mut structs::MQPD, smpo: &mut structs::MQSMPO) -> (&Self::Data, MQTYPE) {
        *smpo.ValueEncoding.as_mut() = constants::MQENC_NATIVE;
        (if *self { &1 } else { &0 }, constants::MQTYPE_BOOLEAN)
    }
}

impl_primitive_setproptype!(MQINT8, constants::MQTYPE_INT8);
impl_primitive_setproptype!(MQINT16, constants::MQTYPE_INT16);
impl_primitive_setproptype!(MQLONG, constants::MQTYPE_INT32);
impl_primitive_setproptype!(MQINT64, constants::MQTYPE_INT64);
impl_primitive_setproptype!(MQFLOAT32, constants::MQTYPE_FLOAT32);
impl_primitive_setproptype!(MQFLOAT64, constants::MQTYPE_FLOAT64);
impl_primitive_setproptype!(Null, constants::MQTYPE_NULL);

impl ReadRaw for Null {}

impl option::SetProperty for str {
    type Data = Self;
    fn apply_mqsetmp(&self, _pd: &mut structs::MQPD, smpo: &mut structs::MQSMPO) -> (&Self::Data, MQTYPE) {
        smpo.ValueCCSID = 1208;
        (self, constants::MQTYPE_STRING)
    }
}

impl<T: AsRef<[MQCHAR]>> option::SetProperty for StringCcsid<T> {
    type Data = [MQCHAR];

    fn apply_mqsetmp(&self, _pd: &mut structs::MQPD, smpo: &mut structs::MQSMPO) -> (&Self::Data, MQTYPE) {
        let CCSID(ccsid) = self.ccsid;
        smpo.ValueCCSID = ccsid;
        (self.data.as_ref(), constants::MQTYPE_STRING)
    }
}

impl<const N: usize> option::SetProperty for MqStr<N> {
    type Data = [u8; N];

    fn apply_mqsetmp(&self, _pd: &mut structs::MQPD, smpo: &mut structs::MQSMPO) -> (&Self::Data, MQTYPE) {
        smpo.ValueCCSID = 1208;
        (self.as_bytes(), constants::MQTYPE_STRING)
    }
}

impl option::SetProperty for [MQBYTE] {
    type Data = Self;
    fn apply_mqsetmp(&self, _pd: &mut structs::MQPD, _smpo: &mut structs::MQSMPO) -> (&Self::Data, MQTYPE) {
        (self, constants::MQTYPE_BYTE_STRING)
    }
}

impl option::SetProperty for Value {
    type Data = [u8];

    fn apply_mqsetmp(&self, pd: &mut structs::MQPD, smpo: &mut structs::MQSMPO) -> (&Self::Data, MQTYPE) {
        #[inline]
        /// Ensure the data is of type `[u8]`
        fn set_as_u8<'a, T: option::SetProperty + ?Sized>(
            value: &'a T,
            pd: &mut structs::MQPD,
            smpo: &mut structs::MQSMPO,
        ) -> (&'a [u8], MQTYPE) {
            let (data, value_type) = value.apply_mqsetmp(pd, smpo);
            (
                // SAFETY: Used downstream by the MQ functions
                unsafe { slice::from_raw_parts(ptr::from_ref(data).cast(), mem::size_of_val(data)) },
                value_type,
            )
        }

        match self {
            Self::Boolean(value) => set_as_u8(value, pd, smpo),
            Self::Int8(value) => set_as_u8(value, pd, smpo),
            Self::Int16(value) => set_as_u8(value, pd, smpo),
            Self::Int32(value) => set_as_u8(value, pd, smpo),
            Self::Int64(value) => set_as_u8(value, pd, smpo),
            Self::Float32(value) => set_as_u8(value, pd, smpo),
            Self::Float64(value) => set_as_u8(value, pd, smpo),
            Self::ByteString(value) => set_as_u8(&**value, pd, smpo),
            Self::String(value) => set_as_u8(value, pd, smpo),
            Self::Null => set_as_u8(&Null, pd, smpo),
        }
    }
}

impl From<option::NameUsage> for Option<NonZero<usize>> {
    fn from(value: option::NameUsage) -> Self {
        match value {
            option::NameUsage::MaxLength(length) => Some(length),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, derive_more::Deref, derive_more::DerefMut)]
pub struct Name<T>(pub T);

impl<T: PartialEq<Y>, Y> PartialEq<Name<Y>> for Name<T> {
    fn eq(&self, other: &Name<Y>) -> bool {
        self.0 == other.0
    }
}

unsafe impl option::PropertyAttr for Name<String> {
    fn property_extract<'p, 's, F>(
        param: &mut option::PropertyParam<'p>,
        mqinqmp: F,
    ) -> ResultComp<(Self, option::PropertyState<'s>)>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        param.name_required = option::NameUsage::AnyLength;
        let impo_options: &mut MQIMPO = param.impo.Options.as_mut();
        impo_options.insert(constants::MQIMPO_CONVERT_VALUE);
        match mqinqmp(param)? {
            Completion(_, Some((rc @ constants::MQRC_PROP_NAME_NOT_CONVERTED, verb))) => {
                Err(Error(constants::MQCC_WARNING, verb, rc))
            }
            other => Ok(other.map(|state| {
                // SAFETY: The `expect` will succeed as the Option is always `Some` when
                // `NameUsage::AnyLength` is specified
                let name = conversion::vec_mqchar_to_byte(state.name.clone().expect("Name should not be None").into_owned());
                // SAFETY: The bytes coming from the MQI library should be correct as there
                // is no conversion error (MQRC_PROP_NAME_NOT_CONVERTED)
                (Self(unsafe { String::from_utf8_unchecked(name) }), state)
            })),
        }
    }
}

unsafe impl<const N: usize> option::PropertyAttr for Name<MqStr<N>> {
    fn property_extract<'p, 's, F>(
        param: &mut option::PropertyParam<'p>,
        mqinqmp: F,
    ) -> ResultComp<(Self, option::PropertyState<'s>)>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        let impo_options: &mut MQIMPO = param.impo.Options.as_mut();
        impo_options.insert(constants::MQIMPO_CONVERT_VALUE);
        param.name_required = option::NameUsage::MaxLength(unsafe { NonZero::new_unchecked(N) });
        match mqinqmp(param)? {
            Completion(_, Some((rc @ constants::MQRC_PROP_NAME_NOT_CONVERTED, verb))) => {
                Err(Error(constants::MQCC_WARNING, verb, rc))
            }
            other => Ok(other.map(|state| {
                let name = state.name.as_ref().expect("Name should not be None");
                let name_str = unsafe { str::from_utf8_unchecked(conversion::slice_mqchar_to_byte(name)) };
                (
                    Self(MqStr::from_str(name_str).expect("buffer size should equal required length")),
                    state,
                )
            })),
        }
    }
}

unsafe impl option::PropertyAttr for Name<StrCcsidOwned> {
    fn property_extract<'p, 's, F>(
        param: &mut option::PropertyParam<'p>,
        mqinqmp: F,
    ) -> ResultComp<(Self, option::PropertyState<'s>)>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        param.name_required = option::NameUsage::AnyLength;
        mqinqmp(param).map_completion(|state| {
            let name = state.name.as_ref().expect("Name should not be None");
            (
                Self(StrCcsidOwned {
                    ccsid: CCSID(param.impo.ReturnedName.VSCCSID),
                    le: MQENC(param.impo.ReturnedEncoding).contains(constants::MQENC_INTEGER_REVERSED),
                    data: name.clone().into_owned(),
                }),
                state,
            )
        })
    }
}

unsafe impl option::PropertyValue for Value {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut option::PropertyParam<'p>, mqinqmp: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        let impo_options: &mut MQIMPO = param.impo.Options.as_mut();
        impo_options.insert(constants::MQIMPO_NONE);
        param.value_type = constants::MQTYPE_AS_SET;
        mqinqmp(param).map_completion(|state| match param.value_type {
            constants::MQTYPE_BOOLEAN => Self::Boolean(MQLONG::as_primitive(&state.value) != 0),
            constants::MQTYPE_STRING => Self::String(StringCcsid {
                ccsid: CCSID(param.impo.ReturnedCCSID),
                data: conversion::bytes_to_cow_mqchar(state.value).into_owned(),
                le: MQENC(param.impo.ReturnedEncoding).contains(constants::MQENC_INTEGER_REVERSED),
            }),
            constants::MQTYPE_BYTE_STRING => Self::ByteString(state.value.into()),
            constants::MQTYPE_INT8 => Self::Int8(MQINT8::as_primitive(&state.value)),
            constants::MQTYPE_INT16 => Self::Int16(MQINT16::as_primitive(&state.value)),
            constants::MQTYPE_INT32 => Self::Int32(MQLONG::as_primitive(&state.value)),
            constants::MQTYPE_INT64 => Self::Int64(MQINT64::as_primitive(&state.value)),
            constants::MQTYPE_FLOAT32 => Self::Float32(MQFLOAT32::as_primitive(&state.value)),
            constants::MQTYPE_FLOAT64 => Self::Float64(MQFLOAT64::as_primitive(&state.value)),
            constants::MQTYPE_NULL => Self::Null,
            _ => unreachable!(),
        })
    }
}

macro_rules! impl_primitive_propertyvalue {
    ($type:ty, $mqtype:path) => {
        impl_as_primitive!($type);
        unsafe impl option::PropertyValue for $type {
            type Error = Error;

            fn property_consume<'p, 's, F>(param: &mut option::PropertyParam<'p>, mqinqmp: F) -> ResultComp<Self>
            where
                F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
            {
                param.value_type = $mqtype;
                let impo_options: &mut MQIMPO = param.impo.Options.as_mut();
                impo_options.insert(constants::MQIMPO_CONVERT_VALUE | constants::MQIMPO_CONVERT_TYPE); // TODO: Oh shit. Rework value type to not convert
                match mqinqmp(param)? {
                    Completion(_, Some((rc @ constants::MQRC_PROP_VALUE_NOT_CONVERTED, verb))) => {
                        Err(Error(constants::MQCC_WARNING, verb, rc))
                    }
                    other => Ok(other.map(|state| Self::as_primitive(&*state.value))),
                }
            }
        }
    };
}

macro_rules! impl_as_primitive {
    ($type:ty) => {
        impl AsPrimitive for $type {
            fn as_primitive(buffer: &[u8]) -> Self {
                Self::from_ne_bytes(buffer.try_into().expect("buffer size should exceed required length"))
            }
        }
    };
}
trait AsPrimitive {
    fn as_primitive(buffer: &[u8]) -> Self;
}

impl_primitive_propertyvalue!(MQFLOAT32, constants::MQTYPE_FLOAT32);
impl_primitive_propertyvalue!(MQFLOAT64, constants::MQTYPE_FLOAT64);
impl_primitive_propertyvalue!(MQINT8, constants::MQTYPE_INT8);
impl_primitive_propertyvalue!(MQINT16, constants::MQTYPE_INT16);
impl_primitive_propertyvalue!(MQLONG, constants::MQTYPE_INT32);
impl_primitive_propertyvalue!(MQINT64, constants::MQTYPE_INT64);

unsafe impl option::PropertyValue for bool {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut option::PropertyParam<'p>, mqinqmp: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        param.value_type = constants::MQTYPE_BOOLEAN;
        let impo_options: &mut MQIMPO = param.impo.Options.as_mut();
        impo_options.insert(constants::MQIMPO_CONVERT_TYPE);
        mqinqmp(param).map_completion(|state| MQLONG::as_primitive(&state.value) != 0)
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(mem::size_of::<MQLONG>())
    }
}

unsafe impl option::PropertyValue for Vec<MQBYTE> {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut option::PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        let impo_options: &mut MQIMPO = param.impo.Options.as_mut();
        impo_options.insert(constants::MQIMPO_CONVERT_TYPE);
        param.value_type = constants::MQTYPE_BYTE_STRING;
        mqinqmp(param).map_completion(|state| state.value.into())
    }
}

unsafe impl<const N: usize> option::PropertyValue for [u8; N] {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut option::PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        let impo_options: &mut MQIMPO = param.impo.Options.as_mut();
        impo_options.insert(constants::MQIMPO_CONVERT_TYPE);
        param.value_type = constants::MQTYPE_BYTE_STRING;
        mqinqmp(param).map_completion(|state| {
            let mut result: [u8; N] = [0; N];
            result.copy_from_slice(&state.value);
            result
        })
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(N)
    }
}

unsafe impl<const N: usize> option::PropertyValue for MqStr<N> {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut option::PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        let impo_options: &mut MQIMPO = param.impo.Options.as_mut();
        impo_options.insert(constants::MQIMPO_CONVERT_VALUE | constants::MQIMPO_CONVERT_TYPE);
        param.value_type = constants::MQTYPE_STRING;
        mqinqmp(param).map_completion(|state| {
            let value_str = unsafe { str::from_utf8_unchecked(&state.value) };
            Self::from_str(value_str).expect("buffer size should equal required length")
        })
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(N)
    }
}

impl<T: AsRef<[u8]>> option::SetProperty for Raw<T> {
    type Data = [u8];

    fn apply_mqsetmp(&self, _pd: &mut structs::MQPD, smpo: &mut structs::MQSMPO) -> (&Self::Data, MQTYPE) {
        let encoding: &mut MQENC = smpo.ValueEncoding.as_mut();
        *encoding = self.metadata.encoding;
        *smpo.ValueCCSID.as_mut() = self.metadata.ccsid;
        (&self.data.as_ref()[..self.metadata.length], self.metadata.value_type)
    }
}

unsafe impl option::PropertyValue for Raw<Vec<u8>> {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut option::PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        let impo_options: &mut MQIMPO = param.impo.Options.as_mut();
        impo_options.insert(constants::MQIMPO_NONE);
        param.value_type = constants::MQTYPE_AS_SET;
        mqinqmp(param).map_completion(|state| {
            let len = state.value.len();
            Self::new(state.value.into_owned(), Metadata::new(len, &param.impo, param.value_type))
        })
    }
}

unsafe impl<const N: usize> option::PropertyValue for Raw<[u8; N]> {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut option::PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        let impo_options: &mut MQIMPO = param.impo.Options.as_mut();
        impo_options.insert(constants::MQIMPO_NONE);
        param.value_type = constants::MQTYPE_AS_SET;
        mqinqmp(param).map_completion(|state| {
            let mut data: [u8; N] = [0; N];
            data[..state.value.len()].copy_from_slice(&state.value);
            let len = data.len();
            Self::new(data, Metadata::new(len, &param.impo, param.value_type))
        })
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(N)
    }
}

unsafe impl option::PropertyValue for String {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut option::PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        let impo_options: &mut MQIMPO = param.impo.Options.as_mut();
        param.value_type = constants::MQTYPE_STRING;
        impo_options.insert(constants::MQIMPO_CONVERT_VALUE | constants::MQIMPO_CONVERT_TYPE);
        match mqinqmp(param)? {
            Completion(_, Some((rc @ constants::MQRC_PROP_VALUE_NOT_CONVERTED, verb))) => {
                Err(Error(constants::MQCC_WARNING, verb, rc))
            }
            // SAFETY: The bytes coming from the MQI library must be correct as there
            // is no conversion error
            other => Ok(other.map(|state| unsafe { str::from_utf8_unchecked(&state.value).to_string() })),
        }
    }
}

unsafe impl option::PropertyValue for StrCcsidOwned {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut option::PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
    {
        let impo_options: &mut MQIMPO = param.impo.Options.as_mut();
        impo_options.insert(constants::MQIMPO_CONVERT_TYPE);
        param.value_type = constants::MQTYPE_STRING;
        mqinqmp(param).map_completion(|state| Self {
            ccsid: CCSID(param.impo.ReturnedCCSID),
            data: conversion::bytes_to_cow_mqchar(state.value).into_owned(),
            le: MQENC(param.impo.ReturnedEncoding).contains(constants::MQENC_INTEGER_REVERSED),
        })
    }
}

#[expect(unused_parens)]
mod impl_property {
    use super::{all_multi_tuples, option};
    use crate::{ResultComp, ResultCompErr, prelude::*};

    macro_rules! impl_propertyvalue_tuple {
        ([$first:ident, $($ty:ident),*]) => {
            #[diagnostic::do_not_recommend]
            unsafe impl<$first, $($ty),*> option::PropertyValue for ($first, $($ty),*)
            where
                $first: option::PropertyValue,
                $($ty: option::PropertyAttr),*
            {
                type Error = $first::Error;

                #[expect(non_snake_case)]
                #[inline]
                fn property_consume<'p, 's, F>(param: &mut option::PropertyParam<'p>, mqi: F) -> ResultCompErr<Self, Self::Error>
                where
                    F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>,
                {
                    let mut rest_outer = None;
                    $first::property_consume(param, |param| {
                        <($($ty),*) as option::PropertyAttr>::property_extract(param, mqi).map_completion(|(rest, state)| {
                            rest_outer = Some(rest);
                            state
                        })
                    })
                    .map_completion(|a| {
                        let ($($ty),*) = rest_outer.expect("rest_outer should be set by extract closure");
                        (a, $($ty),*)
                    })
                }

                fn max_value_size() -> Option<std::num::NonZero<usize>> {
                    $first::max_value_size()
                }
            }

        }
    }

    macro_rules! impl_propertyattr_tuple {
        ([$first:ident, $($ty:ident),*]) => {
            #[diagnostic::do_not_recommend]
            unsafe impl<$first, $($ty),*> option::PropertyAttr for ($first, $($ty),*)
            where
                $first: option::PropertyAttr,
                $($ty: option::PropertyAttr),*
            {
                #[expect(non_snake_case)]
                #[inline]
                fn property_extract<'p, 's, F>(param: &mut option::PropertyParam<'p>, mqi: F) -> ResultComp<(Self, option::PropertyState<'s>)>
                where
                    F: FnOnce(&mut option::PropertyParam<'p>) -> ResultComp<option::PropertyState<'s>>
                {
                    let mut rest_outer = None;
                    $first::property_extract(param, |param| {
                        <($($ty),*) as option::PropertyAttr>::property_extract(param, mqi).map_completion(|(rest, state)| {
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

    all_multi_tuples!(impl_propertyvalue_tuple);
    all_multi_tuples!(impl_propertyattr_tuple);
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{borrow::Cow, error::Error, num::NonZero};

    use libmqm_default as default;

    use super::*;
    use crate::{
        Completion, MqStr, ResultComp, ResultCompErr, ResultCompExt, StrCcsid, StrCcsidOwned, conversion::slice_byte_to_mqchar,
        mqstr, types,
    };

    #[test]
    #[allow(clippy::float_cmp)]
    fn property_value() {
        const BOOL_BYTES: &[u8] = &1i32.to_ne_bytes();

        fn assert_primitive_pv<P: option::PropertyValue + std::cmp::PartialEq + Copy>(
            value: &[u8],
            expected_type: MQTYPE,
            expected_value: P,
        ) {
            assert!(
                execute_pv::<P>(|param| {
                    assert!(
                        MQIMPO(param.impo.Options).contains(constants::MQIMPO_CONVERT_TYPE | constants::MQIMPO_CONVERT_VALUE)
                    );
                    assert_eq!(param.value_type, expected_type);
                    value_property_state(value)
                })
                .is_ok_and(|Completion((value, _), ..)| value == expected_value)
            );
        }

        assert_primitive_pv(&99i8.to_ne_bytes(), constants::MQTYPE_INT8, 99i8);
        assert_primitive_pv(&99i16.to_ne_bytes(), constants::MQTYPE_INT16, 99i16);
        assert_primitive_pv(&99i32.to_ne_bytes(), constants::MQTYPE_INT32, 99i32);
        assert_primitive_pv(&99i64.to_ne_bytes(), constants::MQTYPE_INT64, 99i64);
        assert_primitive_pv(&99f32.to_ne_bytes(), constants::MQTYPE_FLOAT32, 99f32);
        assert_primitive_pv(&99f64.to_ne_bytes(), constants::MQTYPE_FLOAT64, 99f64);

        assert!(
            execute_pv::<bool>(|param| {
                assert!(MQIMPO(param.impo.Options).contains(constants::MQIMPO_CONVERT_TYPE));
                assert_eq!(param.value_type, constants::MQTYPE_BOOLEAN);
                value_property_state(BOOL_BYTES)
            })
            .is_ok_and(|Completion((value, _), ..)| value)
        );

        assert!(
            execute_pv::<Vec<u8>>(|param| {
                assert!(MQIMPO(param.impo.Options).contains(constants::MQIMPO_CONVERT_TYPE));
                assert_eq!(param.value_type, constants::MQTYPE_BYTE_STRING);
                value_property_state(b"test")
            })
            .is_ok_and(|Completion((value, _), ..)| value == b"test")
        );

        assert!(
            execute_pv::<String>(|param| {
                assert!(MQIMPO(param.impo.Options).contains(constants::MQIMPO_CONVERT_TYPE | constants::MQIMPO_CONVERT_VALUE));
                assert_eq!(param.value_type, constants::MQTYPE_STRING);
                value_property_state(b"test")
            })
            .is_ok_and(|Completion((value, _), ..)| value == "test")
        );

        assert!(
            execute_pv::<StrCcsidOwned>(|param| {
                assert!(MQIMPO(param.impo.Options).contains(constants::MQIMPO_CONVERT_TYPE));
                assert!(!MQIMPO(param.impo.Options).contains(constants::MQIMPO_CONVERT_VALUE));
                assert_eq!(param.value_type, constants::MQTYPE_STRING);
                param.impo.ReturnedCCSID = 1208;
                value_property_state(b"test")
            })
            .is_ok_and(|Completion((value, _), ..)| value.ccsid == 1208 && value.data == slice_byte_to_mqchar(b"test"))
        );

        assert!(
            execute_pv::<Raw<Vec<u8>>>(|param| {
                assert_eq!(param.value_type, constants::MQTYPE_AS_SET);
                param.value_type = constants::MQTYPE_STRING;
                value_property_state(b"test")
            })
            .is_ok_and(|Completion((value, _), ..)| &*value == b"test" && value.metadata.value_type == constants::MQTYPE_STRING)
        );

        assert!(
            execute_pv::<Raw<[u8; 4]>>(|param| {
                assert_eq!(param.value_type, constants::MQTYPE_AS_SET);
                param.value_type = constants::MQTYPE_STRING;
                value_property_state(b"test")
            })
            .is_ok_and(|Completion((value, _), ..)| &*value == b"test" && value.metadata.value_type == constants::MQTYPE_STRING)
        );

        assert!(
            execute_pv::<[u8; 4]>(|param| {
                assert!(MQIMPO(param.impo.Options).contains(constants::MQIMPO_CONVERT_TYPE));
                assert_eq!(param.value_type, constants::MQTYPE_BYTE_STRING);
                value_property_state(b"test")
            })
            .is_ok_and(|Completion((value, _), ..)| &value == b"test")
        );

        assert!(
            execute_pv::<Value>(|param| {
                param.value_type = constants::MQTYPE_STRING;
                param.impo.ReturnedCCSID = 1208;
                value_property_state(b"test")
            })
            .is_ok_and(|Completion((value, _), ..)| matches!(value, Value::String(s) if s.ccsid == 1208 && s.data == slice_byte_to_mqchar(b"test")))
        );

        assert!(
            execute_pv::<Value>(|param| {
                param.value_type = constants::MQTYPE_NULL;
                value_property_state(b"")
            })
            .is_ok_and(|Completion((value, _), ..)| matches!(value, Value::Null))
        );
    }

    #[test]
    fn set_property() {
        test_sp("test", |_, smpo, data, mq_type| {
            assert_eq!("test", data);
            assert_eq!(constants::MQTYPE_STRING, mq_type);
            assert_eq!(smpo.ValueCCSID, 1208);
        });

        let mqstr_sub: MqStr<8> = mqstr!("test");
        test_sp(&mqstr_sub, |_, smpo, data, mq_type| {
            assert_eq!(mqstr_sub.as_bytes(), data);
            assert_eq!(constants::MQTYPE_STRING, mq_type);
            assert_eq!(smpo.ValueCCSID, 1208);
        });

        let encoded_str = StrCcsid::from("test");
        test_sp(&encoded_str, |_, smpo, data, mq_type| {
            assert_eq!(encoded_str.data, data);
            assert_eq!(constants::MQTYPE_STRING, mq_type);
            assert_eq!(smpo.ValueCCSID, encoded_str.ccsid.0);
        });

        let byte_str = b"test";
        test_sp(byte_str.as_slice(), |_, _, data, mq_type| {
            assert_eq!(byte_str, data);
            assert_eq!(constants::MQTYPE_BYTE_STRING, mq_type);
        });

        test_sp(&false, |_, _, data, mq_type| {
            assert_eq!(&0, data);
            assert_eq!(constants::MQTYPE_BOOLEAN, mq_type);
        });

        test_sp(&true, |_, _, data, mq_type| {
            assert_eq!(&1, data);
            assert_eq!(constants::MQTYPE_BOOLEAN, mq_type);
        });

        test_simple_sp::<MQINT8>(&99, constants::MQTYPE_INT8);
        test_simple_sp::<MQINT16>(&99, constants::MQTYPE_INT16);
        test_simple_sp::<MQFLOAT32>(&99.0, constants::MQTYPE_FLOAT32);
        test_simple_sp::<MQFLOAT64>(&99.0, constants::MQTYPE_FLOAT64);
        test_simple_sp::<MQLONG>(&99, constants::MQTYPE_INT32);
        test_simple_sp::<MQINT64>(&99, constants::MQTYPE_INT64);

        test_sp(&Value::Null, |_, _, _, mq_type| {
            assert_eq!(mq_type, constants::MQTYPE_NULL);
        });

        test_sp(&Value::ByteString(b"test".into()), |_, _, _, mq_type| {
            assert_eq!(mq_type, constants::MQTYPE_BYTE_STRING);
        });

        test_sp(&Value::String("test".into()), |_, smpo, _, mq_type| {
            assert_eq!(smpo.ValueCCSID, 1208);
            assert_eq!(mq_type, constants::MQTYPE_STRING);
        });

        test_sp(&Value::Float32(99.0), |_, _, _, mq_type| {
            assert_eq!(mq_type, constants::MQTYPE_FLOAT32);
        });

        test_sp(&Value::Float64(99.0), |_, _, _, mq_type| {
            assert_eq!(mq_type, constants::MQTYPE_FLOAT64);
        });

        test_sp(&Value::Boolean(false), |_, _, _, mq_type| {
            assert_eq!(mq_type, constants::MQTYPE_BOOLEAN);
        });

        test_sp(&Value::Int8(99), |_, _, _, mq_type| {
            assert_eq!(mq_type, constants::MQTYPE_INT8);
        });

        test_sp(&Value::Int16(99), |_, _, _, mq_type| {
            assert_eq!(mq_type, constants::MQTYPE_INT16);
        });

        test_sp(&Value::Int32(99), |_, _, _, mq_type| {
            assert_eq!(mq_type, constants::MQTYPE_INT32);
        });

        test_sp(&Value::Int64(99), |_, _, _, mq_type| {
            assert_eq!(mq_type, constants::MQTYPE_INT64);
        });
    }

    fn test_simple_sp<S>(s: &S, mq_type: MQTYPE)
    where
        S::Data: PartialEq<S> + std::fmt::Debug,
        S: option::SetProperty + std::fmt::Debug,
    {
        test_sp(s, |_, _, data, t| {
            assert_eq!(data, s);
            assert_eq!(t, mq_type);
        });
    }

    fn test_sp<S: option::SetProperty + ?Sized>(sp: &S, f: impl FnOnce(&structs::MQPD, &structs::MQSMPO, &S::Data, MQTYPE)) {
        let mut pd = structs::MQPD::new(default::MQPD_DEFAULT);
        let mut smpo = structs::MQSMPO::new(default::MQSMPO_DEFAULT);
        let (data, mq_type) = sp.apply_mqsetmp(&mut pd, &mut smpo);
        f(&pd, &smpo, data, mq_type);
    }

    fn execute_pv<'a, P: option::PropertyValue>(
        f: impl FnMut(&mut option::PropertyParam) -> ResultComp<option::PropertyState<'a>>,
    ) -> ResultCompErr<(P, option::PropertyParam<'a>), P::Error> {
        let mut param = option::PropertyParam {
            impo: structs::MQIMPO::new(default::MQIMPO_DEFAULT),
            value_type: MQTYPE::default(),
            mqpd: structs::MQPD::new(default::MQPD_DEFAULT),
            name_required: option::NameUsage::default(),
        };
        P::property_consume(&mut param, f).map_completion(|value| (value, param))
    }

    #[allow(clippy::unnecessary_wraps)]
    const fn value_property_state(bv: &[u8]) -> ResultComp<option::PropertyState<'_>> {
        Ok(Completion::new(option::PropertyState {
            name: None,
            value: Cow::Borrowed(bv),
        }))
    }

    #[test]
    fn property_attr() -> Result<(), Box<dyn Error>> {
        let (attribute, _) = execute_pa::<Attributes>(|param| {
            param.mqpd.Context = 99;
            Ok(Completion::new(option::PropertyState {
                name: None,
                value: Cow::from(b"test"),
            }))
        })
        .warn_as_error()?;
        assert_eq!(99, attribute.mqpd.Context);

        let (metadata, _) = execute_pa::<Metadata>(|param| {
            param.value_type = constants::MQTYPE_STRING;
            param.impo.ReturnedCCSID = 1208;
            *param.impo.ReturnedEncoding.as_mut() = constants::MQENC_INTEGER_NORMAL;
            Ok(Completion::new(option::PropertyState {
                name: None,
                value: Cow::from(b"test"),
            }))
        })
        .warn_as_error()?;
        assert_eq!(4, metadata.length);
        assert_eq!(CCSID(1208), metadata.ccsid);
        assert_eq!(constants::MQENC_INTEGER_NORMAL, metadata.encoding);

        Ok(())
    }

    #[test]
    fn property_attr_name() -> Result<(), Box<dyn Error>> {
        #[expect(clippy::unnecessary_wraps)]
        fn name_state(name: &[u8]) -> ResultComp<option::PropertyState<'_>> {
            Ok(Completion::new(option::PropertyState {
                name: Some(Cow::from(slice_byte_to_mqchar(name))),
                value: Cow::from(b""),
            }))
        }

        #[expect(clippy::unnecessary_wraps)]
        fn name_state_warning(name: &[u8]) -> ResultComp<option::PropertyState<'_>> {
            Ok(Completion::new_warning(
                option::PropertyState {
                    name: Some(Cow::from(slice_byte_to_mqchar(name))),
                    value: Cow::from(b""),
                },
                (constants::MQRC_PROP_NAME_NOT_CONVERTED, ""),
            ))
        }

        let (name, _) = execute_pa::<Name<String>>(|param| {
            assert_eq!(param.name_required, option::NameUsage::AnyLength);
            assert!(MQIMPO(param.impo.Options).contains(constants::MQIMPO_CONVERT_VALUE));
            name_state(b"name")
        })
        .warn_as_error()?;
        assert_eq!(name, Name("name"));

        execute_pa::<Name<String>>(|_| name_state_warning(b"name")).expect_err("should return error");
        execute_pa::<Name<MqStr<25>>>(|_| name_state_warning(b"name")).expect_err("should return error");

        let (name, _) = execute_pa::<Name<MqStr<25>>>(|param| {
            assert_eq!(
                param.name_required,
                option::NameUsage::MaxLength(unsafe { NonZero::new_unchecked(25) })
            );
            assert!(types::MQIMPO(param.impo.Options).contains(constants::MQIMPO_CONVERT_VALUE));
            name_state(b"name")
        })
        .warn_as_error()?;
        assert_eq!(name, Name("name"));

        let (name, _) = execute_pa::<Name<StrCcsidOwned>>(|param| {
            param.impo.ReturnedName.VSCCSID = 1208;
            assert_eq!(param.name_required, option::NameUsage::AnyLength);
            assert!(!types::MQIMPO(param.impo.Options).contains(constants::MQIMPO_CONVERT_VALUE));
            name_state(b"name")
        })
        .warn_as_error()?;
        assert_eq!(name, Name("name"));

        Ok(())
    }
    fn execute_pa<'a, A: option::PropertyAttr>(
        f: impl FnOnce(&mut option::PropertyParam<'_>) -> ResultComp<option::PropertyState<'a>>,
    ) -> ResultComp<(A, option::PropertyState<'a>)> {
        let mut param = option::PropertyParam {
            impo: structs::MQIMPO::new(default::MQIMPO_DEFAULT),
            value_type: MQTYPE::default(),
            mqpd: structs::MQPD::new(default::MQPD_DEFAULT),
            name_required: option::NameUsage::default(),
        };

        A::property_extract(&mut param, |p| f(p))
    }
}
