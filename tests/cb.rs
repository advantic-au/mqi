mod helpers;

use core::slice;
use std::{error::Error, ptr, sync::Arc, thread};

use helpers::{credentials_app, mq_library};
use mqi::{core::ConnectionHandle, prelude::*, sys, types::QueueName, values, MqStruct, Object, ThreadBlock, ThreadNone, MQMD};

#[test]
fn qm() -> Result<(), Box<dyn Error>> {
    let mut qm = mqi::connect_lib::<ThreadNone, _>(mq_library(), credentials_app()).warn_as_error()?;

    qm.register_event_handler(
        values::MQCBDO(
            sys::MQCBDO_NONE
                | sys::MQCBDO_MC_EVENT_CALL
                | sys::MQCBDO_EVENT_CALL
                | sys::MQCBDO_REGISTER_CALL
                | sys::MQCBDO_DEREGISTER_CALL, // | sys::MQCBDO_START_CALL
                                               // | sys::MQCBDO_STOP_CALL,
        ),
        move |connection, options| {
            println!("{connection:?}");
            println!("{}", values::MQCBCT(options.CallType));
            println!("{}", values::MQCS(options.State));
            println!("{}", values::MQCC(options.CompCode));
            println!("{}", values::MQRC(options.Reason));
            println!("{}", values::MQCBF(options.Flags));
            println!("{}", values::MQRD(options.ReconnectDelay));
        },
    )?;

    qm.disconnect().warn_as_error()?;
    //qm.register_event_handler(MQCBDO(sys::MQCBDO_REGISTER_CALL), &CallbackHandle::from(|_, _: &'_ MqStruct<sys::MQCBC>| ()));
    Ok(())
}

#[test]
fn callback() -> Result<(), Box<dyn Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    fn register_cb<F, M>(cbd: &mut MqStruct<sys::MQCBD>, cb: F)
    where
        F: FnMut(ConnectionHandle, Option<&M>, Option<&MqStruct<sys::MQGMO>>, Option<&[u8]>, &MqStruct<sys::MQCBC>) + 'static,
        M: MQMD,
    {
        let data = Box::into_raw(Box::new(cb));
        cbd.CallbackArea = data.cast();
        cbd.CallbackFunction = call_closure::<F, M> as *mut _;
        cbd.CallbackType = sys::MQCBT_MESSAGE_CONSUMER;
    }

    unsafe extern "C" fn call_closure<F, M>(
        conn: sys::MQHCONN,
        mqmd: sys::PMQVOID,
        gmo: sys::PMQVOID,
        buffer: sys::PMQVOID,
        cbc: *const sys::MQCBC,
    ) where
        F: FnMut(ConnectionHandle, Option<&M>, Option<&MqStruct<sys::MQGMO>>, Option<&[u8]>, &MqStruct<sys::MQCBC>) + 'static,
        M: MQMD,
    {
        unsafe {
            if let Some(context) = cbc.cast::<MqStruct<sys::MQCBC>>().as_ref() {
                let cb_ptr = context.CallbackArea.cast::<F>();
                let cb = &mut *cb_ptr;
                cb(
                    conn.into(),
                    mqmd.cast::<M>().as_ref(),
                    gmo.cast::<MqStruct<sys::MQGMO>>().as_ref(),
                    buffer.as_ref().map(|buffer_ref| {
                        slice::from_raw_parts(
                            ptr::from_ref(buffer_ref).cast(),
                            context
                                .DataLength
                                .try_into()
                                .expect("Callback data length should not exceed maximum positive MQLONG"),
                        )
                    }),
                    context,
                );
            }
        }
    }

    let qm = mqi::connect_lib::<ThreadBlock, _>(mq_library(), credentials_app()).warn_as_error()?;

    let qm = Arc::new(qm);
    let object = Object::open(qm.clone(), (QUEUE, values::MQOO(sys::MQOO_INPUT_AS_Q_DEF))).warn_as_error()?;

    let _ = thread::spawn(move || {
        println!("{:?}", object.handle());
        let b = 2;

        let mut cbd = MqStruct::<sys::MQCBD>::default();
        let mqmd = MqStruct::<sys::MQMD>::default();
        let mut gmo = MqStruct::<sys::MQGMO>::default();
        register_cb(&mut cbd, move |_a, _b: Option<&sys::MQMD2>, _c, _d, _e| {
            println!("{b}");
        });

        gmo.WaitInterval = 1500;
        qm.mq()
            .mqcb(
                qm.handle(),
                values::MQOP(sys::MQOP_REGISTER),
                &cbd,
                Some(object.handle()),
                Some(&*mqmd),
                Some(&gmo),
            )
            .expect("mqcb should not fail");

        let ctlo = MqStruct::<sys::MQCTLO>::default();

        qm.mq()
            .mqctl(qm.handle(), values::MQOP(sys::MQOP_START_WAIT), &ctlo)
            .warn_as_error()
            .expect("mqctl should not fail");

        // Disconnect.
        // object.close().warn_as_error().expect("Bad state");
        // if let Some(connection) = Arc::into_inner(qm) {
        //     connection.disconnect().warn_as_error().expect("Bad state");
        // }
        // connection.disconnect().warn_as_error();
    })
    .join();
    // let ctlo = MqStruct::<sys::MQCTLO>::default();
    // connection
    //     .mq()
    //     .mqctl(connection.handle(), MQOP(sys::MQOP_SUSPEND), &ctlo)
    //     .warn_as_error()?;

    // object.close().warn_as_error()?;

    // // Disconnect.
    // connection.disconnect().warn_as_error()?;

    Ok(())
}
