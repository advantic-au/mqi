#![cfg(feature = "mock")]

use std::{error::Error, sync::Arc, thread};

use mqi::{
    connect_options::{Binding, MqServer, Tls},
    prelude::*,
    test,
    types::{CertificateLabel, CipherSpec, KeyRepo, MessageId, QueueName, FORMAT_NONE},
    values, Properties, ThreadNone,
};

#[test]
fn thread() {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));
    let mut mock = test::mock::connect_ok();
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

#[test]
fn default_binding() -> Result<(), Box<dyn Error>> {
    let mock = test::mock::connect_ok();
    let qm = mqi::connect_lib::<ThreadNone, _>(mock, &Binding::Default).warn_as_error()?;

    // Disconnect.
    qm.disconnect().warn_as_error()?;

    Ok(())
}

#[test]
fn connect() -> Result<(), Box<dyn Error>> {
    let mock = test::mock::connect_ok();
    let def = MqServer::try_from("A/TCP/C")?;

    let tls = Tls::new(
        &KeyRepo(mqstr!("path")),
        Some(&CertificateLabel(mqstr!("label"))),
        &CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256")),
    );
    let _qm = mqi::connect_lib::<ThreadNone, _>(mock, &(tls, def)).warn_as_error()?;

    Ok(())
}
