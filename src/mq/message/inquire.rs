use std::{marker::PhantomData, num::NonZero, ptr};

use libmqm_sys::function;

use crate::core::values::{MQCMHO, MQDMPO, MQIMPO, MQSMPO, MQTYPE};
use crate::core::MessageHandle;
use crate::property::{InqPropertyType, NameUsage, SetPropertyType};
use crate::{core, sys, Buffer as _, Completion, Conn, InqBuffer};

use crate::{EncodedString, Error, MqMask, MqStruct, MqValue, ResultCompErrExt};
use crate::{ResultComp, ResultCompErr, ResultErr};

#[derive(Debug)]
pub struct Message<C: Conn> {
    handle: core::MessageHandle,
    connection: C,
}

impl<C: Conn> Drop for Message<C> {
    fn drop(&mut self) {
        let mqdmho = sys::MQDMHO::default();
        let _ = self
            .connection
            .mq()
            .mqdltmh(Some(self.connection.handle()), &mut self.handle, &mqdmho);
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
        let rn_ref = rn.as_mut();
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
            Some(value.as_mut()),
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

pub struct MsgPropIter<'name, 'message, P, N: EncodedString + ?Sized, C: Conn> {
    name: &'name N,
    message: &'message Message<C>,
    options: MqMask<MQIMPO>,
    _marker: PhantomData<P>,
}

impl<P: InqPropertyType, N: EncodedString + ?Sized, C: Conn> Iterator for MsgPropIter<'_, '_, P, N, C> {
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

impl<C: Conn> Message<C> {
    pub const fn handle(&self) -> &MessageHandle {
        &self.handle
    }

    pub fn new(connection: C, options: MqValue<MQCMHO>) -> ResultErr<Self> {
        let mqcmho = sys::MQCMHO {
            Options: options.value(),
            ..sys::MQCMHO::default()
        };
        connection
            .mq()
            .mqcrtmh(Some(connection.handle()), &mqcmho)
            .map(|handle| Self { handle, connection })
    }

    pub fn property_iter<'message, 'name, P: InqPropertyType + ?Sized, N: EncodedString + ?Sized>(
        &'message self,
        name: &'name N,
        options: MqMask<MQIMPO>,
    ) -> MsgPropIter<'name, 'message, P, N, C> {
        MsgPropIter {
            name,
            message: self,
            options: options | sys::MQIMPO_INQ_NEXT,
            _marker: PhantomData,
        }
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
                VSBufSize: name
                    .as_ref()
                    .len()
                    .try_into()
                    .expect("length of buffer must always fit in an MQLONG"),
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
            self.connection.mq(),
            Some(self.connection.handle()),
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

    pub fn delete_property(&self, name: &(impl EncodedString + ?Sized), options: MqValue<MQDMPO>) -> ResultComp<()> {
        let mut mqdmpo = MqStruct::<sys::MQDMPO>::default();
        mqdmpo.Options = options.value();

        let name_mqcharv = MqStruct::from_encoded_str(name);

        self.connection
            .mq()
            .mqdltmp(Some(self.connection.handle()), &self.handle, &mqdmpo, &name_mqcharv)
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
        self.connection.mq().mqsetmp(
            Some(self.connection.handle()),
            &self.handle,
            &mqsmpo,
            &name_mqcharv,
            &mut mqpd,
            value_type,
            data,
        )
    }
}
