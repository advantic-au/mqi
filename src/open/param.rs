use libmqm_sys as mq;

use super::option;
use crate::{
    CCSID, Conn, EncodedString, result::Error, MqStr, Object, result::ResultComp, StrCcsidOwned, constants,
    macros::{all_multi_tuples, impl_from_str, reverse_ident},
    prelude::*,
    structs,
    types::{MQLONG, MQOO, MQOT, MQPMO, QueueManagerName, QueueName},
};

unsafe impl<'oo, O, T: option::OpenOption<'oo, O>> option::OpenOption<'oo, O> for Option<T> {
    fn apply_param(&self, param: &mut option::OpenParamOption<'oo, O>) {
        if let Some(value) = self {
            value.apply_param(param);
        }
    }
}

unsafe impl<'oo, O> option::OpenOption<'oo, O> for () {
    fn apply_param(&self, _param: &mut option::OpenParamOption<'oo, O>) {}
}

macro_rules! impl_openoption_tuple {
    ([$($rest:ident),*]) => {
        #[expect(non_snake_case)]
        #[diagnostic::do_not_recommend]
        unsafe impl <'oo, O, $($rest),*> option::OpenOption<'oo, O> for ($($rest),*)
        where
            $($rest: option::OpenOption<'oo, O> ),*
        {
            #[inline]
            fn apply_param(&self,param: &mut option::OpenParamOption<'oo, O>){
                let reverse_ident!($($rest),*) = self;
                $($rest.apply_param(param);)*
            }
        }
    };
}

all_multi_tuples!(impl_openoption_tuple);

#[derive(Debug, Clone, Copy)]
pub struct SelectionString<T>(pub T);

#[derive(Debug, Clone, Copy)]
pub struct ObjectString<T>(pub T);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct AlternateUserId(pub MqStr<12>);
impl_from_str!(AlternateUserId, MqStr<12>);

#[derive(Debug, Clone)]
pub struct ResObjectString(pub StrCcsidOwned);

unsafe impl<'a, T: EncodedString + ?Sized, O> option::OpenOption<'a, O> for SelectionString<&'a T> {
    fn apply_param(&self, option::OpenParamOption { mqod, .. }: &mut option::OpenParamOption<'a, O>) {
        mqod.attach_selection_string(self.0);
    }
}

structs::impl_min_version!(['a], structs::MQOD<'a>);

unsafe impl<'a, T: EncodedString + ?Sized, O> option::OpenOption<'a, O> for ObjectString<&'a T> {
    fn apply_param(&self, option::OpenParamOption { mqod, .. }: &mut option::OpenParamOption<'a, O>) {
        *mqod.ObjectType.as_mut() = constants::MQOT_TOPIC;
        mqod.attach_object_string(self.0);
    }
}

unsafe impl<O> option::OpenOption<'_, O> for QueueName {
    fn apply_param(&self, option::OpenParamOption { mqod, .. }: &mut option::OpenParamOption<O>) {
        mqod.ObjectName = self.0.into();
        *mqod.ObjectType.as_mut() = constants::MQOT_Q;
    }
}

unsafe impl<O> option::OpenOption<'_, O> for QueueManagerName {
    fn apply_param(&self, option::OpenParamOption { mqod, .. }: &mut option::OpenParamOption<O>) {
        mqod.ObjectQMgrName = self.0.into();
        *mqod.ObjectType.as_mut() = constants::MQOT_Q_MGR;
    }
}

unsafe impl<'b> option::OpenOption<'b, Self> for MQOO {
    fn apply_param(&self, param: &mut option::OpenParamOption<'b, Self>) {
        param.options.insert(*self);
    }
}

unsafe impl<'b> option::OpenOption<'b, Self> for MQPMO {
    fn apply_param(&self, param: &mut option::OpenParamOption<'b, Self>) {
        param.options.insert(*self);
    }
}

unsafe impl option::OpenOption<'_, MQOO> for AlternateUserId {
    fn apply_param(&self, option::OpenParamOption { mqod, options }: &mut option::OpenParamOption<MQOO>) {
        options.insert(constants::MQOO_ALTERNATE_USER_AUTHORITY);
        mqod.set_min_version(mq::MQOD_VERSION_3);
        mqod.AlternateUserId = self.0.into();
    }
}

unsafe impl option::OpenOption<'_, MQPMO> for AlternateUserId {
    fn apply_param(&self, option::OpenParamOption { mqod, options }: &mut option::OpenParamOption<MQPMO>) {
        options.insert(constants::MQPMO_ALTERNATE_USER_AUTHORITY);
        mqod.set_min_version(mq::MQOD_VERSION_3);
        mqod.AlternateUserId = self.0.into();
    }
}

unsafe impl<S, O> option::OpenAttr<S, O> for Option<QueueName> {
    #[inline]
    fn open_extract<'b, F>(param: &mut option::OpenParamOption<'b, O>, open: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut option::OpenParamOption<'b, O>) -> ResultComp<S>,
    {
        param.mqod.set_min_version(mq::MQOD_VERSION_3); // For ResolvedQName
        open(param).map_completion(|state| {
            (
                Some(QueueName(param.mqod.ResolvedQName.into())).filter(|queue_name| queue_name.has_value()),
                state,
            )
        })
    }
}

unsafe impl<S, O> option::OpenAttr<S, O> for MQOT {
    #[inline]
    fn open_extract<'b, F>(param: &mut option::OpenParamOption<'b, O>, open: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut option::OpenParamOption<'b, O>) -> ResultComp<S>,
    {
        param.mqod.set_min_version(mq::MQOD_VERSION_4);
        open(param).map_completion(|state| (param.mqod.ResolvedType.into(), state))
    }
}

unsafe impl<C: Conn> option::OpenValue<Self> for Object<C> {
    type Error = Error;

    #[inline]
    fn open_consume<'oo, F>(param: &mut option::OpenParam<'oo>, open: F) -> crate::result::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut option::OpenParam<'oo>) -> ResultComp<Self>,
    {
        open(param)
    }
}

unsafe impl<S, O> option::OpenAttr<S, O> for Option<QueueManagerName> {
    #[inline]
    fn open_extract<'a, F>(param: &mut option::OpenParamOption<'a, O>, open: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut option::OpenParamOption<'a, O>) -> ResultComp<S>,
    {
        param.mqod.set_min_version(mq::MQOD_VERSION_3); // For ResolvedQMgrName
        open(param).map_completion(|state| {
            (
                Self::Some(QueueManagerName(param.mqod.ResolvedQMgrName.into())).filter(|queue_name| queue_name.has_value()),
                state,
            )
        })
    }
}

const DEFAULT_RESOBJECTSTRING_LENGTH: MQLONG = 4096;

unsafe impl<S, O> option::OpenAttr<S, O> for Option<ResObjectString> {
    fn open_extract<'a, F>(param: &mut option::OpenParamOption<'a, O>, open: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut option::OpenParamOption<'a, O>) -> ResultComp<S>,
    {
        let od = &mut param.mqod;
        if od.ResObjectString.VSBufSize == 0 {
            od.ResObjectString.VSBufSize = DEFAULT_RESOBJECTSTRING_LENGTH;
        }
        let mut buffer = Vec::with_capacity(
            od.ResObjectString
                .VSBufSize
                .try_into()
                .expect("buffer length should convert to usize"),
        );
        od.ResObjectString.VSPtr = (&raw mut *buffer).cast();

        open(param).map_completion(|state| {
            let od = &mut param.mqod;
            unsafe {
                buffer.set_len(
                    od.ResObjectString
                        .VSLength
                        .try_into()
                        .expect("buffer length should convert to usize"),
                );
            }
            (
                if buffer.is_empty() {
                    None
                } else {
                    Some(ResObjectString(StrCcsidOwned::from_vec(
                        buffer,
                        CCSID(od.ResObjectString.VSCCSID),
                    )))
                },
                state,
            )
        })
    }
}

#[expect(unused_parens)]
mod open_impl {
    use super::option;
    use crate::{result::ResultComp, result::ResultCompErr, macros::all_multi_tuples, prelude::*, types::MQOO};

    macro_rules! impl_openvalue_tuple {
        ([$first:ident, $($ty:ident),*]) => {
            #[diagnostic::do_not_recommend]
            unsafe impl<S, $first, $($ty),*> option::OpenValue<S> for ($first, $($ty),*)
            where
                $first: option::OpenValue<S>,
                $($ty: option::OpenAttr<S, MQOO>),*
            {
                type Error = $first::Error;

                #[expect(non_snake_case)]
                #[inline]
                fn open_consume<'a, F>(param: &mut option::OpenParam<'a>, mqi: F) -> ResultCompErr<Self, Self::Error>
                where
                    F: FnOnce(&mut option::OpenParam<'a>) -> ResultComp<S>,
                {
                    let mut rest_outer = None;
                    $first::open_consume(param, |param| {
                        <($($ty),*) as option::OpenAttr<S, MQOO>>::open_extract(param, mqi).map_completion(|(rest, state)| {
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

    macro_rules! impl_openattr_tuple {
        ([$first:ident, $($ty:ident),*]) => {
            #[diagnostic::do_not_recommend]
            unsafe impl<S, O, $first, $($ty),*> option::OpenAttr<S, O> for ($first, $($ty),*)
            where
                $first: option::OpenAttr<S, O>,
                $($ty: option::OpenAttr<S, O>),*
            {
                #[expect(non_snake_case)]
                #[inline]
                fn open_extract<'a, F>(param: &mut option::OpenParamOption<'a, O>, mqi: F) -> ResultComp<(Self, S)>
                where
                    F: FnOnce(&mut option::OpenParamOption<'a, O>) -> ResultComp<S>
                {
                    let mut rest_outer = None;
                    $first::open_extract(param, |param| {
                        <($($ty),*) as option::OpenAttr<S, O>>::open_extract(param, mqi).map_completion(|(rest, state)| {
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

    all_multi_tuples!(impl_openvalue_tuple);
    all_multi_tuples!(impl_openattr_tuple);
}
