use std::ops::{Deref, DerefMut};
use std::{borrow::Cow, cmp::min, marker::PhantomData, num::NonZero, ptr};

use libmqm_sys::function;

use crate::core::values::{MQCMHO, MQIMPO, MQSMPO, MQTYPE};
use crate::property::{InqPropertyType, NameUsage, SetPropertyType};
use crate::{sys, QueueManagerShare, ResultCompExt};
use crate::{core, Completion};

use crate::EncodedString;
use crate::Error;
use crate::MqMask;
use crate::MqStruct;
use crate::MqValue;
use crate::ResultCompErr;
use crate::ResultCompErrExt;
use crate::MQMD;

use crate::{ResultComp, ResultErr};

pub struct Message<'ch, L: core::Library<MQ: function::MQI>> {
    handle: core::MessageHandle,
    mq: core::MQFunctions<L>,
    connection: &'ch core::ConnectionHandle,
}

impl<L: core::Library<MQ: function::MQI>> Drop for Message<'_, L> {
    fn drop(&mut self) {
        let mqdmho = sys::MQDMHO::default();
        let _ = self.mq.mqdltmh(Some(self.connection), &mut self.handle, &mqdmho);
    }
}

impl<'a> MqStruct<'a, sys::MQCHARV> {
    pub fn from_encoded_str(value: &'a (impl EncodedString + ?Sized)) -> Self {
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

impl<P: InqPropertyType, N: EncodedString + ?Sized, L: core::Library<MQ: function::MQI>> Iterator
    for MsgPropIter<'_, '_, '_, P, N, L>
{
    type Item = ResultCompErr<P, P::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.message.property::<P>(self.name, self.options) {
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

    pub fn property_iter<'m, 'n, P: InqPropertyType + ?Sized, N: EncodedString + ?Sized>(
        &'m self,
        name: &'n N,
        options: MqMask<MQIMPO>,
    ) -> MsgPropIter<'connection, 'n, 'm, P, N, L> {
        MsgPropIter {
            name,
            message: self,
            options: options | sys::MQIMPO_INQ_NEXT,
            _marker: PhantomData,
        }
    }

    pub fn extend_properties(&mut self, iter: impl IntoIterator<Item = (impl EncodedString, impl SetPropertyType)>, location: MqValue<MQSMPO>) -> Result<(), Error> {
        for (name, value) in iter {
            self.set_property(&name, &value, location).warn_as_error()?;
        }
        Ok(())
    } 

    pub fn property<P: InqPropertyType + ?Sized>(
        &self,
        name: &(impl EncodedString + ?Sized),
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

    pub fn set_property(
        &self,
        name: &(impl EncodedString + ?Sized),
        value: &(impl SetPropertyType + ?Sized),
        location: MqValue<MQSMPO>,
    ) -> ResultComp<()> {
        let mut mqpd = MqStruct::<sys::MQPD>::default();
        let mut mqsmpo = MqStruct::<sys::MQSMPO>::default();
        mqsmpo.Options = location.value();
        let (data, value_type) = value.apply_mqinqmp(&mut mqpd, &mut mqsmpo);

        let name_mqcharv = MqStruct::from_encoded_str(name);
        self.mq.mqsetmp(
            self.connection,
            &self.handle,
            &mqsmpo,
            &name_mqcharv,
            &mut mqpd,
            value_type,
            data,
        )
    }
}

impl<L: core::Library<MQ: function::MQI>, H> QueueManagerShare<'_, L, H> {
    pub fn put<B>(&self, mqod: &mut sys::MQOD, mqmd: Option<&mut impl MQMD>, pmo: &mut sys::MQPMO, body: &B) -> ResultComp<()> {
        self.mq().mqput1(self.handle(), mqod, mqmd, pmo, body)
    }
}
