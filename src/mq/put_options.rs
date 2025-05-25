use libmqm_sys::lib as sys;

use crate::{macros::all_multi_tuples, prelude::*, structs, types, constants, Conn, MqStr, Properties, ResultComp};

use super::{
    impl_min_version,
    put::{PutAttr, PutOption, PutParam},
    Object,
};

impl_min_version!(['a], structs::MQPMO<'a>);

#[derive(Debug, Clone, Copy)]
pub struct Context<T>(pub T);

macro_rules! impl_putoption_tuple {
    ([$($rest:ident),*]) => {
        #[expect(non_snake_case)]
        #[diagnostic::do_not_recommend]
        unsafe impl <'po, $($rest),*> PutOption<'po> for ($($rest),*)
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

unsafe impl PutOption<'_> for () {
    fn apply_param(&self, _: &mut PutParam<'_>) {}
}

all_multi_tuples!(impl_putoption_tuple);

#[derive(Debug)]
pub enum PropertyAction<'handle, C: Conn, C2: Conn> {
    Reply(&'handle Properties<C>, &'handle mut Properties<C2>),
    Forward(&'handle Properties<C>, &'handle mut Properties<C2>),
    Report(&'handle Properties<C>, &'handle mut Properties<C2>),
}

unsafe impl<'po, C: Conn> PutOption<'po> for Context<&Object<C>> {
    fn apply_param(&self, (.., pmo): &mut PutParam<'po>) {
        pmo.Context = unsafe { self.0.handle.raw_handle() };
    }
}

unsafe impl<'po, C: Conn> PutOption<'po> for &mut Properties<C> {
    fn apply_param(&self, (.., pmo): &mut PutParam<'po>) {
        pmo.set_min_version(sys::MQPMO_VERSION_3);
        *pmo.Action.as_mut() = constants::MQACTP_NEW;
        pmo.OriginalMsgHandle = unsafe { self.handle().raw_handle() };
    }
}

macro_rules! impl_putoption_deref {
    ($field:tt, $ty:ty) => {
        unsafe impl PutOption<'_> for $ty {
            fn apply_param(&self, (mqmd, _): &mut PutParam<'_>) {
                let mut_field: &mut Self = mqmd.$field.as_mut();
                *mut_field = *self;
            }
        }
    };
}

macro_rules! impl_putoption_mqchar {
    ($field:tt, $ty:ty) => {
        unsafe impl PutOption<'_> for $ty {
            fn apply_param(&self, (mqmd, _): &mut PutParam<'_>) {
                mqmd.$field = *self.as_mqchar();
            }
        }
    };
}

unsafe impl PutOption<'_> for types::MQPMO {
    fn apply_param(&self, (.., pmo): &mut PutParam<'_>) {
        let pmo_options: &mut Self = pmo.Options.as_mut();
        pmo_options.insert(*self);
    }
}

unsafe impl PutOption<'_> for structs::MQMD2 {
    fn apply_param(&self, param: &mut PutParam<'_>) {
        self.clone_into(&mut param.0);
    }
}

unsafe impl PutOption<'_> for types::MQRO {
    fn apply_param(&self, (mqmd, _): &mut PutParam<'_>) {
        let mut_report: &mut Self = mqmd.Report.as_mut();
        mut_report.insert(*self);
    }
}

impl_putoption_deref!(MsgType, types::MQMT);
impl_putoption_deref!(Expiry, types::MQEI);
impl_putoption_deref!(Feedback, types::MQFB);
impl_putoption_deref!(Priority, types::MQPRI);
impl_putoption_deref!(Persistence, types::MQPER);
impl_putoption_deref!(PutApplType, types::MQAT);
impl_putoption_deref!(MsgFlags, types::MQMF);

unsafe impl PutOption<'_> for types::MessageId {
    fn apply_param(&self, (mqmd, _): &mut PutParam<'_>) {
        mqmd.MsgId = self.0;
    }
}

unsafe impl PutOption<'_> for types::CorrelationId {
    fn apply_param(&self, (mqmd, _): &mut PutParam<'_>) {
        mqmd.CorrelId = self.0;
    }
}

unsafe impl PutOption<'_> for types::GroupId {
    fn apply_param(&self, (mqmd, _): &mut PutParam<'_>) {
        mqmd.GroupId = self.0;
    }
}

unsafe impl PutOption<'_> for types::AccountingToken {
    fn apply_param(&self, (mqmd, _): &mut PutParam<'_>) {
        mqmd.AccountingToken = self.0;
    }
}

impl_putoption_mqchar!(PutApplName, types::ApplName);
impl_putoption_mqchar!(ReplyToQ, types::ReplyToQueueName);
impl_putoption_mqchar!(ReplyToQMgr, types::ReplyToQueueManagerName);
impl_putoption_mqchar!(PutDate, types::PutDate);
impl_putoption_mqchar!(PutTime, types::PutTime);
impl_putoption_mqchar!(UserIdentifier, types::UserIdentifier);
impl_putoption_mqchar!(ApplIdentityData, types::ApplIdentityData);
impl_putoption_mqchar!(ApplOriginData, types::ApplOriginData);

unsafe impl<'po, C: Conn, C2: Conn> PutOption<'po> for PropertyAction<'po, C, C2> {
    fn apply_param(&self, (.., pmo): &mut PutParam<'po>) {
        let (action, original, new) = match self {
            PropertyAction::Reply(original, new) => (constants::MQACTP_REPLY, original, new),
            PropertyAction::Forward(original, new) => (constants::MQACTP_FORWARD, original, new),
            PropertyAction::Report(original, new) => (constants::MQACTP_REPORT, original, new),
        };
        pmo.set_min_version(sys::MQPMO_VERSION_3);
        *pmo.Action.as_mut() = action;
        pmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
        pmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
    }
}

unsafe impl PutAttr for structs::MQMD2 {
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

macro_rules! impl_putattr_mqmd_mqstr {
    ($field:tt, $ty:ty) => {
        unsafe impl PutAttr for $ty {
            #[inline]
            fn put_bag_extract<'b, F>(param: &mut PutParam<'b>, put: F) -> ResultComp<Self>
            where
                F: FnOnce(&mut PutParam<'b>) -> ResultComp<()>,
            {
                put(param).map_completion(|()| {
                    let (md, ..) = param;
                    MqStr::from(md.$field).into()
                })
            }
        }
    };
}

macro_rules! impl_putattr_mqmd {
    ($field:tt, $ty:ty) => {
        unsafe impl PutAttr for $ty {
            #[inline]
            fn put_bag_extract<'b, F>(param: &mut PutParam<'b>, put: F) -> ResultComp<Self>
            where
                F: FnOnce(&mut PutParam<'b>) -> ResultComp<()>,
            {
                put(param).map_completion(|()| {
                    let (md, ..) = param;
                    md.$field.into()
                })
            }
        }
    };
}

impl_putattr_mqmd_mqstr!(PutDate, types::PutDate);
impl_putattr_mqmd_mqstr!(PutTime, types::PutTime);
impl_putattr_mqmd!(MsgId, types::MessageId);
impl_putattr_mqmd_mqstr!(UserIdentifier, types::UserIdentifier);
impl_putattr_mqmd!(AccountingToken, types::AccountingToken);
impl_putattr_mqmd_mqstr!(ApplIdentityData, types::ApplIdentityData);
impl_putattr_mqmd!(CorrelId, types::CorrelationId);
impl_putattr_mqmd!(GroupId, types::GroupId);
impl_putattr_mqmd_mqstr!(ApplOriginData, types::ApplOriginData);

#[expect(unused_parens)]
mod impl_put {
    use crate::macros::all_multi_tuples;

    use crate::put::{PutAttr, PutParam};
    use crate::ResultComp;
    use crate::prelude::*;

    macro_rules! impl_putattr_tuple {
        ([$first:ident, $($ty:ident),*]) => {
            #[diagnostic::do_not_recommend]
            unsafe impl<$first, $($ty),*> PutAttr for ($first, $($ty),*)
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

    unsafe impl PutAttr for () {
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

        let mut put_param = (
            structs::MQMD2::new(default::MQMD2_DEFAULT),
            structs::MQPMO::new(default::MQPMO_DEFAULT),
        );

        let source = Properties::new(&qm, MQCMHO::default())?;
        let mut outcome = Properties::new(&qm, MQCMHO::default())?;
        let action = PropertyAction::Reply(&source, &mut outcome);
        action.apply_param(&mut put_param);

        dbg!(put_param);

        Ok(())
    }
}
