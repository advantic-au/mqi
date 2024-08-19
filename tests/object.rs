use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::thread;

use mqi::attribute::{AttributeValue, InqResItem};
use mqi::connect_options::Credentials;
use mqi::open_options::SelectionString;
use mqi::types::{MessageFormat, MessageId, QueueManagerName, QueueName};
use mqi::{get, prelude::*, Message};
use mqi::{attribute, mqstr, sys, Object, QueueManager};

#[test]
fn object() {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    let qm = QueueManager::connect(None, &Credentials::user("app", "app"))
        .warn_as_error()
        .expect("Could not establish connection");

    thread::spawn(move || {
        let mut props = Message::new(&qm, MqValue::default()).expect("property creation");
        props
            .set_property("my_property", "valuex2", MqValue::default())
            .warn_as_error()
            .expect("property set");

        qm.put_message::<()>(QUEUE, &mut props, "Hello")
            .warn_as_error()
            .expect("Put failed");
    })
    .join()
    .expect("Panic from connection thread");
}

#[test]
fn get_message() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));
    let sel = String::from("my_property = 'valuex2'");
    let qm = QueueManager::connect(None, &Credentials::user("app", "app")).warn_as_error()?;

    let object = Object::open::<Object<_>>(
        &qm,
        (QUEUE, SelectionString(&*sel)),
        MqMask::from(sys::MQOO_BROWSE | sys::MQOO_INPUT_AS_Q_DEF),
    )?;
    let mut properties = Message::new(&qm, MqValue::default())?;

    let buffer = vec![0; 4 * 1024]; // Use and consume a vector for the buffer
    let msg = object.get_message::<(MessageId, MessageFormat, get::Headers)>(
        (
            MqMask::from(sys::MQGMO_BROWSE_FIRST), // Browse it
            get::GetConvert::ConvertTo(500, MqMask::from(sys::MQENC_NORMAL)),
            &mut properties,          // Get some properties
            get::GetWait::Wait(2000), // Wait for 2 seconds
        ),
        buffer,
    )?;

    if let Some((rc, verb)) = msg.warning() {
        println!("Warning: {rc} on {verb}");
    }
    let msg = msg.discard_warning();

    match &msg {
        Some((msgid, format, headers)) => {
            for header in headers.all_headers() {
                println!("Header: {header:?}");
            }
            if let Some(header_error) = headers.error() {
                println!("Header parsing error: {header_error}");
            }

            if let Some(rfh2) = headers.header::<sys::MQRFH2>().next() {
                let nv: Cow<str> = rfh2.name_value_data().try_into()?;
                println!("RFH2 name/value data: \"{nv}\"");
            }
            for v in properties.property_iter("%", MqMask::default()) {
                let (name, value): (String, String) = v.warn_as_error()?;
                println!("Property: {name} = {value}");
            }
            println!("Format: \"{}\"", format.fmt);
            println!("MessageId: \"{msgid:?}\"");
        }
        None => println!("No message!"),
    }

    Ok(())
}

#[test]
fn inq_qm() -> Result<(), Box<dyn std::error::Error>> {
    const INQ: &[attribute::AttributeType] = &[
        attribute::MQCA_Q_MGR_NAME,
        attribute::MQCA_ALTERATION_DATE,
        attribute::MQCA_DEAD_LETTER_Q_NAME,
        attribute::MQCA_ALTERATION_TIME,
        attribute::MQCA_CREATION_DATE,
        attribute::MQCA_CREATION_TIME,
        attribute::MQIA_CODED_CHAR_SET_ID,
        attribute::MQCA_DEF_XMIT_Q_NAME,
        // (
        //     // Hmmm... this works. Not documented for MQINQ though.
        //     MqValue::from(sys::MQCA_VERSION),
        //     attribute::ValueType::Str(sys::MQ_VERSION_LENGTH),
        // ),
        attribute::MQIA_COMMAND_LEVEL,
    ];
    let qm = QueueManager::connect(None, &(Credentials::user("app", "app"))).discard_warning()?;
    let object = Object::open::<Object<_>>(&qm, QueueManagerName(mqstr!("QM1")), MqMask::from(sys::MQOO_INQUIRE))?;

    // let mut m = MultiItems::default();
    // m.push_text_item(&TextItem::new::<64>(attribute::MQCA_Q_MGR_DESC, &mqstr!("Warren test"))?);

    // object
    //     .set(&m)
    //     .warn_as_error()?;

    let result = object.inq(INQ.iter())?;
    if let Some((rc, verb)) = result.warning() {
        eprintln!("MQRC warning: {verb} {rc}");
    }

    let values: HashMap<_, _> = result
        .iter()
        .map(InqResItem::into_tuple)
        .collect();

    for (attr, value) in values {
        match value {
            AttributeValue::Text(value) => println!("{attr}: {value:?}"),
            AttributeValue::Long(value) => println!("{attr}: {value}"),
        };
    }

    let r = object.inq_item(attribute::MQCA_DEF_XMIT_Q_NAME).warn_as_error()?;
    println!("{r:?}");

    Ok(())
}

#[test]
fn transaction() -> Result<(), Box<dyn Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    let connection = QueueManager::connect(None, &Credentials::user("app", "app")).warn_as_error()?;
    let object = Object::open::<Object<_>>(&connection, QUEUE, MqMask::from(sys::MQOO_OUTPUT)).warn_as_error()?;

    object.put_message::<()>((), "message").warn_as_error()?;

    Ok(())
}
