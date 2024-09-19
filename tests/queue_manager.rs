use std::{env, error::Error, sync::Arc, thread};

use mqi::{
    connect_options::{ApplName, Binding, ClientDefinition, Credentials, Tls},
    prelude::*,
    sys,
    types::{CertificateLabel, CipherSpec, KeyRepo, MessageId, QueueName, FORMAT_NONE},
    values, Connection, Properties, QueueManager, ShareBlock,
};

#[test]
fn thread() {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));
    let (qm, (tag, id)) =
        Connection::<_, ShareBlock>::connect_with::<(mqi::ConnTag, mqi::ConnectionId)>(Credentials::user("app", "app"))
            .discard_warning() // ignore warning
            .expect("Could not establish connection");
    println!("{:?}", id.0);
    println!("{:?}", tag.0);
    thread::spawn(move || {
        let c = Arc::new(qm);
        let msg = Properties::new(&*c, values::MQCMHO::default()).expect("message created");
        msg.set_property("wally", "test", values::MQSMPO::default())
            .warn_as_error()
            .expect("property set");

        let msgid = QueueManager(c.clone())
            .put_message_with::<MessageId>(QUEUE, (), &("Hello", FORMAT_NONE))
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
    let connection = Connection::<_, ShareBlock>::connect((
        Binding::Default,
        Credentials::user("app", "app"),
        ApplName(mqstr!("readme_example")),
    ))
    .warn_as_error()?;

    // Disconnect.
    connection.disconnect().warn_as_error()?;

    Ok(())
}

#[test]
fn connect() -> Result<(), Box<dyn Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    let def = ClientDefinition::from_mqserver(&env::var("MQSERVER")?)?;
    let tls = Tls::new(
        &KeyRepo(mqstr!("path")),
        Some("password"),
        Some(&CertificateLabel(mqstr!("label"))),
        &CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256")),
    );
    let creds = Credentials::user("app", "app");
    let conn = Connection::<_, ShareBlock>::connect((tls, def, creds)).warn_as_error()?;

    QueueManager(conn)
        .put_message(QUEUE, values::MQPMO(sys::MQPMO_SYNCPOINT), "Hello")
        .warn_as_error()?;

    Ok(())
}
