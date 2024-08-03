use std::cmp::min;
use std::ops::{Deref, DerefMut};
use std::{mem, ptr, slice, str};
use std::{borrow::Cow, num::NonZero};

use libmqm_sys::lib::{MQIMPO_NONE, MQTYPE_STRING};

use crate::{sys, Error, MqMask, MqStr, MqStruct, MqValue, StrCcsidOwned, ReasonCode, StringCcsid};
use crate::core::values::{MQCOPY, MQENC, MQPD, MQTYPE};

pub const INQUIRE_ALL: &str = "%";
pub const INQUIRE_ALL_USR: &str = "usr.%";

pub trait InqPropertyType: Sized {
    type Error: From<Error>;

    const MQTYPE: sys::MQLONG;
    const MQIMPO_VALUE: sys::MQLONG;

    #[must_use]
    fn name_usage() -> NameUsage {
        NameUsage::Ignored
    }

    #[must_use]
    fn max_value_size() -> Option<NonZero<usize>> {
        None
    }

    fn create_from(
        value: Cow<[u8]>,
        name: Option<Cow<[u8]>>,
        value_type: MqValue<MQTYPE>,
        mqimpo: &MqStruct<sys::MQIMPO>,
        mqpd: &MqStruct<'static, sys::MQPD>,
        warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error>;
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
pub struct Metadata<T> {
    pub value: T,
    length: usize,
    ccsid: sys::MQLONG,
    encoding: MqMask<MQENC>,
    value_type: MqValue<MQTYPE>,
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

pub type RawMeta<T> = Metadata<Raw<T>>;
pub type OwnedRawMeta = RawMeta<Vec<u8>>;

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

impl<T> Metadata<T> {
    pub const fn len(&self) -> usize {
        self.length
    }

    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub const fn ccsid(&self) -> sys::MQLONG {
        self.ccsid
    }

    pub const fn encoding(&self) -> MqMask<MQENC> {
        self.encoding
    }

    pub const fn value_type(&self) -> MqValue<MQTYPE> {
        self.value_type
    }
}

impl<T> Deref for Metadata<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Metadata<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: InqPropertyType> InqPropertyType for Metadata<T> {
    type Error = T::Error;

    const MQTYPE: sys::MQLONG = T::MQTYPE;
    const MQIMPO_VALUE: sys::MQLONG = T::MQIMPO_VALUE;

    fn create_from(
        value: Cow<[u8]>,
        name: Option<Cow<[u8]>>,
        value_type: MqValue<MQTYPE>,
        mqimpo: &MqStruct<sys::MQIMPO>,
        mqpd: &MqStruct<'static, sys::MQPD>,
        warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        let length = value.len();
        Ok(Self {
            value: T::create_from(value, name, value_type, mqimpo, mqpd, warning)?,
            length,
            ccsid: mqimpo.ReturnedCCSID,
            encoding: MqMask::from(mqimpo.ReturnedEncoding),
            value_type,
        })
    }

    fn name_usage() -> NameUsage {
        T::name_usage()
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        T::max_value_size()
    }
}

impl<T: InqPropertyType> Deref for Attributes<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: InqPropertyType> DerefMut for Attributes<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: InqPropertyType> Attributes<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            mqpd: MqStruct::default(),
        }
    }

    pub fn set_support(&mut self, support: MqValue<MQPD>) {
        self.mqpd.Support = support.value();
    }

    pub fn support(&self) -> MqValue<MQPD> {
        MqValue::from(self.mqpd.Support)
    }

    pub fn set_context(&mut self, context: MqValue<MQPD>) {
        self.mqpd.Context = context.value();
    }

    pub fn context(&self) -> MqValue<MQPD> {
        MqValue::from(self.mqpd.Context)
    }

    pub fn set_copy_options(&mut self, copy_options: MqValue<MQPD>) {
        self.mqpd.CopyOptions = copy_options.value();
    }

    pub fn copy_options(&self) -> MqMask<MQCOPY> {
        MqMask::from(self.mqpd.CopyOptions)
    }
}

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

// TODO, perhaps this can be nicer with the Error types
impl<N: InqNameType, T: InqPropertyType> InqPropertyType for (N, T)
where
    T::Error: Into<Error>,
    N::Error: Into<Error>,
{
    type Error = Error;
    const MQTYPE: sys::MQLONG = T::MQTYPE;
    const MQIMPO_VALUE: sys::MQLONG = T::MQIMPO_VALUE | N::MQIMPO_NAME;

    fn name_usage() -> NameUsage {
        match (N::max_size(), T::name_usage()) {
            (None, NameUsage::Any | NameUsage::Ignored) => NameUsage::Any,
            (None, usage @ NameUsage::MaxLength(_)) => usage,
            (Some(size), NameUsage::Any | NameUsage::Ignored) => NameUsage::MaxLength(size),
            (Some(size), NameUsage::MaxLength(length)) => NameUsage::MaxLength(min(size, length)),
        }
    }

    fn create_from(
        value: Cow<[u8]>,
        name: Option<Cow<[u8]>>,
        value_type: MqValue<MQTYPE>,
        mqimpo: &MqStruct<sys::MQIMPO>,
        mqpd: &MqStruct<'static, sys::MQPD>,
        warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        // Note: Create the value before the name. When name and value conversion fail, value is reporting in preference over name.
        let value = T::create_from(value, name.clone(), value_type, mqimpo, mqpd, warning).map_err(Into::into)?;
        Ok((
            match name {
                Some(name) => N::create_from(name, mqimpo, warning).map_err(Into::into)?,
                None => N::default(),
            },
            value,
        ))
    }
}

impl InqNameType for String {
    type Error = Error;
    const MQIMPO_NAME: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE;

    fn create_from(
        name: Cow<[u8]>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        match warning {
            Some((rc, verb)) if rc == sys::MQRC_PROP_NAME_NOT_CONVERTED => Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc)),
            // SAFETY: The bytes coming from the MQI library should be correct as there
            // is no conversion error
            _ => Ok(unsafe { str::from_utf8_unchecked(&name).to_string() }),
        }
    }
}

impl<const N: usize> InqNameType for MqStr<N> {
    type Error = Error;
    const MQIMPO_NAME: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE;

    fn max_size() -> Option<NonZero<usize>> {
        N.try_into().ok()
    }

    fn create_from(
        name: Cow<[u8]>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        match warning {
            Some((rc, verb)) if rc == sys::MQRC_PROP_NAME_NOT_CONVERTED => Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc)),
            _ => Ok(Self::from_bytes(&name).expect("buffer size always equals required length")),
        }
    }
}

impl InqNameType for StrCcsidOwned {
    type Error = Error;
    const MQIMPO_NAME: sys::MQLONG = MQIMPO_NONE;

    fn create_from(
        name: Cow<[u8]>,
        mqimpo: &MqStruct<sys::MQIMPO>,
        _warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            ccsid: NonZero::new(mqimpo.ReturnedName.VSCCSID),
            le: (mqimpo.ReturnedEncoding & sys::MQENC_INTEGER_REVERSED) != 0,
            data: name.into_owned(),
        })
    }
}

impl InqPropertyType for Value {
    type Error = Error;
    const MQTYPE: sys::MQLONG = sys::MQTYPE_AS_SET;
    const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_NONE;

    fn create_from(
        value: Cow<[u8]>,
        name: Option<Cow<[u8]>>,
        value_type: MqValue<MQTYPE>,
        mqimpo: &MqStruct<sys::MQIMPO>,
        mqpd: &MqStruct<'static, sys::MQPD>,
        warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        Ok(match value_type.value() {
            sys::MQTYPE_BOOLEAN => Self::Boolean(bool::create_from(value, name, value_type, mqimpo, mqpd, warning)?),
            sys::MQTYPE_STRING => Self::String(<String as InqPropertyType>::create_from(
                value, name, value_type, mqimpo, mqpd, warning,
            )?),
            sys::MQTYPE_BYTE_STRING => Self::ByteString(Vec::create_from(value, name, value_type, mqimpo, mqpd, warning)?),
            sys::MQTYPE_INT8 => Self::Int8(i8::create_from(value, name, value_type, mqimpo, mqpd, warning)?),
            sys::MQTYPE_INT16 => Self::Int16(i16::create_from(value, name, value_type, mqimpo, mqpd, warning)?),
            sys::MQTYPE_INT32 => Self::Int32(i32::create_from(value, name, value_type, mqimpo, mqpd, warning)?),
            sys::MQTYPE_INT64 => Self::Int64(i64::create_from(value, name, value_type, mqimpo, mqpd, warning)?),
            sys::MQTYPE_FLOAT32 => Self::Float32(f32::create_from(value, name, value_type, mqimpo, mqpd, warning)?),
            sys::MQTYPE_FLOAT64 => Self::Float64(f64::create_from(value, name, value_type, mqimpo, mqpd, warning)?),
            sys::MQTYPE_NULL => Self::Null,
            _ => unreachable!(),
        })
    }
}

macro_rules! impl_primitive_inqproptype {
    ($type:ty, $mqtype:path) => {
        impl InqPropertyType for $type {
            type Error = Error;
            const MQTYPE: sys::MQLONG = $mqtype;
            const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE | sys::MQIMPO_CONVERT_TYPE;

            fn create_from(
                value: Cow<[u8]>,
                _name: Option<Cow<[u8]>>,
                _value_type: MqValue<MQTYPE>,
                _mqimpo: &MqStruct<sys::MQIMPO>,
                _mqpd: &MqStruct<'static, sys::MQPD>,
                warning: Option<(ReasonCode, &'static str)>,
            ) -> Result<Self, Self::Error> {
                match warning {
                    Some((rc, verb)) if rc == sys::MQRC_PROP_VALUE_NOT_CONVERTED => {
                        Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc))
                    }
                    _ => Ok(Self::from_ne_bytes(
                        (*value).try_into().expect("buffer size always exceeds required length"),
                    )),
                }
            }
        }
    };
}

impl_primitive_inqproptype!(f32, sys::MQTYPE_FLOAT32);
impl_primitive_inqproptype!(f64, sys::MQTYPE_FLOAT64);
impl_primitive_inqproptype!(i8, sys::MQTYPE_INT8);
impl_primitive_inqproptype!(i16, sys::MQTYPE_INT16);
impl_primitive_inqproptype!(sys::MQLONG, sys::MQTYPE_INT32);
impl_primitive_inqproptype!(sys::MQINT64, sys::MQTYPE_INT64);

impl InqPropertyType for bool {
    type Error = Error;
    const MQTYPE: sys::MQLONG = sys::MQTYPE_BOOLEAN;
    const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_CONVERT_TYPE;

    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(mem::size_of::<Self>())
    }

    fn create_from(
        value: Cow<[u8]>,
        _name: Option<Cow<[u8]>>,
        _value_type: MqValue<MQTYPE>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        _mqpd: &MqStruct<'static, sys::MQPD>,
        _warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        Ok(value[8] != 0)
    }
}

impl InqPropertyType for MqValue<sys::MQLONG> {
    type Error = <sys::MQLONG as InqPropertyType>::Error;
    const MQTYPE: sys::MQLONG = <sys::MQLONG as InqPropertyType>::MQTYPE;
    const MQIMPO_VALUE: sys::MQLONG = <sys::MQLONG as InqPropertyType>::MQIMPO_VALUE;

    fn create_from(
        value: Cow<[u8]>,
        name: Option<Cow<[u8]>>,
        value_type: MqValue<MQTYPE>,
        mqimpo: &MqStruct<sys::MQIMPO>,
        mqpd: &MqStruct<'static, sys::MQPD>,
        warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        Ok(Self::from(sys::MQLONG::create_from(
            value, name, value_type, mqimpo, mqpd, warning,
        )?))
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        sys::MQLONG::max_value_size()
    }
}

impl InqPropertyType for MqMask<sys::MQLONG> {
    type Error = <sys::MQLONG as InqPropertyType>::Error;
    const MQTYPE: sys::MQLONG = <sys::MQLONG as InqPropertyType>::MQTYPE;
    const MQIMPO_VALUE: sys::MQLONG = <sys::MQLONG as InqPropertyType>::MQIMPO_VALUE;

    fn create_from(
        value: Cow<[u8]>,
        name: Option<Cow<[u8]>>,
        value_type: MqValue<MQTYPE>,
        mqimpo: &MqStruct<sys::MQIMPO>,
        mqpd: &MqStruct<'static, sys::MQPD>,
        warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        Ok(Self::from(sys::MQLONG::create_from(
            value, name, value_type, mqimpo, mqpd, warning,
        )?))
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        sys::MQLONG::max_value_size()
    }
}

impl InqPropertyType for Vec<u8> {
    type Error = Error;
    const MQTYPE: sys::MQLONG = sys::MQTYPE_BYTE_STRING;
    const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_CONVERT_TYPE;

    fn create_from(
        value: Cow<[u8]>,
        _name: Option<Cow<[u8]>>,
        _value_type: MqValue<MQTYPE>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        _mqpd: &MqStruct<'static, sys::MQPD>,
        _warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        Ok(value.into_owned())
    }
}

impl<const N: usize> InqPropertyType for [u8; N] {
    type Error = Error;
    const MQTYPE: sys::MQLONG = sys::MQTYPE_BYTE_STRING;
    const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_CONVERT_TYPE;

    fn create_from(
        value: Cow<[u8]>,
        _name: Option<Cow<[u8]>>,
        _value_type: MqValue<MQTYPE>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        _mqpd: &MqStruct<'static, sys::MQPD>,
        _warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        let mut result: [u8; N] = [0; N];
        result.copy_from_slice(&value);
        Ok(result)
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(N)
    }
}

impl<const N: usize> InqPropertyType for MqStr<N> {
    type Error = Error;

    const MQTYPE: sys::MQLONG = sys::MQTYPE_STRING;
    const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE | sys::MQIMPO_CONVERT_TYPE;

    fn create_from(
        value: Cow<[u8]>,
        _name: Option<Cow<[u8]>>,
        _value_type: MqValue<MQTYPE>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        _mqpd: &MqStruct<'static, sys::MQPD>,
        _warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        Ok(Self::from_bytes(&value).expect("buffer size always equals required length"))
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(N)
    }
}

impl<T: InqPropertyType> InqPropertyType for Attributes<T> {
    type Error = T::Error;
    const MQTYPE: sys::MQLONG = T::MQTYPE;
    const MQIMPO_VALUE: sys::MQLONG = T::MQIMPO_VALUE;

    fn create_from(
        value: Cow<[u8]>,
        name: Option<Cow<[u8]>>,
        value_type: MqValue<MQTYPE>,
        mqimpo: &MqStruct<sys::MQIMPO>,
        mqpd: &MqStruct<'static, sys::MQPD>,
        warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        // let length = value.len();
        Ok(Self {
            value: T::create_from(value, name, value_type, mqimpo, mqpd, warning)?,
            mqpd: mqpd.clone(),
        })
    }

    fn name_usage() -> NameUsage {
        T::name_usage()
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        T::max_value_size()
    }
}

impl<T: AsRef<[u8]>> SetPropertyType for RawMeta<T> {
    type Data = [u8];

    fn apply_mqinqmp(&self, _pd: &mut MqStruct<sys::MQPD>, smpo: &mut MqStruct<sys::MQSMPO>) -> (&Self::Data, MqValue<MQTYPE>) {
        smpo.ValueCCSID = self.ccsid;
        smpo.ValueEncoding = self.encoding.value();
        (&self.0.as_ref()[..self.length], self.value_type)
    }
}

impl InqPropertyType for Raw<Vec<u8>> {
    type Error = Error;
    const MQTYPE: sys::MQLONG = sys::MQTYPE_AS_SET;
    const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_NONE;

    fn create_from(
        value: Cow<[u8]>,
        _name: Option<Cow<[u8]>>,
        _value_type: MqValue<MQTYPE>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        _mqpd: &MqStruct<'static, sys::MQPD>,
        _warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        Ok(Self(value.into_owned()))
    }
}

impl<const N: usize> InqPropertyType for Raw<[u8; N]> {
    type Error = Error;
    const MQTYPE: sys::MQLONG = sys::MQTYPE_AS_SET;
    const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_NONE;

    fn create_from(
        value: Cow<[u8]>,
        _name: Option<Cow<[u8]>>,
        _value_type: MqValue<MQTYPE>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        _mqpd: &MqStruct<'static, sys::MQPD>,
        _warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        let mut result: [u8; N] = [0; N];
        result[..value.len()].copy_from_slice(&value);
        Ok(Self(result))
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(N)
    }
}

impl<T> Deref for Raw<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let Self(target) = self;
        target
    }
}

impl<T> DerefMut for Raw<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self(ref mut target) = self;
        target
    }
}

impl<T: Default> Default for Conversion<T> {
    fn default() -> Self {
        Self::Value(T::default())
    }
}

impl<T: InqPropertyType> InqPropertyType for Conversion<T> {
    type Error = T::Error;
    const MQTYPE: sys::MQLONG = T::MQTYPE;
    const MQIMPO_VALUE: sys::MQLONG = T::MQIMPO_VALUE;

    fn create_from(
        value: Cow<[u8]>,
        name: Option<Cow<[u8]>>,
        value_type: MqValue<MQTYPE>,
        mqimpo: &MqStruct<sys::MQIMPO>,
        mqpd: &MqStruct<'static, sys::MQPD>,
        warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        Ok(match warning {
            Some((rc, ..)) if rc == sys::MQRC_PROP_VALUE_NOT_CONVERTED => {
                Self::Raw(value.into_owned(), mqimpo.ReturnedEncoding.into(), mqimpo.ReturnedCCSID)
            }
            _ => Self::Value(T::create_from(value, name, value_type, mqimpo, mqpd, warning)?),
        })
    }

    fn name_usage() -> NameUsage {
        T::name_usage()
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        T::max_value_size()
    }
}

impl<T: InqNameType> InqNameType for Conversion<T> {
    type Error = T::Error;
    const MQIMPO_NAME: sys::MQLONG = T::MQIMPO_NAME;

    fn create_from(
        name: Cow<[u8]>,
        mqimpo: &MqStruct<sys::MQIMPO>,
        warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        Ok(match warning {
            Some((rc, ..)) if rc == sys::MQRC_PROP_NAME_NOT_CONVERTED => {
                Self::Raw(
                    name.into_owned(),
                    MqMask::from(0), /* Encoding not relevant */
                    mqimpo.ReturnedName.VSCCSID,
                )
            }
            _ => Self::Value(T::create_from(name, mqimpo, warning)?),
        })
    }

    fn max_size() -> Option<NonZero<usize>> {
        T::max_size()
    }
}

impl InqPropertyType for String {
    type Error = Error;

    const MQTYPE: sys::MQLONG = sys::MQTYPE_STRING;
    const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE | sys::MQIMPO_CONVERT_TYPE;

    fn create_from(
        value: Cow<[u8]>,
        _name: Option<Cow<[u8]>>,
        _value_type: MqValue<MQTYPE>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        _mqpd: &MqStruct<'static, sys::MQPD>,
        warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        match warning {
            Some((rc, verb)) if rc == sys::MQRC_PROP_VALUE_NOT_CONVERTED => {
                Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc))
            }
            // SAFETY: The bytes coming from the MQI library should be correct as there
            // is no conversion error
            _ => Ok(unsafe { str::from_utf8_unchecked(&value).to_string() }),
        }
    }
}

impl InqPropertyType for StrCcsidOwned {
    type Error = Error;

    const MQTYPE: sys::MQLONG = sys::MQTYPE_STRING;
    const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_CONVERT_TYPE;

    fn create_from(
        value: Cow<[u8]>,
        _name: Option<Cow<[u8]>>,
        _value_type: MqValue<MQTYPE>,
        mqimpo: &MqStruct<sys::MQIMPO>,
        _mqpd: &MqStruct<'static, sys::MQPD>,
        _warning: Option<(ReasonCode, &'static str)>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            ccsid: NonZero::new(mqimpo.ReturnedCCSID),
            data: value.into_owned(),
            le: (mqimpo.ReturnedEncoding & sys::MQENC_INTEGER_REVERSED) != 0,
        })
    }
}
