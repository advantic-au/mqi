use std::array::TryFromSliceError;
use std::borrow::Cow;
use std::convert::Infallible;
use std::mem::size_of;
use std::mem::size_of_val;
use std::num::NonZero;
use std::ops::Deref;
use std::ptr;
use std::string::FromUtf8Error;

use libmqm_sys::function;
use thiserror::Error;

use crate::core;
use crate::core::values::MQIMPO;
use crate::sys;
use crate::Completion;
use crate::EncodedString;
use crate::MQStrError;
use crate::MqMask;
use crate::MqStr;
use crate::MqStruct;
use crate::MqValue;
use crate::ResultCompErr;
use crate::StructBuilder;
use crate::MQMD;
use crate::{
    core::{
        values::{MQCMHO, MQENC, MQSMPO, MQTYPE},
        Library, MQFunctions,
    },
    StrCcsid,
};
use crate::{ResultComp, ResultErr};

use super::QueueManagerShare;

pub struct Message<'ch, L: Library<MQ: function::MQI>> {
    handle: core::MessageHandle,
    mq: MQFunctions<L>,
    connection: &'ch core::ConnectionHandle,
}

pub mod prop {
    pub const INQUIRE_ALL: &str = "%";
    pub const INQUIRE_ALL_USR: &str = "usr.%";
}

// pub struct MsgProp<'mh, 'name, 'ch, L: Library<MQ: function::MQI>> {
//     name: StrCcsid<'name>,
//     mh: &'mh Message<'ch, L>,
//     inq_prop_opts: MqStruct<'static, sys::MQIMPO>,
// }

#[derive(Debug, Clone, Default)]
pub struct PropDetails<T> {
    pub value: T,
    mqpd: MqStruct<'static, sys::MQPD>,
}

#[derive(Debug, Clone, Default)]
pub struct PropDetails2<T: MessagePropertyType2> {
    pub value: T,
    mqpd: MqStruct<'static, sys::MQPD>,
}

#[derive(Debug, Clone, Default)]
pub struct PropOptions<T> {
    pub value: T,
    type_str: MqStr<8>,
}

impl<T> PropOptions<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            type_str: MqStr::default(),
        }
    }
}

impl<T> Deref for PropOptions<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Deref for PropDetails<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: MessagePropertyType> MessagePropertyType for PropOptions<T> {
    type ValueType = T::ValueType;
    type ValueRef = T::ValueRef;
    const MQTYPE: sys::MQLONG = T::MQTYPE;
    const MQIMPO: sys::MQLONG = T::MQIMPO;

    fn value_mut(&mut self) -> &mut Self::ValueRef {
        self.value.value_mut()
    }

    fn receive_mqimpo(&mut self, mqimpo: &MqStruct<sys::MQIMPO>) {
        self.type_str = mqimpo.TypeString.into();
        self.value.receive_mqimpo(mqimpo);
    }
}

impl<T> PropDetails<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            mqpd: MqStruct::default(),
        }
    }

    pub fn support(&self) -> MqValue<core::values::MQPD> {
        MqValue::from(self.mqpd.Support)
    }

    pub fn context(&self) -> MqValue<core::values::MQPD> {
        MqValue::from(self.mqpd.Context)
    }

    pub fn copy_options(&self) -> MqMask<core::values::MQCOPY> {
        MqMask::from(self.mqpd.CopyOptions)
    }
}

impl<T: MessagePropertyType> MessagePropertyType for PropDetails<T> {
    type ValueType = T::ValueType;
    type ValueRef = T::ValueRef;
    const MQTYPE: sys::MQLONG = T::MQTYPE;
    const MQIMPO: sys::MQLONG = T::MQIMPO;

    fn value_mut(&mut self) -> &mut Self::ValueRef {
        self.value.value_mut()
    }

    fn receive_mqpd(&mut self, mqpd: &MqStruct<'static, sys::MQPD>) {
        self.mqpd = mqpd.clone();
        self.value.receive_mqpd(mqpd);
    }
}

pub trait MessagePropertyType2: Sized {
    type Error: std::fmt::Debug;

    const MQTYPE: sys::MQLONG;
    const MQIMPO: sys::MQLONG;

    #[must_use]
    fn buffer_sizing() -> (usize, Option<usize>) {
        (0, None)
    }

    fn create_from(
        buffer: Cow<[u8]>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        mqpd: &MqStruct<'static, sys::MQPD>,
    ) -> Result<Self, Self::Error>;
}

#[allow(clippy::use_self)]
impl MessagePropertyType2 for sys::MQLONG {
    type Error = TryFromSliceError;
    const MQTYPE: sys::MQLONG = sys::MQTYPE_INT32;
    const MQIMPO: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE;

    fn buffer_sizing() -> (usize, Option<usize>) {
        (size_of::<Self>(), Some(size_of::<Self>()))
    }

    fn create_from(
        buffer: Cow<[u8]>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        _mqpd: &MqStruct<'static, sys::MQPD>,
    ) -> Result<Self, Self::Error> {
        let buffer = &*buffer;
        Ok(Self::from_ne_bytes(buffer.try_into()?))
    }
}

impl MessagePropertyType2 for Vec<u8> {
    type Error = Infallible;
    const MQTYPE: sys::MQLONG = sys::MQTYPE_BYTE_STRING;
    const MQIMPO: sys::MQLONG = sys::MQIMPO_NONE;

    fn create_from(
        buffer: Cow<[u8]>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        _mqpd: &MqStruct<'static, sys::MQPD>,
    ) -> Result<Self, Self::Error> {
        Ok(buffer.into_owned())
    }
}

impl<const N: usize> MessagePropertyType2 for MqStr<N> {
    type Error = MQStrError;

    const MQTYPE: sys::MQLONG = sys::MQTYPE_STRING;
    const MQIMPO: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE;

    fn create_from(
        buffer: Cow<[u8]>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        _mqpd: &MqStruct<'static, sys::MQPD>,
    ) -> Result<Self, Self::Error> {
        MqStr::from_bytes(&buffer)
    }
}

impl<T: MessagePropertyType2> MessagePropertyType2 for PropDetails2<T> {
    type Error = T::Error;
    const MQTYPE: sys::MQLONG = T::MQTYPE;
    const MQIMPO: sys::MQLONG = T::MQIMPO;

    fn create_from(
        buffer: Cow<[u8]>,
        mqimpo: &MqStruct<sys::MQIMPO>,
        mqpd: &MqStruct<'static, sys::MQPD>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            value: T::create_from(buffer, mqimpo, mqpd)?,
            mqpd: mqpd.clone(),
        })
    }
}

impl MessagePropertyType2 for String {
    type Error = FromUtf8Error;

    const MQTYPE: sys::MQLONG = sys::MQTYPE_STRING;
    const MQIMPO: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE;

    fn create_from(
        buffer: Cow<[u8]>,
        _mqimpo: &MqStruct<sys::MQIMPO>,
        _mqpd: &MqStruct<'static, sys::MQPD>,
    ) -> Result<Self, Self::Error> {
        String::from_utf8(buffer.into_owned())
    }
}

pub trait MessagePropertyType {
    type ValueType: ?Sized;
    type ValueRef: ?Sized;
    const MQTYPE: sys::MQLONG;
    const MQIMPO: sys::MQLONG;

    fn value_mut(&mut self) -> &mut Self::ValueRef;
    fn receive_mqpd(&mut self, _mqpd: &MqStruct<'static, sys::MQPD>) {}
    fn receive_mqimpo(&mut self, _mqimpo: &MqStruct<sys::MQIMPO>) {}
}

impl<const N: usize> MessagePropertyType for MqStr<N> {
    type ValueType = Self;
    type ValueRef = Self::ValueType;
    const MQTYPE: sys::MQLONG = sys::MQTYPE_STRING;
    const MQIMPO: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE;

    fn value_mut(&mut self) -> &mut Self::ValueRef {
        self
    }
}

impl MessagePropertyType for [u8] {
    type ValueType = Self;
    type ValueRef = Self::ValueType;
    const MQTYPE: sys::MQLONG = sys::MQTYPE_BYTE_STRING;
    const MQIMPO: sys::MQLONG = sys::MQIMPO_NONE;

    fn value_mut(&mut self) -> &mut Self::ValueRef {
        self
    }
}

impl<T> MessagePropertyType for MqValue<T> {
    type ValueType = sys::MQLONG;
    type ValueRef = Self::ValueType;
    const MQTYPE: sys::MQLONG = <Self::ValueType as MessagePropertyType>::MQTYPE;
    const MQIMPO: sys::MQLONG = <Self::ValueType as MessagePropertyType>::MQIMPO;

    fn value_mut(&mut self) -> &mut Self::ValueRef {
        &mut self.0
    }
}

impl<T> MessagePropertyType for MqMask<T> {
    type ValueType = sys::MQLONG;
    type ValueRef = Self::ValueType;
    const MQTYPE: sys::MQLONG = <Self::ValueType as MessagePropertyType>::MQTYPE;
    const MQIMPO: sys::MQLONG = <Self::ValueType as MessagePropertyType>::MQIMPO;

    fn value_mut(&mut self) -> &mut Self::ValueRef {
        &mut self.0
    }
}

#[allow(clippy::use_self)]
impl MessagePropertyType for sys::MQLONG {
    type ValueType = Self;
    type ValueRef = Self::ValueType;
    const MQTYPE: sys::MQLONG = sys::MQTYPE_INT32;
    const MQIMPO: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE;

    fn value_mut(&mut self) -> &mut Self::ValueRef {
        self
    }
}

impl MessagePropertyType for sys::MQINT64 {
    type ValueType = Self;
    type ValueRef = Self::ValueType;
    const MQTYPE: sys::MQLONG = sys::MQTYPE_INT64;
    const MQIMPO: sys::MQLONG = sys::MQIMPO_CONVERT_VALUE;

    fn value_mut(&mut self) -> &mut Self::ValueRef {
        self
    }
}

// fn inq_long<T: ?Sized, Y: ?Sized, L: Library<MQ: function::MQI>>(
//     name: &impl EncodedString,
//     mq: &MQFunctions<L>,
//     ch: &ConnectionHandle,
//     mh: &MessageHandle,
//     options: MqMask<MQIMPO>,
//     returned_name: &mut T,
//     value: Option<&mut Y>,
// ) -> ResultComp<sys::MQLONG> {

// }

// type MsgPropIterItem<'a> = ResultComp<(StrCcsid<'a>, Value<'a>)>;

// impl<L: Library<MQ: function::MQI>> Iterator for MsgPropIter<'_, L> {
//     type Item<'a> = MsgPropIterItem<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         fn next_result<A: Library<MQ: function::MQI>>(it: &mut MsgPropIter<A>) -> MsgPropIterItem {
//             let name_mqcharv = MqStruct::<sys::MQCHARV>::from_encoded_str(name_ref);
//             let mut returning_name = Vec::<u8>::with_capacity(page_size::get());
//             let rn_mqcharv = &mut it.inq_prop_opts.ReturnedName;
//             rn_mqcharv.VSPtr = returning_name.spare_capacity_mut().as_mut_ptr().cast();
//             rn_mqcharv.VSOffset = 0;
//             rn_mqcharv.VSBufSize = returning_name
//                 .capacity()
//                 .try_into()
//                 .expect("page size exceeds maximum positive MQLONG");
//             rn_mqcharv.VSLength = 0;
//             rn_mqcharv.VSCCSID = 0;

//             let mut value = Vec::<u8>::with_capacity(page_size::get());
//             let mut prop_type = MqValue::from(sys::MQTYPE_AS_SET);
//             let mut prop_desc = MqStruct::<sys::MQPD>::default();

//             let inq_length = it.message.mq.mqinqmp(
//                 Some(it.message.connection),
//                 &it.message.handle,
//                 &mut it.inq_prop_opts,
//                 &name_mqcharv,
//                 &mut prop_desc,
//                 &mut prop_type,
//                 value.spare_capacity_mut(),
//             )?;
//             let Completion(length, ..) = inq_length; // TODO: deal with truncation
//             unsafe {
//                 value.set_len(
//                     length
//                         .try_into()
//                         .expect("property length exceeds maximum positive MQLONG"),
//                 );
//                 returning_name.set_len(
//                     it.inq_prop_opts
//                         .ReturnedName
//                         .VSLength
//                         .try_into()
//                         .expect("property name length exceeds maximum positive MQLONG"),
//                 );
//             }
//             it.inq_prop_opts.Options |= sys::MQIMPO_INQ_NEXT;

//             Ok(inq_length.map(|_| {
//                 (
//                     StringCcsid {
//                         ccsid: NonZero::new(it.inq_prop_opts.ReturnedName.VSCCSID),
//                         data: returning_name,
//                     },
//                     Value::from_parts(
//                         &value,
//                         prop_type,
//                         it.inq_prop_opts.ReturnedCCSID,
//                         MqMask::from(it.inq_prop_opts.RequestedEncoding),
//                     ),
//                 )
//             }))
//         }

//         match next_result(self) {
//             Err(Error(cc, _, rc)) if cc == sys::MQCC_FAILED && rc == sys::MQRC_PROPERTY_NOT_AVAILABLE => None,
//             result => Some(result),
//         }
//     }
// }

#[derive(Clone, Debug)]
pub enum Value<'a> {
    Boolean(bool),
    Bytes(&'a [u8]),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    String(&'a str),
    EncodedString(StrCcsid<'a>),
    Null,
    Data(MqValue<MQTYPE>, &'a [u8], Coding),
}

#[derive(Clone, Copy, Debug)]
pub enum Coding {
    NumericEncoding(MqMask<MQENC>),
    TextCCSID(sys::MQLONG),
}

// const ENC_NATIVE_INTEGER: sys::MQLONG = sys::MQENC_INTEGER_MASK as sys::MQLONG & sys::MQENC_NATIVE;
// const ENC_NATIVE_FLOAT: sys::MQLONG = sys::MQENC_FLOAT_MASK as sys::MQLONG & sys::MQENC_NATIVE;

// Create message handle option value
// impl<'a> Value<'a> {
//     #[allow(clippy::cast_possible_wrap)] // Masks are unsigned.
//     pub fn from_parts(
//         data: &'a [u8],
//         prop_type: MqValue<core::values::MQTYPE>,
//         ccsid: sys::MQLONG,
//         encoding: MqMask<MQENC>,
//     ) -> Result<Self, TryFromSliceError> {
//         match prop_type.value() {
//             sys::MQTYPE_BOOLEAN => Ok(Self::Boolean(data[0] != 0)),
//             sys::MQTYPE_BYTE_STRING => Ok(Self::Bytes(data)),
//             sys::MQTYPE_INT16 => data.try_into().map(|d| {
//                 let val = i16::from_ne_bytes(d);
//                 Self::Int16({
//                     match (encoding & (sys::MQENC_INTEGER_MASK as sys::MQLONG)).value() {
//                         sys::MQENC_INTEGER_REVERSED => val.reverse_bits(),
//                         _ => val,
//                     }
//                 })
//             }),
//             sys::MQTYPE_INT32 => data.try_into().map(|d| {
//                 let val = i32::from_ne_bytes(d);
//                 Self::Int32({
//                     match (encoding & (sys::MQENC_INTEGER_MASK as sys::MQLONG)).value() {
//                         sys::MQENC_INTEGER_REVERSED => val.reverse_bits(),
//                         _ => val,
//                     }
//                 })
//             }),
//             sys::MQTYPE_INT64 => data.try_into().map(|d| {
//                 let val = i64::from_ne_bytes(d);
//                 Self::Int64({
//                     match (encoding & (sys::MQENC_INTEGER_MASK as sys::MQLONG)).value() {
//                         sys::MQENC_INTEGER_REVERSED => val.reverse_bits(),
//                         _ => val,
//                     }
//                 })
//             }),
//             sys::MQTYPE_FLOAT32 => data.try_into().map(|d| {
//                 let val = u32::from_ne_bytes(d);
//                 Self::Float32(f32::from_bits(
//                     match (encoding & (sys::MQENC_FLOAT_MASK as sys::MQLONG)).value() {
//                         sys::MQENC_FLOAT_IEEE_REVERSED => val.reverse_bits(),
//                         _ => val,
//                     },
//                 ))
//             }),
//             sys::MQTYPE_FLOAT64 => data.try_into().map(|d| {
//                 let val = u64::from_ne_bytes(d);
//                 Self::Float64(f64::from_bits(
//                     match (encoding & (sys::MQENC_FLOAT_MASK as sys::MQLONG)).value() {
//                         sys::MQENC_FLOAT_IEEE_REVERSED => val.reverse_bits(),
//                         _ => val,
//                     },
//                 ))
//             }),
//             sys::MQTYPE_STRING if ccsid == 1208 => Ok(Self::String(unsafe { from_utf8_unchecked(data) })),
//             sys::MQTYPE_STRING => Ok(Self::EncodedString(StringCcsid {
//                 ccsid: NonZero::new(ccsid),
//                 data: data.into(),
//             })),
//             sys::MQTYPE_NULL => Ok(Self::Null),
//             _ => Ok(Self::Data(prop_type, data.into(), Coding::NumericEncoding(encoding))),
//         }
//     }
// }

impl<L: Library<MQ: function::MQI>> Drop for Message<'_, L> {
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

#[derive(Error, Debug)]
pub enum InqError<T> {
    #[error(transparent)]
    Mq(#[from] crate::Error),
    #[error(transparent)]
    Value(T),
}

impl<'connection, L: Library<MQ: function::MQI>> Message<'connection, L> {
    pub fn new(lib: L, connection: &'connection core::ConnectionHandle, options: MqValue<MQCMHO>) -> ResultErr<Self> {
        let mqcmho = sys::MQCMHO {
            Options: options.value(),
            ..sys::MQCMHO::default()
        };
        let mq = MQFunctions(lib);
        mq.mqcrtmh(Some(connection), &mqcmho)
            .map(|handle| Self { handle, mq, connection })
    }

    pub fn inq2<P: MessagePropertyType2 + ?Sized, N: EncodedString + ?Sized, T: ?Sized>(
        &self,
        name: &N,
        options: MqMask<MQIMPO>,
        returned_name: &mut T,
    ) -> ResultCompErr<P, InqError<P::Error>> {
        let len: i32 = size_of_val(returned_name)
            .try_into()
            .expect("usize could not be converted to a MQLONG");
        let mut mqimpo = MqStruct::new(sys::MQIMPO {
            Options: options.value() | sys::MQIMPO_CONVERT_TYPE | P::MQIMPO,
            ReturnedName: sys::MQCHARV {
                VSPtr: ptr::from_mut(returned_name).cast(),
                VSLength: len,
                VSBufSize: len,
                ..sys::MQCHARV::default()
            },
            ..sys::MQIMPO::default()
        });
        let mut buffer: [u8; 2] = [0; 2];

        let mut vt = MqValue::from(P::MQTYPE);
        let name = MqStruct::from_encoded_str(name);
        let mut mqpd = MqStruct::<sys::MQPD>::default();
        let inq @ Completion(length, ..) = self.mq.mqinqmp(
            Some(self.connection),
            &self.handle,
            &mut mqimpo,
            &name,
            &mut mqpd,
            &mut vt,
            Some(&mut buffer),
        )?;

        let value = P::create_from(
            Cow::from(&buffer.as_slice()[..length.try_into().expect("MQLONG to usize")]),
            &mqimpo,
            &mqpd,
        )
        .map_err(InqError::Value)?;
        Ok(inq.map(|_| value))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inq<P: MessagePropertyType + ?Sized, N: EncodedString + ?Sized, T: ?Sized>(
        &self,
        name: &N,
        options: MqMask<MQIMPO>,
        returned_name: &mut T,
        value: Option<&mut P>,
    ) -> ResultComp<sys::MQLONG> {
        let len: sys::MQLONG = size_of_val(returned_name)
            .try_into()
            .expect("usize could not be converted to a MQLONG");
        let mut mqimpo = MqStruct::new(sys::MQIMPO {
            Options: options.value() | sys::MQIMPO_CONVERT_TYPE | P::MQIMPO,
            ReturnedName: sys::MQCHARV {
                VSPtr: ptr::from_mut(returned_name).cast(),
                VSLength: len,
                VSBufSize: len,
                ..sys::MQCHARV::default()
            },
            ..sys::MQIMPO::default()
        });

        let mut vt = MqValue::from(P::MQTYPE);
        let name = MqStruct::from_encoded_str(name);
        let mut mqpd = MqStruct::<sys::MQPD>::default();

        Ok(match value {
            Some(v) => {
                let inq = self.mq.mqinqmp(
                    Some(self.connection),
                    &self.handle,
                    &mut mqimpo,
                    &name,
                    &mut mqpd,
                    &mut vt,
                    Some(v.value_mut()),
                )?;
                v.receive_mqpd(&mqpd);
                inq
            }
            None => self.mq.mqinqmp(
                Some(self.connection),
                &self.handle,
                &mut mqimpo,
                &name,
                &mut mqpd,
                &mut vt,
                None::<&mut P::ValueType>,
            )?,
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

    // pub fn inq_properties(&self, name: impl Into<String>) -> MsgProp<'_, L> {
    //     MsgProp {
    //         name: name.into(),
    //         message: self,
    //         inq_prop_opts: MqStruct::<sys::MQIMPO>::default(),
    //     }
    // }
}

impl<L: Library<MQ: function::MQI>, H> QueueManagerShare<'_, L, H> {
    pub fn put<B>(
        &self,
        mqod: &mut sys::MQOD,
        mqmd: Option<&mut impl MQMD>,
        pmo: &mut sys::MQPMO,
        body: &B,
    ) -> ResultComp<()> {
        self.mq().mqput1(self.handle(), mqod, mqmd, pmo, body)
    }
}
