use crate::{macros::all_multi_tuples, prelude::*, types::MQPMO, sys, types, Conn, MqStruct, Properties, ResultComp};

use super::{
    impl_mqstruct_min_version,
    put::{PutAttr, PutOption, PutParam},
    Object,
};

impl_mqstruct_min_version!(sys::MQPMO);

#[derive(Debug, Clone, Copy)]
pub struct Context<T>(pub T);

macro_rules! impl_putoption_tuple {
    ([$($rest:ident),*]) => {
        #[expect(non_snake_case)]
        #[diagnostic::do_not_recommend]
        impl <'po, $($rest),*> PutOption<'po> for ($($rest),*)
        where
            $($rest: PutOption<'po> ),*
        {
            #[inline]
            fn apply_param(&self, param: &mut PutParam<'po>) {
                let $crate::macros::reverse_ident!($($rest),*) = self;
                $($rest.apply_param(param);)*
            }
        }
    };
}

impl PutOption<'_> for () {
    fn apply_param(&self, _: &mut PutParam<'_>) {}
}

all_multi_tuples!(impl_putoption_tuple);

#[derive(Debug)]
pub enum PropertyAction<'handle, C: Conn, C2: Conn> {
    Reply(&'handle Properties<C>, &'handle mut Properties<C2>),
    Forward(&'handle Properties<C>, &'handle mut Properties<C2>),
    Report(&'handle Properties<C>, &'handle mut Properties<C2>),
}

impl<'po, C: Conn> PutOption<'po> for Context<&Object<C>> {
    fn apply_param(&self, (.., pmo): &mut PutParam<'po>) {
        pmo.Context = unsafe { self.0.handle.raw_handle() };
    }
}

impl<'po, C: Conn> PutOption<'po> for &mut Properties<C> {
    fn apply_param(&self, (.., pmo): &mut PutParam<'po>) {
        pmo.set_min_version(sys::MQPMO_VERSION_3);
        pmo.Action = sys::MQACTP_NEW;
        pmo.OriginalMsgHandle = unsafe { self.handle().raw_handle() };
    }
}

impl PutOption<'_> for MQPMO {
    fn apply_param(&self, (.., pmo): &mut PutParam<'_>) {
        pmo.Options |= self.0;
    }
}

impl PutOption<'_> for MqStruct<'static, sys::MQMD2> {
    fn apply_param(&self, param: &mut PutParam<'_>) {
        self.clone_into(&mut param.0);
    }
}

impl<'po, C: Conn, C2: Conn> PutOption<'po> for PropertyAction<'po, C, C2> {
    fn apply_param(&self, (.., pmo): &mut PutParam<'po>) {
        let (action, original, new) = match self {
            PropertyAction::Reply(original, new) => (sys::MQACTP_REPLY, original, new),
            PropertyAction::Forward(original, new) => (sys::MQACTP_FORWARD, original, new),
            PropertyAction::Report(original, new) => (sys::MQACTP_REPORT, original, new),
        };
        pmo.set_min_version(sys::MQPMO_VERSION_3);
        pmo.Action = action;
        pmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
        pmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
    }
}

impl PutAttr for MqStruct<'static, sys::MQMD2> {
    #[inline]
    fn put_bag_extract<'b, F>(param: &mut PutParam<'b>, put: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut PutParam<'b>) -> ResultComp<()>,
    {
        put(param).map_completion(|()| {
            let (md, ..) = param;
            md.clone()
        })
    }
}

impl PutAttr for types::MessageId {
    #[inline]
    fn put_bag_extract<'b, F>(param: &mut PutParam<'b>, put: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut PutParam<'b>) -> ResultComp<()>,
    {
        put(param).map_completion(|()| {
            let (md, ..) = param;
            Self(md.MsgId.into())
        })
    }
}

impl PutAttr for types::CorrelationId {
    #[inline]
    fn put_bag_extract<'b, F>(param: &mut PutParam<'b>, put: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut PutParam<'b>) -> ResultComp<()>,
    {
        put(param).map_completion(|()| {
            let (md, ..) = param;
            Self(md.CorrelId.into())
        })
    }
}

impl PutAttr for Option<types::UserIdentifier> {
    #[inline]
    fn put_bag_extract<'b, F>(param: &mut PutParam<'b>, put: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut PutParam<'b>) -> ResultComp<()>,
    {
        put(param).map_completion(|()| {
            let (md, ..) = param;
            types::UserIdentifier::new(md.UserIdentifier)
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
            #[diagnostic::do_not_recommend]
            impl<$first, $($ty),*> PutAttr for ($first, $($ty),*)
            where
                $first: PutAttr,
                $($ty: PutAttr),*
            {
                #[expect(non_snake_case)]
                #[inline]
                fn put_bag_extract<'p, F>(param: &mut PutParam<'p>, mqi: F) -> ResultComp<Self>
                where
                    F: FnOnce(&mut PutParam<'p>) -> ResultComp<()>
                {
                    let mut rest_outer = None;
                    $first::put_bag_extract(param, |param| {
                        <($($ty),*) as PutAttr>::put_bag_extract(param, mqi).map_completion(|rest| {
                            rest_outer = Some(rest);
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

    impl PutAttr for () {
        #[inline]
        fn put_bag_extract<'p, F>(param: &mut PutParam<'p>, mqi: F) -> ResultComp<Self>
        where
            F: FnOnce(&mut PutParam<'p>) -> ResultComp<()>,
            Self: Sized,
        {
            mqi(param)
        }
    }

    all_multi_tuples!(impl_putattr_tuple);
}

#[cfg(test)]
#[cfg(feature = "mock")]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use std::error::Error;

    use crate::put::PutOption;
    use crate::test::mock;
    use crate::Properties;
    use crate::types::MQCMHO;

    use super::*;

    use libmqm_default as default;

    #[test]
    fn property_action() -> Result<(), Box<dyn Error>> {
        let qm = mock::connect_ok(|mock_library| {
            let mut seq = mockall::Sequence::new();
            mock_library.properties_ok(0xf0f0, 1, &mut seq);
            mock_library.properties_ok(0x0e0e, 1, &mut seq);
        });

        let mut put_param = (MqStruct::new(default::MQMD2_DEFAULT), MqStruct::new(default::MQPMO_DEFAULT));

        let source = Properties::new(&qm, MQCMHO::default())?;
        let mut outcome = Properties::new(&qm, MQCMHO::default())?;
        let action = PropertyAction::Reply(&source, &mut outcome);
        action.apply_param(&mut put_param);

        dbg!(put_param);

        Ok(())
    }
}
