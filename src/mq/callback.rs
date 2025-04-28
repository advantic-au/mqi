use libmqm_sys::Mqi;
use libmqm_default as default;

use crate::types;
use crate::{
    core::{Library, MqFunctions},
    sys, Error, MqStruct, constants,
};

use super::{Conn as _, Connection, ConnectionRef};

struct CallbackData<F, L> {
    options: types::MQCBDO,
    closure: F,
    mq: MqFunctions<L>,
}

extern "C" fn event_callback<L, H, F>(
    hconn: sys::MQHCONN,
    _: sys::PMQVOID,
    _: sys::PMQVOID,
    _: sys::PMQVOID,
    cbc: *const sys::MQCBC,
) where
    L: Library<MQ: Mqi> + Clone,
    F: FnMut(ConnectionRef<L, H>, &MqStruct<sys::MQCBC>),
{
    unsafe {
        if let Some(context) = cbc.cast::<MqStruct<sys::MQCBC>>().as_ref() {
            if let Some(CallbackData {
                options, closure, mq, ..
            }) = context.CallbackArea.cast::<CallbackData<F, L>>().as_mut()
            {
                let is_deregister = types::MQCBCT(context.CallType) == constants::MQCBCT_DEREGISTER_CALL;
                if !is_deregister || options.contains(constants::MQCBDO_DEREGISTER_CALL) {
                    closure(ConnectionRef::from_parts(hconn.into(), mq.clone()), context);
                }
                if is_deregister {
                    // Recreate the box so it deallocates / drops
                    let _ = Box::<CallbackData<F, L>>::from_raw(context.CallbackArea.cast());
                }
            }
        }
    }
}

impl<L, H> Connection<L, H>
where
    L: Library<MQ: Mqi> + Clone,
{
    pub fn register_event_handler<F>(&mut self, options: types::MQCBDO, closure: F) -> Result<(), Error>
    where
        F: FnMut(ConnectionRef<L, H>, &MqStruct<sys::MQCBC>),
    {
        let cb_data: *mut CallbackData<F, L> = Box::into_raw(Box::from(CallbackData {
            options,
            closure,
            mq: self.mq().clone(),
        }));
        let mut cbd = MqStruct::new(default::MQCBD_DEFAULT);
        cbd.CallbackArea = cb_data.cast();
        *cbd.Options.as_mut() = options | constants::MQCBDO_DEREGISTER_CALL; // Always register for the deregister call
        cbd.CallbackFunction = event_callback::<L, H, F> as *mut _;
        *cbd.CallbackType.as_mut() = constants::MQCBT_EVENT_HANDLER;

        self.mq()
            .mqcb(self.handle(), constants::MQOP_REGISTER, &cbd, None, None::<&sys::MQMD>, None)?;

        Ok(())
    }
}
