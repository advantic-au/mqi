use std::{
    ffi::c_void,
    ptr,
    sync::{Arc, Mutex},
};

use libmqm_sys::{self as mq, mock::MockMq};

use crate::{constants, types};

const MQCBC_DEFAULT: libmqm_sys::MQCBC = mq::MQCBC {
    StrucId: [67, 66, 67, 32],
    Version: mq::MQCBC_VERSION_2,
    CallType: 0,
    Hobj: mq::MQHO_NONE,
    CallbackArea: ptr::null_mut(),
    ConnectionArea: ptr::null_mut(),
    CompCode: mq::MQCC_OK,
    Reason: mq::MQRC_NONE,
    State: mq::MQCS_NONE,
    DataLength: 0,
    BufferLength: 0,
    Flags: mq::MQCBCF_NONE,
    ReconnectDelay: mq::MQRD_NO_DELAY,
};

pub fn mock_cb(mock_library: &mut MockMq, cb_type: types::MQCBT) {
    type MqCbFn = unsafe extern "C" fn(_: mq::MQHCONN, _: mq::PMQVOID, _: mq::PMQVOID, _: mq::PMQVOID, _: *const mq::MQCBC);
    #[derive(Clone)]
    struct MqCallback(*mut c_void, *mut c_void);
    unsafe impl Send for MqCallback {}

    let cb: Arc<Mutex<Option<MqCallback>>> = Arc::default();
    let cb_init = cb.clone();

    mock_library
        .expect_MQCB()
        .withf(move |_, op, cbd, _, _, _, _, _| {
            let cbd = cbd.expect("MQCBD should be non-null");
            let op = types::MQOP(*op);
            let callback_type = types::MQCBT(cbd.CallbackType);
            callback_type == cb_type
                && (op.contains(constants::MQOP_REGISTER) || op.contains(constants::MQOP_DEREGISTER))
        })
        .returning(move |hconn, op, cbd, _, _, _, cc, rc| {
            let cbd = cbd.expect("MQCBD should be non-null");
            let op = types::MQOP(op);
            let mut cb_init_lock = cb_init.lock().expect("mutex retrieval should succeed");
            let existing = match op {
                constants::MQOP_REGISTER => {
                    let cb_new = Some(MqCallback(cbd.CallbackArea, cbd.CallbackFunction));
                    cb_init_lock.clone_from(&cb_new);
                    cb_new
                }
                constants::MQOP_DEREGISTER => cb_init_lock.take(),
                _ => None,
            };
            drop(cb_init_lock);
            if let Some(MqCallback(area, function)) = existing {
                let mut cb_context = libmqm_sys::MQCBC {
                    CallType: match op {
                        constants::MQOP_REGISTER => libmqm_sys::MQCBCT_REGISTER_CALL,
                        constants::MQOP_DEREGISTER => libmqm_sys::MQCBCT_DEREGISTER_CALL,
                        _ => unreachable!(),
                    },
                    CallbackArea: area,
                    ..MQCBC_DEFAULT
                };
                unsafe {
                    let fn_cb = std::mem::transmute::<*const c_void, MqCbFn>(function);
                    fn_cb(hconn, ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), &raw mut cb_context);
                }
            }

            super::mqi_outcome_ok(cc, rc);
        });
    mock_library.expect_MQDISC().returning(move |hconn, cc, rc| {
        let callback = cb.lock().expect("mutex retrieval should succeed").take();
        if let Some(MqCallback(area, function)) = callback {
            let mut cbc = libmqm_sys::MQCBC {
                CallbackArea: area,
                CallType: libmqm_sys::MQCBCT_DEREGISTER_CALL,
                ..MQCBC_DEFAULT
            };
            unsafe {
                let fn_cb = std::mem::transmute::<*const c_void, MqCbFn>(function);
                fn_cb(*hconn, ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), &raw mut cbc);
            }
        }
        super::mqi_outcome_ok(cc, rc);
    });
}
