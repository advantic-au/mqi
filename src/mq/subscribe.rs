use crate::{
    core::{self, values, ObjectHandle},
    sys, Error, MqMask, ResultComp, ResultCompErr,
};

use super::{open_options::ObjectString, Conn, ConsumeValue2, EncodedString, ExtractValue2, MqStruct, MqiOption, Object};
use crate::ResultCompErrExt as _;

#[derive(Debug)]
pub struct Subscription<C: Conn> {
    handle: core::SubscriptionHandle,
    connection: C,
    close_options: MqMask<values::MQCO>,
}

#[derive(Debug)]
pub struct SubscribeParam<'a, C: Conn> {
    pub sd: MqStruct<'a, sys::MQSD>,
    pub close_options: MqMask<values::MQCO>,
    handles: (sys::MQLONG, Option<Object<C>>),
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

pub trait SubscribeValue<C: Conn>: for<'a> ConsumeValue2<SubscribeParam<'a, C>, Subscription<C>> {}
pub trait SubscribeOption<'so, C: Conn>: MqiOption<SubscribeParam<'so, C>> {}

// Blanket implementation for SubscribeValue<C>
impl<T, C: Conn> SubscribeValue<C> for T where for<'so> Self: ConsumeValue2<SubscribeParam<'so, C>, Subscription<C>> {}

impl<'so, C: Conn, A: MqiOption<SubscribeParam<'so, C>>> SubscribeOption<'so, C> for A {}

impl<C: Conn + Clone> Subscription<C> {
    pub fn subscribe<'so, R>(
        connection: C,
        subscribe_option: impl SubscribeOption<'so, C>,
    ) -> ResultCompErr<R, <R as ConsumeValue2<SubscribeParam<'so, C>, Self>>::Error>
    where
        R: SubscribeValue<C>,
    {
        let mut so = SubscribeParam {
            close_options: MqMask::default(),
            sd: MqStruct::default(),
            handles: (sys::MQHO_NONE, None),
        };

        subscribe_option.apply_param(&mut so);

        R::consume(&mut so, |param| {
            let mut obj_handle = ObjectHandle::from(param.handles.0);
            connection
                .mq()
                .mqsub(connection.handle(), &mut param.sd, &mut obj_handle)
                .map_completion(|sub_handle| {
                    // Create an Object if there is a unique one issued from the call
                    let new_raw_handle = unsafe { obj_handle.raw_handle() };
                    param.handles.1 = match (param.handles.0, new_raw_handle) {
                        (_, sys::MQHO_NONE) => None,
                        (original, new) if original == new => None,
                        (_, new) => Some(unsafe { Object::from_parts(connection.clone(), ObjectHandle::from(new)) }),
                    };
                    Self {
                        handle: sub_handle,
                        connection,
                        close_options: param.close_options,
                    }
                })
        })
    }
}

impl<'a, C: Conn, T: EncodedString + ?Sized> MqiOption<SubscribeParam<'a, C>> for ObjectString<&'a T> {
    #[inline]
    fn apply_param(self, param: &mut SubscribeParam<'a, C>) {
        param.sd.attach_object_string(self.0);
    }
}

impl<C: Conn> MqiOption<SubscribeParam<'_, C>> for &Object<C> {
    #[inline]
    fn apply_param(self, param: &mut SubscribeParam<'_, C>) {
        param.handles.0 = unsafe { self.raw_handle() };
    }
}

// Set the close options for the subscription when opening
impl<C: Conn> MqiOption<SubscribeParam<'_, C>> for MqMask<values::MQCO> {
    #[inline]
    fn apply_param(self, param: &mut SubscribeParam<'_, C>) {
        param.close_options |= self;
    }
}

impl<C: Conn> MqiOption<SubscribeParam<'_, C>> for MqMask<values::MQSO> {
    #[inline]
    fn apply_param(self, param: &mut SubscribeParam<'_, C>) {
        param.sd.Options |= self.value();
    }
}

impl<C: Conn, P> ConsumeValue2<P, Self> for Subscription<C> {
    type Error = Error;

    fn consume<F>(param: &mut P, subscribe: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut P) -> ResultComp<Self>,
    {
        subscribe(param)
    }
}

// Return the optional handle of a managed subscription
impl<'b, C: Conn, S> ExtractValue2<SubscribeParam<'b, C>, S> for Option<Object<C>> {
    #[inline]
    fn extract<F>(param: &mut SubscribeParam<'b, C>, subscribe: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut SubscribeParam<'b, C>) -> ResultComp<S>,
    {
        subscribe(param).map_completion(|state| (param.handles.1.take(), state))
    }
}
