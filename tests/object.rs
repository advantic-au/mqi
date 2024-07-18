use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::thread;

use mqi::property::Attributes;
use mqi::{prelude::*, property, Message};
use mqi::{
    inq, mqstr, sys, ConnectionOptions, Credentials, InqReqItem, InqReqType, MqStr, MqStruct, Object, ObjectName, QueueManager,
    StructBuilder,
};

#[test]
fn object() {
    const QUEUE: ObjectName = mqstr!("DEV.QUEUE.1");
    let cb = ConnectionOptions::default_binding().credentials(Credentials::user("app", "app"));
    let (qm, ..) = QueueManager::new(None, &cb)
        .warn_as_error()
        .expect("Could not establish connection");

    thread::spawn(move || {
        let mut od = sys::MQOD::default();
        let mut md = sys::MQMD::default();
        let mut pmo = sys::MQPMO::default();

        QUEUE.copy_into_mqchar(&mut od.ObjectName);
        od.ObjectType = sys::MQOT_Q;

        qm.put(&mut od, Some(&mut md), &mut pmo, b"Hello ")
            .warn_as_error()
            .expect("Put failed");
    })
    .join()
    .expect("Panic from connection thread");
}

#[test]
fn get_message() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: ObjectName = mqstr!("DEV.QUEUE.1");
    let buffer = vec![0; 2 * 1024 * 1024]; //2M
    let cb = ConnectionOptions::default_binding().credentials(Credentials::user("app", "app"));
    let (qm, ..) = QueueManager::new(None, &cb).warn_as_error()?;

    let mut od = MqStruct::<sys::MQOD>::default();
    od.ObjectName = QUEUE.into();
    od.ObjectType = sys::MQOT_Q;
    let object = Object::open(&qm, &od, MqMask::from(sys::MQOO_INPUT_AS_Q_DEF))?;
    let mut properties = Message::new(&qm, MqValue::default())?;
    let result: Option<mqi::GetMqmd<Vec<u8>>> = object
        .get_message(
            // Get a vector with an MQMD
            MqMask::default(),     // Just the default GET options
            mqi::ANY_MESSAGE,      // No selection criteria
            Some(2000),            // Wait 2 seconds
            Some(&mut properties), // Populate the properties
            buffer,                // Use a vec as buffer
        )
        .warn_as_error()?;

    for v in properties.property_iter(property::INQUIRE_ALL, MqMask::default()) {
        let (name, value): (String, Attributes<String>) = v.warn_as_error()?;
        println!("{name:?}, {value:?}");
    }

    println!("{result:?}");

    Ok(())
}

#[test]
fn inq_qm() -> Result<(), Box<dyn std::error::Error>> {
    const INQ: &[InqReqType] = &[
        inq::MQCA_Q_MGR_NAME,
        inq::MQCA_ALTERATION_DATE,
        inq::MQCA_DEAD_LETTER_Q_NAME,
        inq::MQCA_ALTERATION_TIME,
        inq::MQCA_CREATION_DATE,
        inq::MQCA_CREATION_TIME,
        inq::MQIA_CODED_CHAR_SET_ID,
        inq::MQCA_DEF_XMIT_Q_NAME,
        (
            // Hmmm... this works. Not documented for MQINQ though.
            MqValue::from(sys::MQCA_VERSION),
            InqReqItem::Str(sys::MQ_VERSION_LENGTH),
        ),
        inq::MQIA_COMMAND_LEVEL,
    ];
    let Completion((qm, ..), ..) = QueueManager::new(
        None,
        &ConnectionOptions::default_binding().credentials(Credentials::user("app", "app")),
    )?;
    let mut od = MqStruct::<sys::MQOD>::default();
    od.ObjectQMgrName = mqstr!("QM1").into();
    od.ObjectType = sys::MQOT_Q_MGR;
    let object = Object::open(&qm, &od, MqMask::from(sys::MQOO_INQUIRE))?;

    let result = object.inq(INQ)?;
    if let Some((rc, verb)) = result.warning() {
        eprintln!("MQRC warning: {verb} {rc}");
    }

    let values: HashMap<_, _> = result
        .iter()
        .map(|(attr, value)| {
            (
                attr,
                match value {
                    mqi::InqResItem::Str(value) => Cow::from(value),
                    mqi::InqResItem::Long(value) => Cow::from(value.to_string()),
                },
            )
        })
        .collect();

    for (attr, value) in values {
        println!("{attr}: {value}");
    }

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
