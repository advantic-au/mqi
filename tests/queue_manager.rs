use std::{error::Error, thread};

use mqi::{mqstr, sys, ConnectionOptions, Credentials, MqStr, ObjectName, QueueManager, ResultCompExt, Tls};

#[test]
fn thread() {
    const QUEUE: ObjectName = mqstr!("DEV.QUEUE.1");
    let cb = ConnectionOptions::default_binding().credentials(Credentials::user("app", "app"));
    let (conn, ..) = QueueManager::new(None, &cb)
        .warn_as_error()
        .expect("Could not establish connection");
    thread::spawn(move || {
        let mut od = sys::MQOD::default();
        let mut md = sys::MQMD::default();
        let mut pmo = sys::MQPMO::default();

        QUEUE.copy_into_mqchar(&mut od.ObjectName);
        conn.put(&mut od, Some(&mut md), &mut pmo, b"Hello ")
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
    let connection_options = ConnectionOptions::default_binding()
        .application_name(Some(mqstr!("readme_example")))
        .credentials(Credentials::user("app", "app"));

    // Connect to the default queue manager (None) with the provided `connection_options`
    // Treat all MQCC_WARNING as an error
    let (connection, ..) = QueueManager::new(None, &connection_options).warn_as_error()?;

    // Disconnect.
    connection.disconnect().warn_as_error()?;

    Ok(())
}

#[test]
fn connect() -> Result<(), Box<dyn Error>> {
    const QUEUE: ObjectName = mqstr!("DEV.QUEUE.1");
    let mut od = sys::MQOD::default();
    let mut md = sys::MQMD2::default();
    let mut pmo = sys::MQPMO::default();

    QUEUE.copy_into_mqchar(&mut od.ObjectName);
    od.ObjectType = sys::MQOT_Q;
    let cb = ConnectionOptions::from_mqserver("DEV.APP.SVRCONN/TCP/192.168.92.15(1414)")?
        .tls(
            &mqstr!("TLS_AES_128_GCM_SHA256"),
            Tls::new(&mqstr!("path"), Some("password"), Some(&mqstr!("label"))),
        )
        .application_name(Some(mqstr!("rust_testing")))
        .credentials(Credentials::user("app", "app"));

    let (conn, ..) = QueueManager::new(None, &cb).warn_as_error()?;

    pmo.Options |= sys::MQPMO_SYNCPOINT;
    conn.put(&mut od, Some(&mut md), &mut pmo, b"Hello").warn_as_error()?;

    Ok(())
}
