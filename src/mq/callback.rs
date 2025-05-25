use libmqm_sys::Mqi;
use libmqm_sys::lib as sys;
use libmqm_default as default;

use crate::types;
use crate::{
    core::{Library, MqFunctions},
    Error, constants,
    prelude::*,
    structs,
};

use super::{Connection, ConnectionRef};

struct CallbackData<F, L> {
    options: types::MQCBDO,
    closure: F,
    mq: MqFunctions<L>,
}

unsafe extern "C" fn event_callback<L, H, F>(
    hconn: sys::MQHCONN,
    _mqmd: sys::PMQVOID,   // Not used for MQCBT_EVENT_HANDLER
    _mqgmo: sys::PMQVOID,  // Not used for MQCBT_EVENT_HANDLER
    _buffer: sys::PMQVOID, // Not used for MQCBT_EVENT_HANDLER
    cbc: *const sys::MQCBC,
) where
    L: Library<MQ: Mqi> + Clone,
    F: FnMut(ConnectionRef<L, H>, &structs::MQCBC),
{
    // unsafe {
    // SAFETY: MQCBC will always be non-null
    if let Some(context) = unsafe { cbc.cast::<structs::MQCBC>().as_ref() } {
        // SAFETY: CallbackArea is always set by `register_event_handler`
        if let Some(CallbackData { options, closure, mq }) = unsafe { context.CallbackArea.cast::<CallbackData<F, L>>().as_mut() }
        {
            let is_deregister = types::MQCBCT(context.CallType) == constants::MQCBCT_DEREGISTER_CALL;
            if !is_deregister || options.contains(constants::MQCBDO_DEREGISTER_CALL) {
                closure(ConnectionRef::from_parts(hconn.into(), mq.clone()), context);
            }
            if is_deregister {
                // Recreate the box so it deallocates / drops
                // SAFETY: The only place the Callback handler is reconstructed
                let _ = unsafe { Box::<CallbackData<F, L>>::from_raw(context.CallbackArea.cast()) };
            }
        }
    }
    // }
}

impl<L, H> Connection<L, H>
where
    L: Library<MQ: Mqi> + Clone,
{
    /// # Safety
    /// Consumers of [`register_event_handler`](Connection::register_event_handler) must handle and read the pointers in [`MQCBDO`](types::MQCBDO) correctly
    pub unsafe fn register_event_handler<F>(&mut self, options: types::MQCBDO, closure: F) -> Result<(), Error>
    where
        F: FnMut(ConnectionRef<L, H>, &structs::MQCBC),
    {
        let cb_data: *mut CallbackData<F, L> = Box::into_raw(Box::from(CallbackData {
            options,
            closure,
            mq: self.mq().clone(),
        }));
        let mut cbd = structs::MQCBD::new(default::MQCBD_DEFAULT);
        cbd.CallbackArea = cb_data.cast();
        *cbd.Options.as_mut() = options | constants::MQCBDO_DEREGISTER_CALL; // Always register for the deregister call
        cbd.CallbackFunction = event_callback::<L, H, F> as *mut _;
        *cbd.CallbackType.as_mut() = constants::MQCBT_EVENT_HANDLER;

        // SAFETY: MQCBD registered with valid pointers
        unsafe {
            self.mq()
                .mqcb(self.handle(), constants::MQOP_REGISTER, &cbd, None, None::<&sys::MQMD>, None)
        }?;

        Ok(())
    }
}
