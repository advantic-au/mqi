use crate::{Error, ResultComp, ResultCompErr, ResultCompErrExt};

use super::{macros::all_multi_tuples, types};

pub trait MqiOption<P> {
    fn apply_param(self, param: &mut P);
}

// pub trait ConsumeValue<P, S>: Sized {
//     type Error: From<Error>;

//     fn consume_from(state: S, param: &P, warning: Option<types::Warning>) -> Result<Self, Self::Error>;
// }

pub trait ConsumeValue2<P, S>: Sized {
    type Error: From<Error> + std::fmt::Debug;

    fn consume<F>(param: &mut P, mqi: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut P) -> ResultComp<S>;
}

pub trait ExtractValue2<P, S>: Sized {
    fn extract<F>(param: &mut P, mqi: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut P) -> ResultComp<S>;
}

macro_rules! impl_consumevalue2 {
    ($first:ident, [$($ty:ident),*]) => {
        impl<P, S, $first, $($ty),*> ConsumeValue2<P, S> for ($first, $($ty),*)
        where
            $first: ConsumeValue2<P, S>,
            $($ty: ExtractValue2<P, S>),*
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
                    <($($ty),*) as ExtractValue2<P, S>>::extract(param, mqi).map_completion(|(rest, state)| {
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

macro_rules! impl_extractvalue2 {
    ($first:ident, [$($ty:ident),*]) => {
        impl<P, S, $first, $($ty),*> ExtractValue2<P, S> for ($first, $($ty),*)
        where
            $first: ExtractValue2<P, S>,
            $($ty: ExtractValue2<P, S>),*
        {
            #[allow(unused_parens, non_snake_case)]
            #[inline]
            fn extract<F>(param: &mut P, mqi: F) -> ResultComp<(Self, S)>
            where
                F: FnOnce(&mut P) -> ResultComp<S>
            {
                let mut rest_outer = None;
                $first::extract(param, |param| {
                    <($($ty),*) as ExtractValue2<P, S>>::extract(param, mqi).map_completion(|(rest, state)| {
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

all_multi_tuples!(impl_extractvalue2);
all_multi_tuples!(impl_consumevalue2);

impl<P, S> ConsumeValue2<P, S> for () {
    type Error = Error;

    fn consume<F>(param: &mut P, mqi: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut P) -> ResultComp<S>,
    {
        mqi(param).map_completion(|_state| ())
    }
}

impl<P, S> ExtractValue2<P, S> for () {
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

impl<F, P> MqiOption<P> for F
where
    F: FnOnce(&mut P),
{
    fn apply_param(self, param: &mut P) {
        self(param);
    }
}

impl<P> MqiOption<P> for () {
    fn apply_param(self, _param: &mut P) {}
}

// TODO: Future me (and others) will hate me if I don't document what is happening here
macro_rules! impl_mqioption {
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

all_multi_tuples!(impl_mqioption);
