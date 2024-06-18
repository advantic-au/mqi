use std::collections::HashMap;
use std::error::Error;
use std::thread;

use mqi::{inq, mqstr, sys, QueueManager, ConnectionOptions, Credentials, InqReqType, MqStr, MqStruct, Object, ObjectName, StructBuilder};
use mqi::prelude::*;

#[test]
fn object() {
    const QUEUE: ObjectName = mqstr!("DEV.QUEUE.1");
    let cb = ConnectionOptions::default_binding().credentials(Credentials::user("app", "app"));
    let (qm, ..) = QueueManager::new(None, &cb).warn_as_error().expect("Could not establish connection");

    thread::spawn(move || {
        let mut od = sys::MQOD::default();
        let mut md = sys::MQMD::default();
        let mut pmo = sys::MQPMO::default();

        QUEUE.copy_into_mqchar(&mut od.ObjectName);
        od.ObjectType = sys::MQOT_Q;

        qm.put(&mut od, Some(&mut md), &mut pmo, b"Hello ").warn_as_error().expect("Put failed");
    })
    .join()
    .expect("Panic from connection thread");
}

#[test]
fn inq_qm() -> Result<(), Box<dyn std::error::Error>> {
    const INQ: &[InqReqType] = &[
        //inq::MQCA_DEF_XMIT_Q_NAME,
        inq::MQCA_ALTERATION_DATE,
        // MQIA_CODED_CHAR_SET_ID,
        //inq::MQCA_Q_MGR_NAME,
        inq::MQCA_ALTERATION_TIME,
    ];
    let (conn, ..) = QueueManager::new(
        None,
        &ConnectionOptions::default_binding().credentials(Credentials::user("app", "app")),
    )
    .warn_as_error()?;
    let mut od = MqStruct::<sys::MQOD>::default();
    od.Version = sys::MQOD_VERSION_4;
    od.ObjectName = mqstr!("DEV.QUEUE.1").into();
    od.ObjectType = sys::MQOT_Q;
    let object = Object::open(&conn, &od, MqMask::from(sys::MQOO_INQUIRE)).warn_as_error()?;

    let result = object.inq(INQ)?;
    if let Some(rc) = result.warning() {
        eprintln!("MQRC warning: {rc}");
    }
    let a: HashMap<_, _> = result.collect();
    println!("{a:?}");

    Ok(())
}

#[test]
fn transaction() -> Result<(), Box<dyn Error>> {
    const QUEUE: ObjectName = mqstr!("DEV.QUEUE.1");
    let cb = ConnectionOptions::default_binding().credentials(Credentials::user("app", "app"));

    let cb = cb.build();
    let mut od = MqStruct::<sys::MQOD>::default();
    let mut md = MqStruct::<sys::MQMD2>::default();
    let mut pmo = MqStruct::<sys::MQPMO>::default();

    QUEUE.copy_into_mqchar(&mut od.ObjectName);
    let (connection, ..) = QueueManager::new(None, &cb).warn_as_error()?;
    let object = Object::open(&connection, &od, MqMask::from(sys::MQOO_OUTPUT)).warn_as_error()?;

    object.put(&mut *md, &mut pmo, b"Hello ").warn_as_error()?;

    Ok(())
    //        sync.backout().expect("Backout failed");
}
