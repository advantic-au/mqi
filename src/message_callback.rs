#![allow(clippy::significant_drop_tightening)]

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

        let _ = Self::deregister_message_consumer_internal(object.as_object());

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

    pub fn deregister_message_consumer(self) -> ResultComp<O> {
        let result = Self::deregister_message_consumer_internal(self.object.as_object());
        let object = unsafe { std::ptr::read(&raw const self.object) };
        let _ = ManuallyDrop::new(self); // Suppress drop
        result.map_completion(|()| object)
    }

    fn deregister_message_consumer_internal(obj: &Object<O::AsConnection>) -> ResultComp<()> {
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
        let _ = Self::deregister_message_consumer_internal(self.object.as_object());
    }
}

#[cfg(all(test, feature = "mock"))]
#[cfg_attr(coverage_nightly, coverage(off))]
#[expect(clippy::ref_option_ref)]
mod tests {
    use mockall::{Sequence, automock};

    use super::*;
    use crate::{MqStr, test::mock, types::QueueManagerName};

    #[automock]
    trait Callback {
        #[expect(clippy::elidable_lifetime_names)]
        fn callback<'a, 'b, 'c, 'd>(
            &self,
            object: &Object<mock::MockConnection>,
            cbc: &structs::MQCBC<'a>,
            mqmd: Option<&'b MQMD>,
            gmo: Option<&'c structs::MQGMO>,
            buffer: Option<&'d [u8]>,
        );
    }

    #[test]
    fn register_message_consumer() -> Result<(), Box<dyn std::error::Error>> {
        let connection = mock::connect_ok(|mock| {
            let mut seq = mockall::Sequence::new();
            mock::open_ok(mock, 0x0c0c, 1, &mut seq);
            mock::callback::mock_cb(mock, constants::MQCBT_MESSAGE_CONSUMER);
        });

        let mut cb = MockCallback::new();
        let mut seq = Sequence::new();
        for call_type in [constants::MQCBCT_REGISTER_CALL, constants::MQCBCT_DEREGISTER_CALL] {
            cb.expect_callback()
                .withf(move |_obj, cbc, _md, _gmo, _buffer| types::MQCBCT(cbc.CallType) == call_type)
                .once()
                .return_const(())
                .in_sequence(&mut seq);
        }

        let object = Object::open(connection, &QueueManagerName(MqStr::empty())).warn_as_error()?;

        let callback = MessageCallback::register_message_consumer(
            &object,
            constants::MQCBDO_DEREGISTER_CALL | constants::MQCBDO_REGISTER_CALL,
            &(),
            |obj, cbc, md, gmo, buffer| cb.callback(obj, cbc, md, gmo, buffer),
        )
        .warn_as_error();

        let result = callback?.deregister_message_consumer();

        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn deregister_message_consumer() -> Result<(), Box<dyn std::error::Error>> {
        let connection = mock::connect_ok(|mock| {
            let mut seq = mockall::Sequence::new();
            mock::open_ok(mock, 0x0c0c, 1, &mut seq);
            mock::callback::mock_cb(mock, constants::MQCBT_MESSAGE_CONSUMER);
        });
        let object = Object::open(connection, &QueueManagerName(MqStr::empty())).warn_as_error()?;
        let callback =
            MessageCallback::register_message_consumer(&object, constants::MQCBDO_NONE, &(), |_obj, _cbc, _md, _gmo, _buffer| {
                panic!("Should not be called");
            })
            .warn_as_error()?;

        let result = callback.deregister_message_consumer();
        assert!(result.is_ok());
        Ok(())
    }
}
