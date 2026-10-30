use libmqm_sys as mq;

use super::option;
use crate::{
    MqStr, Object, Properties, connection::AsConnection, constants, macros::all_multi_tuples, prelude::*, result::ResultComp,
    structs, types,
};

structs::impl_min_version!(['a], structs::MQPMO<'a>);

#[derive(Debug, Clone, Copy)]
pub struct Context<T>(pub T);

macro_rules! impl_putoption_tuple {
    ([$($rest:ident),*]) => {
        #[expect(non_snake_case)]
        #[diagnostic::do_not_recommend]
        unsafe impl <'po, $($rest),*> option::PutOption<'po> for ($($rest),*)
        where
            $($rest: option::PutOption<'po> ),*
        {
            #[inline]
            fn apply_param(&self, param: &mut option::PutParam<'po>) {
                let $crate::macros::reverse_ident!($($rest),*) = self;
                $($rest.apply_param(param);)*
            }
        }
    };
}

unsafe impl option::PutOption<'_> for () {
    fn apply_param(&self, _: &mut option::PutParam<'_>) {}
}

unsafe impl<'po, T: option::PutOption<'po>> option::PutOption<'po> for Option<T> {
    fn apply_param(&self, param: &mut option::PutParam<'po>) {
        if let Some(value) = self {
            value.apply_param(param);
        }
    }
}

all_multi_tuples!(impl_putoption_tuple);

#[derive(Debug)]
pub enum PropertyAction<'handle, C: AsConnection, C2: AsConnection> {
    Reply(&'handle Properties<C>, &'handle mut Properties<C2>),
    Forward(&'handle Properties<C>, &'handle mut Properties<C2>),
    Report(&'handle Properties<C>, &'handle mut Properties<C2>),
}

unsafe impl<'po, C: AsConnection> option::PutOption<'po> for Context<&Object<C>> {
    fn apply_param(&self, (.., pmo): &mut option::PutParam<'po>) {
        pmo.Context = self.0.handle.raw_handle();
    }
}

unsafe impl<'po, C: AsConnection> option::PutOption<'po> for &mut Properties<C> {
    fn apply_param(&self, (.., pmo): &mut option::PutParam<'po>) {
        pmo.set_min_version(mq::MQPMO_VERSION_3);
        *pmo.Action.as_mut() = constants::MQACTP_NEW;
        pmo.OriginalMsgHandle = self.handle().raw_handle();
    }
}

macro_rules! impl_putoption_deref {
    ($field:tt, $ty:ty) => {
        unsafe impl option::PutOption<'_> for $ty {
            fn apply_param(&self, (mqmd, _): &mut option::PutParam<'_>) {
                let mut_field: &mut Self = mqmd.$field.as_mut();
                *mut_field = *self;
            }
        }
    };
}

macro_rules! impl_putoption_mqchar {
    ($field:tt, $ty:ty) => {
        unsafe impl option::PutOption<'_> for $ty {
            fn apply_param(&self, (mqmd, _): &mut option::PutParam<'_>) {
                mqmd.$field = *self.as_mqchar();
            }
        }
    };
}

unsafe impl option::PutOption<'_> for types::MQPMO {
    fn apply_param(&self, (.., pmo): &mut option::PutParam<'_>) {
        let pmo_options: &mut Self = pmo.Options.as_mut();
        pmo_options.insert(*self);
    }
}

unsafe impl option::PutOption<'_> for structs::MQMD {
    fn apply_param(&self, param: &mut option::PutParam<'_>) {
        self.clone_into(&mut param.0);
    }
}

unsafe impl option::PutOption<'_> for types::MQRO {
    fn apply_param(&self, (mqmd, _): &mut option::PutParam<'_>) {
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

unsafe impl option::PutOption<'_> for types::MessageId {
    fn apply_param(&self, (mqmd, _): &mut option::PutParam<'_>) {
        mqmd.MsgId = self.0;
    }
}

unsafe impl option::PutOption<'_> for types::CorrelationId {
    fn apply_param(&self, (mqmd, _): &mut option::PutParam<'_>) {
        mqmd.CorrelId = self.0;
    }
}

unsafe impl option::PutOption<'_> for types::GroupId {
    fn apply_param(&self, (mqmd, _): &mut option::PutParam<'_>) {
        mqmd.GroupId = self.0;
    }
}

unsafe impl option::PutOption<'_> for types::AccountingToken {
    fn apply_param(&self, (mqmd, _): &mut option::PutParam<'_>) {
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

unsafe impl<'po, C: AsConnection, C2: AsConnection> option::PutOption<'po> for PropertyAction<'po, C, C2> {
    fn apply_param(&self, (.., pmo): &mut option::PutParam<'po>) {
        let (action, original, new) = match self {
            PropertyAction::Reply(original, new) => (constants::MQACTP_REPLY, original, new),
            PropertyAction::Forward(original, new) => (constants::MQACTP_FORWARD, original, new),
            PropertyAction::Report(original, new) => (constants::MQACTP_REPORT, original, new),
        };
        pmo.set_min_version(mq::MQPMO_VERSION_3);
        *pmo.Action.as_mut() = action;
        pmo.OriginalMsgHandle = original.handle().raw_handle();
        pmo.NewMsgHandle = new.handle().raw_handle();
    }
}

unsafe impl option::PutAttr for structs::MQMD {
    #[inline]
    fn put_extract<'b, F>(param: &mut option::PutParam<'b>, put: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut option::PutParam<'b>) -> ResultComp<()>,
    {
        put(param).map_completion(|()| {
            let (md, ..) = param;
            md.clone()
        })
    }
}

macro_rules! impl_putattr_mqmd_mqstr {
    ($field:tt, $ty:ty) => {
        unsafe impl option::PutAttr for $ty {
            #[inline]
            fn put_extract<'b, F>(param: &mut option::PutParam<'b>, put: F) -> ResultComp<Self>
            where
                F: FnOnce(&mut option::PutParam<'b>) -> ResultComp<()>,
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
        unsafe impl option::PutAttr for $ty {
            #[inline]
            fn put_extract<'b, F>(param: &mut option::PutParam<'b>, put: F) -> ResultComp<Self>
            where
                F: FnOnce(&mut option::PutParam<'b>) -> ResultComp<()>,
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
    use super::option;
    use crate::{macros::all_multi_tuples, prelude::*, result::ResultComp};

    macro_rules! impl_putattr_tuple {
        ([$first:ident, $($ty:ident),*]) => {
            #[diagnostic::do_not_recommend]
            unsafe impl<$first, $($ty),*> option::PutAttr for ($first, $($ty),*)
            where
                $first: option::PutAttr,
                $($ty: option::PutAttr),*
            {
                #[expect(non_snake_case)]
                #[inline]
                fn put_extract<'p, F>(param: &mut option::PutParam<'p>, mqi: F) -> ResultComp<Self>
                where
                    F: FnOnce(&mut option::PutParam<'p>) -> ResultComp<()>
                {
                    let mut rest_outer = None;
                    $first::put_extract(param, |param| {
                        <($($ty),*) as option::PutAttr>::put_extract(param, mqi).map_completion(|rest| {
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

    unsafe impl option::PutAttr for () {
        #[inline]
        fn put_extract<'p, F>(param: &mut option::PutParam<'p>, mqi: F) -> ResultComp<Self>
        where
            F: FnOnce(&mut option::PutParam<'p>) -> ResultComp<()>,
            Self: Sized,
        {
            mqi(param)
        }
    }

    all_multi_tuples!(impl_putattr_tuple);
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use libmqm_default as default;

    use super::{option, *};
    use crate::{constants, put::PutOption as _, result::Completion};

    fn put_param() -> option::PutParam<'static> {
        (
            structs::MQMD::new(default::MQMD_DEFAULT),
            structs::MQPMO::new(default::MQPMO_DEFAULT),
        )
    }

    #[test]
    fn put_option_unit_and_option_wrapper() {
        let mut param = put_param();
        let original_priority = param.0.Priority;
        <() as option::PutOption>::apply_param(&(), &mut param);
        assert_eq!(param.0.Priority, original_priority);

        let skipped = None::<types::MQPRI>;
        skipped.apply_param(&mut param);
        assert_eq!(param.0.Priority, original_priority);

        let applied = Some(types::MQPRI(17));
        applied.apply_param(&mut param);
        assert_eq!(types::MQPRI(param.0.Priority), types::MQPRI(17));
    }

    #[test]
    fn put_option_tuple_applies_leftmost_value_last() {
        let mut param = put_param();
        let options = (types::MQPRI(3), Some(types::MQPRI(7)));

        options.apply_param(&mut param);

        assert_eq!(types::MQPRI(param.0.Priority), types::MQPRI(3));
    }

    #[test]
    fn put_option_sets_message_descriptor_fields() {
        let mut param = put_param();
        let message_id = types::MessageId([1; 24]);
        let correlation_id = types::CorrelationId([2; 24]);
        let group_id = types::GroupId([3; 24]);
        let accounting_token = types::AccountingToken([4; libmqm_sys::MQ_ACCOUNTING_TOKEN_LENGTH]);
        let application_name = types::ApplName(crate::mqstr!("test-app"));
        let reply_queue = types::ReplyToQueueName(crate::mqstr!("reply-queue"));
        let reply_queue_manager = types::ReplyToQueueManagerName(crate::mqstr!("reply-qmgr"));
        let put_date = types::PutDate(crate::mqstr!("20261030"));
        let put_time = types::PutTime(crate::mqstr!("12345678"));
        let user_identifier = types::UserIdentifier(crate::mqstr!("test-user"));
        let identity_data = types::ApplIdentityData(crate::mqstr!("identity"));
        let origin_data = types::ApplOriginData(crate::mqstr!("test"));
        let options = (
            types::MQMT(1),
            types::MQEI(2),
            types::MQFB(3),
            types::MQPRI(4),
            types::MQPER(5),
            types::MQAT(6),
            types::MQMF(7),
        );
        let identifier_options = (message_id, correlation_id, group_id, accounting_token);
        let character_options = (
            application_name,
            reply_queue,
            reply_queue_manager,
            put_date,
            put_time,
            user_identifier,
            identity_data,
            origin_data,
        );

        options.apply_param(&mut param);
        identifier_options.apply_param(&mut param);
        character_options.apply_param(&mut param);

        assert_eq!(types::MQMT(param.0.MsgType), types::MQMT(1));
        assert_eq!(types::MQEI(param.0.Expiry), types::MQEI(2));
        assert_eq!(types::MQFB(param.0.Feedback), types::MQFB(3));
        assert_eq!(types::MQPRI(param.0.Priority), types::MQPRI(4));
        assert_eq!(types::MQPER(param.0.Persistence), types::MQPER(5));
        assert_eq!(types::MQAT(param.0.PutApplType), types::MQAT(6));
        assert_eq!(types::MQMF(param.0.MsgFlags), types::MQMF(7));
        assert_eq!(param.0.MsgId, message_id.0);
        assert_eq!(param.0.CorrelId, correlation_id.0);
        assert_eq!(param.0.GroupId, group_id.0);
        assert_eq!(param.0.AccountingToken, accounting_token.0);
        assert_eq!(param.0.PutApplName, *application_name.as_mqchar());
        assert_eq!(param.0.ReplyToQ, *reply_queue.as_mqchar());
        assert_eq!(param.0.ReplyToQMgr, *reply_queue_manager.as_mqchar());
        assert_eq!(param.0.PutDate, *put_date.as_mqchar());
        assert_eq!(param.0.PutTime, *put_time.as_mqchar());
        assert_eq!(param.0.UserIdentifier, *user_identifier.as_mqchar());
        assert_eq!(param.0.ApplIdentityData, *identity_data.as_mqchar());
        assert_eq!(param.0.ApplOriginData, *origin_data.as_mqchar());
    }

    #[test]
    fn put_option_merges_flags_and_copies_message_descriptor() {
        let mut param = put_param();
        let message_descriptor = structs::MQMD::new(libmqm_sys::MQMD {
            Priority: 19,
            ..default::MQMD_DEFAULT
        });
        let options = (
            constants::MQPMO_SYNCPOINT,
            constants::MQPMO_NEW_MSG_ID,
            constants::MQRO_EXCEPTION,
            constants::MQRO_EXPIRATION,
        );
        options.apply_param(&mut param);

        assert!(types::MQPMO(param.1.Options).contains(constants::MQPMO_SYNCPOINT | constants::MQPMO_NEW_MSG_ID));
        assert_eq!(
            types::MQRO(param.0.Report),
            constants::MQRO_EXCEPTION | constants::MQRO_EXPIRATION
        );

        message_descriptor.apply_param(&mut param);
        assert_eq!(types::MQPRI(param.0.Priority), types::MQPRI(19));
    }

    #[test]
    fn put_attributes_extract_message_descriptor_and_preserve_warning() {
        let mut param = put_param();
        let message_id = types::MessageId([1; 24]);
        let correlation_id = types::CorrelationId([2; 24]);
        let group_id = types::GroupId([3; 24]);
        let accounting_token = types::AccountingToken([4; libmqm_sys::MQ_ACCOUNTING_TOKEN_LENGTH]);
        let put_date = types::PutDate(crate::mqstr!("20261030"));
        let put_time = types::PutTime(crate::mqstr!("12345678"));
        let user_identifier = types::UserIdentifier(crate::mqstr!("test-user"));
        let identity_data = types::ApplIdentityData(crate::mqstr!("identity"));
        let origin_data = types::ApplOriginData(crate::mqstr!("test"));
        let mut call_count = 0;
        let warning = (constants::MQRC_TRUNCATED_MSG_ACCEPTED, "MQPUT");

        let result = <(
            types::PutDate,
            types::PutTime,
            types::MessageId,
            types::UserIdentifier,
            types::AccountingToken,
            types::ApplIdentityData,
            types::CorrelationId,
            types::GroupId,
        ) as option::PutAttr>::put_extract(&mut param, |(md, _)| {
            call_count += 1;
            md.PutDate = *put_date.as_mqchar();
            md.PutTime = *put_time.as_mqchar();
            md.MsgId = message_id.0;
            md.UserIdentifier = *user_identifier.as_mqchar();
            md.AccountingToken = accounting_token.0;
            md.ApplIdentityData = *identity_data.as_mqchar();
            md.CorrelId = correlation_id.0;
            md.GroupId = group_id.0;
            Ok(Completion::new_warning((), warning))
        })
        .expect("attribute extraction succeeds");

        assert_eq!(call_count, 1);
        assert_eq!(
            result.0,
            (
                put_date,
                put_time,
                message_id,
                user_identifier,
                accounting_token,
                identity_data,
                correlation_id,
                group_id,
            )
        );
        assert_eq!(result.warning(), Some(warning));

        let origin_result = <types::ApplOriginData as option::PutAttr>::put_extract(&mut param, |(md, _)| {
            md.ApplOriginData = *origin_data.as_mqchar();
            Ok(Completion::new(()))
        })
        .expect("attribute extraction succeeds");
        assert_eq!(origin_result.0, origin_data);
    }

    #[test]
    fn put_attribute_error_is_propagated() {
        let mut param = put_param();
        let error = crate::result::Error(constants::MQCC_FAILED, "MQPUT", constants::MQRC_NOT_AUTHORIZED);

        let result = <types::MessageId as option::PutAttr>::put_extract(&mut param, |_| Err(error));

        assert!(result.is_err_and(|e| e.0 == constants::MQCC_FAILED && e.2 == constants::MQRC_NOT_AUTHORIZED));
    }
}

#[cfg(test)]
#[cfg(feature = "mock")]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use std::error::Error;

    use libmqm_default as default;

    use super::*;
    use crate::{Properties, put::PutOption as _, test::mock, types::MQCMHO};

    #[test]
    fn property_action() -> Result<(), Box<dyn Error>> {
        let qm = mock::connect_ok(|mock_library| {
            let mut seq = mockall::Sequence::new();
            mock::properties_ok(mock_library, 0xf0f0, 1, &mut seq);
            mock::properties_ok(mock_library, 0x0e0e, 1, &mut seq);
            mock::properties_ok(mock_library, 0xaaaa, 1, &mut seq);
            mock::properties_ok(mock_library, 0xbbbb, 1, &mut seq);
        });

        let source = Properties::new(&qm, MQCMHO::default())?;
        let mut reply = Properties::new(&qm, MQCMHO::default())?;
        let mut forward = Properties::new(&qm, MQCMHO::default())?;
        let mut report = Properties::new(&qm, MQCMHO::default())?;

        for (action, action_value, new_handle) in [
            (PropertyAction::Reply(&source, &mut reply), constants::MQACTP_REPLY, 0x0e0e),
            (
                PropertyAction::Forward(&source, &mut forward),
                constants::MQACTP_FORWARD,
                0xaaaa,
            ),
            (PropertyAction::Report(&source, &mut report), constants::MQACTP_REPORT, 0xbbbb),
        ] {
            let mut put_param = (
                structs::MQMD::new(default::MQMD_DEFAULT),
                structs::MQPMO::new(default::MQPMO_DEFAULT),
            );
            action.apply_param(&mut put_param);

            assert_eq!(put_param.1.Version, mq::MQPMO_VERSION_3);
            assert_eq!(put_param.1.Action, action_value.0);
            assert_eq!(put_param.1.OriginalMsgHandle, 0xf0f0);
            assert_eq!(put_param.1.NewMsgHandle, new_handle);
        }

        let mut put_param = (
            structs::MQMD::new(default::MQMD_DEFAULT),
            structs::MQPMO::new(default::MQPMO_DEFAULT),
        );
        #[expect(clippy::unnecessary_mut_passed)]
        (&mut reply).apply_param(&mut put_param);
        assert_eq!(put_param.1.Version, mq::MQPMO_VERSION_3);
        assert_eq!(put_param.1.Action, constants::MQACTP_NEW.0);
        assert_eq!(put_param.1.OriginalMsgHandle, 0x0e0e);

        Ok(())
    }
}
