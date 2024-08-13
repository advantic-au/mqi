use crate::{
    core::{self, ObjectHandle},
    sys, ResultComp,
};

use super::{Conn, MqStruct, MqiAttr, MqiOption, MqiValue, Object};
use crate::ResultCompErrExt as _;
pub type SubscribeOption<'a, C> = (MqStruct<'a, sys::MQSD>, ObjectHandle, Option<Object<C>>);

pub struct Subscription<C> {
    handle: core::SubscriptionHandle,
    connection: C,
}

impl<C: Conn + Copy> Subscription<C> {
    pub fn subscribe<R>(connection: C, options: impl for<'a> MqiOption<'a, SubscribeOption<'a, C>>) -> ResultComp<R>
    where
        R: for<'a> MqiValue<'a, Self, Param<'a> = SubscribeOption<'a, C>>,
    {
        let mut so = (MqStruct::default(), ObjectHandle::from(sys::MQHO_NONE), None);
        options.apply_param(&mut so);

        R::from_mqi(&mut so, |param| {
            let original_raw_handle = unsafe { param.1.raw_handle() };
            let result = connection
                .mq()
                .mqsub(connection.handle(), &mut param.0, &mut param.1)
                .map_completion(|sub_handle| Self { handle: sub_handle, connection });
            let new_raw_handle = unsafe { param.1.raw_handle() };

            let obj_handle = match (original_raw_handle, new_raw_handle) {
                (_, sys::MQHO_NONE) => None,
                (original, new) if original == new => None,
                (_, new) => Some(unsafe { Object::from_parts(connection, ObjectHandle::from(new)) })
            };
            
            param.2 = obj_handle;

            result
            
        })
    }
}

impl<'b, C: Conn> MqiValue<'b, Self> for Subscription<C> {
    type Param<'a> = SubscribeOption<'a, C>;

    fn from_mqi<F: FnOnce(&mut Self::Param<'b>) -> ResultComp<Self>>(
        param: &mut Self::Param<'b>,
        subscribe: F,
    ) -> ResultComp<Self> {
        subscribe(param)
    }
}

impl<'b, C: Conn> MqiAttr<SubscribeOption<'b, C>> for Option<Object<C>> {
    fn apply_param<Y, F: for<'a> FnOnce(&'a mut SubscribeOption<'b, C>) -> Y>(
        param: &mut SubscribeOption<'b, C>,
        subscribe: F,
    ) -> (Self, Y) {
        let result = subscribe(param);
        (param.2.take(), result)
    }
}
