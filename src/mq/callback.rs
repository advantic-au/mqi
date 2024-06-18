use std::{marker::PhantomData, pin::Pin, ptr};

use libmqm_sys::function;

use crate::{
    core::{ConnectionHandle, Library},
    sys, MqMask, MqStruct, QueueManagerShare, MQCBDO,
};

/// Holds the callback function and ties it to the lifetime of the connection
pub struct CallbackHandle<'a, F> {
    _callback: Pin<Box<F>>,
    _marker: PhantomData<&'a mut ()>,
}

fn event_callback<F: FnMut(ConnectionHandle, &MqStruct<sys::MQCBC>)>(
    conn: sys::MQHCONN,
    _: sys::PMQVOID,
    _: sys::PMQVOID,
    _: sys::PMQVOID,
    cbc: *const sys::MQCBC,
) {
    unsafe {
        if let Some(context) = cbc.cast::<MqStruct<sys::MQCBC>>().as_ref() {
            let cb = &mut *context.CallbackArea.cast::<F>();
            cb(conn.into(), context);
        }
    }
}

impl<L: Library<MQ: function::MQI>, H> QueueManagerShare<L, H> {
    pub fn register_event_handler<F: FnMut(ConnectionHandle, &MqStruct<sys::MQCBC>)>(
        &self,
        options: MqMask<MQCBDO>,
        callback: F,
    ) -> Result<CallbackHandle<'_, F>, crate::Error> {
        let mut cbd = MqStruct::<sys::MQCBD>::default();
        let cb_pin = Box::pin(callback);
        cbd.CallbackArea = ptr::addr_of!(*cb_pin).cast_mut().cast();
        cbd.Options = options.0;
        cbd.CallbackFunction = event_callback::<F> as *mut _;
        cbd.CallbackType = sys::MQCBT_EVENT_HANDLER;

        self.mq().mqcb(
            self.handle(),
            MqMask::from(sys::MQOP_REGISTER),
            &cbd,
            None,
            None::<&sys::MQMD>,
            None,
        )?;

        Ok(CallbackHandle {
            _callback: cb_pin,
            _marker: PhantomData,
        })
    }
}
