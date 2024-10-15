use crate::{macros::all_option_tuples, prelude::*, sys, types, values, Conn, MqStruct, MqiAttr, Properties, ResultComp};

use super::{
    put::{PutOption, PutParam},
    Object,
};

#[derive(Debug, Clone, Copy)]
pub struct Context<T>(pub T);

all_option_tuples!(PutOption, PutParam);

#[derive(Debug)]
pub enum PropertyAction<'handle, C: Conn> {
    Reply(&'handle Properties<C>, &'handle mut Properties<C>),
    Forward(&'handle Properties<C>, &'handle mut Properties<C>),
    Report(&'handle Properties<C>, &'handle mut Properties<C>),
}

impl<C: Conn> PutOption for Context<&Object<C>> {
    fn apply_param(self, (.., pmo): &mut PutParam) {
        pmo.Context = unsafe { self.0.handle.raw_handle() };
    }
}

impl<C: Conn> PutOption for &mut Properties<C> {
    fn apply_param(self, (.., pmo): &mut PutParam) {
        pmo.Action = sys::MQACTP_NEW;
        pmo.OriginalMsgHandle = unsafe { self.handle().raw_handle() };
    }
}

impl PutOption for values::MQPMO {
    fn apply_param(self, (.., pmo): &mut PutParam) {
        pmo.Options |= self.value();
    }
}

impl PutOption for MqStruct<'static, sys::MQMD2> {
    fn apply_param(self, param: &mut PutParam) {
        self.clone_into(&mut param.0);
    }
}

impl<'handle, C: Conn> PutOption for PropertyAction<'handle, C> {
    fn apply_param(self, (.., pmo): &mut PutParam) {
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
            (Self(md.MsgId.into()), state)
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
            (Self(md.CorrelId.into()), state)
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
