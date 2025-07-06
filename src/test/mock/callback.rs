use std::{
    ffi::c_void,
    ptr,
    sync::{Arc, Mutex},
};

use libmqm_sys::{self as mq, mock::MockMq};

use crate::{constants, types};

#[must_use]
pub fn connect_ok_event_cb() -> MockMq {
    type MqCbFn = unsafe extern "C" fn(_: mq::MQHCONN, _: mq::PMQVOID, _: mq::PMQVOID, _: mq::PMQVOID, _: *const mq::MQCBC);
    struct MqCallback(*const c_void, *const c_void);
    unsafe impl Send for MqCallback {}

    let mut mock_library = MockMq::new();
    super::connx_outcome(&mut mock_library, 0x0d0d, constants::MQCC_OK, constants::MQRC_NONE);
    let cb: Arc<Mutex<Option<MqCallback>>> = Arc::default();
    let cb_init = cb.clone();
    mock_library
        .expect_MQCB()
        .withf(|_, _, cbd, _, _, _, _, _| {
            let cbd = cbd.expect("MQCBD should be non-null");
            types::MQCBT(cbd.CallbackType) == constants::MQCBT_EVENT_HANDLER
                && types::MQCBDO(cbd.Options).contains(constants::MQCBDO_DEREGISTER_CALL)
        })
        .returning(move |_, _, cbd, _, _, _, cc, rc| {
            let cbd = cbd.expect("MQCBD should be non-null");
            cb_init
                .lock()
                .expect("mutex retrieval should succeed")
                .replace(MqCallback(cbd.CallbackArea, cbd.CallbackFunction));
            super::mqi_outcome_ok(cc, rc);
        });
    mock_library.expect_MQDISC().returning(move |hconn, cc, rc| {
        let callback = cb.lock().expect("mutex retrieval should succeed").take();
        if let Some(MqCallback(area, function)) = callback {
            let mut cbc = mq::MQCBC {
                StrucId: [67, 66, 67, 32],
                Version: mq::MQCBC_VERSION_2,
                CallType: mq::MQCBCT_DEREGISTER_CALL,
                Hobj: mq::MQHO_NONE,
                CallbackArea: area.cast_mut(),
                ConnectionArea: ptr::null_mut(),
                CompCode: mq::MQCC_OK,
                Reason: mq::MQRC_NONE,
                State: mq::MQCS_NONE,
                DataLength: 0,
                BufferLength: 0,
                Flags: mq::MQCBCF_NONE,
                ReconnectDelay: mq::MQRD_NO_DELAY,
            };
            unsafe {
                let fn_cb = std::mem::transmute::<*const c_void, MqCbFn>(function);
                fn_cb(*hconn, ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), &raw mut cbc);
            }
        }
        super::mqi_outcome_ok(cc, rc);
    });
    mock_library
}
