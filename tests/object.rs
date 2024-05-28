use std::collections::HashMap;
use std::thread;

use mqi::{mqstr, sys, Connection, ConnectionOptions, Credentials, InqReqType, MqStr, Object, ObjectName, StructBuilder};
use mqi::inq::{MQCA_DEF_XMIT_Q_NAME, MQIA_CODED_CHAR_SET_ID};
use mqi::prelude::*;

#[test]
fn object() {
    const QUEUE: ObjectName = mqstr!("DEV.QUEUE.1");
    let cb = ConnectionOptions::default_binding().credentials(Credentials::user("app", "app"));
    let conn = Connection::new(None, &cb).expect("Could not establish connection");

    thread::spawn(move || {
        let mut od = sys::MQOD::default();
        let mut md = sys::MQMD::default();
        let mut pmo = sys::MQPMO::default();

        QUEUE.copy_into_mqchar(&mut od.ObjectName);
        od.ObjectType = sys::MQOT_Q_MGR;

        conn.put(&od, &mut md, &mut pmo, b"Hello ").expect("Put failed");
    })
    .join()
    .expect("Panic from connection thread");
}

#[test]
fn inq_qm() {
    const INQ: &[InqReqType] = &[
        MQCA_DEF_XMIT_Q_NAME,
        // MQCA_ALTERATION_DATE,
        MQIA_CODED_CHAR_SET_ID,
        // MQCA_Q_MGR_NAME,
        // MQCA_ALTERATION_TIME,
    ];
    let conn = Connection::new(
        None,
        &ConnectionOptions::default_binding().credentials(Credentials::user("app", "app")),
    )
    .warn_as_error()
    .expect("Could not establish connection");
    let mut od = sys::MQOD::default();
    mqstr!("QM1").copy_into_mqchar(&mut od.ObjectName);
    od.ObjectType = sys::MQOT_Q_MGR;
    let object = Object::open(&conn, &od, sys::MQOO_INQUIRE).expect("Unable to OPEN object");

    let result = object.inq(INQ.iter()).expect("Unable to INQ object");
    if let Some(rc) = result.warning() {
        eprintln!("MQRC warning: {rc}");
    }
    let a: HashMap<_, _> = result.collect();
    println!("{a:?}");
}

#[test]
fn transaction() {
    const QUEUE: ObjectName = mqstr!("DEV.QUEUE.1");
    let cb = ConnectionOptions::default_binding().credentials(Credentials::user("app", "app"));

    let cb = cb.build();
    let mut od = sys::MQOD::default();
    let mut md = sys::MQMD::default();
    let mut pmo = sys::MQPMO::default();

    QUEUE.copy_into_mqchar(&mut od.ObjectName);
    let connection = Connection::new(None, &cb)
        .warn_as_error()
        .expect("Could not establish connection");
    let object = Object::open(&connection, &od, sys::MQOO_OUTPUT).expect("Could not open object");

    object.put(&mut md, &mut pmo, b"Hello ").expect("Put failed");
    //        sync.backout().expect("Backout failed");
}
