mod helpers;

use std::{env, error::Error, thread};

use helpers::{credentials_app, mq_library};
use mqi::{
    connect_options::{Binding, MqServer, Tls},
    prelude::*,
    sys,
    types::{CertificateLabel, CipherSpec, KeyRepo, MessageId, QueueName, FORMAT_NONE},
    values, Properties, ThreadNoBlock, ThreadNone,
};

#[test]
fn thread() {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));
    let (qm, (tag, id)) =
        mqi::connect_lib_with::<(mqi::ConnTag, mqi::ConnectionId), ThreadNoBlock, _>(mq_library(), credentials_app())
            .discard_warning() // ignore warning
            .expect("connection should be established");
    println!("{:?}", id.0);
    println!("{:?}", tag.0);
    thread::spawn(move || {
        let msg = Properties::new(qm.connection_ref(), values::MQCMHO::default()).expect("message created");
        msg.set_property("wally", "test", values::MQSMPO::default())
            .warn_as_error()
            .expect("property set should not fail");

        let msgid = qm
            .connection_ref()
            .put_message_with::<MessageId>(QUEUE, (), &("Hello", FORMAT_NONE))
            .warn_as_error()
            .expect("message put should not fail");
        println!("{msgid:?}");
    })
    .join()
    .expect("thread join should not fail");
}

#[test]
fn default_binding() -> Result<(), Box<dyn Error>> {
    let qm = mqi::connect_lib::<ThreadNone, _>(mq_library(), (Binding::Default, credentials_app())).warn_as_error()?;

    // Disconnect.
    qm.disconnect().warn_as_error()?;

    Ok(())
}

#[test]
fn connect() -> Result<(), Box<dyn Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    let env = env::var("MQSERVER")?;
    let def = MqServer::try_from(&*env)?;

    let tls = Tls::new(
        &KeyRepo(mqstr!("path")),
        Some("password"),
        Some(&CertificateLabel(mqstr!("label"))),
        &CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256")),
    );
    let qm = mqi::connect_lib::<ThreadNone, _>(mq_library(), (tls, def, credentials_app())).warn_as_error()?;

    qm.put_message(QUEUE, values::MQPMO(sys::MQPMO_SYNCPOINT), "Hello")
        .warn_as_error()?;

    Ok(())
}
