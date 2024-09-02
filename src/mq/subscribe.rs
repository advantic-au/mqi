use crate::{
    core::{self, values, ObjectHandle},
    sys, MqiOption, MqiValue, ResultComp, ResultCompErr,
};

use super::{Conn, MqStruct, Object};
use crate::ResultCompErrExt as _;

#[derive(Debug)]
pub struct Subscription<C: Conn> {
    handle: core::SubscriptionHandle,
    connection: C,
    close_options: values::MQCO,
}

pub struct SubscribeState<C: Conn> {
    pub subscription: Subscription<C>,
    pub object: Option<Object<C>>,
}

#[derive(Debug)]
pub struct SubscribeParam<'a> {
    pub sd: MqStruct<'a, sys::MQSD>,
    pub close_options: values::MQCO,
    pub provided_object: sys::MQLONG,
}

impl<C: Conn> Subscription<C> {
    pub fn close(self) -> ResultComp<()> {
        let mut s = self;
        s.connection
            .mq()
            .mqclose(s.connection.handle(), &mut s.handle, s.close_options)
    }
}

impl<C: Conn> Drop for Subscription<C> {
    fn drop(&mut self) {
        // TODO: handle close failure
        if self.handle.is_closeable() {
            let _ = self
                .connection
                .mq()
                .mqclose(self.connection.handle(), &mut self.handle, self.close_options);
        }
    }
}

pub trait SubscribeValue<C: Conn>: for<'so> MqiValue<SubscribeParam<'so>, SubscribeState<C>> {}
pub trait SubscribeOption<'so, C: Conn>: MqiOption<SubscribeParam<'so>> {}

// Blanket implementation for SubscribeValue<C>
impl<T, C: Conn> SubscribeValue<C> for T where for<'so> Self: MqiValue<SubscribeParam<'so>, SubscribeState<C>> {}

impl<'so, C: Conn, A: MqiOption<SubscribeParam<'so>>> SubscribeOption<'so, C> for A {}

impl<C: Conn + Clone> Subscription<C> {
    pub fn subscribe<'so, R>(
        connection: C,
        subscribe_option: impl SubscribeOption<'so, C>,
    ) -> ResultCompErr<R, <R as MqiValue<SubscribeParam<'so>, SubscribeState<C>>>::Error>
    where
        R: SubscribeValue<C>,
    {
        let mut so = SubscribeParam {
            close_options: values::MQCO::default(),
            sd: MqStruct::default(),
            provided_object: sys::MQHO_NONE,
        };

        subscribe_option.apply_param(&mut so);

        R::consume(&mut so, |param| {
            let mut obj_handle = ObjectHandle::from(param.provided_object);
            connection
                .mq()
                .mqsub(connection.handle(), &mut param.sd, &mut obj_handle)
                .map_completion(|sub_handle| {
                    // Create an Object if there is a unique one issued from the call
                    let new_raw_handle = unsafe { obj_handle.raw_handle() };
                    let object = match (param.provided_object, new_raw_handle) {
                        (_, sys::MQHO_NONE) => None,
                        (original, new) if original == new => None,
                        (_, new) => Some(unsafe { Object::from_parts(connection.clone(), ObjectHandle::from(new)) }),
                    };
                    SubscribeState {
                        subscription: Self {
                            handle: sub_handle,
                            connection,
                            close_options: param.close_options,
                        },
                        object,
                    }
                })
        })
    }
}
