use libmqm_sys::function;

use crate::{
    core::{values::MQCBDO, Library, MqFunctions},
    sys, Error, MqStruct,
};

use super::{values::MQOP, Conn as _, Connection, ConnectionRef};

struct CallbackData<F, L> {
    options: MQCBDO,
    closure: F,
    mq: MqFunctions<L>,
}

fn event_callback<L, H, F>(hconn: sys::MQHCONN, _: sys::PMQVOID, _: sys::PMQVOID, _: sys::PMQVOID, cbc: *const sys::MQCBC)
where
    L: Library<MQ: function::Mqi> + Clone,
    F: FnMut(ConnectionRef<'_, L, H>, &MqStruct<sys::MQCBC>),
{
    unsafe {
        if let Some(context) = cbc.cast::<MqStruct<sys::MQCBC>>().as_ref() {
            if let Some(CallbackData {
                options, closure, mq, ..
            }) = context.CallbackArea.cast::<CallbackData<F, L>>().as_mut()
            {
                if (context.CallType != sys::MQCBCT_DEREGISTER_CALL) || (*options & sys::MQCBDO_DEREGISTER_CALL) != 0 {
                    closure(ConnectionRef::from_parts(hconn.into(), mq.clone()), context);
                }
                if context.CallType == sys::MQCBCT_DEREGISTER_CALL {
                    // Recreate the box so it deallocates / drops
                    let _ = Box::<CallbackData<F, L>>::from_raw(context.CallbackArea.cast());
                }
            }
        }
    }
}

impl<L, H> Connection<L, H>
where
    L: Library<MQ: function::Mqi> + Clone,
{
    pub fn register_event_handler<F>(&mut self, options: MQCBDO, closure: F) -> Result<(), Error>
    where
        F: FnMut(ConnectionRef<'_, L, H>, &MqStruct<sys::MQCBC>),
    {
        let cb_data: *mut CallbackData<F, L> = Box::into_raw(Box::from(CallbackData {
            options,
            closure,
            mq: self.mq().clone(),
        }));
        let mut cbd = MqStruct::<sys::MQCBD>::default();
        cbd.CallbackArea = cb_data.cast();
        cbd.Options = (options | sys::MQCBDO_DEREGISTER_CALL).0; // Always register for the deregister call
        cbd.CallbackFunction = event_callback::<L, H, F> as *mut _;
        cbd.CallbackType = sys::MQCBT_EVENT_HANDLER;

        self.mq()
            .mqcb(self.handle(), MQOP(sys::MQOP_REGISTER), &cbd, None, None::<&sys::MQMD>, None)?;

        Ok(())
    }
}
