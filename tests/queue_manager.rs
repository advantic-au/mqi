use std::{error::Error, sync::Arc, thread};

use mqi::{
    connect_options::{ApplName, Binding, ClientDefinition, Credentials, Tls},
    mqstr, sys,
    types::{ChannelName, CipherSpec, ConnectionName, MessageId, QueueName, FORMAT_NONE},
    Properties, MqMask, MqValue, QueueManager, ResultCompErrExt, ResultCompExt,
};

#[test]
fn thread() {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));
    let (qm, tag, id) =
        QueueManager::connect::<(QueueManager<_>, mqi::ConnTag, mqi::ConnectionId)>(None, &Credentials::user("app", "app"))
            .discard_warning() // ignore warning
            .expect("Could not establish connection");
    println!("{:?}", id.0);
    println!("{:?}", tag.0);
    thread::spawn(move || {
        let c = Arc::new(qm);
        let msg = Properties::new(&*c, MqValue::default()).expect("message created");
        msg.set_property("wally", "test", MqValue::default())
            .warn_as_error()
            .expect("property set");

        let msgid = c
            .put_message::<MessageId>(QUEUE, (), &(b"Hello", FORMAT_NONE))
            .warn_as_error()
            .expect("Put failed");
        println!("{msgid:?}");
    })
    .join()
    .expect("Failed to join");
}

#[test]
fn default_binding() -> Result<(), Box<dyn Error>> {
    // Use the default binding which is controlled through the MQI usually using environment variables
    // eg `MQSERVER = '...'``

    // Connect to the default queue manager (None) with the provided options
    // Treat all MQCC_WARNING as an error
    let connection: QueueManager<_> = QueueManager::connect(
        None,
        &(
            Binding::Default,
            Credentials::user("app", "app"),
            ApplName(mqstr!("readme_example")),
        ),
    )
    .warn_as_error()?;

    // Disconnect.
    connection.disconnect().warn_as_error()?;

    Ok(())
}

#[test]
fn connect() -> Result<(), Box<dyn Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    let def = ClientDefinition::new_client(
        &ChannelName(mqstr!("DEV.APP.SVRCONN")),
        &ConnectionName(mqstr!("192.168.92.15(1414)")),
        None,
    );
    let tls = Tls::new(
        &mqstr!("path"),
        Some("password"),
        Some(&mqstr!("label")),
        &CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256")),
    );
    let creds = Credentials::user("app", "app");
    let conn: QueueManager<_> = QueueManager::connect(None, &(tls, def, creds)).warn_as_error()?;

    conn.put_message::<()>(QUEUE, MqMask::from(sys::MQPMO_SYNCPOINT), "Hello")
        .warn_as_error()?;

    Ok(())
}
