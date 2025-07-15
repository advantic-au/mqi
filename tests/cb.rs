use std::sync::atomic::{AtomicUsize, Ordering};

use mqi::{prelude::*, ConnectionCallback, connection::ThreadNone, constants, test};

#[test]
fn qm() -> Result<(), Box<dyn std::error::Error>> {
    #[allow(clippy::allow_attributes, unused_mut)]
    let mut connection;
    #[cfg(feature = "mock")]
    {
        connection = test::mock::connect_ok(|mock_library| {
            let mut seq = mockall::Sequence::new();
            test::mock::open_ok(mock_library, 0x0c0c, 1, &mut seq);
            test::mock::get_error(mock_library, constants::MQRC_NO_MSG_AVAILABLE, 1, &mut seq);
        });
    }
    #[cfg(not(feature = "mock"))]
    {
        let creds = test::credentials();
        let cred_options: mqi::connection::Credentials<_> = creds.as_ref().into();
        connection = mqi::connect_lib::<ThreadNone, _>(test::mq_library(), &cred_options).warn_as_error()?;
    }

    let first_counter = AtomicUsize::new(0);
    let second_counter = AtomicUsize::new(0);
    let r = ConnectionCallback::new(connection.connection_ref());

    r.event_handler(
        constants::MQCBDO_REGISTER_CALL | constants::MQCBDO_DEREGISTER_CALL,
        |_, _| { let _ = first_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed); },
    )
    .warn_as_error()?;

    assert_eq!(first_counter.load(Ordering::Relaxed), 1);

    r.event_handler(
        constants::MQCBDO_REGISTER_CALL | constants::MQCBDO_DEREGISTER_CALL,
        |_, _| { let _ = second_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed); },
    )
    .warn_as_error()?;

    assert_eq!(first_counter.load(Ordering::Relaxed), 2);
    assert_eq!(second_counter.load(Ordering::Relaxed), 1);


    drop(r);

    assert_eq!(second_counter.load(Ordering::Relaxed), 2);


    Ok(())
}

// #[test]
// #[ignore = "experimental, and mocks not set up correctly"]
// fn callback() -> Result<(), Box<dyn Error>> {
//     fn register_cb<F, M>(cbd: &mut structs::MQCBD, cb: F)
//     where
//         F: FnMut(ConnectionHandle, Option<&M>, Option<&structs::MQGMO>, Option<&[u8]>, &structs::MQCBC) + 'static,
//         M: MQMD,
//     {
//         let data = Box::into_raw(Box::new(cb));
//         cbd.CallbackArea = data.cast();
//         cbd.CallbackFunction = call_closure::<F, M> as *mut _;
//         *cbd.CallbackType.as_mut() = constants::MQCBT_MESSAGE_CONSUMER;
//     }

//     unsafe extern "C" fn call_closure<F, M>(
//         conn: mq::MQHCONN,
//         mqmd: mq::PMQVOID,
//         gmo: mq::PMQVOID,
//         buffer: mq::PMQVOID,
//         cbc: *const mq::MQCBC,
//     ) where
//         F: FnMut(ConnectionHandle, Option<&M>, Option<&structs::MQGMO>, Option<&[u8]>, &structs::MQCBC) + 'static,
//         M: MQMD,
//     {
//         unsafe {
//             if let Some(context) = cbc.cast::<structs::MQCBC>().as_ref() {
//                 let cb_ptr = context.CallbackArea.cast::<F>();
//                 let cb = &mut *cb_ptr;
//                 cb(
//                     conn.into(),
//                     mqmd.cast::<M>().as_ref(),
//                     gmo.cast::<structs::MQGMO>().as_ref(),
//                     buffer.as_ref().map(|buffer_ref| {
//                         slice::from_raw_parts(
//                             ptr::from_ref(buffer_ref).cast(),
//                             context
//                                 .DataLength
//                                 .try_into()
//                                 .expect("Callback data length should not exceed maximum positive MQLONG"),
//                         )
//                     }),
//                     context,
//                 );
//             }
//         }
//     }

//     let mut mock_library = mock::callback::connect_ok_event_cb();
//     let mut seq = mockall::Sequence::new();
//     mock::open_ok(&mut mock_library, 0x0c0c, 1, &mut seq);
//     mock_library.expect_MQCTL().returning(|_, _, _, cc, rc| {
//         mock::mqi_outcome_ok(cc, rc);
//     });

//     let qm = mqi::connect_lib::<ThreadBlock, _>(mock_library, &()).warn_as_error()?;

//     let qm = Arc::new(qm);
//     let object = Object::open(qm.clone(), &()).warn_as_error()?;

//     let _ = thread::spawn(move || {
//         println!("{:?}", object.handle());
//         let b = 2;

//         let mut cbd = structs::MQCBD::new(default::MQCBD_DEFAULT);
//         let mqmd = structs::MQMD::new(default::MQMD_DEFAULT);
//         let mut gmo = structs::MQGMO::new(default::MQGMO_DEFAULT);
//         register_cb(&mut cbd, move |_a, _b: Option<&mq::MQMD>, _c, _d, _e| {
//             println!("{b}");
//         });

//         gmo.WaitInterval = 1500;
//         unsafe {
//             qm.mq()
//                 .mqcb(
//                     qm.handle(),
//                     constants::MQOP_REGISTER,
//                     Some(&cbd),
//                     Some(object.handle()),
//                     Some(&*mqmd),
//                     Some(&gmo),
//                 )
//                 .expect("mqcb should not fail");
//         };

//         let ctlo = structs::MQCTLO::new(default::MQCTLO_DEFAULT);

//         unsafe {
//             qm.mq()
//                 .mqctl(qm.handle(), constants::MQOP_START_WAIT, &ctlo)
//                 .warn_as_error()
//                 .expect("mqctl should not fail");
//         };

//         // Disconnect.
//         // object.close().warn_as_error().expect("Bad state");
//         // if let Some(connection) = Arc::into_inner(qm) {
//         //     connection.disconnect().warn_as_error().expect("Bad state");
//         // }
//         // connection.disconnect().warn_as_error();
//     })
//     .join();
//     // let ctlo = MqStruct::<sys::MQCTLO>::default();
//     // connection
//     //     .mq()
//     //     .mqctl(connection.handle(), constants::MQOP_SUSPEND, &ctlo)
//     //     .warn_as_error()?;

//     // object.close().warn_as_error()?;

//     // // Disconnect.
//     // connection.disconnect().warn_as_error()?;

//     Ok(())
// }
