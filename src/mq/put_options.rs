use crate::{macros::all_option_tuples, prelude::*, sys, types, values, Conn, MqStruct, Properties, ResultComp};

use super::{
    put::{PutAttr, PutOption, PutParam},
    Object,
};

#[derive(Debug, Clone, Copy)]
pub struct Context<T>(pub T);

all_option_tuples!(PutOption, PutParam);

#[derive(Debug)]
pub enum PropertyAction<'handle, C: Conn> {
    Reply(&'handle Properties<C>, &'handle mut Properties<C>),
    Forward(&'handle Properties<C>, &'handle mut Properties<C>),
    Report(&'handle Properties<C>, &'handle mut Properties<C>),
}

impl<C: Conn> PutOption for Context<&Object<C>> {
    fn apply_param(self, (.., pmo): &mut PutParam) {
        pmo.Context = unsafe { self.0.handle.raw_handle() };
    }
}

impl<C: Conn> PutOption for &mut Properties<C> {
    fn apply_param(self, (.., pmo): &mut PutParam) {
        pmo.Action = sys::MQACTP_NEW;
        pmo.OriginalMsgHandle = unsafe { self.handle().raw_handle() };
    }
}

impl PutOption for values::MQPMO {
    fn apply_param(self, (.., pmo): &mut PutParam) {
        pmo.Options |= self.value();
    }
}

impl PutOption for MqStruct<'static, sys::MQMD2> {
    fn apply_param(self, param: &mut PutParam) {
        self.clone_into(&mut param.0);
    }
}

impl<'handle, C: Conn> PutOption for PropertyAction<'handle, C> {
    fn apply_param(self, (.., pmo): &mut PutParam) {
        match self {
            PropertyAction::Reply(original, new) => {
                pmo.Action = sys::MQACTP_REPLY;
                pmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
                pmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
            }
            PropertyAction::Forward(original, new) => {
                pmo.Action = sys::MQACTP_FORWARD;
                pmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
                pmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
            }
            PropertyAction::Report(original, new) => {
                pmo.Action = sys::MQACTP_REPORT;
                pmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
                pmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
            }
        }
    }
}

impl PutAttr for MqStruct<'static, sys::MQMD2> {
    #[inline]
    fn extract<'b, F>(param: &mut PutParam<'b>, put: F) -> ResultComp<(Self, ())>
    where
        F: FnOnce(&mut PutParam<'b>) -> ResultComp<()>,
    {
        put(param).map_completion(|state| {
            let (md, ..) = param;
            (md.clone(), state)
        })
    }
}

impl PutAttr for types::MessageId {
    #[inline]
    fn extract<'b, F>(param: &mut PutParam<'b>, put: F) -> ResultComp<(Self, ())>
    where
        F: FnOnce(&mut PutParam<'b>) -> ResultComp<()>,
    {
        put(param).map_completion(|state| {
            let (md, ..) = param;
            (Self(md.MsgId.into()), state)
        })
    }
}

impl PutAttr for types::CorrelationId {
    #[inline]
    fn extract<'b, F>(param: &mut PutParam<'b>, put: F) -> ResultComp<(Self, ())>
    where
        F: FnOnce(&mut PutParam<'b>) -> ResultComp<()>,
    {
        put(param).map_completion(|state| {
            let (md, ..) = param;
            (Self(md.CorrelId.into()), state)
        })
    }
}

impl PutAttr for Option<types::UserIdentifier> {
    #[inline]
    fn extract<'b, F>(param: &mut PutParam<'b>, put: F) -> ResultComp<(Self, ())>
    where
        F: FnOnce(&mut PutParam<'b>) -> ResultComp<()>,
    {
        put(param).map_completion(|state| {
            let (md, ..) = param;
            (types::UserIdentifier::new(md.UserIdentifier), state)
        })
    }
}

#[expect(unused_parens)]
mod impl_put {
    use crate::macros::all_multi_tuples;

    use crate::put::{PutAttr, PutParam};
    use crate::ResultComp;
    use crate::prelude::*;

    macro_rules! impl_putattr_tuple {
        ([$first:ident, $($ty:ident),*]) => {
            impl<$first, $($ty),*> PutAttr for ($first, $($ty),*)
            where
                $first: PutAttr,
                $($ty: PutAttr),*
            {
                #[expect(non_snake_case)]
                #[inline]
                fn extract<'p, F>(param: &mut PutParam<'p>, mqi: F) -> ResultComp<(Self, ())>
                where
                    F: FnOnce(&mut PutParam<'p>) -> ResultComp<()>
                {
                    let mut rest_outer = None;
                    $first::extract(param, |param| {
                        <($($ty),*) as PutAttr>::extract(param, mqi).map_completion(|(rest, state)| {
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

    impl PutAttr for () {
        #[inline]
        fn extract<'p, F>(param: &mut PutParam<'p>, mqi: F) -> ResultComp<(Self, ())>
        where
            F: FnOnce(&mut PutParam<'p>) -> ResultComp<()>,
            Self: Sized,
        {
            mqi(param).map_completion(|()| ((), ()))
        }
    }

    all_multi_tuples!(impl_putattr_tuple);
}
