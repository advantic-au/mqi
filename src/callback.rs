use std::marker::PhantomData;

use libmqm_default as default;
use libmqm_sys::{self as mq, MQMD, Mqi};

use crate::{constants, result::ResultComp, structs, types, Conn, Connection, ConnectionRef, Library, MqFunctions};

pub struct ConnectionCallback<'a, C: Conn> {
    connection: C,
    _cb: PhantomData<&'a ()>,
}

impl<'cb, C: Conn> ConnectionCallback<'cb, C> {
    pub const fn new(connection: C) -> Self {
        Self {
            connection,
            _cb: PhantomData,
        }
    }

    pub fn event_handler<F: FnMut(ConnectionRef<C::Lib, C::Thread>, &structs::MQCBC) + Send + 'cb>(
        &self,
        options: types::MQCBDO,
        closure: F,
    ) -> ResultComp<()>
    where
        C::Lib: Clone,
    {
        let functions = self.connection.mq();
        let handle = self.connection.handle();

        let cb_data: *mut CallbackData<'cb, C::Lib, C::Thread> = Box::into_raw(Box::from(CallbackData {
            options,
            closure: Box::new(closure),
            mq: functions.clone(),
        }));
        let mut cbd = structs::MQCBD::new(default::MQCBD_DEFAULT);
        let _ = unsafe { functions.mqcb(handle, constants::MQOP_DEREGISTER, Some(&cbd), None, None::<&MQMD>, None) };

        cbd.CallbackArea = cb_data.cast();
        *cbd.Options.as_mut() = options | constants::MQCBDO_DEREGISTER_CALL; // Always register for the deregister call
        cbd.CallbackFunction = event_callback::<C::Lib, C::Thread> as *mut _;
        *cbd.CallbackType.as_mut() = constants::MQCBT_EVENT_HANDLER;

        // SAFETY: MQCBD registered with valid pointers
        unsafe { functions.mqcb(handle, constants::MQOP_REGISTER, Some(&cbd), None, None::<&MQMD>, None) }
    }

    pub fn unregister(&mut self) -> ResultComp<()> {
        let functions = self.connection.mq();
        let handle = self.connection.handle();
        let cbd = structs::MQCBD::new(default::MQCBD_DEFAULT);
        unsafe { functions.mqcb(handle, constants::MQOP_DEREGISTER, Some(&cbd), None, None::<&MQMD>, None) }
    }
}

impl<C: Conn> Drop for ConnectionCallback<'_, C> {
    fn drop(&mut self) {
        let _ = self.unregister();
    }
}

impl<C: std::ops::Deref<Target = Connection<L, H>> + Conn, L: Library<MQ: Mqi>, H> std::ops::Deref for ConnectionCallback<'_, C> {
    type Target = Connection<L, H>;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl<C: Conn> Conn for ConnectionCallback<'_, C> {
    type Lib = C::Lib;
    type Thread = C::Thread;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        self.connection.mq()
    }

    fn handle(&self) -> crate::handle::ConnectionHandle {
        self.connection.handle()
    }
}

type BoxedEventCallback<'a, L, H> = Box<dyn FnMut(ConnectionRef<L, H>, &structs::MQCBC) + Send + 'a>;

struct CallbackData<'a, L: Library<MQ: Mqi>, H> {
    options: types::MQCBDO,
    closure: BoxedEventCallback<'a, L, H>,
    mq: MqFunctions<L>,
}

unsafe extern "C" fn event_callback<L, H>(
    hconn: mq::MQHCONN,
    _mqmd: mq::PMQVOID,   // Not used for MQCBT_EVENT_HANDLER
    _mqgmo: mq::PMQVOID,  // Not used for MQCBT_EVENT_HANDLER
    _buffer: mq::PMQVOID, // Not used for MQCBT_EVENT_HANDLER
    cbc: *const mq::MQCBC,
) where
    L: Library<MQ: Mqi> + Clone,
{
    // SAFETY: MQCBC will always be non-null
    if let Some(context) = unsafe { cbc.cast::<structs::MQCBC>().as_ref() } {
        // SAFETY: CallbackArea is always set by `event_handler`
        if let Some(CallbackData { options, closure, mq }) = unsafe { context.CallbackArea.cast::<CallbackData<L, H>>().as_mut() }
        {
            let is_deregister = types::MQCBCT(context.CallType) == constants::MQCBCT_DEREGISTER_CALL;
            if !is_deregister || options.contains(constants::MQCBDO_DEREGISTER_CALL) {
                closure(ConnectionRef::from_parts(hconn.into(), mq.clone()), context);
            }
            if is_deregister {
                // Recreate the box so it deallocates / drops
                // SAFETY: The only place the Callback handler is reconstructed
                let _ = unsafe { Box::<CallbackData<L, H>>::from_raw(context.CallbackArea.cast()) };
            }
        }
    }
}
