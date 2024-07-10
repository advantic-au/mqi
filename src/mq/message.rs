use std::ops::{Deref, DerefMut};
use std::{borrow::Cow, cmp::min, marker::PhantomData, mem::size_of, num::NonZero, ptr, str::from_utf8_unchecked};

use libmqm_sys::function;

use crate::core::values::{MQCMHO, MQCOPY, MQENC, MQIMPO, MQPD, MQSMPO, MQTYPE};
use crate::sys;
use crate::{core, Completion, ReasonCode};

use crate::EncodedString;
use crate::Error;
use crate::MqMask;
use crate::MqStr;
use crate::MqStruct;
use crate::MqValue;
use crate::ResultCompErr;
use crate::ResultCompErrExt;
use crate::StructBuilder;
use crate::MQMD;

use crate::{ResultComp, ResultErr};

use super::QueueManagerShare;

pub struct Message<'ch, L: core::Library<MQ: function::MQI>> {
    handle: core::MessageHandle,
    mq: core::MQFunctions<L>,
    connection: &'ch core::ConnectionHandle,
}

pub mod prop {
    pub const INQUIRE_ALL: &str = "%";
    pub const INQUIRE_ALL_USR: &str = "usr.%";
}

#[derive(Debug, Clone, Default)]
pub struct PropDetails<T: PropertyType> {
    pub value: T,
    mqpd: MqStruct<'static, sys::MQPD>,
    length: usize,
    ccsid: sys::MQLONG,
    encoding: MqMask<MQENC>,
}

impl<T: PropertyType> Deref for PropDetails<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: PropertyType> DerefMut for PropDetails<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: PropertyType> PropDetails<T> {
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

    pub fn support(&self) -> MqValue<MQPD> {
        MqValue::from(self.mqpd.Support)
    }

    pub fn context(&self) -> MqValue<MQPD> {
        MqValue::from(self.mqpd.Context)
    }

    pub fn copy_options(&self) -> MqMask<MQCOPY> {
        MqMask::from(self.mqpd.CopyOptions)
    }
}

pub trait NameType: Sized + Default {
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

#[derive(Debug)]
pub enum NameUsage {
    Ignored,
    MaxLength(NonZero<usize>),
    Any,
}

impl From<NameUsage> for Option<NonZero<usize>> {
    fn from(value: NameUsage) -> Self {
        match value {
            NameUsage::MaxLength(length) => Some(length),
            _ => None,
        }
    }
}

pub trait PropertyType: Sized {
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

// TODO, perhaps this can be nicer
impl<N: NameType, T: PropertyType> PropertyType for (N, T)
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

impl NameType for String {
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
            _ => Ok(unsafe { from_utf8_unchecked(&name).to_string() }),
        }
    }
}

impl<const N: usize> NameType for MqStr<N> {
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

impl PropertyType for Value {
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
            sys::MQTYPE_STRING => Self::String(<String as PropertyType>::create_from(
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

macro_rules! impl_primitive_proptype {
    ($type:ty, $mqtype:path) => {
        impl PropertyType for $type {
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

impl_primitive_proptype!(f32, sys::MQTYPE_FLOAT32);
impl_primitive_proptype!(f64, sys::MQTYPE_FLOAT64);
impl_primitive_proptype!(i8, sys::MQTYPE_INT8);
impl_primitive_proptype!(i16, sys::MQTYPE_INT16);
impl_primitive_proptype!(sys::MQLONG, sys::MQTYPE_INT32);
impl_primitive_proptype!(sys::MQINT64, sys::MQTYPE_INT64);

impl PropertyType for bool {
    type Error = Error;
    const MQTYPE: sys::MQLONG = sys::MQTYPE_BOOLEAN;
    const MQIMPO_VALUE: sys::MQLONG = sys::MQIMPO_CONVERT_TYPE;

    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(size_of::<Self>())
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

impl PropertyType for MqValue<sys::MQLONG> {
    type Error = <sys::MQLONG as PropertyType>::Error;
    const MQTYPE: sys::MQLONG = <sys::MQLONG as PropertyType>::MQTYPE;
    const MQIMPO_VALUE: sys::MQLONG = <sys::MQLONG as PropertyType>::MQIMPO_VALUE;

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

impl PropertyType for MqMask<sys::MQLONG> {
    type Error = <sys::MQLONG as PropertyType>::Error;
    const MQTYPE: sys::MQLONG = <sys::MQLONG as PropertyType>::MQTYPE;
    const MQIMPO_VALUE: sys::MQLONG = <sys::MQLONG as PropertyType>::MQIMPO_VALUE;

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

impl PropertyType for Vec<u8> {
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

impl<const N: usize> PropertyType for [u8; N] {
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

impl<const N: usize> PropertyType for MqStr<N> {
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

impl<T: PropertyType> PropertyType for PropDetails<T> {
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
            mqpd: mqpd.clone(),
            length,
            ccsid: mqimpo.ReturnedCCSID,
            encoding: mqimpo.RequestedEncoding.into(),
        })
    }

    fn name_usage() -> NameUsage {
        T::name_usage()
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        T::max_value_size()
    }
}

#[derive(Debug, Clone)]
pub enum Conversion<T> {
    Value(T),
    Raw(Vec<u8>, MqMask<MQENC>, sys::MQLONG),
}

#[derive(Debug, Clone)]
pub struct Raw<T>(T);

impl PropertyType for Raw<Vec<u8>> {
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

impl<const N: usize> PropertyType for Raw<[u8; N]> {
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
        result.copy_from_slice(&value);
        Ok(Self(result))
    }

    fn max_value_size() -> Option<NonZero<usize>> {
        NonZero::new(N)
    }
}

impl<T: Default> Default for Conversion<T> {
    fn default() -> Self {
        Self::Value(T::default())
    }
}

impl<T: PropertyType> PropertyType for Conversion<T> {
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

impl<T: NameType> NameType for Conversion<T> {
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

impl PropertyType for String {
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
            _ => Ok(unsafe { from_utf8_unchecked(&value).to_string() }),
        }
    }
}

#[derive(Clone, Debug)]
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

impl<L: core::Library<MQ: function::MQI>> Drop for Message<'_, L> {
    fn drop(&mut self) {
        let mqdmho = sys::MQDMHO::default();
        let _ = self.mq.mqdltmh(Some(self.connection), &mut self.handle, &mqdmho);
    }
}

impl<'a> MqStruct<'a, sys::MQCHARV> {
    pub fn from_encoded_str<T: EncodedString + ?Sized>(value: &'a T) -> Self {
        let data = value.data();
        let len = data
            .len()
            .try_into()
            .expect("string length exceeds maximum positive MQLONG for MQCHARV");
        MqStruct::new(sys::MQCHARV {
            VSPtr: ptr::from_ref(data).cast_mut().cast(),
            VSLength: len,
            VSBufSize: len,
            VSCCSID: value.ccsid().map_or(0, NonZero::into),
            ..sys::MQCHARV::default()
        })
    }
}

enum InqBuffer<'a, T> {
    Slice(&'a mut [T]),
    Owned(Vec<T>),
}

impl<'a, T> InqBuffer<'a, T> {
    fn truncate(self, len: usize) -> Self {
        let buf_len = self.len();
        match self {
            Self::Slice(s) => Self::Slice(&mut s[..min(len, buf_len)]),
            Self::Owned(mut v) => {
                v.truncate(len);
                Self::Owned(v)
            }
        }
    }
}

impl<'a, T> Deref for InqBuffer<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match self {
            InqBuffer::Slice(s) => s,
            InqBuffer::Owned(o) => o,
        }
    }
}

impl<'a, T> DerefMut for InqBuffer<'a, T> {
    // Cow doesn't have DerefMut
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            InqBuffer::Slice(s) => s,
            InqBuffer::Owned(o) => &mut *o,
        }
    }
}

impl<'a, T: Clone> From<InqBuffer<'a, T>> for Cow<'a, [T]>
where
    [T]: ToOwned,
{
    fn from(value: InqBuffer<'a, T>) -> Self {
        match value {
            InqBuffer::Slice(s) => Cow::from(&*s),
            InqBuffer::Owned(o) => o.into(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn inqmp<'a, 'b, A: core::Library<MQ: function::MQI>>(
    mq: &core::MQFunctions<A>,
    connection_handle: Option<&core::ConnectionHandle>,
    message_handle: &core::MessageHandle,
    mqimpo: &mut MqStruct<sys::MQIMPO>,
    name: &MqStruct<sys::MQCHARV>,
    mqpd: &mut MqStruct<sys::MQPD>,
    value_type: &mut MqValue<MQTYPE>,
    mut value: InqBuffer<'a, u8>,
    max_value_size: Option<NonZero<usize>>,
    mut returned_name: Option<InqBuffer<'b, u8>>,
    max_name_size: Option<NonZero<usize>>,
) -> ResultCompErr<(InqBuffer<'a, u8>, Option<InqBuffer<'b, u8>>), core::MqInqError> {
    if let Some(rn) = returned_name.as_mut() {
        let rn_ref: &mut [u8] = rn;
        mqimpo.ReturnedName.VSPtr = rn_ref.as_mut_ptr().cast();
        mqimpo.ReturnedName.VSBufSize = rn_ref.len().try_into().expect("length always converts to usize");
    } else {
        mqimpo.ReturnedName.VSPtr = ptr::null_mut();
    }

    match (
        mq.mqinqmp(
            connection_handle,
            message_handle,
            mqimpo,
            name,
            mqpd,
            value_type,
            Some(&mut *value),
        ),
        returned_name,
    ) {
        (Err(core::MqInqError::Length(length, Error(_, _, rc))), rn)
            if rc == sys::MQRC_PROPERTY_VALUE_TOO_BIG
                && max_value_size.map_or(true, |max_len| Into::<usize>::into(max_len) > value.len()) =>
        {
            let len = length.try_into().expect("length always converts to usize");
            let value_vec = InqBuffer::Owned(vec![0; len]);
            inqmp(
                mq,
                connection_handle,
                message_handle,
                mqimpo,
                name,
                mqpd,
                value_type,
                value_vec,
                max_value_size,
                rn,
                max_name_size,
            )
        }
        (Err(core::MqInqError::Length(length, Error(_, _, rc))), Some(rn))
            if rc == sys::MQRC_PROPERTY_NAME_TOO_BIG
                && max_name_size.map_or(true, |max_len| Into::<usize>::into(max_len) > rn.len()) =>
        {
            let len = length.try_into().expect("length always converts to usize");
            let name_vec = InqBuffer::Owned(vec![0; len]);
            inqmp(
                mq,
                connection_handle,
                message_handle,
                mqimpo,
                name,
                mqpd,
                value_type,
                value,
                max_value_size,
                Some(name_vec),
                max_name_size,
            )
        }
        (other, rn) => other.map_completion(|length| {
            (
                value.truncate(length.try_into().expect("length always convertable to usize")),
                rn.map(|name| {
                    name.truncate(
                        mqimpo
                            .ReturnedName
                            .VSLength
                            .try_into()
                            .expect("length always convertable to usize"),
                    )
                }),
            )
        }),
    }
}

pub struct MsgPropIter<'connection, 'name, 'message, P, N: EncodedString + ?Sized, L: core::Library<MQ: function::MQI>> {
    name: &'name N,
    message: &'message Message<'connection, L>,
    options: MqMask<MQIMPO>,
    _marker: PhantomData<P>,
}

impl<P: PropertyType, N: EncodedString + ?Sized, L: core::Library<MQ: function::MQI>> Iterator
    for MsgPropIter<'_, '_, '_, P, N, L>
{
    type Item = ResultCompErr<P, P::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.message.inq::<P, N>(self.name, self.options) {
            Ok(Completion(Some(value), warning)) => Some(Ok(Completion(value, warning))),
            Ok(Completion(None, _)) => None,
            Err(e) => Some(Err(e)),
        };

        self.options |= sys::MQIMPO_INQ_NEXT;

        result
    }
}

impl<'connection, L: core::Library<MQ: function::MQI>> Message<'connection, L> {
    pub fn new(lib: L, connection: &'connection core::ConnectionHandle, options: MqValue<MQCMHO>) -> ResultErr<Self> {
        let mqcmho = sys::MQCMHO {
            Options: options.value(),
            ..sys::MQCMHO::default()
        };
        let mq = core::MQFunctions(lib);
        mq.mqcrtmh(Some(connection), &mqcmho)
            .map(|handle| Self { handle, mq, connection })
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn inq_iter<'m, 'n, P: PropertyType + ?Sized, N: EncodedString + ?Sized>(
        &'m self,
        name: &'n N,
        options: MqMask<MQIMPO>,
    ) -> MsgPropIter<'connection, 'n, 'm, P, N, L> {
        MsgPropIter::<P, N, L> {
            name,
            message: self,
            options: options | sys::MQIMPO_INQ_NEXT,
            _marker: PhantomData,
        }
    }

    pub fn inq<P: PropertyType + ?Sized, N: EncodedString + ?Sized>(
        &self,
        name: &N,
        options: MqMask<MQIMPO>,
    ) -> ResultCompErr<Option<P>, P::Error> {
        const DEFAULT_BUF_SIZE: usize = 1024;
        let mut val_return_buffer = [0; DEFAULT_BUF_SIZE]; // Returned value buffer
        let mut name_return_buffer = [0; DEFAULT_BUF_SIZE]; // Returned name buffer

        let mut inq_name_buffer = match P::name_usage() {
            NameUsage::Ignored => None,
            used => {
                let buf = InqBuffer::Slice(name_return_buffer.as_mut_slice());
                Some(match used {
                    NameUsage::MaxLength(length) => buf.truncate(length.into()),
                    _ => buf,
                })
            }
        };
        let mut mqimpo = MqStruct::new(sys::MQIMPO {
            Options: options.value() | P::MQIMPO_VALUE,
            ReturnedName: inq_name_buffer.as_mut().map_or_else(Default::default, |name| sys::MQCHARV {
                VSPtr: ptr::from_mut(&mut *name).cast(),
                VSBufSize: name.len().try_into().expect("length of buffer must always fit in an MQLONG"),
                ..sys::MQCHARV::default()
            }),
            ..sys::MQIMPO::default()
        });

        let mut inq_value_buffer = InqBuffer::Slice(val_return_buffer.as_mut_slice());
        inq_value_buffer = match P::max_value_size() {
            Some(max_size) => inq_value_buffer.truncate(max_size.into()),
            None => inq_value_buffer,
        };
        let mut vt = MqValue::from(P::MQTYPE);
        let name = MqStruct::from_encoded_str(name);
        let mut mqpd = MqStruct::<sys::MQPD>::default();

        let inq = match inqmp(
            &self.mq,
            Some(self.connection),
            &self.handle,
            &mut mqimpo,
            &name,
            &mut mqpd,
            &mut vt,
            inq_value_buffer,
            P::max_value_size(),
            inq_name_buffer,
            P::name_usage().into()
        )
        .map_err(Into::<Error>::into) // Convert the error into an ordinary MQ error
        {
            Err(Error(cc, _, rc)) if cc == sys::MQCC_FAILED && rc == sys::MQRC_PROPERTY_NOT_AVAILABLE => {
                Ok(Completion::new(None)) // Convert property not available to a result of Option::None
            }
            other => other.map_completion(Some),
        }?;

        Ok(match inq {
            Completion(Some((value, name)), warning) => Completion(
                Some(P::create_from(
                    value.into(),
                    name.map(Into::into),
                    vt,
                    &mqimpo,
                    &mqpd,
                    warning,
                )?),
                warning,
            ),
            comp => comp.map(|_| None),
        })
    }

    pub fn set_property<N: EncodedString + ?Sized, V: EncodedString + ?Sized>(
        &self,
        name: &N,
        value: &V,
        location: MqValue<MQSMPO>,
        mqpd: &impl StructBuilder<sys::MQPD>,
    ) -> ResultComp<()> {
        let mut mqpd = mqpd.build();
        let mut mqsmpo = MqStruct::<sys::MQSMPO>::default();
        mqsmpo.Options = location.value();
        mqsmpo.ValueCCSID = value.ccsid().map_or(0, NonZero::into);

        let name_mqcharv = MqStruct::from_encoded_str(name);

        // TODO: work out what we should get from MQPD
        self.mq.mqsetmp(
            self.connection,
            &self.handle,
            &mqsmpo,
            &name_mqcharv,
            &mut mqpd,
            MqValue::from(sys::MQTYPE_STRING),
            value.data(),
        )
    }
}

impl<L: core::Library<MQ: function::MQI>, H> QueueManagerShare<'_, L, H> {
    pub fn put<B>(&self, mqod: &mut sys::MQOD, mqmd: Option<&mut impl MQMD>, pmo: &mut sys::MQPMO, body: &B) -> ResultComp<()> {
        self.mq().mqput1(self.handle(), mqod, mqmd, pmo, body)
    }
}
