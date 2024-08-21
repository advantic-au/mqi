use crate::{
    core::{self, values, ObjectHandle},
    sys, MqMask, ResultComp,
};

use super::{open_options::ObjectString, Conn, EncodedString, MqStruct, MqiAttr, MqiOption, MqiValue, Object};
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

pub trait SubscribeValue<T, C: Conn>: for<'a> MqiValue<T, Param<'a> = SubscribeParam<'a, C>> {}
pub trait SubscribeOption<'so, C: Conn>: MqiOption<SubscribeParam<'so, C>> {}

impl<T, C: Conn, A: for<'a> MqiValue<T, Param<'a> = SubscribeParam<'a, C>>> SubscribeValue<T, C> for A {}
impl<'so, C: Conn, A: MqiOption<SubscribeParam<'so, C>>> SubscribeOption<'so, C> for A {}

impl<C: Conn + Clone> Subscription<C> {
    pub fn subscribe<'so, R>(connection: C, subscribe_option: impl SubscribeOption<'so, C>) -> ResultComp<R>
    where
        R: SubscribeValue<Self, C>,
    {
        let mut so = SubscribeParam {
            close_options: MqMask::default(),
            sd: MqStruct::default(),
            handles: (sys::MQHO_NONE, None),
        };

        subscribe_option.apply_param(&mut so);

        R::from_mqi(&mut so, |param| {
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

//  `Subscription` is the primary value for a subscribe action
impl<C: Conn> MqiValue<Self> for Subscription<C> {
    type Param<'a> = SubscribeParam<'a, C>;

    #[inline]
    fn from_mqi<F: FnOnce(&mut Self::Param<'_>) -> ResultComp<Self>>(
        param: &mut Self::Param<'_>,
        subscribe: F,
    ) -> ResultComp<Self> {
        subscribe(param)
    }
}

// Return the optional handle of a managed subscription
impl<'b, C: Conn> MqiAttr<SubscribeParam<'b, C>> for Option<Object<C>> {
    #[inline]
    fn from_mqi<Y, F: FnOnce(&mut SubscribeParam<'b, C>) -> Y>(param: &mut SubscribeParam<'b, C>, subscribe: F) -> (Self, Y) {
        let result = subscribe(param);
        (param.handles.1.take(), result)
    }
}
