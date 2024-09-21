use core::slice;
use std::{error::Error, ptr, sync::Arc, thread};

use mqi::{
    connect_options::{ApplName, Binding, Credentials},
    core::ConnectionHandle,
    prelude::*,
    sys,
    types::QueueName,
    values::{MQCBCT, MQCBDO, MQCBF, MQCC, MQCS, MQOO, MQOP, MQRC, MQRD},
    Conn as _, MqStruct, Object, ThreadBlock, ThreadNone, MQMD,
};

#[test]
fn qm() -> Result<(), Box<dyn Error>> {
    let mut qm = mqi::connect::<ThreadNone>(Credentials::user("app", "app")).warn_as_error()?;

    qm.register_event_handler(
        MQCBDO(sys::MQCBDO_REGISTER_CALL | sys::MQCBDO_DEREGISTER_CALL),
        move |connection, options| {
            println!("{connection:?}");
            println!("{}", MQCBCT(options.CallType));
            println!("{}", MQCS(options.State));
            println!("{}", MQCC(options.CompCode));
            println!("{}", MQRC(options.Reason));
            println!("{}", MQCBF(options.Flags));
            println!("{}", MQRD(options.ReconnectDelay));

        },
    )?;
    //qm.register_event_handler(MQCBDO(sys::MQCBDO_REGISTER_CALL), &CallbackHandle::from(|_, _: &'_ MqStruct<sys::MQCBC>| ()));
    Ok(())
}

#[test]
fn callback() -> Result<(), Box<dyn Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    fn register_cb<
        F: FnMut(ConnectionHandle, Option<&M>, Option<&MqStruct<sys::MQGMO>>, Option<&[u8]>, &MqStruct<sys::MQCBC>) + 'static,
        M: MQMD,
    >(
        cbd: &mut MqStruct<sys::MQCBD>,
        cb: F,
    ) {
        let data = Box::into_raw(Box::new(cb));
        cbd.CallbackArea = data.cast();
        cbd.CallbackFunction = call_closure::<F, M> as *mut _;
        cbd.CallbackType = sys::MQCBT_MESSAGE_CONSUMER;
    }

    unsafe extern "C" fn call_closure<
        F: FnMut(ConnectionHandle, Option<&M>, Option<&MqStruct<sys::MQGMO>>, Option<&[u8]>, &MqStruct<sys::MQCBC>) + 'static,
        M: MQMD,
    >(
        conn: sys::MQHCONN,
        mqmd: sys::PMQVOID,
        gmo: sys::PMQVOID,
        buffer: sys::PMQVOID,
        cbc: *const sys::MQCBC,
    ) {
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
                                .expect("Callback data length exceeds maximum positive MQLONG"),
                        )
                    }),
                    context,
                );
            }
        }
    }

    // Use the default binding which is controlled through the MQI usually using environment variables
    // eg `MQSERVER = '...'``
    // let connection_options = ConnectionOptions::default_binding()
    //     .application_name(Some(mqstr!("readme_example")))
    //     .credentials(Credentials::user("app", "app"));

    // Connect to the default queue manager (None) with the provided `connection_options`
    // Treat all MQCC_WARNING as an error
    let qm = mqi::connect::<ThreadBlock>((
        Binding::Default,
        ApplName(mqstr!("readme_example")),
        Credentials::user("app", "app"),
    ))
    .warn_as_error()?;

    let qm = Arc::new(qm);
    let object = Object::open(qm.clone(), (QUEUE, MQOO(sys::MQOO_INPUT_AS_Q_DEF))).warn_as_error()?;

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
                MQOP(sys::MQOP_REGISTER),
                &cbd,
                Some(object.handle()),
                Some(&*mqmd),
                Some(&gmo),
            )
            .expect("Bad state");

        let ctlo = MqStruct::<sys::MQCTLO>::default();

        qm.mq()
            .mqctl(qm.handle(), MQOP(sys::MQOP_START_WAIT), &ctlo)
            .warn_as_error()
            .expect("Bad state");

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
