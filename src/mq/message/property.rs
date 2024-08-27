use core::str;
use std::ops::Deref as _;
use std::{mem, ptr, slice};
use std::{borrow::Cow, num::NonZero};

use libmqm_sys::lib::MQTYPE_STRING;

use crate::{
    sys, types, Completion, ConsumeValue2, Error, ExtractValue2, MqMask, MqStr, MqStruct, MqValue, ReasonCode, ResultComp,
    ResultCompErrExt, StrCcsidOwned, StringCcsid,
};
use crate::core::values::{MQENC, MQTYPE};

pub const INQUIRE_ALL: &str = "%";
pub const INQUIRE_ALL_USR: &str = "usr.%";

pub struct PropertyState<'s> {
    pub name: Option<Cow<'s, [u8]>>,
    pub value: Cow<'s, [u8]>,
}

pub struct PropertyParam<'p> {
    pub value_type: MqValue<MQTYPE>,
    pub impo: MqStruct<'p, sys::MQIMPO>,
    pub mqpd: MqStruct<'static, sys::MQPD>,
}

pub trait PropertyConsume: for<'p, 's> ConsumeValue2<PropertyParam<'p>, PropertyState<'s>, Error: Into<Error>> {
    const MQTYPE: sys::MQLONG;
    const MQIMPO: sys::MQLONG;

    #[must_use]
    fn name_usage() -> NameUsage {
        NameUsage::Ignored
    }

    #[must_use]
    fn max_value_size() -> Option<NonZero<usize>> {
        None
    }
}

pub trait PropertyExtract: for<'p, 's> ExtractValue2<PropertyParam<'p>, PropertyState<'s>> {
    // const MQIMPO: sys::MQLONG;
}

pub trait SetPropertyType {
    type Data: std::fmt::Debug + ?Sized;
    fn apply_mqinqmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>);
}

pub trait InqNameType: Sized + Default {
    type Error: From<Error>;
    const MQIMPO_NAME: sys::MQLONG;

    #[must_use]
    fn max_size() -> Option<NonZero<usize>> {
        None
    }

    fn create_from(
        name: Cow<[u8]>,
        mqimpo: &MqStruct<sys::MQIMPO>,
        warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error>;
}

#[derive(Debug, Clone, Default)]
pub struct Attributes<T> {
    pub value: T,
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

#[derive(Debug)]
pub enum NameUsage {
    Ignored,
    MaxLength(NonZero<usize>),
    Any,
}

#[derive(Debug, Clone)]
pub enum Conversion<T> {
    Value(T),
    Raw(Vec<u8>, MqMask<MQENC>, sys::MQLONG),
}

// pub type RawMeta<T> = Metadata<Raw<T>>;
// pub type OwnedRawMeta = RawMeta<Vec<u8>>;

#[derive(Debug, Clone)]
pub struct Raw<T>(T);

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
    String(String),
    Null,
}

impl Metadata {
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

impl<'p, 'a> ExtractValue2<PropertyParam<'p>, PropertyState<'a>> for Metadata {
    fn extract<F>(param: &mut PropertyParam<'p>, inqmp: F) -> ResultComp<(Self, PropertyState<'a>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'a>>,
    {
        inqmp(param).map_completion(|state| {
            (
                Self {
                    length: state.value.len(),
                    ccsid: param.impo.ReturnedCCSID,
                    encoding: MqMask::from(param.impo.ReturnedEncoding),
                    value_type: param.value_type,
                },
                state,
            )
        })
    }
}

// impl<T: PropertyConsume> Attributes<T> {
//     pub fn new(value: T) -> Self {
//         Self {
//             value,
//             mqpd: MqStruct::default(),
//         }
//     }

//     pub fn set_support(&mut self, support: MqValue<MQPD>) {
//         self.mqpd.Support = support.value();
//     }

//     pub fn support(&self) -> MqValue<MQPD> {
//         MqValue::from(self.mqpd.Support)
//     }

//     pub fn set_context(&mut self, context: MqValue<MQPD>) {
//         self.mqpd.Context = context.value();
//     }

//     pub fn context(&self) -> MqValue<MQPD> {
//         MqValue::from(self.mqpd.Context)
//     }

//     pub fn set_copy_options(&mut self, copy_options: MqValue<MQPD>) {
//         self.mqpd.CopyOptions = copy_options.value();
//     }

//     pub fn copy_options(&self) -> MqMask<MQCOPY> {
//         MqMask::from(self.mqpd.CopyOptions)
//     }
// }

macro_rules! impl_primitive_setproptype {
    ($type:ty, $mqtype:path) => {
        impl SetPropertyType for $type {
            type Data = Self;
            fn apply_mqinqmp(
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

impl SetPropertyType for str {
    type Data = Self;
    fn apply_mqinqmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        smpo.ValueCCSID = 1208;
        (self, MqValue::from(sys::MQTYPE_STRING))
    }
}

impl SetPropertyType for String {
    type Data = <str as SetPropertyType>::Data;
    fn apply_mqinqmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        self.deref().apply_mqinqmp(pd, smpo)
    }
}

impl<T: AsRef<[u8]>> SetPropertyType for StringCcsid<T> {
    type Data = [u8];

    fn apply_mqinqmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        smpo.ValueCCSID = self.ccsid.map_or(0, Into::into);
        (self.data.as_ref(), MqValue::from(MQTYPE_STRING))
    }
}

impl SetPropertyType for Vec<u8> {
    type Data = <[u8] as SetPropertyType>::Data;
    fn apply_mqinqmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        self.deref().apply_mqinqmp(pd, smpo)
    }
}

impl<const N: usize> SetPropertyType for MqStr<N> {
    type Data = [u8];

    fn apply_mqinqmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        smpo.ValueCCSID = 1208;
        (self.as_bytes(), MqValue::from(sys::MQTYPE_STRING))
    }
}

impl<T: SetPropertyType> SetPropertyType for Attributes<T> {
    type Data = T::Data;

    fn apply_mqinqmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        self.mqpd.clone_into(pd);
        self.value.apply_mqinqmp(pd, smpo)
    }
}

impl SetPropertyType for [u8] {
    type Data = Self;
    fn apply_mqinqmp(&self, _pd: &mut MqStruct<sys::MQPD>, _smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        (self, MqValue::from(sys::MQTYPE_BYTE_STRING))
    }
}

impl SetPropertyType for Value {
    type Data = [u8];

    fn apply_mqinqmp(&self, pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        #[inline]
        /// Ensure the data is of type `[u8]`
        fn set_as_u8<'a, T: SetPropertyType + ?Sized>(
            value: &'a T,
            pd: &mut MqStruct<sys::MQPD>,
            smpo: &mut MqStruct<sys::MQSMPO>,
        ) -> (&'a [u8], MqValue<MQTYPE>) {
            let (data, value_type) = value.apply_mqinqmp(pd, smpo);
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
            Self::String(value) => set_as_u8(&**value, pd, smpo),
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

impl<'p, 's> ExtractValue2<PropertyParam<'p>, PropertyState<'s>> for Name<String> {
    fn extract<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
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

impl<'p, 's, const N: usize> ExtractValue2<PropertyParam<'p>, PropertyState<'s>> for Name<MqStr<N>> {
    fn extract<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
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

// TODO, perhaps this can be nicer with the Error types
// impl<N: InqNameType, T: PropertyConsume> PropertyConsume for (N, T)
// where
//     T::Error: Into<Error>,
//     N::Error: Into<Error>,
// {
//     type Error = Error;
//     const MQTYPE: sys::MQLONG = T::MQTYPE;
//     const MQIMPO_VALUE: sys::MQLONG = T::MQIMPO_VALUE | N::MQIMPO_NAME;

//     fn name_usage() -> NameUsage {
//         match (N::max_size(), T::name_usage()) {
//             (None, NameUsage::Any | NameUsage::Ignored) => NameUsage::Any,
//             (None, usage @ NameUsage::MaxLength(_)) => usage,
//             (Some(size), NameUsage::Any | NameUsage::Ignored) => NameUsage::MaxLength(size),
//             (Some(size), NameUsage::MaxLength(length)) => NameUsage::MaxLength(min(size, length)),
//         }
//     }

//     fn create_from(
//         value: Cow<[u8]>,
//         name: Option<Cow<[u8]>>,
//         value_type: MqValue<MQTYPE>,
//         mqimpo: &MqStruct<sys::MQIMPO>,
//         mqpd: &MqStruct<'static, sys::MQPD>,
//         warning: Option<(ReasonCode, &'static str)>,
//     ) -> Result<Self, Self::Error> {
//         // Note: Create the value before the name. When name and value conversion fail, value is reporting in preference over name.
//         let value = T::create_from(value, name.clone(), value_type, mqimpo, mqpd, warning).map_err(Into::into)?;
//         Ok((
//             match name {
//                 Some(name) => N::create_from(name, mqimpo, warning).map_err(Into::into)?,
//                 None => N::default(),
//             },
//             value,
//         ))
//     }
// }

impl<'p, 's> ExtractValue2<PropertyParam<'p>, PropertyState<'s>> for Name<StrCcsidOwned> {
    fn extract<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
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

impl PropertyConsume for Value {
    const MQTYPE: sys::MQLONG = sys::MQTYPE_AS_SET;
    const MQIMPO: sys::MQLONG = sys::MQIMPO_NONE;
}

impl<'p, 's> ConsumeValue2<PropertyParam<'p>, PropertyState<'s>> for Value {
    type Error = Error;

    fn consume<F>(param: &mut PropertyParam<'p>, mqinqmp: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
    {
        match param.value_type.value() {
            sys::MQTYPE_BOOLEAN => bool::consume(param, mqinqmp).map_completion(Self::Boolean),
            sys::MQTYPE_STRING => String::consume(param, mqinqmp).map_completion(Self::String),
            sys::MQTYPE_BYTE_STRING => Vec::consume(param, mqinqmp).map_completion(Self::ByteString),
            sys::MQTYPE_INT8 => i8::consume(param, mqinqmp).map_completion(Self::Int8),
            sys::MQTYPE_INT16 => i16::consume(param, mqinqmp).map_completion(Self::Int16),
            sys::MQTYPE_INT32 => i32::consume(param, mqinqmp).map_completion(Self::Int32),
            sys::MQTYPE_INT64 => i64::consume(param, mqinqmp).map_completion(Self::Int64),
            sys::MQTYPE_FLOAT32 => f32::consume(param, mqinqmp).map_completion(Self::Float32),
            sys::MQTYPE_FLOAT64 => f64::consume(param, mqinqmp).map_completion(Self::Float64),
            sys::MQTYPE_NULL => Self::Null,
            _ => unreachable!(),
        }
    }

    // fn consume_from(
    //     state: PropertyState<'s>,
    //     param: &PropertyParam<'p>,
    //     warning: Option<crate::types::Warning>,
    // ) -> Result<Self, Self::Error> {
    //     Ok(match param.value_type.value() {
    //         sys::MQTYPE_BOOLEAN => Self::Boolean(bool::consume_from(state, param, warning)?),
    //         sys::MQTYPE_STRING => Self::String(String::consume_from(state, param, warning)?),
    //         sys::MQTYPE_BYTE_STRING => Self::ByteString(Vec::consume_from(state, param, warning)?),
    //         sys::MQTYPE_INT8 => Self::Int8(i8::consume_from(state, param, warning)?),
    //         sys::MQTYPE_INT16 => Self::Int16(i16::consume_from(state, param, warning)?),
    //         sys::MQTYPE_INT32 => Self::Int32(i32::consume_from(state, param, warning)?),
    //         sys::MQTYPE_INT64 => Self::Int64(i64::consume_from(state, param, warning)?),
    //         sys::MQTYPE_FLOAT32 => Self::Float32(f32::consume_from(state, param, warning)?),
    //         sys::MQTYPE_FLOAT64 => Self::Float64(f64::consume_from(state, param, warning)?),
    //         sys::MQTYPE_NULL => Self::Null,
    //         _ => unreachable!(),
    //     })
    // }
}

// impl<'s, P> ConsumeValue2<P, PropertyState<'s>> for i32 {
//     type Error = Error;

//     fn consume<F>(param: &mut P, mqinqmp: F) -> ResultComp<Self>
//     where
//         F: FnOnce(&mut P) -> ResultComp<PropertyState<'s>>,
//     {
//         match mqinqmp(param)? {
//             Completion(_, Some((rc, verb))) if rc == sys::MQRC_PROP_VALUE_NOT_CONVERTED => {
//                 Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc))
//             }
//             other => Ok(other.map(|state| {
//                 Self::from_ne_bytes(*state.value)
//                     .try_into()
//                     .expect("buffer size always exceeds required length")
//             })),
//         }
//     }
// }

macro_rules! impl_primitive_propertyconsume {
    ($type:ty, $mqtype:path) => {
        impl PropertyConsume for $type {
            const MQTYPE: sys::MQLONG = $mqtype;
            const MQIMPO: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE | sys::MQIMPO_CONVERT_TYPE;
        }

        impl<'s, P> ConsumeValue2<P, PropertyState<'s>> for $type {
            type Error = Error;

            fn consume<F>(param: &mut P, mqinqmp: F) -> ResultComp<Self>
            where
                F: FnOnce(&mut P) -> ResultComp<PropertyState<'s>>,
            {
                match mqinqmp(param)? {
                    Completion(_, Some((rc, verb))) if rc == sys::MQRC_PROP_VALUE_NOT_CONVERTED => {
                        Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc))
                    }
                    other => Ok(other.map(|state| {
                        Self::from_ne_bytes(*state.value)
                            .try_into()
                            .expect("buffer size always exceeds required length")
                    })),
                }
            }
        }
    };
}

impl_primitive_propertyconsume!(f32, sys::MQTYPE_FLOAT32);
impl_primitive_propertyconsume!(f64, sys::MQTYPE_FLOAT64);
impl_primitive_propertyconsume!(i8, sys::MQTYPE_INT8);
impl_primitive_propertyconsume!(i16, sys::MQTYPE_INT16);
impl_primitive_propertyconsume!(sys::MQLONG, sys::MQTYPE_INT32);
impl_primitive_propertyconsume!(sys::MQINT64, sys::MQTYPE_INT64);

impl PropertyConsume for bool {
    const MQTYPE: sys::MQLONG = sys::MQTYPE_BOOLEAN;
    const MQIMPO: sys::MQLONG = sys::MQIMPO_CONVERT_TYPE;

    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(mem::size_of::<Self>())
    }
}

impl<'s, P> ConsumeValue2<P, PropertyState<'s>> for bool {
    type Error = Error;

    fn consume<F>(param: &mut P, mqinqmp: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut P) -> ResultComp<PropertyState<'s>>,
    {
        mqinqmp(param).map_completion(|state| state.value[8] != 0)
    }
}

impl PropertyConsume for MqValue<sys::MQLONG> {
    const MQTYPE: sys::MQLONG = <sys::MQLONG as PropertyConsume>::MQTYPE;
    const MQIMPO: sys::MQLONG = <sys::MQLONG as PropertyConsume>::MQIMPO;

    fn max_value_size() -> Option<NonZero<usize>> {
        sys::MQLONG::max_value_size()
    }
}

impl<P> ConsumeValue<P, PropertyState<'_>> for MqValue<sys::MQLONG> {
    type Error = Error;

    fn consume_from(state: PropertyState<'_>, param: &P, warning: Option<types::Warning>) -> Result<Self, Self::Error> {
        Ok(Self::from(sys::MQLONG::consume_from(state, param, warning)?))
    }
}

impl PropertyConsume for MqMask<sys::MQLONG> {
    const MQTYPE: sys::MQLONG = <sys::MQLONG as PropertyConsume>::MQTYPE;
    const MQIMPO: sys::MQLONG = <sys::MQLONG as PropertyConsume>::MQIMPO;

    fn max_value_size() -> Option<NonZero<usize>> {
        sys::MQLONG::max_value_size()
    }
}

impl<P> ConsumeValue<P, PropertyState<'_>> for MqMask<sys::MQLONG> {
    type Error = Error;

    fn consume_from(state: PropertyState<'_>, param: &P, warning: Option<types::Warning>) -> Result<Self, Self::Error> {
        Ok(Self::from(sys::MQLONG::consume_from(state, param, warning)?))
    }
}

impl PropertyConsume for Vec<u8> {
    const MQTYPE: sys::MQLONG = sys::MQTYPE_BYTE_STRING;
    const MQIMPO: sys::MQLONG = sys::MQIMPO_CONVERT_TYPE;
}

impl<P> ConsumeValue<P, PropertyState<'_>> for Vec<u8> {
    type Error = Error;

    fn consume_from(state: PropertyState<'_>, _param: &P, _warning: Option<types::Warning>) -> Result<Self, Self::Error> {
        Ok(state.value.into_owned())
    }
}

// impl<const N: usize> PropertyConsume for [u8; N] {
//     type Error = Error;
//     const MQTYPE: sys::MQLONG = sys::MQTYPE_BYTE_STRING;
//     const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_CONVERT_TYPE;

//     fn create_from(
//         value: Cow<[u8]>,
//         _name: Option<Cow<[u8]>>,
//         _value_type: MqValue<MQTYPE>,
//         _mqimpo: &MqStruct<sys::MQIMPO>,
//         _mqpd: &MqStruct<'static, sys::MQPD>,
//         _warning: Option<(ReasonCode, &'static str)>,
//     ) -> Result<Self, Self::Error> {
//         let mut result: [u8; N] = [0; N];
//         result.copy_from_slice(&value);
//         Ok(result)
//     }

//     fn max_value_size() -> Option<NonZero<usize>> {
//         NonZero::new(N)
//     }
// }

// impl<const N: usize> PropertyConsume for MqStr<N> {
//     type Error = Error;

//     const MQTYPE: sys::MQLONG = sys::MQTYPE_STRING;
//     const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE | sys::MQIMPO_CONVERT_TYPE;

//     fn create_from(
//         value: Cow<[u8]>,
//         _name: Option<Cow<[u8]>>,
//         _value_type: MqValue<MQTYPE>,
//         _mqimpo: &MqStruct<sys::MQIMPO>,
//         _mqpd: &MqStruct<'static, sys::MQPD>,
//         _warning: Option<(ReasonCode, &'static str)>,
//     ) -> Result<Self, Self::Error> {
//         Ok(Self::from_bytes(&value).expect("buffer size always equals required length"))
//     }

//     fn max_value_size() -> Option<NonZero<usize>> {
//         NonZero::new(N)
//     }
// }

// impl<T: PropertyConsume> PropertyConsume for Attributes<T> {
//     type Error = T::Error;
//     const MQTYPE: sys::MQLONG = T::MQTYPE;
//     const MQIMPO_VALUE: sys::MQLONG = T::MQIMPO_VALUE;

//     fn create_from(
//         value: Cow<[u8]>,
//         name: Option<Cow<[u8]>>,
//         value_type: MqValue<MQTYPE>,
//         mqimpo: &MqStruct<sys::MQIMPO>,
//         mqpd: &MqStruct<'static, sys::MQPD>,
//         warning: Option<(ReasonCode, &'static str)>,
//     ) -> Result<Self, Self::Error> {
//         // let length = value.len();
//         Ok(Self {
//             value: T::create_from(value, name, value_type, mqimpo, mqpd, warning)?,
//             mqpd: mqpd.clone(),
//         })
//     }

//     fn name_usage() -> NameUsage {
//         T::name_usage()
//     }

//     fn max_value_size() -> Option<NonZero<usize>> {
//         T::max_value_size()
//     }
// }

// impl<T: AsRef<[u8]>> SetPropertyType for RawMeta<T> {
//     type Data = [u8];

//     fn apply_mqinqmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
//         smpo.ValueCCSID = self.ccsid;
//         smpo.ValueEncoding = self.encoding.value();
//         (&self.0.as_ref()[..self.length], self.value_type)
//     }
// }

// impl PropertyConsume for Raw<Vec<u8>> {
//     type Error = Error;
//     const MQTYPE: sys::MQLONG = sys::MQTYPE_AS_SET;
//     const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_NONE;

//     fn create_from(
//         value: Cow<[u8]>,
//         _name: Option<Cow<[u8]>>,
//         _value_type: MqValue<MQTYPE>,
//         _mqimpo: &MqStruct<sys::MQIMPO>,
//         _mqpd: &MqStruct<'static, sys::MQPD>,
//         _warning: Option<(ReasonCode, &'static str)>,
//     ) -> Result<Self, Self::Error> {
//         Ok(Self(value.into_owned()))
//     }
// }

// impl<const N: usize> PropertyConsume for Raw<[u8; N]> {
//     type Error = Error;
//     const MQTYPE: sys::MQLONG = sys::MQTYPE_AS_SET;
//     const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_NONE;

//     fn create_from(
//         value: Cow<[u8]>,
//         _name: Option<Cow<[u8]>>,
//         _value_type: MqValue<MQTYPE>,
//         _mqimpo: &MqStruct<sys::MQIMPO>,
//         _mqpd: &MqStruct<'static, sys::MQPD>,
//         _warning: Option<(ReasonCode, &'static str)>,
//     ) -> Result<Self, Self::Error> {
//         let mut result: [u8; N] = [0; N];
//         result[..value.len()].copy_from_slice(&value);
//         Ok(Self(result))
//     }

//     fn max_value_size() -> Option<NonZero<usize>> {
//         NonZero::new(N)
//     }
// }

// impl<T> Deref for Raw<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         let Self(target) = self;
//         target
//     }
// }

// impl<T> DerefMut for Raw<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         let Self(ref mut target) = self;
//         target
//     }
// }

// impl<T: Default> Default for Conversion<T> {
//     fn default() -> Self {
//         Self::Value(T::default())
//     }
// }

// impl<T: PropertyConsume> PropertyConsume for Conversion<T> {
//     type Error = T::Error;
//     const MQTYPE: sys::MQLONG = T::MQTYPE;
//     const MQIMPO_VALUE: sys::MQLONG = T::MQIMPO_VALUE;

//     fn create_from(
//         value: Cow<[u8]>,
//         name: Option<Cow<[u8]>>,
//         value_type: MqValue<MQTYPE>,
//         mqimpo: &MqStruct<sys::MQIMPO>,
//         mqpd: &MqStruct<'static, sys::MQPD>,
//         warning: Option<(ReasonCode, &'static str)>,
//     ) -> Result<Self, Self::Error> {
//         Ok(match warning {
//             Some((rc, ..)) if rc == sys::MQRC_PROP_VALUE_NOT_CONVERTED => {
//                 Self::Raw(value.into_owned(), mqimpo.ReturnedEncoding.into(), mqimpo.ReturnedCCSID)
//             }
//             _ => Self::Value(T::create_from(value, name, value_type, mqimpo, mqpd, warning)?),
//         })
//     }

//     fn name_usage() -> NameUsage {
//         T::name_usage()
//     }

//     fn max_value_size() -> Option<NonZero<usize>> {
//         T::max_value_size()
//     }
// }

// impl<T: InqNameType> InqNameType for Conversion<T> {
//     type Error = T::Error;
//     const MQIMPO_NAME: sys::MQLONG = T::MQIMPO_NAME;

//     fn create_from(
//         name: Cow<[u8]>,
//         mqimpo: &MqStruct<sys::MQIMPO>,
//         warning: Option<(ReasonCode, &'static str)>,
//     ) -> Result<Self, Self::Error> {
//         Ok(match warning {
//             Some((rc, ..)) if rc == sys::MQRC_PROP_NAME_NOT_CONVERTED => {
//                 Self::Raw(
//                     name.into_owned(),
//                     MqMask::from(0), /* Encoding not relevant */
//                     mqimpo.ReturnedName.VSCCSID,
//                 )
//             }
//             _ => Self::Value(T::create_from(name, mqimpo, warning)?),
//         })
//     }

//     fn max_size() -> Option<NonZero<usize>> {
//         T::max_size()
//     }
// }

impl PropertyConsume for String {
    const MQTYPE: sys::MQLONG = sys::MQTYPE_STRING;
    const MQIMPO: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE | sys::MQIMPO_CONVERT_TYPE;
}

impl<'s, P> ConsumeValue<P, PropertyState<'s>> for String {
    type Error = Error;

    fn consume_from(state: PropertyState<'s>, _param: &P, warning: Option<types::Warning>) -> Result<Self, Self::Error> {
        match warning {
            Some((rc, verb)) if rc == sys::MQRC_PROP_VALUE_NOT_CONVERTED => {
                Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc))
            }
            // SAFETY: The bytes coming from the MQI library should be correct as there
            // is no conversion error
            _ => Ok(unsafe { str::from_utf8_unchecked(&state.value).to_string() }),
        }
    }
}

impl PropertyConsume for StrCcsidOwned {
    const MQTYPE: sys::MQLONG = sys::MQTYPE_STRING;
    const MQIMPO: sys::MQLONG = sys::MQIMPO_CONVERT_TYPE;
}

impl<'p, 's> ConsumeValue<PropertyParam<'p>, PropertyState<'s>> for StrCcsidOwned {
    type Error = Error;

    fn consume_from(
        state: PropertyState<'s>,
        param: &PropertyParam<'p>,
        _warning: Option<types::Warning>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            ccsid: NonZero::new(param.impo.ReturnedCCSID),
            data: state.value.into_owned(),
            le: (param.impo.ReturnedEncoding & sys::MQENC_INTEGER_REVERSED) != 0,
        })
    }
}
