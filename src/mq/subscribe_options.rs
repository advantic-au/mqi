use crate::{values, Error, MqiAttr, MqiValue, ResultComp, ResultCompErr};

use super::{open_options::ObjectString, Conn, EncodedString, Object, SubscribeOption, SubscribeParam, SubscribeState, Subscription};
use crate::prelude::*;

impl<'so, T: EncodedString + ?Sized> SubscribeOption<'so> for ObjectString<&'so T> {
    #[inline]
    fn apply_param(self, param: &mut SubscribeParam<'so>) {
        param.sd.attach_object_string(self.0);
    }
}

impl<C: Conn> SubscribeOption<'_> for &Object<C> {
    #[inline]
    fn apply_param(self, param: &mut SubscribeParam) {
        param.provided_object = unsafe { self.handle.raw_handle() };
    }
}

// Set the close options for the subscription when opening
impl SubscribeOption<'_> for values::MQCO {
    #[inline]
    fn apply_param(self, param: &mut SubscribeParam) {
        param.close_options |= self;
    }
}

impl SubscribeOption<'_> for values::MQSO {
    #[inline]
    fn apply_param(self, param: &mut SubscribeParam) {
        param.sd.Options |= self.value();
    }
}

impl<C: Conn, P> MqiValue<P, SubscribeState<C>> for Subscription<C> {
    type Error = Error;

    fn consume<F>(param: &mut P, subscribe: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut P) -> ResultComp<SubscribeState<C>>,
    {
        subscribe(param).map_completion(|state| state.subscription)
    }
}

// Return the optional handle of a managed subscription
impl<C: Conn, P> MqiAttr<P, SubscribeState<C>> for Option<Object<C>> {
    #[inline]
    fn extract<F>(param: &mut P, subscribe: F) -> ResultComp<(Self, SubscribeState<C>)>
    where
        F: FnOnce(&mut P) -> ResultComp<SubscribeState<C>>,
    {
        subscribe(param).map_completion(|mut state| (state.object.take(), state))
    }
}
