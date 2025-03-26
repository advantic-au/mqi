#![cfg(feature = "mock")]

use std::{sync::Arc, thread};

use mqi::{
    prelude::*,
    sys, test,
    types::{MessageId, QueueName, FORMAT_NONE},
    values, Properties,
};

#[test]
fn thread() {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    let mut mock = test::mock::MockFunctions::new();
    mock.connx_outcome(0x0d0d, sys::MQCC_OK, sys::MQRC_NONE);
    mock.disc_outcome(sys::MQCC_OK, sys::MQRC_NONE);

    let mut seq = mockall::Sequence::new();
    mock.properties_ok(0xf0f0, 1, &mut seq);
    mock.expect_MQSETMP().returning(|_, _, _, _, _, _, _, _, cc, rc| {
        // TODO: assert values set
        test::mock::MockFunctions::mqi_outcome_ok(cc, rc);
    });
    mock.expect_MQPUT1().returning(|_, _, _, _, _, _, cc, rc| {
        // TODO: assert values set
        test::mock::MockFunctions::mqi_outcome_ok(cc, rc);
    });

    let (qm, (tag, id)) = mqi::connect_lib_with::<(mqi::ConnTag, mqi::ConnectionId), mqi::ThreadBlock, _>(mock, &())
        .discard_warning() // ignore warning
        .expect("connection should be established");
    let qm = Arc::new(qm);
    println!("Connection ID: {id}");
    println!("{:?}", tag.0);
    thread::spawn(move || {
        let msg = Properties::new(qm.clone(), values::MQCMHO::default()).expect("message created");
        msg.set_property("wally", "test", values::MQSMPO::default())
            .warn_as_error()
            .expect("property set should not fail");

        let msgid: MessageId = qm
            .put_message_with(&QUEUE, &(), &("Hello", FORMAT_NONE))
            .warn_as_error()
            .expect("message put should not fail");
        println!("Message ID: {msgid}");
    })
    .join()
    .expect("thread join should not fail");
}
