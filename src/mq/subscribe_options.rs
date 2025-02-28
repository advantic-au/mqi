use crate::{macros::all_option_tuples, values, Error, ResultComp, ResultCompErr};

use super::{
    open_options::ObjectString, Conn, EncodedString, Object, SubscribeAttr, SubscribeOption, SubscribeParam,
    SubscribeRequestOption, SubscribeRequestParam, SubscribeState, SubscribeValue, Subscription,
};
use crate::prelude::*;

all_option_tuples!('so, SubscribeOption, SubscribeParam<'so>);
all_option_tuples!(SubscribeRequestOption, SubscribeRequestParam);

impl<'so, T: EncodedString + ?Sized> SubscribeOption<'so> for ObjectString<&'so T> {
    #[inline]
    fn apply_param(&self, param: &mut SubscribeParam<'so>) {
        param.sd.attach_object_string(self.0);
    }
}

impl<C: Conn> SubscribeOption<'_> for &Object<C> {
    #[inline]
    fn apply_param(&self, param: &mut SubscribeParam) {
        param.provided_object = unsafe { self.handle.raw_handle() };
    }
}

// Set the close options for the subscription when opening
impl SubscribeOption<'_> for values::MQCO {
    #[inline]
    fn apply_param(&self, param: &mut SubscribeParam) {
        param.close_options |= *self;
    }
}

impl SubscribeOption<'_> for values::MQSO {
    #[inline]
    fn apply_param(&self, param: &mut SubscribeParam) {
        param.sd.Options |= self.value();
    }
}

impl SubscribeRequestOption for values::MQSR {
    #[inline]
    fn apply_param(&self, param: &mut super::SubscribeRequestParam) {
        param.sr = *self;
    }
}

impl SubscribeRequestOption for values::MQSRO {
    fn apply_param(&self, param: &mut super::SubscribeRequestParam) {
        param.sro.Options |= self.0;
    }
}

impl<C: Conn> SubscribeValue<C> for Subscription<C> {
    type Error = Error;

    #[inline]
    fn consume<'so, F>(param: &mut SubscribeParam<'so>, subscribe: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut SubscribeParam<'so>) -> ResultComp<SubscribeState<C>>,
    {
        subscribe(param).map_completion(|state| state.subscription)
    }
}

// Return the optional handle of a managed subscription
impl<C: Conn> SubscribeAttr<C> for Option<Object<C>> {
    #[inline]
    fn extract<'so, F>(param: &mut SubscribeParam<'so>, subscribe: F) -> ResultComp<(Self, SubscribeState<C>)>
    where
        F: FnOnce(&mut SubscribeParam<'so>) -> ResultComp<SubscribeState<C>>,
    {
        subscribe(param).map_completion(|mut state| (state.object.take(), state))
    }
}

#[expect(unused_parens)]
mod impl_subscribe {
    use super::{SubscribeValue, SubscribeAttr, SubscribeState, SubscribeParam};
    use crate::macros::all_multi_tuples;
    use crate::{Conn, ResultCompErr, ResultComp};
    use crate::prelude::*;

    macro_rules! impl_subscribevalue_tuple {
        ([$first:ident, $($ty:ident),*]) => {
            #[diagnostic::do_not_recommend]
            impl<C: Conn, $first, $($ty),*> SubscribeValue<C> for ($first, $($ty),*)
            where
                $first: SubscribeValue<C>,
                $($ty: SubscribeAttr<C>),*
            {
                type Error = $first::Error;

                #[expect(non_snake_case)]
                #[inline]
                fn consume<'sp, F>(param: &mut SubscribeParam<'sp>, mqi: F) -> ResultCompErr<Self, Self::Error>
                where
                    F: FnOnce(&mut SubscribeParam<'sp>) -> ResultComp<SubscribeState<C>>,
                {
                    let mut rest_outer = None;
                    $first::consume(param, |param| {
                        <($($ty),*) as SubscribeAttr<C>>::extract(param, mqi).map_completion(|(rest, state)| {
                            rest_outer = Some(rest);
                            state
                        })
                    })
                    .map_completion(|a| {
                        let ($($ty),*) = rest_outer.expect("rest_outer should be set by extract closure");
                        (a, $($ty),*)
                    })
                }
            }
        }
    }

    macro_rules! impl_subscribeattr_tuple {
        ([$first:ident, $($ty:ident),*]) => {
            #[diagnostic::do_not_recommend]
            impl<C: Conn, $first, $($ty),*> SubscribeAttr<C> for ($first, $($ty),*)
            where
                $first: SubscribeAttr<C>,
                $($ty: SubscribeAttr<C>),*
            {
                #[expect(non_snake_case)]
                #[inline]
                fn extract<'sp, F>(param: &mut SubscribeParam<'sp>, mqi: F) -> ResultComp<(Self, SubscribeState<C>)>
                where
                    F: FnOnce(&mut SubscribeParam<'sp>) -> ResultComp<SubscribeState<C>>
                {
                    let mut rest_outer = None;
                    $first::extract(param, |param| {
                        <($($ty),*) as SubscribeAttr<C>>::extract(param, mqi).map_completion(|(rest, state)| {
                            rest_outer = Some(rest);
                            state
                        })
                    })
                    .map_completion(|(a, s)| {
                        let ($($ty),*) = rest_outer.expect("rest_outer should be set by extract closure");
                        ((a, $($ty),*), s)
                    })
                }
            }
        }
    }

    impl<C: Conn> SubscribeValue<C> for () {
        type Error = crate::Error;

        #[inline]
        fn consume<'so, F>(param: &mut SubscribeParam<'so>, mqi: F) -> ResultCompErr<Self, Self::Error>
        where
            F: FnOnce(&mut SubscribeParam<'so>) -> ResultComp<SubscribeState<C>>,
        {
            mqi(param).map_completion(|_| ())
        }
    }

    impl<C: Conn> SubscribeAttr<C> for () {
        #[inline]
        fn extract<'so, F>(param: &mut SubscribeParam<'so>, mqi: F) -> ResultComp<(Self, SubscribeState<C>)>
        where
            F: FnOnce(&mut SubscribeParam<'so>) -> ResultComp<SubscribeState<C>>,
            Self: Sized,
        {
            mqi(param).map_completion(|state| ((), state))
        }
    }

    all_multi_tuples!(impl_subscribevalue_tuple);
    all_multi_tuples!(impl_subscribeattr_tuple);
}
