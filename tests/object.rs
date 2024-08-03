use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::thread;

use mqi::get::{GetConvert, GetMessage, GetWait};
use mqi::put::Properties;
use mqi::sys::MQENC_NORMAL;
use mqi::{get, prelude::*, Message, StrCcsidCow};
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
            .set_property("my_property", "valuex2", MqValue::default())
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
    let cb = ConnectionOptions::default_binding().credentials(Credentials::user("app", "app"));
    let (qm, ..) = QueueManager::new(None, &cb).warn_as_error()?;

    let mut od = MqStruct::<sys::MQOD>::default();
    od.ObjectName = QUEUE.into();
    let object = Object::open(&qm, &od, MqMask::from(sys::MQOO_BROWSE | sys::MQOO_INPUT_AS_Q_DEF))?;
    let mut properties = Message::new(&qm, MqValue::default())?;

    let buffer = vec![0; 4]; // Use and consume a vector for the buffer
    let msg: Completion<Option<get::Headers<StrCcsidCow>>> = object.get_message(
        (
            MqMask::from(sys::MQGMO_BROWSE_FIRST), // Browse it
            GetConvert::ConvertTo(500, MqMask::from(MQENC_NORMAL)),
            &mut properties,
            // CorrelationId([1; sys::MQ_CORREL_ID_LENGTH]),
            GetWait::Wait(2000), // Wait for 2 seconds
        ),
        buffer,
    )?;

    if let Some((rc, verb)) = msg.warning() {
        println!("Warning: {rc} on {verb}");
    }
    let msg = msg.discard_warning();

    match &msg {
        Some(msg) => {
            for header in msg.all_headers() {
                println!("Header: {header:?}");
            }
            if let Some(header_error) = msg.header_error() {
                println!("Header parsing error: {header_error}");
            }

            if let Some(rfh2) = msg.header::<sys::MQRFH2>().next() {
                let nv: Cow<str> = rfh2.name_value_data().try_into()?;
                println!("RFH2 name/value data: \"{nv}\"");
            }
            for v in properties.property_iter("%", MqMask::default()) {
                let (name, value): (String, String) = v.warn_as_error()?;
                println!("Property: {name} = {value}");
            }
            println!("Format: \"{}\"", msg.format().format);
            println!("Payload: \"{:?}\"", msg.payload());
        }
        None => println!("No message!"),
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
