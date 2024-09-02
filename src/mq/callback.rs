use libmqm_sys::function;

use crate::{
    core::{
        values::{self, MQCBDO},
        ConnectionHandle, Library,
    },
    sys, Error, MqStruct, QueueManagerShare,
};

type CallbackData<F> = (MQCBDO, F);

fn event_callback<F: FnMut(&ConnectionHandle, &MqStruct<sys::MQCBC>)>(
    conn: sys::MQHCONN,
    _: sys::PMQVOID,
    _: sys::PMQVOID,
    _: sys::PMQVOID,
    cbc: *const sys::MQCBC,
) {
    unsafe {
        if let Some(context) = cbc.cast::<MqStruct<sys::MQCBC>>().as_ref() {
            if let Some((options, cb)) = context.CallbackArea.cast::<CallbackData<F>>().as_mut() {
                if (context.CallType != sys::MQCBCT_DEREGISTER_CALL) || (*options & sys::MQCBDO_DEREGISTER_CALL) != 0 {
                    cb(&conn.into(), context);
                }
                if context.CallType == sys::MQCBCT_DEREGISTER_CALL {
                    let _ = Box::<CallbackData<F>>::from_raw(context.CallbackArea.cast());
                    // Recreate the box so it deallocates
                }
            }
        }
    }
}

impl<'a, L: Library<MQ: function::MQI>, H> QueueManagerShare<'a, L, H> {
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn register_event_handler<F: FnMut(&ConnectionHandle, &MqStruct<sys::MQCBC>) + 'a + Send>(
        &mut self,
        options: MQCBDO,
        closure: F,
    ) -> Result<(), Error> {
        let cb_data: *mut CallbackData<F> = Box::into_raw(Box::from((options, closure)));
        let mut cbd = MqStruct::<sys::MQCBD>::default();
        cbd.CallbackArea = cb_data.cast();
        cbd.Options = (options | sys::MQCBDO_DEREGISTER_CALL).0; // Always register for the deregister call
        cbd.CallbackFunction = event_callback::<F> as *mut _;
        cbd.CallbackType = sys::MQCBT_EVENT_HANDLER;

        self.mq().mqcb(
            self.handle(),
            values::MQOP(sys::MQOP_REGISTER),
            &cbd,
            None,
            None::<&sys::MQMD>,
            None,
        )?;

        Ok(())
    }
}
