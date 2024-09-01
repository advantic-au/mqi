use crate::{core::values, sys, types, Conn, Properties, MqMask, MqStruct, ResultComp, ResultCompErrExt, MqiAttr, MqiOption};

use super::put::{PropertyAction, PutParam};

impl<C: Conn> MqiOption<PutParam<'_>> for &mut Properties<C> {
    fn apply_param(self, (.., pmo): &mut PutParam<'_>) {
        pmo.Action = sys::MQACTP_NEW;
        pmo.OriginalMsgHandle = unsafe { self.handle().raw_handle() };
    }
}

impl MqiOption<PutParam<'_>> for MqMask<values::MQPMO> {
    fn apply_param(self, (.., pmo): &mut PutParam<'_>) {
        pmo.Options |= self.value();
    }
}

impl<'handle, C: Conn> MqiOption<PutParam<'_>> for PropertyAction<'handle, C> {
    fn apply_param(self, (.., pmo): &mut PutParam<'_>) {
        match self {
            PropertyAction::Reply(original, new) => {
                pmo.Action = sys::MQACTP_REPLY;
                pmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
                pmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
            }
            PropertyAction::Forward(original, new) => {
                pmo.Action = sys::MQACTP_FORWARD;
                pmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
                pmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
            }
            PropertyAction::Report(original, new) => {
                pmo.Action = sys::MQACTP_REPORT;
                pmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
                pmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
            }
        }
    }
}

impl<'b, S> MqiAttr<PutParam<'b>, S> for MqStruct<'static, sys::MQMD2> {
    #[inline]
    fn extract<F>(param: &mut PutParam<'b>, put: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut PutParam<'b>) -> ResultComp<S>,
    {
        put(param).map_completion(|state| {
            let (md, ..) = param;
            (md.clone(), state)
        })
    }
}

impl<'b, S> MqiAttr<PutParam<'b>, S> for types::MessageId {
    #[inline]
    fn extract<F>(param: &mut PutParam<'b>, put: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut PutParam<'b>) -> ResultComp<S>,
    {
        put(param).map_completion(|state| {
            let (md, ..) = param;
            (Self(md.MsgId), state)
        })
    }
}

impl<'b, S> MqiAttr<PutParam<'b>, S> for types::CorrelationId {
    #[inline]
    fn extract<F>(param: &mut PutParam<'b>, put: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut PutParam<'b>) -> ResultComp<S>,
    {
        put(param).map_completion(|state| {
            let (md, ..) = param;
            (Self(md.CorrelId), state)
        })
    }
}

impl<'b, S> MqiAttr<PutParam<'b>, S> for Option<types::UserIdentifier> {
    #[inline]
    fn extract<F>(param: &mut PutParam<'b>, put: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut PutParam<'b>) -> ResultComp<S>,
    {
        put(param).map_completion(|state| {
            let (md, ..) = param;
            (types::UserIdentifier::new(md.UserIdentifier), state)
        })
    }
}
