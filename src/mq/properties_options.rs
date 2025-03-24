#![expect(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]

use core::str;
use std::{mem, ptr, slice};
use std::{borrow::Cow, num::NonZero};

use crate::conversion;
use crate::core::ReadRaw;
use crate::macros::reverse_ident;

use libmqm_sys::lib::MQTYPE_STRING;

use crate::macros::all_multi_tuples;
use crate::{prelude::*, ResultCompErr};
use crate::{sys, Completion, Error, MqStr, MqStruct, ResultComp, StrCcsidOwned, StringCcsid};
use crate::values::{self, CCSID, MQENC, MQTYPE};

pub const INQUIRE_ALL: &str = "%";
pub const INQUIRE_ALL_USR: &str = "usr.%";

#[derive(Debug, Clone)]
pub struct PropertyState<'s> {
    pub name: Option<Cow<'s, [sys::MQCHAR]>>,
    pub value: Cow<'s, [u8]>,
}

#[derive(Clone, Debug)]
pub struct PropertyParam<'p> {
    pub value_type: MQTYPE,
    pub impo: MqStruct<'p, sys::MQIMPO>,
    pub mqpd: MqStruct<'static, sys::MQPD>,
    pub name_required: NameUsage,
}

pub trait PropertyValue {
    type Error: From<Error> + Into<Error> + std::fmt::Debug;

    fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqi: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
        Self: std::marker::Sized;

    #[must_use]
    fn max_value_size() -> Option<NonZero<usize>> {
        None
    }
}

pub trait PropertyAttr {
    fn property_extract<'p, 's, F>(param: &mut PropertyParam<'p>, mqi: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
        Self: Sized;
}

pub trait SetProperty {
    type Data: ReadRaw + ?Sized;
    fn apply_mqsetmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MQTYPE);
}

pub trait SetPropertyAttr {
    fn apply_mqsetmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>);
}

macro_rules! impl_setproperty_tuple {
    ([$first:ident, $($ty:ident),*]) => {
        #[diagnostic::do_not_recommend]
        impl<$first, $($ty),*> SetProperty for ($first, $($ty),*)
        where
            $first: SetProperty,
            $($ty: SetPropertyAttr),*
        {
            type Data = $first::Data;

            #[allow(non_snake_case,unused_parens)]
            fn apply_mqsetmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MQTYPE) {
                let reverse_ident!($first, $($ty),*) = self;
                $first.apply_mqsetmp(pd, smpo);
                $($ty.apply_mqsetmp(pd, smpo));*
            }
        }
    };
}

impl SetPropertyAttr for Attributes {
    fn apply_mqsetmp(&self, pd: &mut MqStruct<sys::MQPD>, _smpo: &mut MqStruct<sys::MQSMPO>) {
        self.mqpd.clone_into(pd);
    }
}

all_multi_tuples!(impl_setproperty_tuple);

#[derive(Debug, Clone)]
pub struct Attributes {
    mqpd: MqStruct<'static, sys::MQPD>,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub length: usize,
    pub ccsid: sys::MQLONG,
    pub encoding: MQENC,
    pub value_type: MQTYPE,
}

#[derive(Debug, Clone, Copy)]
pub struct Null;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NameUsage {
    #[default]
    Ignored,
    MaxLength(NonZero<usize>),
    AnyLength,
}

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
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    ByteString(Vec<sys::MQBYTE>),
    String(StrCcsidOwned),
    Null,
}

impl Metadata {
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(length: usize, impo: &MqStruct<sys::MQIMPO>, value_type: values::MQTYPE) -> Self {
        Self {
            length,
            ccsid: impo.ReturnedCCSID,
            encoding: values::MQENC(impo.ReturnedEncoding),
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
    pub const fn ccsid(&self) -> sys::MQLONG {
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

impl PropertyAttr for Metadata {
    #[inline]
    fn property_extract<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        mqinqmp(param).map_completion(|state| (Self::new(state.value.len(), &param.impo, param.value_type), state))
    }
}

impl PropertyAttr for Attributes {
    #[inline]
    fn property_extract<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
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
    pub fn set_support(&mut self, support: values::MQPD) {
        self.mqpd.Support = support.value();
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn, reason = "false positive")]
    pub fn support(&self) -> values::MQPD {
        values::MQPD(self.mqpd.Support)
    }

    #[allow(clippy::missing_const_for_fn, reason = "false positive")]
    pub fn set_context(&mut self, context: values::MQPD) {
        self.mqpd.Context = context.value();
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn, reason = "false positive")]
    pub fn context(&self) -> values::MQPD {
        values::MQPD(self.mqpd.Context)
    }

    #[allow(clippy::missing_const_for_fn, reason = "false positive")]
    pub fn set_copy_options(&mut self, copy_options: values::MQPD) {
        self.mqpd.CopyOptions = copy_options.value();
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn, reason = "false positive")]
    pub fn copy_options(&self) -> values::MQCOPY {
        values::MQCOPY(self.mqpd.CopyOptions)
    }
}

macro_rules! impl_primitive_setproptype {
    ($type:ty, $mqtype:path) => {
        impl SetProperty for $type {
            type Data = Self;
            fn apply_mqsetmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MQTYPE) {
                smpo.ValueEncoding = sys::MQENC_NATIVE;
                (self, values::MQTYPE($mqtype))
            }
        }
    };
}

impl SetProperty for bool {
    type Data = sys::MQLONG;
    fn apply_mqsetmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MQTYPE) {
        smpo.ValueEncoding = sys::MQENC_NATIVE;
        (if *self { &1 } else { &0 }, values::MQTYPE(sys::MQTYPE_BOOLEAN))
    }
}

impl_primitive_setproptype!(i8, sys::MQTYPE_INT8);
impl_primitive_setproptype!(i16, sys::MQTYPE_INT16);
impl_primitive_setproptype!(i32, sys::MQTYPE_INT32);
impl_primitive_setproptype!(i64, sys::MQTYPE_INT64);
impl_primitive_setproptype!(f32, sys::MQTYPE_FLOAT32);
impl_primitive_setproptype!(f64, sys::MQTYPE_FLOAT64);
impl_primitive_setproptype!(Null, sys::MQTYPE_NULL);

impl ReadRaw for Null {}

impl SetProperty for str {
    type Data = Self;
    fn apply_mqsetmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MQTYPE) {
        smpo.ValueCCSID = 1208;
        (self, MQTYPE(sys::MQTYPE_STRING))
    }
}

impl<T: AsRef<[sys::MQCHAR]>> SetProperty for StringCcsid<T> {
    type Data = [sys::MQCHAR];

    fn apply_mqsetmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MQTYPE) {
        let CCSID(ccsid) = self.ccsid;
        smpo.ValueCCSID = ccsid;
        (self.data.as_ref(), values::MQTYPE(MQTYPE_STRING))
    }
}

impl<const N: usize> SetProperty for MqStr<N> {
    type Data = [u8; N];

    fn apply_mqsetmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MQTYPE) {
        smpo.ValueCCSID = 1208;
        (self.as_bytes(), MQTYPE(sys::MQTYPE_STRING))
    }
}

impl SetProperty for [sys::MQBYTE] {
    type Data = Self;
    fn apply_mqsetmp(&self, _pd: &mut MqStruct<sys::MQPD>, _smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MQTYPE) {
        (self, MQTYPE(sys::MQTYPE_BYTE_STRING))
    }
}

impl SetProperty for Value {
    type Data = [u8];

    fn apply_mqsetmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MQTYPE) {
        #[inline]
        /// Ensure the data is of type `[u8]`
        fn set_as_u8<'a, T: SetProperty + ?Sized>(
            value: &'a T,
            pd: &mut MqStruct<sys::MQPD>,
            smpo: &mut MqStruct<sys::MQSMPO>,
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

impl From<NameUsage> for Option<NonZero<usize>> {
    fn from(value: NameUsage) -> Self {
        match value {
            NameUsage::MaxLength(length) => Some(length),
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

impl PropertyAttr for Name<String> {
    fn property_extract<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.name_required = NameUsage::AnyLength;
        param.impo.Options |= sys::MQIMPO_CONVERT_VALUE;
        match mqinqmp(param)? {
            Completion(_, Some((rc @ values::MQRC(sys::MQRC_PROP_NAME_NOT_CONVERTED), verb))) => {
                Err(Error(values::MQCC(sys::MQCC_WARNING), verb, rc))
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

impl<const N: usize> PropertyAttr for Name<MqStr<N>> {
    fn property_extract<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.name_required = NameUsage::MaxLength(unsafe { NonZero::new_unchecked(N) });
        param.impo.Options |= sys::MQIMPO_CONVERT_VALUE;
        match mqinqmp(param)? {
            Completion(_, Some((rc @ values::MQRC(sys::MQRC_PROP_NAME_NOT_CONVERTED), verb))) => {
                Err(Error(values::MQCC(sys::MQCC_WARNING), verb, rc))
            }
            other => Ok(other.map(|state| {
                let name = state.name.as_ref().expect("Name should not be None");
                (
                    Self(MqStr::from_mqchar_slice(name).expect("buffer size should equal required length")),
                    state,
                )
            })),
        }
    }
}

impl PropertyAttr for Name<StrCcsidOwned> {
    fn property_extract<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.name_required = NameUsage::AnyLength;
        mqinqmp(param).map_completion(|state| {
            let name = state.name.as_ref().expect("Name should not be None");
            (
                Self(StrCcsidOwned {
                    ccsid: CCSID(param.impo.ReturnedName.VSCCSID),
                    le: (param.impo.ReturnedEncoding & sys::MQENC_INTEGER_REVERSED) != 0,
                    data: name.clone().into_owned(),
                }),
                state,
            )
        })
    }
}

impl PropertyValue for Value {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MQTYPE(sys::MQTYPE_AS_SET);
        param.impo.Options |= sys::MQIMPO_NONE;
        mqinqmp(param).map_completion(|state| match param.value_type.value() {
            sys::MQTYPE_BOOLEAN => Self::Boolean(i32::as_primitive(&state.value) != 0),
            sys::MQTYPE_STRING => Self::String(StringCcsid {
                ccsid: CCSID(param.impo.ReturnedCCSID),
                data: conversion::bytes_to_cow_mqchar(state.value).into_owned(),
                le: (param.impo.ReturnedEncoding & sys::MQENC_INTEGER_REVERSED) != 0,
            }),
            sys::MQTYPE_BYTE_STRING => Self::ByteString(state.value.into()),
            sys::MQTYPE_INT8 => Self::Int8(i8::as_primitive(&state.value)),
            sys::MQTYPE_INT16 => Self::Int16(i16::as_primitive(&state.value)),
            sys::MQTYPE_INT32 => Self::Int32(i32::as_primitive(&state.value)),
            sys::MQTYPE_INT64 => Self::Int64(i64::as_primitive(&state.value)),
            sys::MQTYPE_FLOAT32 => Self::Float32(f32::as_primitive(&state.value)),
            sys::MQTYPE_FLOAT64 => Self::Float64(f64::as_primitive(&state.value)),
            sys::MQTYPE_NULL => Self::Null,
            _ => unreachable!(),
        })
    }
}

macro_rules! impl_primitive_propertyvalue {
    ($type:ty, $mqtype:path) => {
        impl_as_primitive!($type);
        impl PropertyValue for $type {
            type Error = Error;

            fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<Self>
            where
                F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
            {
                param.value_type = MQTYPE($mqtype);
                param.impo.Options |= sys::MQIMPO_CONVERT_VALUE | sys::MQIMPO_CONVERT_TYPE; // TODO: Oh shit. Rework value type to not convert
                match mqinqmp(param)? {
                    Completion(_, Some((rc @ values::MQRC(sys::MQRC_PROP_VALUE_NOT_CONVERTED), verb))) => {
                        Err(Error(values::MQCC(sys::MQCC_WARNING), verb, rc))
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

impl_primitive_propertyvalue!(f32, sys::MQTYPE_FLOAT32);
impl_primitive_propertyvalue!(f64, sys::MQTYPE_FLOAT64);
impl_primitive_propertyvalue!(i8, sys::MQTYPE_INT8);
impl_primitive_propertyvalue!(i16, sys::MQTYPE_INT16);
impl_primitive_propertyvalue!(sys::MQLONG, sys::MQTYPE_INT32);
impl_primitive_propertyvalue!(sys::MQINT64, sys::MQTYPE_INT64);

impl PropertyValue for bool {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MQTYPE(sys::MQTYPE_BOOLEAN);
        param.impo.Options |= sys::MQIMPO_CONVERT_TYPE;
        mqinqmp(param).map_completion(|state| sys::MQLONG::as_primitive(&state.value) != 0)
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(mem::size_of::<sys::MQLONG>())
    }
}

impl PropertyValue for Vec<sys::MQBYTE> {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MQTYPE(sys::MQTYPE_BYTE_STRING);
        param.impo.Options |= sys::MQIMPO_CONVERT_TYPE;
        mqinqmp(param).map_completion(|state| state.value.into())
    }
}

impl<const N: usize> PropertyValue for [u8; N] {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MQTYPE(sys::MQTYPE_BYTE_STRING);
        param.impo.Options |= sys::MQIMPO_CONVERT_TYPE;
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

impl<const N: usize> PropertyValue for MqStr<N> {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MQTYPE(sys::MQTYPE_BYTE_STRING);
        param.impo.Options |= sys::MQIMPO_CONVERT_VALUE | sys::MQIMPO_CONVERT_TYPE;
        mqinqmp(param)
            .map_completion(|state| Self::from_byte_slice(&state.value).expect("buffer size should equal required length"))
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(N)
    }
}

impl<T: AsRef<[u8]>> SetProperty for Raw<T> {
    type Data = [u8];

    fn apply_mqsetmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MQTYPE) {
        smpo.ValueCCSID = self.metadata.ccsid;
        smpo.ValueEncoding = self.metadata.encoding.value();
        (&self.data.as_ref()[..self.metadata.length], self.metadata.value_type)
    }
}

impl PropertyValue for Raw<Vec<u8>> {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MQTYPE(sys::MQTYPE_AS_SET);
        param.impo.Options |= sys::MQIMPO_NONE;
        mqinqmp(param).map_completion(|state| {
            let len = state.value.len();
            Self::new(state.value.into_owned(), Metadata::new(len, &param.impo, param.value_type))
        })
    }
}

impl<const N: usize> PropertyValue for Raw<[u8; N]> {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MQTYPE(sys::MQTYPE_AS_SET);
        param.impo.Options |= sys::MQIMPO_NONE;
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

impl PropertyValue for String {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MQTYPE(sys::MQTYPE_STRING);
        param.impo.Options |= sys::MQIMPO_CONVERT_VALUE | sys::MQIMPO_CONVERT_TYPE;
        match mqinqmp(param)? {
            Completion(_, Some((rc @ values::MQRC(sys::MQRC_PROP_VALUE_NOT_CONVERTED), verb))) => {
                Err(Error(values::MQCC(sys::MQCC_WARNING), verb, rc))
            }
            // SAFETY: The bytes coming from the MQI library must be correct as there
            // is no conversion error
            other => Ok(other.map(|state| unsafe { str::from_utf8_unchecked(&state.value).to_string() })),
        }
    }
}

impl PropertyValue for StrCcsidOwned {
    type Error = Error;

    fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MQTYPE(sys::MQTYPE_STRING);
        param.impo.Options |= sys::MQIMPO_CONVERT_TYPE;
        mqinqmp(param).map_completion(|state| Self {
            ccsid: CCSID(param.impo.ReturnedCCSID),
            data: conversion::bytes_to_cow_mqchar(state.value).into_owned(),
            le: (param.impo.ReturnedEncoding & sys::MQENC_INTEGER_REVERSED) != 0,
        })
    }
}

#[expect(unused_parens)]
mod impl_property {
    use super::{all_multi_tuples, PropertyAttr, PropertyParam, PropertyState, PropertyValue};
    use crate::{ResultCompErr, ResultComp};
    use crate::prelude::*;

    macro_rules! impl_propertyvalue_tuple {
        ([$first:ident, $($ty:ident),*]) => {
            #[diagnostic::do_not_recommend]
            impl<$first, $($ty),*> PropertyValue for ($first, $($ty),*)
            where
                $first: PropertyValue,
                $($ty: PropertyAttr),*
            {
                type Error = $first::Error;

                #[expect(non_snake_case)]
                #[inline]
                fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqi: F) -> ResultCompErr<Self, Self::Error>
                where
                    F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
                {
                    let mut rest_outer = None;
                    $first::property_consume(param, |param| {
                        <($($ty),*) as PropertyAttr>::property_extract(param, mqi).map_completion(|(rest, state)| {
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
            impl<$first, $($ty),*> PropertyAttr for ($first, $($ty),*)
            where
                $first: PropertyAttr,
                $($ty: PropertyAttr),*
            {
                #[expect(non_snake_case)]
                #[inline]
                fn property_extract<'p, 's, F>(param: &mut PropertyParam<'p>, mqi: F) -> ResultComp<(Self, PropertyState<'s>)>
                where
                    F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>
                {
                    let mut rest_outer = None;
                    $first::property_extract(param, |param| {
                        <($($ty),*) as PropertyAttr>::property_extract(param, mqi).map_completion(|(rest, state)| {
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

    use crate::{
        conversion::slice_byte_to_mqchar,
        mqstr,
        properties_options::{Metadata, Name},
        sys,
        values::MQTYPE,
        Completion, MqStr, MqStruct, ResultComp, ResultCompExt, StrCcsid, StrCcsidOwned,
    };

    use super::{Attributes, NameUsage, PropertyAttr, PropertyParam, PropertyState, SetProperty, Value};

    #[test]
    fn set_property() {
        test_sp("test", |_, smpo, data, mq_type| {
            assert_eq!("test", data);
            assert_eq!(MQTYPE(sys::MQTYPE_STRING), mq_type);
            assert_eq!(smpo.ValueCCSID, 1208);
        });

        let mqstr_sub: MqStr<8> = mqstr!("test");
        test_sp(&mqstr_sub, |_, smpo, data, mq_type| {
            assert_eq!(mqstr_sub.as_bytes(), data);
            assert_eq!(MQTYPE(sys::MQTYPE_STRING), mq_type);
            assert_eq!(smpo.ValueCCSID, 1208);
        });

        let encoded_str = StrCcsid::from("test");
        test_sp(&encoded_str, |_, smpo, data, mq_type| {
            assert_eq!(encoded_str.data, data);
            assert_eq!(MQTYPE(sys::MQTYPE_STRING), mq_type);
            assert_eq!(smpo.ValueCCSID, encoded_str.ccsid.0);
        });

        let byte_str = b"test";
        test_sp(byte_str.as_slice(), |_, _, data, mq_type| {
            assert_eq!(byte_str, data);
            assert_eq!(MQTYPE(sys::MQTYPE_BYTE_STRING), mq_type);
        });

        test_sp(&false, |_, _, data, mq_type| {
            assert_eq!(&0, data);
            assert_eq!(MQTYPE(sys::MQTYPE_BOOLEAN), mq_type);
        });

        test_sp(&true, |_, _, data, mq_type| {
            assert_eq!(&1, data);
            assert_eq!(MQTYPE(sys::MQTYPE_BOOLEAN), mq_type);
        });

        test_simple_sp::<i8>(&99, MQTYPE(sys::MQTYPE_INT8));
        test_simple_sp::<i16>(&99, MQTYPE(sys::MQTYPE_INT16));
        test_simple_sp::<f32>(&99.0, MQTYPE(sys::MQTYPE_FLOAT32));
        test_simple_sp::<f64>(&99.0, MQTYPE(sys::MQTYPE_FLOAT64));
        test_simple_sp::<sys::MQLONG>(&99, MQTYPE(sys::MQTYPE_INT32));
        test_simple_sp::<sys::MQINT64>(&99, MQTYPE(sys::MQTYPE_INT64));

        test_sp(&Value::Null, |_, _, _, mq_type| {
            assert_eq!(mq_type, MQTYPE(sys::MQTYPE_NULL));
        });

        test_sp(&Value::ByteString(b"test".into()), |_, _, _, mq_type| {
            assert_eq!(mq_type, MQTYPE(sys::MQTYPE_BYTE_STRING));
        });

        test_sp(&Value::String("test".into()), |_, smpo, _, mq_type| {
            assert_eq!(smpo.ValueCCSID, 1208);
            assert_eq!(mq_type, MQTYPE(sys::MQTYPE_STRING));
        });

        test_sp(&Value::Float32(99.0), |_, _, _, mq_type| {
            assert_eq!(mq_type, MQTYPE(sys::MQTYPE_FLOAT32));
        });

        test_sp(&Value::Float64(99.0), |_, _, _, mq_type| {
            assert_eq!(mq_type, MQTYPE(sys::MQTYPE_FLOAT64));
        });

        test_sp(&Value::Boolean(false), |_, _, _, mq_type| {
            assert_eq!(mq_type, MQTYPE(sys::MQTYPE_BOOLEAN));
        });

        test_sp(&Value::Int8(99), |_, _, _, mq_type| {
            assert_eq!(mq_type, MQTYPE(sys::MQTYPE_INT8));
        });

        test_sp(&Value::Int16(99), |_, _, _, mq_type| {
            assert_eq!(mq_type, MQTYPE(sys::MQTYPE_INT16));
        });

        test_sp(&Value::Int32(99), |_, _, _, mq_type| {
            assert_eq!(mq_type, MQTYPE(sys::MQTYPE_INT32));
        });

        test_sp(&Value::Int64(99), |_, _, _, mq_type| {
            assert_eq!(mq_type, MQTYPE(sys::MQTYPE_INT64));
        });
    }

    fn test_simple_sp<S>(s: &S, mq_type: MQTYPE)
    where
        S::Data: PartialEq<S> + std::fmt::Debug,
        S: SetProperty + std::fmt::Debug,
    {
        test_sp(s, |_, _, data, t| {
            assert_eq!(data, s);
            assert_eq!(t, mq_type);
        });
    }

    fn test_sp<S: SetProperty + ?Sized>(sp: &S, f: impl FnOnce(&MqStruct<sys::MQPD>, &MqStruct<sys::MQSMPO>, &S::Data, MQTYPE)) {
        let mut pd = MqStruct::new(default::MQPD_DEFAULT);
        let mut smpo = MqStruct::new(default::MQSMPO_DEFAULT);
        let (data, mq_type) = sp.apply_mqsetmp(&mut pd, &mut smpo);
        f(&pd, &smpo, data, mq_type);
    }

    #[test]
    fn property_attr() -> Result<(), Box<dyn Error>> {
        let (attribute, _) = execute_pa::<Attributes>(|param| {
            param.mqpd.Context = 99;
            Ok(Completion::new(PropertyState {
                name: None,
                value: Cow::from(b"test"),
            }))
        })
        .warn_as_error()?;
        assert_eq!(99, attribute.mqpd.Context);

        let (metadata, _) = execute_pa::<Metadata>(|param| {
            param.value_type = MQTYPE(sys::MQTYPE_STRING);
            param.impo.ReturnedCCSID = 1208;
            param.impo.ReturnedEncoding = sys::MQENC_INTEGER_NORMAL;
            Ok(Completion::new(PropertyState {
                name: None,
                value: Cow::from(b"test"),
            }))
        })
        .warn_as_error()?;
        assert_eq!(4, metadata.length);
        assert_eq!(1208, metadata.ccsid);
        assert_eq!(sys::MQENC_INTEGER_NORMAL, metadata.encoding.0);

        Ok(())
    }

    #[test]
    fn property_attr_name() -> Result<(), Box<dyn Error>> {
        fn name_state(name: &[u8]) -> PropertyState {
            PropertyState {
                name: Some(Cow::from(slice_byte_to_mqchar(name))),
                value: Cow::from(b""),
            }
        }

        let (name, _) = execute_pa::<Name<String>>(|param| {
            assert_eq!(param.name_required, NameUsage::AnyLength);
            assert_ne!(param.impo.Options & sys::MQIMPO_CONVERT_VALUE, 0);
            Ok(Completion::new(name_state(b"name")))
        })
        .warn_as_error()?;
        assert_eq!(name, Name("name"));

        let (name, _) = execute_pa::<Name<MqStr<25>>>(|param| {
            assert_eq!(
                param.name_required,
                NameUsage::MaxLength(unsafe { NonZero::new_unchecked(25) })
            );
            assert_ne!(param.impo.Options & sys::MQIMPO_CONVERT_VALUE, 0);
            Ok(Completion::new(name_state(b"name")))
        })
        .warn_as_error()?;
        assert_eq!(name, Name("name"));

        let (name, _) = execute_pa::<Name<StrCcsidOwned>>(|param| {
            param.impo.ReturnedName.VSCCSID = 1208;
            assert_eq!(param.name_required, NameUsage::AnyLength);
            assert_eq!(param.impo.Options & sys::MQIMPO_CONVERT_VALUE, 0);
            Ok(Completion::new(name_state(b"name")))
        })
        .warn_as_error()?;
        assert_eq!(name, Name("name"));

        Ok(())
    }
    fn execute_pa<'a, A: PropertyAttr>(
        f: impl FnOnce(&mut PropertyParam<'_>) -> ResultComp<PropertyState<'a>>,
    ) -> ResultComp<(A, PropertyState<'a>)> {
        let mut param = PropertyParam {
            impo: MqStruct::new(default::MQIMPO_DEFAULT),
            value_type: MQTYPE::default(),
            mqpd: MqStruct::new(default::MQPD_DEFAULT),
            name_required: NameUsage::default(),
        };

        A::property_extract(&mut param, |p| f(p))
    }
}
