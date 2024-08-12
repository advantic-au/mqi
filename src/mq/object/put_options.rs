use crate::{core::values, sys, types, Conn, Message, MqMask, MqStruct, MqiAttr, MqiOption};

use super::put::{Properties, PutParam};

impl<C: Conn> MqiOption<'_, PutParam<'_>> for &mut Message<C> {
    fn apply_param(&self, (.., pmo): &mut PutParam<'_>) {
        pmo.Action = sys::MQACTP_NEW;
        pmo.OriginalMsgHandle = unsafe { self.handle().raw_handle() };
    }
}

impl MqiOption<'_, PutParam<'_>> for MqMask<values::MQPMO> {
    fn apply_param(&self, (.., pmo): &mut PutParam<'_>) {
        pmo.Options |= self.value();
    }
}

impl<'handle, C: Conn> MqiOption<'_, PutParam<'_>> for Properties<'handle, C> {
    fn apply_param(&self, (.., pmo): &mut PutParam<'_>) {
        match self {
            Properties::Reply(original, new) => {
                pmo.Action = sys::MQACTP_REPLY;
                pmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
                pmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
            }
            Properties::Forward(original, new) => {
                pmo.Action = sys::MQACTP_FORWARD;
                pmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
                pmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
            }
            Properties::Report(original, new) => {
                pmo.Action = sys::MQACTP_REPORT;
                pmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
                pmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
            }
        }
    }
}

impl<'b> MqiAttr<PutParam<'b>> for MqStruct<'static, sys::MQMD2> {
    fn apply_param<Y, F: for<'a> FnOnce(&'a mut PutParam<'b>) -> Y>(param: &mut PutParam<'b>, put: F) -> (Self, Y) {
        let put_result = put(param);
        let (md, ..) = param;
        (md.clone(), put_result)
    }
}

impl<'b> MqiAttr<PutParam<'b>> for types::MessageId {
    #[inline]
    fn apply_param<Y, F: for<'a> FnOnce(&'a mut PutParam<'b>) -> Y>(param: &mut PutParam<'b>, put: F) -> (Self, Y) {
        let put_result = put(param);
        (Self(param.0.MsgId), put_result)
    }
}

impl<'b> MqiAttr<PutParam<'b>> for types::CorrelationId {
    #[inline]
    fn apply_param<Y, F: for<'a> FnOnce(&'a mut PutParam<'b>) -> Y>(param: &mut PutParam<'b>, put: F) -> (Self, Y) {
        let put_result = put(param);
        (Self(param.0.CorrelId), put_result)
    }
}

impl<'b> MqiAttr<PutParam<'b>> for Option<types::UserIdentifier> {
    #[inline]
    fn apply_param<Y, F: for<'a> FnOnce(&'a mut PutParam<'b>) -> Y>(param: &mut PutParam<'b>, put: F) -> (Self, Y) {
        let put_result = put(param);
        (types::UserIdentifier::new(param.0.UserIdentifier), put_result)
    }
}
