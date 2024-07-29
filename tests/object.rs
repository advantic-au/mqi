use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::thread;

use mqi::get::GetMessage;
use mqi::put::Properties;
use mqi::{get, prelude::*, Message};
use mqi::{inq, mqstr, sys, ConnectionOptions, Credentials, MqStruct, Object, ObjectName, QueueManager, StructBuilder};

#[test]
fn object() {
    const QUEUE: ObjectName = mqstr!("DEV.QUEUE.1");
    let cb = ConnectionOptions::default_binding().credentials(Credentials::user("app", "app"));
    let (qm, ..) = QueueManager::new(None, &cb)
        .warn_as_error()
        .expect("Could not establish connection");

    thread::spawn(move || {
        let mut od = MqStruct::<sys::MQOD>::default();

        QUEUE.copy_into_mqchar(&mut od.ObjectName);
        od.ObjectType = sys::MQOT_Q;

        let props = Message::new(&qm, MqValue::default()).expect("property creation");
        props
            .set_property("my_property", "value", MqValue::default())
            .warn_as_error()
            .expect("property set");

        qm.put_message::<()>(&mut od, MqMask::default(), &Properties::New(Some(&props)), "Hello")
            .warn_as_error()
            .expect("Put failed");
    })
    .join()
    .expect("Panic from connection thread");
}

#[test]
fn get_message() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: ObjectName = mqstr!("DEV.QUEUE.1");
    //let mut buffer = vec![0u8; 2 * 1024 * 1024]; //2M
    // let mut buffer = [0u8; 2*1024];
    let cb = ConnectionOptions::default_binding().credentials(Credentials::user("app", "app"));
    let (qm, ..) = QueueManager::new(None, &cb).warn_as_error()?;

    let mut od = MqStruct::<sys::MQOD>::default();
    od.ObjectName = QUEUE.into();
    od.ObjectType = sys::MQOT_Q;
    let object = Object::open(&qm, &od, MqMask::from(sys::MQOO_INPUT_AS_Q_DEF))?;
    let mut properties = Message::new(&qm, MqValue::default())?;
    let mut buffer = [0u8; 2 * 1024];
    let msg: Completion<Option<get::Mqmd<Cow<str>>>> = object
        .get_message(
            // Get a vector with an MQMD
            MqMask::default(),     // Just the default GET options
            get::ANY_MESSAGE,      // No selection criteria
            Some(2000),            // Wait 2 seconds
            Some(&mut properties), // Populate the properties
            buffer.as_mut_slice(), // Use the stack as buffer
        )?;

    let msg = msg.discard_warning(); // We don't care about warning  s, right?

    println!("{}", msg.map_or("{no message}".into(), GetMessage::into_payload));

    for v in properties.property_iter("%", MqMask::default()) {
        let (name, value): (String, String) = v.warn_as_error()?;
        println!("{name}: {value}");
    }

    Ok(())
}

#[test]
fn inq_qm() -> Result<(), Box<dyn std::error::Error>> {
    const INQ: &[inq::InqReqType] = &[
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
            inq::InqReqItem::Str(sys::MQ_VERSION_LENGTH),
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
                    inq::InqResItem::Str(value) => Cow::from(value),
                    inq::InqResItem::Long(value) => Cow::from(value.to_string()),
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

    QUEUE.copy_into_mqchar(&mut od.ObjectName);
    let (connection, ..) = QueueManager::new(None, &cb).warn_as_error()?;
    let object = Object::open(&connection, &od, MqMask::from(sys::MQOO_OUTPUT)).warn_as_error()?;

    object
        .put_message::<()>(MqMask::default(), &Properties::default(), "message")
        .warn_as_error()?;

    Ok(())
}
