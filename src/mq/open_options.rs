use std::ptr;

use crate::{
    macros::all_multi_tuples,
    prelude::*,
    sys,
    types::{QueueManagerName, QueueName},
    values::{CCSID, MQOO, MQOT, MQPMO},
    Conn, EncodedString, Error, MqStr, MqiAttr, MqiValue, ResultComp, StrCcsidOwned,
};

use super::{impl_mqstruct_min_version, types::impl_from_str, Object, OpenOption, OpenParam, OpenParamOption, OpenValue};

impl<'oo, O, T: OpenOption<'oo, O>> OpenOption<'oo, O> for Option<T> {
    fn apply_param(self, param: &mut OpenParamOption<'oo, O>) {
        if let Some(value) = self {
            value.apply_param(param);
        }
    }
}

impl<'oo, O> OpenOption<'oo, O> for () {
    fn apply_param(self, _param: &mut OpenParamOption<'oo, O>) {}
}

macro_rules! impl_openoption_tuple {
    ([$first:ident, $($rest:ident),*]) => {
        #[expect(non_snake_case)]
        impl <'oo, O, $first, $($rest),*> OpenOption<'oo, O> for ($first, $($rest),*)
        where
            $first: OpenOption<'oo, O>,
            $($rest: OpenOption<'oo, O> ),*
        {
            #[inline]
            fn apply_param(self,param: &mut OpenParamOption<'oo, O>){
                let($first, $($rest),*) = self;
                ($($rest),*).apply_param(param);
                $first.apply_param(param);
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

impl<'a, T: EncodedString + ?Sized, O> OpenOption<'a, O> for SelectionString<&'a T> {
    fn apply_param(self, OpenParamOption { mqod, .. }: &mut OpenParamOption<'a, O>) {
        mqod.attach_selection_string(self.0);
    }
}

impl_mqstruct_min_version!(sys::MQOD);

impl<'a, T: EncodedString + ?Sized, O> OpenOption<'a, O> for ObjectString<&'a T> {
    fn apply_param(self, OpenParamOption { mqod, .. }: &mut OpenParamOption<'a, O>) {
        mqod.set_min_version(sys::MQOD_VERSION_4);
        mqod.ObjectType = sys::MQOT_TOPIC;
        mqod.attach_object_string(self.0);
    }
}

impl<'b, O> OpenOption<'b, O> for QueueName {
    fn apply_param(self, OpenParamOption { mqod, .. }: &mut OpenParamOption<O>) {
        mqod.ObjectName = self.0.into();
        mqod.ObjectType = sys::MQOT_Q;
    }
}

impl<'b, O> OpenOption<'b, O> for QueueManagerName {
    fn apply_param(self, OpenParamOption { mqod, .. }: &mut OpenParamOption<O>) {
        mqod.ObjectQMgrName = self.0.into();
        mqod.ObjectType = sys::MQOT_Q_MGR;
    }
}

impl<'b> OpenOption<'b, Self> for MQOO {
    fn apply_param(self, param: &mut OpenParamOption<'b, Self>) {
        param.options |= self;
    }
}

impl<'b> OpenOption<'b, Self> for MQPMO {
    fn apply_param(self, param: &mut OpenParamOption<'b, Self>) {
        param.options |= self;
    }
}

impl<'b> OpenOption<'b, MQOO> for AlternateUserId {
    fn apply_param(self, OpenParamOption { mqod, options }: &mut OpenParamOption<MQOO>) {
        *options |= sys::MQOO_ALTERNATE_USER_AUTHORITY;
        mqod.set_min_version(sys::MQOD_VERSION_3);
        mqod.AlternateUserId = self.0.into();
    }
}

impl<'b> OpenOption<'b, MQPMO> for AlternateUserId {
    fn apply_param(self, OpenParamOption { mqod, options }: &mut OpenParamOption<MQPMO>) {
        *options |= sys::MQPMO_ALTERNATE_USER_AUTHORITY;
        mqod.set_min_version(sys::MQOD_VERSION_3);
        mqod.AlternateUserId = self.0.into();
    }
}

impl<'b, O, S> MqiAttr<OpenParamOption<'b, O>, S> for Option<QueueName> {
    fn extract<F>(param: &mut OpenParamOption<'b, O>, open: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut OpenParamOption<'b, O>) -> ResultComp<S>,
    {
        open(param).map_completion(|state| {
            (
                Some(QueueName(param.mqod.ResolvedQName.into())).filter(|queue_name| queue_name.has_value()),
                state,
            )
        })
    }
}

impl<'b, O, S> MqiAttr<OpenParamOption<'b, O>, S> for MQOT {
    fn extract<F>(param: &mut OpenParamOption<'b, O>, open: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut OpenParamOption<'b, O>) -> ResultComp<S>,
    {
        param.mqod.set_min_version(sys::MQOD_VERSION_4);
        open(param).map_completion(|state| (param.mqod.ObjectType.into(), state))
    }
}

// Blanket implementation of OpenValue
impl<T, S> OpenValue<S> for T where for<'oo> Self: MqiValue<OpenParam<'oo>, S> {}

impl<C: Conn, P> MqiValue<P, Self> for Object<C> {
    type Error = Error;

    fn consume<F>(param: &mut P, open: F) -> crate::ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut P) -> ResultComp<Self>,
    {
        open(param)
    }
}

impl<'a, O, S> MqiAttr<OpenParamOption<'a, O>, S> for Option<QueueManagerName> {
    fn extract<F>(param: &mut OpenParamOption<'a, O>, open: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut OpenParamOption<'a, O>) -> ResultComp<S>,
    {
        open(param).map_completion(|state| {
            (
                Self::Some(QueueManagerName(param.mqod.ResolvedQMgrName.into())).filter(|queue_name| queue_name.has_value()),
                state,
            )
        })
    }
}

const DEFAULT_RESOBJECTSTRING_LENGTH: sys::MQLONG = 4096;

impl<'a, O, S> MqiAttr<OpenParamOption<'a, O>, S> for Option<ResObjectString> {
    fn extract<F>(param: &mut OpenParamOption<'a, O>, open: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut OpenParamOption<'a, O>) -> ResultComp<S>,
    {
        let od = &mut param.mqod;
        if od.ResObjectString.VSBufSize == 0 {
            od.ResObjectString.VSBufSize = DEFAULT_RESOBJECTSTRING_LENGTH;
        }
        let mut buffer: Vec<_> = Vec::with_capacity(
            od.ResObjectString
                .VSBufSize
                .try_into()
                .expect("buffer length should convert to usize"),
        );
        od.ResObjectString.VSPtr = ptr::from_mut(&mut *buffer).cast();

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
