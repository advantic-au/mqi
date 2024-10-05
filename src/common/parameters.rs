#![expect(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]

use crate::{Error, ResultComp, ResultCompErr, prelude::*};

use super::macros::all_multi_tuples;

pub trait MqiValue<P, S> {
    type Error: From<Error> + std::fmt::Debug;

    fn consume<F>(param: &mut P, mqi: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut P) -> ResultComp<S>,
        Self: std::marker::Sized;
}

pub trait MqiAttr<P, S> {
    fn extract<F>(param: &mut P, mqi: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut P) -> ResultComp<S>,
        Self: Sized;
}

pub struct Callback<F>(pub F);

macro_rules! impl_mqivalue_tuple {
    ([$first:ident, $($ty:ident),*]) => {
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
                    let ($($ty),*) = rest_outer.expect("rest_outer should be set by extract closure");
                    (a, $($ty),*)
                })
            }
        }
    }
}

macro_rules! impl_mqiattr_tuple {
    ([$first:ident, $($ty:ident),*]) => {
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
                    let ($($ty),*) = rest_outer.expect("rest_outer should be set by extract closure");
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
