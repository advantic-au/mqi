#![expect(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]

use crate::{Error, ResultComp, ResultCompErr, ResultCompErrExt};

use super::macros::all_multi_tuples;

pub trait MqiOption<P> {
    fn apply_param(self, param: &mut P);
}

pub trait MqiValue<P, S>: Sized {
    type Error: From<Error> + std::fmt::Debug;

    fn consume<F>(param: &mut P, mqi: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut P) -> ResultComp<S>;
}

pub trait MqiAttr<P, S>: Sized {
    fn extract<F>(param: &mut P, mqi: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut P) -> ResultComp<S>;
}

pub struct Callback<F>(pub F);

impl<P, F> MqiOption<P> for Callback<F>
where
    F: FnOnce(&mut P),
{
    fn apply_param(self, param: &mut P) {
        self.0(param);
    }
}

macro_rules! impl_mqivalue_tuple {
    ($first:ident, [$($ty:ident),*]) => {
        impl<P, S, $first, $($ty),*> MqiValue<P, S> for ($first, $($ty),*)
        where
            $first: MqiValue<P, S>,
            $($ty: MqiAttr<P, S>),*
        {
            type Error = $first::Error;

            #[allow(unused_parens, non_snake_case)]
            #[inline]
            fn consume<F>(param: &mut P, mqi: F) -> ResultCompErr<Self, Self::Error>
            where
                F: FnOnce(&mut P) -> ResultComp<S>,
            {
                let mut rest_outer = None;
                $first::consume(param, |param| {
                    <($($ty),*) as MqiAttr<P, S>>::extract(param, mqi).map_completion(|(rest, state)| {
                        rest_outer = Some(rest);
                        state
                    })
                })
                .map_completion(|a| {
                    let ($($ty),*) = rest_outer.expect("rest_outer set by extract closure");
                    (a, $($ty),*)
                })
            }
        }
    }
}

macro_rules! impl_mqiattr_tuple {
    ($first:ident, [$($ty:ident),*]) => {
        impl<P, S, $first, $($ty),*> MqiAttr<P, S> for ($first, $($ty),*)
        where
            $first: MqiAttr<P, S>,
            $($ty: MqiAttr<P, S>),*
        {
            #[allow(unused_parens, non_snake_case)]
            #[inline]
            fn extract<F>(param: &mut P, mqi: F) -> ResultComp<(Self, S)>
            where
                F: FnOnce(&mut P) -> ResultComp<S>
            {
                let mut rest_outer = None;
                $first::extract(param, |param| {
                    <($($ty),*) as MqiAttr<P, S>>::extract(param, mqi).map_completion(|(rest, state)| {
                        rest_outer = Some(rest);
                        state
                    })
                })
                .map_completion(|(a, s)| {
                    let ($($ty),*) = rest_outer.expect("rest_outer set by extract closure");
                    ((a, $($ty),*), s)
                })
            }
        }
    }
}

all_multi_tuples!(impl_mqiattr_tuple);
all_multi_tuples!(impl_mqivalue_tuple);

impl<P, S> MqiValue<P, S> for () {
    type Error = Error;

    fn consume<F>(param: &mut P, mqi: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut P) -> ResultComp<S>,
    {
        mqi(param).map_completion(|_state| ())
    }
}

impl<P, S> MqiAttr<P, S> for () {
    fn extract<F>(param: &mut P, mqi: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut P) -> ResultComp<S>,
    {
        mqi(param).map_completion(|state| ((), state))
    }
}

impl<P, T: MqiOption<P>> MqiOption<P> for Option<T> {
    fn apply_param(self, param: &mut P) {
        if let Some(option) = self {
            option.apply_param(param);
        }
    }
}

impl<P> MqiOption<P> for () {
    fn apply_param(self, _param: &mut P) {}
}

// TODO: Future me (and others) will hate me if I don't document what is happening here
macro_rules! impl_mqioption_tuple {
    ($first:ident, [$($ty:ident),*]) => {
        #[allow(non_snake_case,unused_parens)]
        impl<P, $first, $($ty, )*> MqiOption<P> for ($first, $($ty, )*)
        where
            $first: MqiOption<P>,
            $($ty: MqiOption<P>,)*
        {
            #[inline]
            fn apply_param(self, param: &mut P) {
                let ($first, $($ty, )*) = self;
                ($($ty),*).apply_param(param);
                $first.apply_param(param);
            }
        }
    }
}

all_multi_tuples!(impl_mqioption_tuple);
