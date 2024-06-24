use std::{pin::Pin, ptr};

use libmqm_sys::function;

use crate::{
    core::{mqai, ConnectionHandle, Library}, sys, Error, MqMask, MqStruct, QueueManagerShare, MQCBDO
};

/// Holds the callback function and ties it to the lifetime of the connection
pub struct CallbackHandle<F> {
    callback: Pin<Box<F>>,
}

impl<'a, L: Library<MQ: function::MQI>,  H> QueueManagerShare<'a, L, H> {
    fn event_callback<F: FnMut(&'_ QueueManagerShare<'_, L, H>, &'_ MqStruct<sys::MQCBC>)>(
        conn: sys::MQHCONN,
        _: sys::PMQVOID,
        _: sys::PMQVOID,
        _: sys::PMQVOID,
        cbc: *const sys::MQCBC,
    ) {
        unsafe {
            if let Some(context) = cbc.cast::<MqStruct<sys::MQCBC>>().as_ref() {
                let (mq, cb) = &mut *context.CallbackArea.cast::<(L, F)>();
                let qm = Self {
                    handle: conn.into(),
                    mq,
                    _share: std::marker::PhantomData,
                    _ref: std::marker::PhantomData,
                };
                cb(&qm, context);
                if context.CallType == sys::MQCBCT_DEREGISTER_CALL {
                    let _ = Box::<(L, F)>::from_raw(context.CallbackArea.cast());
                }
            }
        }
    }
    
}

impl<F: FnMut(ConnectionHandle, &'_ MqStruct<sys::MQCBC>)> From<F> for CallbackHandle<F> {
    fn from(f: F)-> Self {
        Self { callback: Box::pin(f) }
    }
}

impl<'a, L: Library<MQ: function::MQI>, H> QueueManagerShare<'a, L, H> {

    pub fn register_event_handler<F: FnMut(&QueueManagerShare<'_, L, H>, &'_ MqStruct<sys::MQCBC>)>(
        &mut self,
        options: MqMask<MQCBDO>,
        handle: &'a F,
    ) -> Result<(), Error> {
        let mut cbd = MqStruct::<sys::MQCBD>::default();
        cbd.CallbackArea = ptr::addr_of!(*handle).cast_mut().cast();
        cbd.Options = options.0;
        cbd.CallbackFunction = QueueManagerShare::<L, H>::event_callback::<F> as *mut _;
        cbd.CallbackType = sys::MQCBT_EVENT_HANDLER;

        self.mq().mqcb(
            self.handle(),
            MqMask::from(sys::MQOP_REGISTER),
            &cbd,
            None,
            None::<&sys::MQMD>,
            None,
        )?;

        Ok(())
    }
}
