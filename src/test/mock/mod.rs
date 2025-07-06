#![expect(clippy::allow_attributes)]
#![expect(non_snake_case)]

use std::{cmp, rc::Rc, slice};

#[cfg(feature = "mqai")]
use libmqm_sys::{self as mq, mock::MockMq};

use crate::{Connection, Library, ResultCompExt, ThreadNone, connect_lib, constants, put::PutMessage, types};

pub mod callback;

pub unsafe fn copy_to_mq_data(data: &[u8], buf_len: mq::MQLONG, buf_target: mq::PMQVOID, data_len: mq::PMQLONG) -> mq::MQLONG {
    let write_len = cmp::min(size_of_val(data), buf_len.try_into().expect("convertable buffer length"));
    let target = unsafe { slice::from_raw_parts_mut(buf_target.cast(), write_len) };
    target.copy_from_slice(&data[..write_len]);
    let write_len_long = write_len.try_into().expect("convertable buffer length");
    unsafe { *data_len = write_len_long };
    write_len_long
}

pub fn connx_outcome(mock: &mut MockMq, hconn: mq::MQHCONN, comp_code: types::MQCC, reason: types::MQRC) {
    mock.expect_MQCONNX().returning(
        move |_, _, pHconn: &mut mq::MQHCONN, pCompCode: &mut mq::MQLONG, pReason: &mut mq::MQLONG| {
            *pHconn = hconn;
            mqi_outcome(pCompCode, pReason, comp_code, reason);
        },
    );
}

pub fn disc_outcome(mock: &mut MockMq, comp_code: types::MQCC, reason: types::MQRC) {
    mock.expect_MQDISC()
        .returning(move |_, pCompCode: &mut mq::MQLONG, pReason: &mut mq::MQLONG| {
            mqi_outcome(pCompCode, pReason, comp_code, reason);
        });
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub const fn mqi_outcome(pCompCode: &mut mq::MQLONG, pReason: &mut mq::MQLONG, comp_code: types::MQCC, reason: types::MQRC) {
    *pCompCode = comp_code.0;
    *pReason = reason.0;
}

pub const fn mqi_outcome_ok(pCompCode: &mut mq::MQLONG, pReason: &mut mq::MQLONG) {
    mqi_outcome(pCompCode, pReason, constants::MQCC_OK, constants::MQRC_NONE);
}

pub fn properties_ok(mock: &mut MockMq, hMsg: mq::MQHMSG, count: impl Into<mockall::TimesRange>, seq: &mut mockall::Sequence) {
    mock.expect_MQCRTMH()
        .returning(move |_, _, hmsg, comp_code, reason| {
            *hmsg = hMsg;
            mqi_outcome_ok(comp_code, reason);
        })
        .times(count)
        .in_sequence(seq);

    mock.expect_MQDLTMH()
        .withf(move |_, &msg, _, _, _| msg == hMsg)
        .returning(|_, _, _, comp_code, reason| {
            mqi_outcome_ok(comp_code, reason);
        });
}

pub fn get_error(mock: &mut MockMq, mqrc: types::MQRC, count: impl Into<mockall::TimesRange>, seq: &mut mockall::Sequence) {
    mock.expect_MQGET()
        .returning(move |_, _, _, _, _, _, _, cc, rc| mqi_outcome(cc, rc, constants::MQCC_FAILED, mqrc))
        .times(count)
        .in_sequence(seq);
}

pub fn get_ok(
    mock: &mut MockMq,
    message: &'static (impl PutMessage + ?Sized),
    count: impl Into<mockall::TimesRange>,
    seq: &mut mockall::Sequence,
) {
    mock.expect_MQGET()
        .returning_st(move |_, _, mqmd, _, buffer_len, buffer, data_length, cc, rc| {
            let md: &mut mq::MQMD = unsafe { &mut *(mqmd.cast()) };
            let fmt = message.format().fmt.into_ascii();
            md.Format = *unsafe { &*std::ptr::from_ref(fmt.as_ref()).cast() };
            md.Encoding = message.format().encoding.0;

            let msg = message.render();
            let mock_length = unsafe { copy_to_mq_data(&msg, buffer_len, buffer, data_length) };

            mqi_outcome(
                cc,
                rc,
                constants::MQCC_OK,
                if mock_length < buffer_len {
                    constants::MQRC_TRUNCATED_MSG_ACCEPTED
                } else {
                    constants::MQRC_NONE
                },
            );
        })
        .times(count)
        .in_sequence(seq);
}

pub fn open_ok(mock: &mut MockMq, hObj: mq::MQHOBJ, count: impl Into<mockall::TimesRange>, seq: &mut mockall::Sequence) {
    mock.expect_MQOPEN()
        .returning(move |_, _, _, hobj, comp_code, reason| {
            *hobj = hObj;
            mqi_outcome_ok(comp_code, reason);
        })
        .times(count)
        .in_sequence(seq);

    close_ok(mock, hObj);
}

fn close_ok(mock: &mut MockMq, hObj: mq::MQHOBJ) {
    mock.expect_MQCLOSE()
        .withf(move |_, &obj, _, _, _| obj == hObj)
        .returning(|_, obj, _, comp_code, reason| {
            *obj = mq::MQHO_UNUSABLE_HOBJ;
            mqi_outcome_ok(comp_code, reason);
        });
}

pub fn subscribe_managed_ok(
    mock: &mut MockMq,
    hObj: mq::MQHOBJ,
    hSub: mq::MQHOBJ,
    count: impl Into<mockall::TimesRange>,
    seq: &mut mockall::Sequence,
) {
    mock.expect_MQSUB()
        .withf(|_, mqsd, obj, _, _, _| {
            // let sd: &mq::MQSD = unsafe { &*(mqsd.cast()) };
            mqsd.Options & mq::MQSO_MANAGED != 0 && !obj.is_none()
        })
        .returning(move |_, _, obj, sub, cc, rc| {
            if let Some(obj) = obj {
                *obj = hObj;
            }
            *sub = hSub;
            mqi_outcome_ok(cc, rc);
        })
        .times(count)
        .in_sequence(seq);

    close_ok(mock, hObj);
    close_ok(mock, hSub);
}

#[cfg(feature = "mqai")]
pub mod mqai {
    use libmqm_constants::constants;
    use libmqm_sys::{Mqai, mock::MockMq};

    use crate::{Library, types};

    pub fn get_bag_error(
        mock: &mut MockMq,
        mqrc: types::MQRC,
        count: impl Into<mockall::TimesRange>,
        seq: &mut mockall::Sequence,
    ) {
        mock.expect_mqGetBag()
            .returning(move |_, _, _, _, _, cc, rc| super::mqi_outcome(cc, rc, constants::MQCC_FAILED, mqrc))
            .times(count)
            .in_sequence(seq);
    }

    pub fn get_bag_ok(mock: &mut MockMq, count: impl Into<mockall::TimesRange>, seq: &mut mockall::Sequence) {
        mock.expect_mqGetBag()
            .returning(move |_, _, _, _, _, cc, rc| super::mqi_outcome_ok(cc, rc))
            .times(count)
            .in_sequence(seq);
    }

    pub fn real_bag(mock: &mut MockMq, mqai: impl Library<MQ: Mqai> + Clone + Send + 'static) {
        unsafe {
            // TODO: Add more bag function passthroughs
            let mq = mqai.clone();
            mock.expect_mqCreateBag()
                .returning(move |option, bag, cc, rc| mq.lib().mqCreateBag(option, bag, cc, rc));
            let mq = mqai.clone();
            mock.expect_mqDeleteBag()
                .returning(move |bag, cc, rc| mq.lib().mqDeleteBag(bag, cc, rc));
            let mq = mqai.clone();
            mock.expect_mqSetInteger()
                .returning(move |a, b, c, d, e, f| mq.lib().mqSetInteger(a, b, c, d, e, f));
            let mq = mqai.clone();
            mock.expect_mqAddInteger()
                .returning(move |a, b, c, d, e| mq.lib().mqAddInteger(a, b, c, d, e));
            let mq = mqai.clone();
            mock.expect_mqInquireInteger()
                .returning(move |a, b, c, d, e, f| mq.lib().mqInquireInteger(a, b, c, d, e, f));
            let mq = mqai;
            mock.expect_mqAddString()
                .returning(move |a, b, c, d, e, f| mq.lib().mqAddString(a, b, c, d, e, f));
        }
    }
}

impl Library for MockMq {
    type MQ = Self;

    fn lib(&self) -> &Self::MQ {
        self
    }
}

pub fn connect_ok<F>(f: F) -> Connection<Rc<MockMq>, ThreadNone>
where
    F: FnOnce(&mut MockMq),
{
    let mut mock = MockMq::new();
    connx_outcome(&mut mock, 0x0d0d, constants::MQCC_OK, constants::MQRC_NONE);
    disc_outcome(&mut mock, constants::MQCC_OK, constants::MQRC_NONE);
    f(&mut mock);

    connect_lib(Rc::from(mock), &())
        .warn_as_error()
        .expect("should not fail or produce a warning")
}
