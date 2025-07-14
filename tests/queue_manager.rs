#![cfg(feature = "mock")]

use std::{sync::Arc, thread};

use libmqm_sys::mock::MockMq;
use mqi::{
    Properties, connection, constants,
    prelude::*,
    test::mock,
    types::{FORMAT_NONE, MQCMHO, MQSMPO, MessageId, QueueName},
};

#[test]
fn thread() {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    let mut mock = MockMq::new();
    mock::connx_outcome(&mut mock, 0x0d0d, constants::MQCC_OK, constants::MQRC_NONE);
    mock::disc_outcome(&mut mock, constants::MQCC_OK, constants::MQRC_NONE);

    let mut seq = mockall::Sequence::new();
    mock::properties_ok(&mut mock, 0xf0f0, 1, &mut seq);
    mock.expect_MQSETMP().returning(|_, _, _, _, _, _, _, _, cc, rc| {
        // TODO: assert values set
        mock::mqi_outcome_ok(cc, rc);
    });
    mock.expect_MQPUT1().returning(|_, _, _, _, _, _, cc, rc| {
        // TODO: assert values set
        mock::mqi_outcome_ok(cc, rc);
    });

    let (qm, (tag, id)) =
        mqi::connect_lib_with::<(connection::ConnTag, connection::ConnectionId), connection::ThreadBlock, _>(mock, &())
            .discard_warning() // ignore warning
            .expect("connection should be established");
    let qm = Arc::new(qm);
    println!("Connection ID: {id}");
    println!("{:?}", tag.0);
    thread::spawn(move || {
        let msg = Properties::new(qm.clone(), MQCMHO::default()).expect("message created");
        msg.set_property("wally", "test", MQSMPO::default())
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
