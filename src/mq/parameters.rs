use crate::{ResultComp, ResultCompErrExt};

use super::macros::all_multi_tuples;

pub trait MqiOption<P> {
    fn apply_param(self, param: &mut P);
}

pub trait MqiAttr<P>: Sized {
    fn from_mqi<Y, F: FnOnce(&mut P) -> Y>(param: &mut P, mqi: F) -> (Self, Y);
}

pub trait MqiValue<T>: Sized {
    type Param<'a>;

    #[allow(unused_variables)]
    fn from_mqi<F: FnOnce(&mut Self::Param<'_>) -> ResultComp<T>>(param: &mut Self::Param<'_>, mqi: F) -> ResultComp<Self>;
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

impl<P> MqiAttr<P> for () {
    #[inline]
    fn from_mqi<Y, F: FnOnce(&mut P) -> Y>(param: &mut P, mqi: F) -> (Self, Y) {
        ((), mqi(param))
    }
}

// TODO: Future me (and others) will hate me if I don't document what is happening here
macro_rules! impl_mqivalue {
    ($first:ident, [$($ty:ident),*]) => {
        #[allow(non_snake_case,unused_parens)]
        impl<$first $(, $ty)*> MqiValue<$first> for ($first $(, $ty)*)
        where
            $first: MqiValue<$first>,
            $($ty: for <'a> MqiAttr<$first::Param<'a>>,)*
        {
            type Param<'a> = $first::Param<'a>;

            #[inline]
            fn from_mqi<F: FnOnce(&mut Self::Param<'_>) -> ResultComp<$first>>(param: &mut Self::Param<'_>, mqi: F) -> ResultComp<Self> {
                let (($($ty),*), mqi_result) = <($($ty),*) as MqiAttr<Self::Param<'_>>>::from_mqi(param, |p| $first::from_mqi(p, mqi));
                mqi_result.map_completion(|mqi| (mqi $(, $ty)*))
            }
        }
    };
}

// TODO: Future me (and others) will hate me if I don't document what is happening here
macro_rules! impl_mqiattr {
    ($first:ident, [$($ty:ident),*]) => {
        #[allow(non_snake_case,unused_parens)]
        impl<$first, $($ty, )* P> MqiAttr<P> for ($first, $($ty),*)
        where
            $first: MqiAttr<P>,
            $($ty: MqiAttr<P>,)*
        {
            #[inline]
            fn from_mqi<Y, F: FnOnce(&mut P) -> Y>(param: &mut P, mqi: F) -> (Self, Y) {
                let ($first, (($($ty),*), y)) = $first::from_mqi(param, |f| <($($ty),*) as MqiAttr<P>>::from_mqi(f, mqi));
                (($first, $($ty),*), y)
            }
        }
    }
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

all_multi_tuples!(impl_mqivalue);
all_multi_tuples!(impl_mqiattr);
all_multi_tuples!(impl_mqioption);