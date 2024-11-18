use crate::{macros::all_multi_tuples, prelude::*, sys, types, values, Conn, MqStruct, Properties, ResultComp};

use super::{
    put::{PutAttr, PutOption, PutParam},
    Object,
};

#[derive(Debug, Clone, Copy)]
pub struct Context<T>(pub T);

macro_rules! impl_putoption_tuple {
    ([$first:ident, $($rest:ident),*]) => {
        #[expect(non_snake_case)]
        impl <'po, $first, $($rest),*> PutOption<'po> for ($first, $($rest),*)
        where
            $first: PutOption<'po>,
            $($rest: PutOption<'po> ),*
        {
            #[inline]
            fn apply_param(self, param: &mut PutParam<'po>) {
                let($first, $($rest),*) = self;
                ($($rest),*).apply_param(param);
                $first.apply_param(param);
            }
        }
    };
}

impl PutOption<'_> for () {
    fn apply_param(self, _: &mut PutParam<'_>) {}
}

all_multi_tuples!(impl_putoption_tuple);

#[derive(Debug)]
pub enum PropertyAction<'handle, C: Conn, C2: Conn> {
    Reply(&'handle Properties<C>, &'handle mut Properties<C2>),
    Forward(&'handle Properties<C>, &'handle mut Properties<C2>),
    Report(&'handle Properties<C>, &'handle mut Properties<C2>),
}

impl<'po, C: Conn> PutOption<'po> for Context<&Object<C>> {
    fn apply_param(self, (.., pmo): &mut PutParam<'po>) {
        pmo.Context = unsafe { self.0.handle.raw_handle() };
    }
}

impl<'po, C: Conn> PutOption<'po> for &mut Properties<C> {
    fn apply_param(self, (.., pmo): &mut PutParam<'po>) {
        pmo.Action = sys::MQACTP_NEW;
        pmo.OriginalMsgHandle = unsafe { self.handle().raw_handle() };
    }
}

impl PutOption<'_> for values::MQPMO {
    fn apply_param(self, (.., pmo): &mut PutParam<'_>) {
        pmo.Options |= self.value();
    }
}

impl PutOption<'_> for MqStruct<'static, sys::MQMD2> {
    fn apply_param(self, param: &mut PutParam<'_>) {
        self.clone_into(&mut param.0);
    }
}

impl<'po, C: Conn, C2: Conn> PutOption<'po> for PropertyAction<'po, C, C2> {
    fn apply_param(self, (.., pmo): &mut PutParam<'po>) {
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

#[cfg(test)]
#[cfg(feature = "mock")]
mod test {
    use std::error::Error;

    use crate::put::PutOption;
    use crate::test::mock;
    use crate::{connect_lib, values, Properties, ThreadNone};
    use crate::prelude::*;

    use super::PropertyAction;

    #[test]
    fn property_action() -> Result<(), Box<dyn Error>> {
        let mut mock_library = mock::connect_ok();
        let mut seq = mockall::Sequence::new();

        mock_library.properties_ok(0xf0f0, 1, &mut seq);
        mock_library.properties_ok(0x0e0e, 1, &mut seq);

        let qm = connect_lib::<ThreadNone, _>(mock_library, ()).warn_as_error()?;

        let mut put_param = Default::default();

        let source = Properties::new(&qm, values::MQCMHO::default())?;
        let mut outcome = Properties::new(&qm, values::MQCMHO::default())?;
        let action = PropertyAction::Reply(&source, &mut outcome);
        action.apply_param(&mut put_param);

        dbg!(put_param);

        Ok(())
    }
}
