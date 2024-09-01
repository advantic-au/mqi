use std::{marker::PhantomData, num::NonZero, ptr};

use libmqm_sys::function;

use crate::core::values::{MQCMHO, MQDMPO, MQIMPO, MQSMPO, MQTYPE};
use crate::core::MessageHandle;
use crate::properties_options::{NameUsage, PropertyValue, PropertyParam, PropertyState, SetProperty};
use crate::{core, sys, Buffer as _, Completion, Conn, InqBuffer};

use crate::{EncodedString, Error, MqMask, MqStruct, MqValue, ResultCompErrExt};
use crate::{ResultComp, ResultCompErr, ResultErr};

#[derive(Debug)]
pub struct Properties<C: Conn> {
    handle: core::MessageHandle,
    connection: C,
}

impl<C: Conn> Drop for Properties<C> {
    fn drop(&mut self) {
        let mqdmho = sys::MQDMHO::default();

        if self.handle.is_deleteable() {
            let _ = self
                .connection
                .mq()
                .mqdltmh(Some(self.connection.handle()), &mut self.handle, &mqdmho);
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
    message: &'message Properties<C>,
    options: MqMask<MQIMPO>,
    _marker: PhantomData<P>,
}

impl<P: PropertyValue, N: EncodedString + ?Sized, C: Conn> Iterator for MsgPropIter<'_, '_, P, N, C> {
    type Item = ResultCompErr<P, Error>;

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

impl<C: Conn> Properties<C> {
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

    pub fn property_iter<'message, 'name, P, N>(
        &'message self,
        name: &'name N,
        options: MqMask<MQIMPO>,
    ) -> MsgPropIter<'name, 'message, P, N, C>
    where
        P: PropertyValue + ?Sized,
        N: EncodedString + ?Sized,
    {
        MsgPropIter {
            name,
            message: self,
            options: options | sys::MQIMPO_INQ_NEXT,
            _marker: PhantomData,
        }
    }

    pub fn property<P>(&self, name: &(impl EncodedString + ?Sized), options: MqMask<MQIMPO>) -> ResultCompErr<Option<P>, Error>
    where
        P: PropertyValue + ?Sized,
    {
        const DEFAULT_BUF_SIZE: usize = 1024;
        let mut val_return_buffer = [0; DEFAULT_BUF_SIZE]; // Returned value buffer
        let mut name_return_buffer = [0; DEFAULT_BUF_SIZE]; // Returned name buffer

        let mut property_not_available = false;

        let mut param = PropertyParam {
            impo: MqStruct::new(sys::MQIMPO {
                Options: options.value(),
                ..sys::MQIMPO::default()
            }),
            ..PropertyParam::default()
        };

        let mut inq_value_buffer = InqBuffer::Slice(val_return_buffer.as_mut_slice());
        inq_value_buffer = match P::max_value_size() {
            Some(max_size) => inq_value_buffer.truncate(max_size.into()),
            None => inq_value_buffer,
        };
        let name = MqStruct::from_encoded_str(name);

        let result = P::consume(&mut param, |param| {
            let mut inq_name_buffer = match param.name_required {
                NameUsage::Ignored => None,
                used => {
                    let buf = InqBuffer::Slice(name_return_buffer.as_mut_slice());
                    Some(match used {
                        NameUsage::MaxLength(length) => buf.truncate(length.into()),
                        _ => buf,
                    })
                }
            };
            param.impo.ReturnedName = inq_name_buffer.as_mut().map_or_else(Default::default, |name| sys::MQCHARV {
                VSPtr: ptr::from_mut(&mut *name).cast(),
                VSBufSize: name
                    .as_ref()
                    .len()
                    .try_into()
                    .expect("length of buffer must always fit in an MQLONG"),
                ..sys::MQCHARV::default()
            });

            let mqi_inqmp = inqmp(
                self.connection.mq(),
                Some(self.connection.handle()),
                &self.handle,
                &mut param.impo,
                &name,
                &mut param.mqpd,
                &mut param.value_type,
                inq_value_buffer,
                P::max_value_size(),
                inq_name_buffer,
                param.name_required.into(),
            )
            .map_err(Into::into) // Convert the error into an ordinary MQ error
            .map_completion(|(value, name)| PropertyState {
                name: name.map(Into::into),
                value: value.into(),
            });

            property_not_available = mqi_inqmp
                .as_ref()
                .is_err_and(|&Error(cc, _, rc)| cc == sys::MQCC_FAILED && rc == sys::MQRC_PROPERTY_NOT_AVAILABLE);

            mqi_inqmp
        });

        if property_not_available {
            Ok(Completion::new(None))
        } else {
            result.map_completion(Some).map_err(Into::into)
        }
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
        value: &(impl SetProperty + ?Sized),
        location: MqValue<MQSMPO>,
    ) -> ResultComp<()> {
        let mut mqpd = MqStruct::<sys::MQPD>::default();
        let mut mqsmpo = MqStruct::<sys::MQSMPO>::default();
        mqsmpo.Options = location.value();
        let (data, value_type) = value.apply_mqsetmp(&mut mqpd, &mut mqsmpo);

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

    pub fn close(self) -> ResultErr<()> {
        let mut s = self;
        let mqdmho = sys::MQDMHO::default();
        s.connection.mq().mqdltmh(Some(s.connection.handle()), &mut s.handle, &mqdmho)
    }
}
