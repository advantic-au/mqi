use std::{
    collections::HashMap,
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
    struct MqCallback(*mut c_void, *mut c_void, types::MQCBDO);
    unsafe impl Send for MqCallback {}

    type CbMap = HashMap<(mq::MQHCONN, Option<mq::MQHOBJ>), MqCallback>;
    let cb_map: Arc<Mutex<CbMap>> = Arc::default();
    let cb_disc = cb_map.clone();

    mock_library
        .expect_MQCB()
        .withf(move |_, op, cbd, _, _, _, _, _| {
            let cbd = cbd.expect("MQCBD should be non-null");
            let op = types::MQOP(*op);
            let callback_type = types::MQCBT(cbd.CallbackType);
            callback_type == cb_type && (op.contains(constants::MQOP_REGISTER) || op.contains(constants::MQOP_DEREGISTER))
        })
        .returning(move |hconn, op, cbd, hobj, _, _, cc, rc| {
            let cbd = cbd.expect("MQCBD should be non-null");
            let op = types::MQOP(op);

            let maybe_hobj = Some(hobj).filter(|o| *o != mq::MQHO_NONE);

            if op == constants::MQOP_REGISTER {
                let mut cb_lock = cb_map.lock().expect("Callback write acquired");
                let cb_new = MqCallback(cbd.CallbackArea, cbd.CallbackFunction, types::MQCBDO(cbd.Options));
                let _old = cb_lock.insert((hconn, maybe_hobj), cb_new);
            }

            let cb_lookup = maybe_hobj
                .map(|ho| (hconn, Some(ho)))
                .into_iter()
                .chain(std::iter::once((hconn, None)))
                .filter_map(|(hconn, maybe_hobj)| {
                    cb_map
                        .lock()
                        .expect("Callback read acquired")
                        .get(&(hconn, maybe_hobj))
                        .cloned()
                });

            for MqCallback(area, function, options) in cb_lookup {
                // Execute the callback
                let mut cb_context = libmqm_sys::MQCBC {
                    CallType: match op {
                        constants::MQOP_REGISTER => libmqm_sys::MQCBCT_REGISTER_CALL,
                        constants::MQOP_DEREGISTER => libmqm_sys::MQCBCT_DEREGISTER_CALL,
                        _ => unreachable!(),
                    },
                    CallbackArea: area,
                    ..MQCBC_DEFAULT
                };
                if options.0 & op.0 != 0 {
                    unsafe {
                        let fn_cb = std::mem::transmute::<*const c_void, MqCbFn>(function);
                        fn_cb(hconn, ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), &raw mut cb_context);
                    }
                }
            }

            if op == constants::MQOP_DEREGISTER {
                let mut cb_lock = cb_map.lock().expect("Callback write acquired");
                let _old = cb_lock.remove(&(hconn, maybe_hobj));
            }

            super::mqi_outcome_ok(cc, rc);
        });
    mock_library.expect_MQDISC().returning(move |hconn, cc, rc| {
        let callback = cb_disc.lock().expect("Callback write acquired").remove(&(*hconn, None));
        if let Some(MqCallback(area, function, options)) = callback {
            let mut cbc = libmqm_sys::MQCBC {
                CallbackArea: area,
                CallType: libmqm_sys::MQCBCT_DEREGISTER_CALL,
                ..MQCBC_DEFAULT
            };
            if options.contains(constants::MQCBDO_DEREGISTER_CALL) {
                unsafe {
                    let fn_cb = std::mem::transmute::<*const c_void, MqCbFn>(function);
                    fn_cb(*hconn, ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), &raw mut cbc);
                }
            }
        }
        super::mqi_outcome_ok(cc, rc);
    });
}
