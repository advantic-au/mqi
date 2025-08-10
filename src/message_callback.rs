use std::{marker::PhantomData, mem::ManuallyDrop};

use libmqm_default as default;
use libmqm_sys::{self as mq, MQMD, Mqi};

use crate::{
    Connection, ConnectionRef, Library, MqFunctions, Object,
    connection::AsConnection,
    constants,
    get::{GetOption, GetParam},
    prelude::*,
    result::{Completion, ResultComp},
    structs, types,
};

#[derive(Debug)]
struct CallbackData<L: Library<MQ: Mqi>, F> {
    options: types::MQCBDO,
    closure: F,
    mq: MqFunctions<L>,
}

/// Manage the message handler callback of an object
#[must_use]
#[derive(Debug)]
pub struct MessageCallback<'a, C: AsConnection> {
    object: ManuallyDrop<Object<C>>,
    _cb: PhantomData<&'a ()>,
}

impl<C: AsConnection> Object<C> {
    fn register_message_handler<'cb, F>(
        &self,
        options: types::MQCBDO,
        get_options: impl GetOption,
        closure: F,
    ) -> ResultComp<MessageCallback<'cb, C>>
    where
        C: Clone,
        F: FnMut(ConnectionRef<C::Lib, C::Thread>, &structs::MQCBC, &MQMD, &[u8]) + Send + 'cb,
        C::Lib: Clone,
    {
        let connection = self.connection.as_connection();
        let cb_data: *mut CallbackData<C::Lib, F> = Box::into_raw(Box::from(CallbackData {
            options,
            closure,
            mq: connection.mq.clone(),
        }));
        let mut cbd = structs::MQCBD::new(default::MQCBD_DEFAULT);
        *cbd.CallbackType.as_mut() = constants::MQCBT_MESSAGE_CONSUMER;

        let mut param = GetParam {
            md: structs::MQMD::new(default::MQMD_DEFAULT),
            gmo: structs::MQGMO::new(default::MQGMO_DEFAULT),
        };

        get_options.apply_param(&mut param);
        assert!(param.gmo.Version <= mq::MQGMO_CURRENT_VERSION);
        assert!(param.md.Version <= mq::MQMD_CURRENT_VERSION);

        cbd.CallbackArea = cb_data.cast();
        *cbd.Options.as_mut() = options | constants::MQCBDO_DEREGISTER_CALL; // Always register for the deregister call
        cbd.CallbackFunction = message_callback::<C::Lib, C::Thread, F> as *mut _;

        let mqcb_result = unsafe {
            connection.mq.mqcb(
                connection.handle,
                constants::MQOP_DEREGISTER,
                Some(&cbd),
                Some(&self.handle),
                Some(&*param.md),
                Some(&*param.gmo),
            )
        };

        mqcb_result.map_completion(|()| MessageCallback {
            object: ManuallyDrop::new(Self {
                handle: self.handle,
                connection: self.connection.clone(),
                drop_close_options: self.drop_close_options,
            }),
            _cb: PhantomData,
        })
    }
}

unsafe extern "C" fn message_callback<L, H, F>(
    hconn: mq::MQHCONN,
    mqmd: mq::PMQVOID,
    _mqgmo: mq::PMQVOID,  // Not used for MQCBT_EVENT_HANDLER
    _buffer: mq::PMQVOID, // Not used for MQCBT_EVENT_HANDLER
    cbc: *const mq::MQCBC,
) where
     L: Library<MQ: Mqi> + Clone,
{
    // SAFETY: MQCBC will always be non-null
    if let Some(context) = unsafe { cbc.cast::<structs::MQCBC>().as_ref() } {
        // SAFETY: CallbackArea is always set by `register_message_handler`
        if let Some(CallbackData { options, closure, mq }) = unsafe { context.CallbackArea.cast::<CallbackData<L, F>>().as_mut() }
        {
            let is_deregister = types::MQCBCT(context.CallType) == constants::MQCBCT_DEREGISTER_CALL;

            if is_deregister && options.contains(constants::MQCBDO_DEREGISTER_CALL) {
                closure()
            }

            if !is_deregister || options.contains(constants::MQCBDO_DEREGISTER_CALL) {


                // SAFETY: mqmd and buffer are provided by MQ for message callbacks
                if let (Some(md), Some(buf_ptr)) = (unsafe { mqmd.cast::<MQMD>().as_ref() }, unsafe { buffer.as_ref() }) {
                    // For message callbacks, we need to determine the buffer length
                    // This would typically come from the message descriptor or be passed separately
                    // For now, we'll use an empty slice as a placeholder
                    let buffer_slice = unsafe { std::slice::from_raw_parts(buf_ptr.cast::<u8>(), context.DataLength as usize) };
                    closure(ConnectionRef::from_parts(hconn.into(), mq.clone()), context, md, buffer_slice);
                }
            }
            if is_deregister {
                // Recreate the box so it deallocates / drops
                // SAFETY: The only place the Callback handler is reconstructed
                let _ = unsafe { Box::<CallbackData<L, F>>::from_raw(context.CallbackArea.cast()) };
            }
        }
    }
}

// impl<'cb, O> MessageCallback<'cb, O> {
//     /// Wrap an object to manage the message handler callback
//     pub const fn new(object: O) -> Self {
//         Self {
//             object: ManuallyDrop::new(object),
//             _cb: PhantomData,
//         }
//     }

//     /// Register a message handler for the object
//     pub fn register_message_handler<F, L, H>(
//         &self,
//         options: types::MQCBDO,
//         closure: F,
//     ) -> ResultComp<()>
//     where
//         O: std::ops::Deref<Target = Object<L, H>>,
//         F: FnMut(ConnectionRef<L, H>, &structs::MQCBC, &MQMD, &[u8]) + Send + 'cb,
//         L: Library<MQ: Mqi> + Clone,
//     {
//         let Object { connection, handle, .. } = &**self.object;
//         let Connection { mq, handle: conn_handle, .. } = connection.as_connection();

//         let cb_data: *mut CallbackData<L, F> = Box::into_raw(Box::from(CallbackData {
//             options,
//             closure,
//             mq: mq.clone(),
//         }));
//         let mut cbd = structs::MQCBD::new(default::MQCBD_DEFAULT);
//         *cbd.CallbackType.as_mut() = constants::MQCBT_MESSAGE_CONSUMER;
//         let _ = unsafe { mq.mqcb(*conn_handle, constants::MQOP_DEREGISTER, Some(&cbd), Some(*handle), None::<&MQMD>, None) };

//         cbd.CallbackArea = cb_data.cast();
//         *cbd.Options.as_mut() = options | constants::MQCBDO_DEREGISTER_CALL; // Always register for the deregister call
//         cbd.CallbackFunction = message_callback::<L, H, F> as *mut _;

//         // SAFETY: MQCBD registered with valid pointers
//         unsafe { mq.mqcb(*conn_handle, constants::MQOP_REGISTER, Some(&cbd), Some(*handle), None::<&MQMD>, None) }
//     }

//     /// Unregister the message handler and return the original object
//     pub fn unregister_message_handler<L, H>(self) -> ResultComp<O>
//     where
//         O: std::ops::Deref<Target = Object<L, H>>,
//         L: Library<MQ: Mqi>,
//     {
//         let mut self_mut = self;
//         let Object { connection, handle, .. } = &**self_mut.object;
//         let Connection { mq, handle: conn_handle, .. } = connection.as_connection();

//         let mut cbd = structs::MQCBD::new(default::MQCBD_DEFAULT);
//         *cbd.CallbackType.as_mut() = constants::MQCBT_MESSAGE_CONSUMER;
//         let result = unsafe { mq.mqcb(*conn_handle, constants::MQOP_DEREGISTER, Some(&cbd), Some(*handle), None::<&MQMD>, None) };
//         let wrapped = unsafe { ManuallyDrop::take(&mut self_mut.object) };
//         let _ = ManuallyDrop::new(self_mut); // Suppress drop of self
//         result.map_completion(|()| wrapped)
//     }
// }

// impl<O> Drop for MessageCallback<'_, O> {
//     fn drop(&mut self) {
//         // We need to constrain O to have access to the Object fields for cleanup
//         // This is a best-effort cleanup in the drop implementation
//         // The actual cleanup logic would need to be implemented based on the Object structure
//     }
// }

// impl<O, L, H> std::ops::Deref for MessageCallback<'_, O>
// where
//     O: std::ops::Deref<Target = Object<L, H>>,
//     L: Library<MQ: Mqi>,
// {
//     type Target = Object<L, H>;

//     fn deref(&self) -> &Self::Target {
//         &self.object
//     }
// }

// #[derive(Debug)]
// struct CallbackData<L: Library<MQ: Mqi>, F> {
//     options: types::MQCBDO,
//     closure: F,
//     mq: MqFunctions<L>,
// }

// unsafe extern "C" fn message_callback<L, H, F>(
//     hconn: mq::MQHCONN,
//     mqmd: mq::PMQVOID,
//     _mqgmo: mq::PMQVOID,  // Could be used for get message options if needed
//     buffer: mq::PMQVOID,
//     cbc: *const mq::MQCBC,
// ) where
//     L: Library<MQ: Mqi> + Clone,
//     F: FnMut(ConnectionRef<L, H>, &structs::MQCBC, &MQMD, &[u8]) + Send,
// {
//     // SAFETY: MQCBC will always be non-null
//     if let Some(context) = unsafe { cbc.cast::<structs::MQCBC>().as_ref() } {
//         // SAFETY: CallbackArea is always set by `register_message_handler`
//         if let Some(CallbackData { options, closure, mq }) = unsafe { context.CallbackArea.cast::<CallbackData<L, F>>().as_mut() }
//         {
//             let is_deregister = types::MQCBCT(context.CallType) == constants::MQCBCT_DEREGISTER_CALL;
//             if !is_deregister || options.contains(constants::MQCBDO_DEREGISTER_CALL) {
//                 // SAFETY: mqmd and buffer are provided by MQ for message callbacks
//                 if let (Some(md), Some(buf_ptr)) = (unsafe { mqmd.cast::<MQMD>().as_ref() }, unsafe { buffer.as_ref() }) {
//                     // For message callbacks, we need to determine the buffer length
//                     // This would typically come from the message descriptor or be passed separately
//                     // For now, we'll use an empty slice as a placeholder
//                     let buffer_slice = unsafe { std::slice::from_raw_parts(buf_ptr.cast::<u8>(), context.DataLength as usize) };
//                     closure(ConnectionRef::from_parts(hconn.into(), mq.clone()), context, md, buffer_slice);
//                 }
//             }
//             if is_deregister {
//                 // Recreate the box so it deallocates / drops
//                 // SAFETY: The only place the Callback handler is reconstructed
//                 let _ = unsafe { Box::<CallbackData<L, F>>::from_raw(context.CallbackArea.cast()) };
//             }
//         }
//     }
// }
