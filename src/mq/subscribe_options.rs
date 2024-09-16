use crate::{core::values, Error, MqiAttr, MqiOption, MqiValue, ResultComp, ResultCompErr};

use super::{open_options::ObjectString, Connection, EncodedString, Object, SubscribeParam, SubscribeState, Subscription};
use crate::ResultCompErrExt as _;

impl<'so, T: EncodedString + ?Sized> MqiOption<SubscribeParam<'so>> for ObjectString<&'so T> {
    #[inline]
    fn apply_param(self, param: &mut SubscribeParam<'so>) {
        param.sd.attach_object_string(self.0);
    }
}

impl<C: Connection> MqiOption<SubscribeParam<'_>> for &Object<C> {
    #[inline]
    fn apply_param(self, param: &mut SubscribeParam<'_>) {
        param.provided_object = unsafe { self.handle.raw_handle() };
    }
}

// Set the close options for the subscription when opening
impl MqiOption<SubscribeParam<'_>> for values::MQCO {
    #[inline]
    fn apply_param(self, param: &mut SubscribeParam<'_>) {
        param.close_options |= self;
    }
}

impl MqiOption<SubscribeParam<'_>> for values::MQSO {
    #[inline]
    fn apply_param(self, param: &mut SubscribeParam<'_>) {
        param.sd.Options |= self.value();
    }
}

impl<C: Connection, P> MqiValue<P, SubscribeState<C>> for Subscription<C> {
    type Error = Error;

    fn consume<F>(param: &mut P, subscribe: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut P) -> ResultComp<SubscribeState<C>>,
    {
        subscribe(param).map_completion(|state| state.subscription)
    }
}

// Return the optional handle of a managed subscription
impl<C: Connection, P> MqiAttr<P, SubscribeState<C>> for Option<Object<C>> {
    #[inline]
    fn extract<F>(param: &mut P, subscribe: F) -> ResultComp<(Self, SubscribeState<C>)>
    where
        F: FnOnce(&mut P) -> ResultComp<SubscribeState<C>>,
    {
        subscribe(param).map_completion(|mut state| (state.object.take(), state))
    }
}
