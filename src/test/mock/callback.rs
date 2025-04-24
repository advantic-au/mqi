use std::{
    ffi::c_void,
    ptr,
    sync::{Arc, Mutex},
};

use super::MockFunctions;
use crate::{sys, constants};

impl MockFunctions {
    #[must_use]
    pub fn connect_ok_event_cb() -> Self {
        type MqCbFn =
            unsafe extern "C" fn(_: sys::MQHCONN, _: sys::PMQVOID, _: sys::PMQVOID, _: sys::PMQVOID, _: *const sys::MQCBC);
        struct MqCallback(*const c_void, *const c_void);
        unsafe impl Send for MqCallback {}

        let mut mock_library = Self::new();
        mock_library.connx_outcome(0x0d0d, constants::MQCC_OK, constants::MQRC_NONE);
        let cb: Arc<Mutex<Option<MqCallback>>> = Arc::default();
        let cb_init = cb.clone();
        mock_library
            .expect_MQCB()
            .withf(|_, _, cbd_ptr, _, _, _, _, _| {
                let cbd = unsafe { cbd_ptr.cast::<sys::MQCBD>().as_ref().expect("MQCBD should be non-null") };
                cbd.CallbackType == sys::MQCBT_EVENT_HANDLER && (cbd.Options & sys::MQCBDO_DEREGISTER_CALL != 0)
            })
            .returning(move |_, _, cbd_ptr, _, _, _, cc, rc| {
                let cbd = unsafe { cbd_ptr.cast::<sys::MQCBD>().as_ref().expect("MQCBD should be non-null") };
                cb_init
                    .lock()
                    .expect("mutex retrieval should succeed")
                    .replace(MqCallback(cbd.CallbackArea, cbd.CallbackFunction));
                Self::mqi_outcome_ok(cc, rc);
            });
        mock_library.expect_MQDISC().returning(move |hconn, cc, rc| {
            let callback = cb.lock().expect("mutex retrieval should succeed").take();
            if let Some(MqCallback(area, function)) = callback {
                let mut cbc = sys::MQCBC {
                    StrucId: [67, 66, 67, 32],
                    Version: sys::MQCBC_VERSION_2,
                    CallType: sys::MQCBCT_DEREGISTER_CALL,
                    Hobj: sys::MQHO_NONE,
                    CallbackArea: area.cast_mut(),
                    ConnectionArea: ptr::null_mut(),
                    CompCode: sys::MQCC_OK,
                    Reason: sys::MQRC_NONE,
                    State: sys::MQCS_NONE,
                    DataLength: 0,
                    BufferLength: 0,
                    Flags: sys::MQCBCF_NONE,
                    ReconnectDelay: sys::MQRD_NO_DELAY,
                };
                unsafe {
                    let fn_cb = std::mem::transmute::<*const c_void, MqCbFn>(function);
                    fn_cb(*hconn, ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), &raw mut cbc);
                }
            }
            Self::mqi_outcome_ok(cc, rc);
        });
        mock_library
    }
}
