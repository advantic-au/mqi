use std::{marker::PhantomData, mem::ManuallyDrop};

use libmqm_default as default;
use libmqm_sys::{self as mq, MQMD};

use crate::{
    Object,
    connection::AsConnection,
    constants,
    get::{GetOption, GetParam},
    object::AsObject,
    prelude::*,
    result::ResultComp,
    structs::{self, MQGMO},
    types,
};

#[derive(Debug)]
struct CallbackData<O, F> {
    options: types::MQCBDO,
    object: O,
    closure: F,
}

/// Manage the message handler callback of an object
#[must_use]
#[derive(Debug)]
pub struct MessageCallback<'a, O: AsObject> {
    object: O,
    _cb: PhantomData<&'a ()>,
}

impl<'cb, O: AsObject> MessageCallback<'cb, O> {
    pub fn register_message_consumer<F>(
        object: O,
        options: types::MQCBDO,
        get_options: &impl GetOption,
        closure: F,
    ) -> ResultComp<Self>
    where
        O: Clone + Send,
        F: FnMut(&Object<O::AsConnection>, &structs::MQCBC, Option<&MQMD>, Option<&MQGMO>, Option<&[u8]>) + Send + 'cb,
    {
        let cb_data: *mut CallbackData<O, F> = Box::into_raw(Box::from(CallbackData {
            options,
            object: object.clone(),
            closure,
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
        cbd.CallbackFunction = message_callback::<O, F> as *mut _;

        let obj = object.as_object();
        let conn = obj.connection.as_connection();
        let mqcb_result = unsafe {
            conn.mq.mqcb(
                conn.handle,
                constants::MQOP_REGISTER,
                Some(&cbd),
                Some(&obj.handle),
                Some(&*param.md),
                Some(&*param.gmo),
            )
        };

        mqcb_result.map_completion(|()| Self {
            object,
            _cb: PhantomData,
        })
    }

    pub fn unregister_message_consumer(self) -> ResultComp<O> {
        let result: Result<crate::result::Completion<()>, crate::result::Error> = self.unregister_message_consumer_internal();
        let object = unsafe { std::ptr::read(&raw const self.object) };
        let _ = ManuallyDrop::new(self); // Suppress drop
        result.map_completion(|()| object)
    }

    fn unregister_message_consumer_internal(&self) -> ResultComp<()> {
        let obj = self.object.as_object();
        let conn = obj.connection.as_connection();
        let mut cbd = structs::MQCBD::new(default::MQCBD_DEFAULT);
        *cbd.CallbackType.as_mut() = constants::MQCBT_MESSAGE_CONSUMER;
        unsafe {
            conn.mq.mqcb(
                conn.handle,
                constants::MQOP_DEREGISTER,
                Some(&cbd),
                Some(&obj.handle),
                None::<&mq::MQMD>,
                None,
            )
        }
    }
}

unsafe extern "C" fn message_callback<O, F>(
    _hconn: mq::MQHCONN,
    mqmd: mq::PMQVOID,
    mqgmo: mq::PMQVOID,
    buffer: mq::PMQVOID,
    cbc: *const mq::MQCBC,
) where
    O: AsObject,
    F: FnMut(&Object<O::AsConnection>, &structs::MQCBC, Option<&MQMD>, Option<&MQGMO>, Option<&[u8]>),
{
    // SAFETY: MQCBC will always be non-null
    if let Some(context) = unsafe { cbc.cast::<structs::MQCBC>().as_ref() } {
        // SAFETY: CallbackArea is always set by `register_message_handler`
        if let Some(CallbackData {
            options,
            object,
            closure,
        }) = unsafe { context.CallbackArea.cast::<CallbackData<O, F>>().as_mut() }
        {
            let is_deregister = types::MQCBCT(context.CallType) == constants::MQCBCT_DEREGISTER_CALL;

            if !is_deregister || options.contains(constants::MQCBDO_DEREGISTER_CALL) {
                // SAFETY: mqmd and buffer are provided by MQ for message callbacks
                let md = unsafe { mqmd.cast::<MQMD>().as_ref() };
                let gmo = unsafe { mqgmo.cast::<MQGMO>().as_ref() };
                let buffer_slice = unsafe { buffer.cast::<u8>().as_ref() }.map(|buf_ptr| {
                    #[expect(clippy::cast_sign_loss, reason = "buffer length is always within bounds of usize")]
                    unsafe {
                        std::slice::from_raw_parts(buf_ptr, context.DataLength as usize)
                    }
                });
                closure(object.as_object(), context, md, gmo, buffer_slice);
            }
            if is_deregister {
                // Recreate the box so it deallocates / drops
                // SAFETY: The only place the Callback handler is reconstructed
                let _ = unsafe { Box::<CallbackData<O, F>>::from_raw(context.CallbackArea.cast()) };
            }
        }
    }
}

impl<O: AsObject> Drop for MessageCallback<'_, O> {
    fn drop(&mut self) {
        let _ = self.unregister_message_consumer_internal();
    }
}

#[cfg(all(test, feature = "mock"))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        collections::HashMap,
        sync::{
            Arc, Mutex,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use super::*;
    use crate::{
        MqStr,
        test::mock,
        types::QueueManagerName,
    };

    #[test]
    fn register_message_consumer() -> Result<(), Box<dyn std::error::Error>> {
        let connection = mock::connect_ok(|mock| {
            let mut seq = mockall::Sequence::new();
            mock::open_ok(mock, 0x0c0c, 1, &mut seq);
            mock::callback::mock_cb(mock, constants::MQCBT_MESSAGE_CONSUMER);
        });
        let object = Object::open(connection, &QueueManagerName(MqStr::empty())).warn_as_error()?;

        let counters = Arc::new(Mutex::new(HashMap::new()));
        let counters_cb = counters.clone();

        let callback = MessageCallback::register_message_consumer(
            &object,
            constants::MQCBDO_DEREGISTER_CALL,
            &(),
            move |_obj, cbc, _md, _gmo, _buffer| {
                let mut guard = counters_cb.lock().expect("Arc lock should be available");
                let counter: &mut AtomicUsize = guard.entry(types::MQCBCT(cbc.CallType)).or_default();
                counter.fetch_add(1, Ordering::Relaxed);
            },
        );

        assert!(
            counters
                .lock()
                .expect("Arc lock should be available")
                .get(&constants::MQCBCT_REGISTER_CALL)
                .is_some_and(|c| c.load(Ordering::Relaxed) == 1)
        );

        assert!(callback.is_ok());
        Ok(())
    }

    #[test]
    fn unregister_message_consumer() -> Result<(), Box<dyn std::error::Error>> {
        let connection = mock::connect_ok(|mock| {
            let mut seq = mockall::Sequence::new();
            mock::open_ok(mock, 0x0c0c, 1, &mut seq);
            mock::callback::mock_cb(mock, constants::MQCBT_MESSAGE_CONSUMER);
        });
        let object = Object::open(connection, &QueueManagerName(MqStr::empty())).warn_as_error()?;
        let callback =
            MessageCallback::register_message_consumer(&object, constants::MQCBDO_NONE, &(), |_obj, _cbc, _md, _gmo, _buffer| {})
                .warn_as_error()?;

        let result = callback.unregister_message_consumer();
        assert!(result.is_ok());
        Ok(())
    }
}
