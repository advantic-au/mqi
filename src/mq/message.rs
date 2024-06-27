use std::num::NonZero;
use std::ptr;

use libmqm_sys::function;

use crate::core;
use crate::core::values::MQCMHO;
use crate::core::values::MQENC;
use crate::core::values::MQSMPO;
use crate::core::values::MQTYPE;
use crate::core::Library;
use crate::core::MQFunctions;
use crate::sys;
use crate::Completion;
use crate::EncodedString;
use crate::MqMask;
use crate::MqStruct;
use crate::MqValue;
use crate::StringCcsid;
use crate::StructBuilder;
use crate::MQMD;
use crate::{Error, ResultComp, ResultErr};

use super::QueueManagerShare;

pub struct Message<'ch, L: Library<MQ: function::MQI>> {
    handle: core::MessageHandle,
    mq: MQFunctions<L>,
    connection: &'ch core::ConnectionHandle,
}

pub struct MsgPropIter<'mh, L: Library<MQ: function::MQI>> {
    name: String,
    message: &'mh Message<'mh, L>,
    inq_prop_opts: MqStruct<'static, sys::MQIMPO>,
}

type MsgPropIterItem = ResultComp<(StringCcsid, Value)>;

impl<L: Library<MQ: function::MQI>> Iterator for MsgPropIter<'_, L> {
    type Item = MsgPropIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        fn next_result<A: Library<MQ: function::MQI>>(it: &mut MsgPropIter<A>) -> MsgPropIterItem {
            let name_ref: &str = &it.name;
            let name_mqcharv = MqStruct::<sys::MQCHARV>::from_encoded_str(name_ref);
            let mut returning_name = Vec::<u8>::with_capacity(page_size::get());
            let rn_mqcharv = &mut it.inq_prop_opts.ReturnedName;
            rn_mqcharv.VSPtr = returning_name.spare_capacity_mut().as_mut_ptr().cast();
            rn_mqcharv.VSOffset = 0;
            rn_mqcharv.VSBufSize = returning_name
                .capacity()
                .try_into()
                .expect("page size exceeds maximum positive MQLONG");
            rn_mqcharv.VSLength = 0;
            rn_mqcharv.VSCCSID = 0;

            let mut value = Vec::<u8>::with_capacity(page_size::get());
            let mut prop_type = MqValue::from(sys::MQTYPE_AS_SET);
            let mut prop_desc = MqStruct::<sys::MQPD>::default();

            let inq_length = it.message.mq.mqinqmp(
                Some(it.message.connection),
                &it.message.handle,
                &mut it.inq_prop_opts,
                &name_mqcharv,
                &mut prop_desc,
                &mut prop_type,
                value.spare_capacity_mut(),
            )?;
            let Completion(length, ..) = inq_length;
            unsafe {
                value.set_len(
                    length
                        .try_into()
                        .expect("property length exceeds maximum positive MQLONG"),
                );
                returning_name.set_len(
                    it.inq_prop_opts
                        .ReturnedName
                        .VSLength
                        .try_into()
                        .expect("property name length exceeds maximum positive MQLONG"),
                );
            }
            it.inq_prop_opts.Options |= sys::MQIMPO_INQ_NEXT;

            Ok(inq_length.map(|_| {
                (
                    StringCcsid {
                        ccsid: NonZero::new(it.inq_prop_opts.ReturnedName.VSCCSID),
                        data: returning_name,
                    },
                    Value::from_parts(
                        &value,
                        prop_type,
                        it.inq_prop_opts.ReturnedCCSID,
                        MqMask::from(it.inq_prop_opts.RequestedEncoding),
                    ),
                )
            }))
        }

        match next_result(self) {
            Err(Error(cc, _, rc)) if cc == sys::MQCC_FAILED && rc == sys::MQRC_PROPERTY_NOT_AVAILABLE => None,
            result => Some(result),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Boolean(bool),
    Bytes(Vec<u8>),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    String(String),
    EncodedString(StringCcsid),
    Null,
    Data(MqValue<MQTYPE>, Vec<u8>, Coding),
}

#[derive(Clone, Copy, Debug)]
pub enum Coding {
    NumericEncoding(MqMask<MQENC>),
    TextCCSID(sys::MQLONG),
}

/// Create message handle option value
impl Value {
    #[must_use]
    #[allow(clippy::cast_possible_wrap)] // Masks are unsigned.
    pub fn from_parts(
        data: &[u8],
        prop_type: MqValue<core::values::MQTYPE>,
        ccsid: sys::MQLONG,
        encoding: MqMask<MQENC>,
    ) -> Self {
        static ENC_NATIVE_INTEGER: sys::MQLONG = sys::MQENC_INTEGER_MASK as sys::MQLONG & sys::MQENC_NATIVE;
        static ENC_NATIVE_FLOAT: sys::MQLONG = sys::MQENC_FLOAT_MASK as sys::MQLONG & sys::MQENC_NATIVE;
        match prop_type.value() {
            sys::MQTYPE_BOOLEAN => Ok(Self::Boolean(data[0] != 0)),
            sys::MQTYPE_BYTE_STRING => Ok(Self::Bytes(data.to_owned())),
            sys::MQTYPE_INT16 if (encoding & ENC_NATIVE_INTEGER) != 0 => {
                data.try_into().map(|d| Self::Int16(i16::from_ne_bytes(d)))
            }
            sys::MQTYPE_INT32 if (encoding & ENC_NATIVE_INTEGER) != 0 => {
                data.try_into().map(|d| Self::Int32(i32::from_ne_bytes(d)))
            }
            sys::MQTYPE_INT64 if (encoding & ENC_NATIVE_INTEGER) != 0 => {
                data.try_into().map(|d| Self::Int64(i64::from_ne_bytes(d)))
            }
            sys::MQTYPE_FLOAT32 if (encoding & ENC_NATIVE_FLOAT) != 0 => {
                data.try_into().map(|d| Self::Float32(f32::from_ne_bytes(d)))
            }
            sys::MQTYPE_FLOAT64 if (encoding & ENC_NATIVE_FLOAT) != 0 => {
                data.try_into().map(|d| Self::Float64(f64::from_ne_bytes(d)))
            }
            sys::MQTYPE_STRING if ccsid == 1208 => Ok(Self::String(String::from_utf8_lossy(data).into_owned())),
            sys::MQTYPE_STRING => Ok(Self::EncodedString(StringCcsid {
                ccsid: NonZero::new(ccsid),
                data: data.into(),
            })),
            sys::MQTYPE_NULL => Ok(Self::Null),
            _ => Ok(Self::Data(prop_type, data.into(), Coding::NumericEncoding(encoding))),
        }
        .expect("Unexpected data length")
    }
}

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

    pub fn inq_properties(&self, name: impl Into<String>) -> MsgPropIter<'_, L> {
        MsgPropIter {
            name: name.into(),
            message: self,
            inq_prop_opts: MqStruct::<sys::MQIMPO>::default(),
        }
    }
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
