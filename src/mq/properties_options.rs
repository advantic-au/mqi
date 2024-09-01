use core::str;
use std::ops::Deref as _;
use std::{mem, ptr, slice};
use std::{borrow::Cow, num::NonZero};

use crate::macros::reverse_ident;

use libmqm_sys::lib::MQTYPE_STRING;

// use crate::core::macros::all_multi_tuples;
use crate::macros::all_multi_tuples;
use crate::{
    sys, Completion, Error, MqMask, MqStr, MqStruct, MqValue, MqiAttr, MqiValue, ResultComp, ResultCompErrExt, StrCcsidOwned,
    StringCcsid,
};
use crate::core::values::{self, MQENC, MQTYPE};

pub const INQUIRE_ALL: &str = "%";
pub const INQUIRE_ALL_USR: &str = "usr.%";

#[derive(Debug, Clone)]
pub struct PropertyState<'s> {
    pub name: Option<Cow<'s, [u8]>>,
    pub value: Cow<'s, [u8]>,
}

#[derive(Clone, Debug, Default)]
pub struct PropertyParam<'p> {
    pub value_type: MqValue<MQTYPE>,
    pub impo: MqStruct<'p, sys::MQIMPO>,
    pub mqpd: MqStruct<'static, sys::MQPD>,
    pub name_required: NameUsage,
}

pub trait PropertyValue: for<'p, 's> MqiValue<PropertyParam<'p>, PropertyState<'s>, Error: Into<Error>> {
    #[must_use]
    fn max_value_size() -> Option<NonZero<usize>> {
        None
    }
}

pub trait SetProperty {
    type Data: std::fmt::Debug + ?Sized;
    fn apply_mqsetmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>);
}

pub trait SetPropertyAttr {
    fn apply_mqsetmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>);
}

macro_rules! impl_setproperty_tuple {
    ($first:ident, [$($ty:ident),*]) => {
        impl<$first, $($ty),*> SetProperty for ($first, $($ty),*)
        where
            $first: SetProperty,
            $($ty: SetPropertyAttr),*
        {
            type Data = $first::Data;

            #[allow(non_snake_case,unused_parens)]
            fn apply_mqsetmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
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

macro_rules! impl_propertyvalue_tuple {
    ($first:ident, [$($ty:ident),*]) => {
        impl<$first, $($ty),*> PropertyValue for ($first, $($ty),*)
            where
                $first: PropertyValue,
                ($first, $($ty),*): for<'p, 's> MqiValue<PropertyParam<'p>, PropertyState<'s>, Error: Into<Error>>
        {
            fn max_value_size() -> Option<NonZero<usize>> {
                $first::max_value_size()
            }
        }
    };
}

all_multi_tuples!(impl_propertyvalue_tuple);

#[derive(Debug, Clone, Default)]
pub struct Attributes {
    mqpd: MqStruct<'static, sys::MQPD>,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub length: usize,
    pub ccsid: sys::MQLONG,
    pub encoding: MqMask<MQENC>,
    pub value_type: MqValue<MQTYPE>,
}

#[derive(Debug, Clone, Copy)]
pub struct Null;

#[derive(Debug, Clone, Copy, Default)]
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
    ByteString(Vec<u8>),
    String(StrCcsidOwned),
    Null,
}

impl Metadata {
    #[must_use]
    pub fn new(length: usize, impo: &MqStruct<'_, sys::MQIMPO>, value_type: MqValue<values::MQTYPE>) -> Self {
        Self {
            length,
            ccsid: impo.ReturnedCCSID,
            encoding: MqMask::from(impo.ReturnedEncoding),
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
    pub const fn encoding(&self) -> MqMask<MQENC> {
        self.encoding
    }

    #[must_use]
    pub const fn value_type(&self) -> MqValue<MQTYPE> {
        self.value_type
    }
}

impl<'p, 'a> MqiAttr<PropertyParam<'p>, PropertyState<'a>> for Metadata {
    fn extract<F>(param: &mut PropertyParam<'p>, inqmp: F) -> ResultComp<(Self, PropertyState<'a>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'a>>,
    {
        inqmp(param).map_completion(|state| (Self::new(state.value.len(), &param.impo, param.value_type), state))
    }
}

impl<'p, S> MqiAttr<PropertyParam<'p>, S> for Attributes {
    fn extract<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<S>,
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
    pub fn set_support(&mut self, support: MqValue<values::MQPD>) {
        self.mqpd.Support = support.value();
    }

    #[must_use]
    pub fn support(&self) -> MqValue<values::MQPD> {
        MqValue::from(self.mqpd.Support)
    }

    pub fn set_context(&mut self, context: MqValue<values::MQPD>) {
        self.mqpd.Context = context.value();
    }

    #[must_use]
    pub fn context(&self) -> MqValue<values::MQPD> {
        MqValue::from(self.mqpd.Context)
    }

    pub fn set_copy_options(&mut self, copy_options: MqValue<values::MQPD>) {
        self.mqpd.CopyOptions = copy_options.value();
    }

    #[must_use]
    pub fn copy_options(&self) -> MqMask<values::MQCOPY> {
        MqMask::from(self.mqpd.CopyOptions)
    }
}

macro_rules! impl_primitive_setproptype {
    ($type:ty, $mqtype:path) => {
        impl SetProperty for $type {
            type Data = Self;
            fn apply_mqsetmp(
                &self,
                _pd: &mut MqStruct<sys::MQPD>,
                smpo: &mut MqStruct<sys::MQSMPO>,
            ) -> (&Self::Data, MqValue<MQTYPE>) {
                smpo.ValueEncoding = sys::MQENC_NATIVE;
                (self, MqValue::from($mqtype))
            }
        }
    };
}

impl_primitive_setproptype!(bool, sys::MQTYPE_BOOLEAN);
impl_primitive_setproptype!(i8, sys::MQTYPE_INT8);
impl_primitive_setproptype!(i16, sys::MQTYPE_INT16);
impl_primitive_setproptype!(i32, sys::MQTYPE_INT32);
impl_primitive_setproptype!(i64, sys::MQTYPE_INT64);
impl_primitive_setproptype!(f32, sys::MQTYPE_FLOAT32);
impl_primitive_setproptype!(f64, sys::MQTYPE_FLOAT64);
impl_primitive_setproptype!(Null, sys::MQTYPE_NULL);

impl SetProperty for str {
    type Data = Self;
    fn apply_mqsetmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        smpo.ValueCCSID = 1208;
        (self, MqValue::from(sys::MQTYPE_STRING))
    }
}

impl SetProperty for String {
    type Data = <str as SetProperty>::Data;
    fn apply_mqsetmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        self.deref().apply_mqsetmp(pd, smpo)
    }
}

impl<T: AsRef<[u8]>> SetProperty for StringCcsid<T> {
    type Data = [u8];

    fn apply_mqsetmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        smpo.ValueCCSID = self.ccsid.map_or(0, Into::into);
        (self.data.as_ref(), MqValue::from(MQTYPE_STRING))
    }
}

impl SetProperty for Vec<u8> {
    type Data = <[u8] as SetProperty>::Data;
    fn apply_mqsetmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        self.deref().apply_mqsetmp(pd, smpo)
    }
}

impl<const N: usize> SetProperty for MqStr<N> {
    type Data = [u8];

    fn apply_mqsetmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        smpo.ValueCCSID = 1208;
        (self.as_bytes(), MqValue::from(sys::MQTYPE_STRING))
    }
}

impl SetProperty for [u8] {
    type Data = Self;
    fn apply_mqsetmp(&self, _pd: &mut MqStruct<sys::MQPD>, _smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        (self, MqValue::from(sys::MQTYPE_BYTE_STRING))
    }
}

impl SetProperty for Value {
    type Data = [u8];

    fn apply_mqsetmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        #[inline]
        /// Ensure the data is of type `[u8]`
        fn set_as_u8<'a, T: SetProperty + ?Sized>(
            value: &'a T,
            pd: &mut MqStruct<sys::MQPD>,
            smpo: &mut MqStruct<sys::MQSMPO>,
        ) -> (&'a [u8], MqValue<MQTYPE>) {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Deref, derive_more::DerefMut)]
pub struct Name<T>(pub T);

impl<'p, 's> MqiAttr<PropertyParam<'p>, PropertyState<'s>> for Name<String> {
    fn extract<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.name_required = NameUsage::AnyLength;
        param.impo.Options |= sys::MQIMPO_CONVERT_VALUE;
        match mqinqmp(param)? {
            Completion(_, Some((rc, verb))) if rc == sys::MQRC_PROP_NAME_NOT_CONVERTED => {
                Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc))
            }
            other => Ok(other.map(|state| {
                // SAFETY: The bytes coming from the MQI library should be correct as there
                // is no conversion error
                // The unwrap will succeed as the Option is always some if this code is executed
                let name = state.name.as_ref().expect("Option is always Some here");
                (Self(unsafe { str::from_utf8_unchecked(name).to_string() }), state)
            })),
        }
    }
}

impl<'p, 's, const N: usize> MqiAttr<PropertyParam<'p>, PropertyState<'s>> for Name<MqStr<N>> {
    fn extract<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.name_required = NameUsage::MaxLength(unsafe { NonZero::new_unchecked(N) });
        param.impo.Options |= sys::MQIMPO_CONVERT_VALUE;
        match mqinqmp(param)? {
            Completion(_, Some((rc, verb))) if rc == sys::MQRC_PROP_NAME_NOT_CONVERTED => {
                Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc))
            }
            other => Ok(other.map(|state| {
                let name = state.name.as_ref().expect("Option is always Some here");
                (
                    Self(MqStr::from_bytes(name).expect("buffer size always equals required length")),
                    state,
                )
            })),
        }
    }
}

impl<'p, 's> MqiAttr<PropertyParam<'p>, PropertyState<'s>> for Name<StrCcsidOwned> {
    fn extract<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.name_required = NameUsage::AnyLength;
        mqinqmp(param).map_completion(|state| {
            let name = state.name.as_ref().expect("Option is always Some");
            (
                Self(StrCcsidOwned {
                    ccsid: NonZero::new(param.impo.ReturnedName.VSCCSID),
                    le: (param.impo.ReturnedEncoding & sys::MQENC_INTEGER_REVERSED) != 0,
                    data: name.clone().into_owned(),
                }),
                state,
            )
        })
    }
}

impl PropertyValue for Value {}

impl<'p, 's> MqiValue<PropertyParam<'p>, PropertyState<'s>> for Value {
    type Error = Error;

    fn consume<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MqValue::from(sys::MQTYPE_AS_SET);
        param.impo.Options |= sys::MQIMPO_NONE;
        mqinqmp(param).map_completion(|state| match param.value_type.value() {
            sys::MQTYPE_BOOLEAN => Self::Boolean(state.value[8] != 0),
            sys::MQTYPE_STRING => Self::String(StringCcsid {
                ccsid: NonZero::new(param.impo.ReturnedCCSID),
                data: state.value.into_owned(),
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
        impl PropertyValue for $type {}
        impl<'p, 's> MqiValue<PropertyParam<'p>, PropertyState<'s>> for $type {
            type Error = Error;

            fn consume<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<Self>
            where
                F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
            {
                param.value_type = MqValue::from($mqtype);
                param.impo.Options |= sys::MQIMPO_CONVERT_VALUE | sys::MQIMPO_CONVERT_TYPE; // TODO: Oh shit. Rework value type to not convert
                match mqinqmp(param)? {
                    Completion(_, Some((rc, verb))) if rc == sys::MQRC_PROP_VALUE_NOT_CONVERTED => {
                        Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc))
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
                Self::from_ne_bytes(buffer.try_into().expect("buffer size always exceeds required length"))
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
    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(mem::size_of::<Self>())
    }
}

impl<'p, 's> MqiValue<PropertyParam<'p>, PropertyState<'s>> for bool {
    type Error = Error;

    fn consume<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MqValue::from(sys::MQTYPE_BOOLEAN);
        param.impo.Options |= sys::MQIMPO_CONVERT_TYPE;
        mqinqmp(param).map_completion(|state| state.value[8] != 0)
    }
}

impl PropertyValue for MqValue<sys::MQLONG> {
    fn max_value_size() -> Option<NonZero<usize>> {
        sys::MQLONG::max_value_size()
    }
}

impl<'p, 's> MqiValue<PropertyParam<'p>, PropertyState<'s>> for MqValue<sys::MQLONG> {
    type Error = Error;

    fn consume<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        sys::MQLONG::consume(param, mqinqmp).map_completion(Self::from)
    }
}

impl PropertyValue for MqMask<sys::MQLONG> {
    fn max_value_size() -> Option<NonZero<usize>> {
        sys::MQLONG::max_value_size()
    }
}

impl<'p, 's> MqiValue<PropertyParam<'p>, PropertyState<'s>> for MqMask<sys::MQLONG> {
    type Error = Error;

    fn consume<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        sys::MQLONG::consume(param, mqinqmp).map_completion(Self::from)
    }
}

impl<'p, 's> MqiValue<PropertyParam<'p>, PropertyState<'s>> for Vec<u8> {
    type Error = Error;

    fn consume<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MqValue::from(sys::MQTYPE_BYTE_STRING);
        param.impo.Options |= sys::MQIMPO_CONVERT_TYPE;
        mqinqmp(param).map_completion(|state| state.value.into())
    }
}

impl<const N: usize> PropertyValue for [u8; N] {
    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(N)
    }
}

impl<'p, 's, const N: usize> MqiValue<PropertyParam<'p>, PropertyState<'s>> for [u8; N] {
    type Error = Error;

    fn consume<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MqValue::from(sys::MQTYPE_BYTE_STRING);
        param.impo.Options |= sys::MQIMPO_CONVERT_TYPE;
        mqinqmp(param).map_completion(|state| {
            let mut result: [u8; N] = [0; N];
            result.copy_from_slice(&state.value);
            result
        })
    }
}

impl<const N: usize> PropertyValue for MqStr<N> {
    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(N)
    }
}

impl<'p, 's, const N: usize> MqiValue<PropertyParam<'p>, PropertyState<'s>> for MqStr<N> {
    type Error = Error;

    fn consume<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MqValue::from(sys::MQTYPE_BYTE_STRING);
        param.impo.Options |= sys::MQIMPO_CONVERT_VALUE | sys::MQIMPO_CONVERT_TYPE;
        mqinqmp(param).map_completion(|state| Self::from_bytes(&state.value).expect("buffer size always equals required length"))
    }
}

impl<T: AsRef<[u8]>> SetProperty for Raw<T> {
    type Data = [u8];

    fn apply_mqsetmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        smpo.ValueCCSID = self.metadata.ccsid;
        smpo.ValueEncoding = self.metadata.encoding.value();
        (&self.data.as_ref()[..self.metadata.length], self.metadata.value_type)
    }
}

impl PropertyValue for Raw<Vec<u8>> {}

impl<'p, 's> MqiValue<PropertyParam<'p>, PropertyState<'s>> for Raw<Vec<u8>> {
    type Error = Error;

    fn consume<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MqValue::from(sys::MQTYPE_AS_SET);
        param.impo.Options |= sys::MQIMPO_NONE;
        mqinqmp(param).map_completion(|state| {
            let len = state.value.len();
            Self::new(state.value.into_owned(), Metadata::new(len, &param.impo, param.value_type))
        })
    }
}

impl<const N: usize> PropertyValue for Raw<[u8; N]> {
    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(N)
    }
}

impl<'p, 's, const N: usize> MqiValue<PropertyParam<'p>, PropertyState<'s>> for Raw<[u8; N]> {
    type Error = Error;

    fn consume<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MqValue::from(sys::MQTYPE_AS_SET);
        param.impo.Options |= sys::MQIMPO_NONE;
        mqinqmp(param).map_completion(|state| {
            let mut data: [u8; N] = [0; N];
            data[..state.value.len()].copy_from_slice(&state.value);
            let len = data.len();
            Self::new(data, Metadata::new(len, &param.impo, param.value_type))
        })
    }
}

impl PropertyValue for String {}

impl<'p, 's> MqiValue<PropertyParam<'p>, PropertyState<'s>> for String {
    type Error = Error;

    fn consume<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MqValue::from(sys::MQTYPE_STRING);
        param.impo.Options |= sys::MQIMPO_CONVERT_VALUE | sys::MQIMPO_CONVERT_TYPE;
        match mqinqmp(param)? {
            Completion(_, Some((rc, verb))) if rc == sys::MQRC_PROP_VALUE_NOT_CONVERTED => {
                Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc))
            }
            // SAFETY: The bytes coming from the MQI library must be correct as there
            // is no conversion error
            other => Ok(other.map(|state| unsafe { str::from_utf8_unchecked(&state.value).to_string() })),
        }
    }
}

impl PropertyValue for StrCcsidOwned {}
impl<'p, 's> MqiValue<PropertyParam<'p>, PropertyState<'s>> for StrCcsidOwned {
    type Error = Error;

    fn consume<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        param.value_type = MqValue::from(sys::MQTYPE_STRING);
        param.impo.Options |= sys::MQIMPO_CONVERT_TYPE;
        mqinqmp(param).map_completion(|state| Self {
            ccsid: NonZero::new(param.impo.ReturnedCCSID),
            data: state.value.into_owned(),
            le: (param.impo.ReturnedEncoding & sys::MQENC_INTEGER_REVERSED) != 0,
        })
    }
}
