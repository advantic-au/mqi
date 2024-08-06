use std::{error::Error, sync::Arc, thread};

use mqi::{
    connect_options::Credentials, mqstr, sys, types::QueueName, Message, MqMask, MqValue, QueueManager, ResultCompExt
};

#[test]
fn thread() {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));
    let conn = QueueManager::connect(None, &Credentials::user("app", "app").build_csp())
        .warn_as_error()
        .expect("Could not establish connection");
    thread::spawn(move || {
        let c = Arc::new(conn);
        let mut msg = Message::new(&*c, MqValue::default()).expect("message created");
        msg.set_property("wally", "test", MqValue::default())
            .warn_as_error()
            .expect("property set");

        c.put_message::<()>(QUEUE, &mut msg, b"Hello ".as_slice())
            .warn_as_error()
            .expect("Put failed");
    })
    .join()
    .expect("Failed to join");
}

#[test]
fn default_binding() -> Result<(), Box<dyn Error>> {
    // Use the default binding which is controlled through the MQI usually using environment variables
    // eg `MQSERVER = '...'``
    // let connection_options = ConnectionOptions::default_binding()
    //     .application_name(Some(mqstr!("readme_example")))
    //     .credentials(Credentials::user("app", "app"));

    // Connect to the default queue manager (None) with the provided `connection_options`
    // Treat all MQCC_WARNING as an error
    let connection: QueueManager<_> = QueueManager::connect(None, ()).warn_as_error()?;

    // Disconnect.
    connection.disconnect().warn_as_error()?;

    Ok(())
}

#[test]
fn connect() -> Result<(), Box<dyn Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));
    // let mut od = MqStruct::<sys::MQOD>::default();

    // QUEUE.copy_into_mqchar(&mut od.ObjectName);
    // od.ObjectType = sys::MQOT_Q;
    // let cb = ConnectionOptions::from_mqserver("DEV.APP.SVRCONN/TCP/192.168.92.15(1414)")?
    //     .tls(
    //         &mqstr!("TLS_AES_128_GCM_SHA256"),
    //         Tls::new(&mqstr!("path"), Some("password"), Some(&mqstr!("label"))),
    //     )
    //     .application_name(Some(mqstr!("rust_testing")))
    //     .credentials(Credentials::user("app", "app"));

    let conn: QueueManager<_> = QueueManager::connect(None, ()).warn_as_error()?;

    conn.put_message::<()>(QUEUE, MqMask::from(sys::MQPMO_SYNCPOINT), "Hello")
        .warn_as_error()?;

    Ok(())
}
