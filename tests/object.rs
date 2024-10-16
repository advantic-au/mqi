mod helpers;

use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::thread;

use helpers::{credentials_app, mq_library};

use mqi::{prelude::*, ThreadNoBlock, ThreadNone};
use mqi::attribute::{AttributeType, AttributeValue, InqResItem};
use mqi::values::{self, CCSID};
use mqi::open_options::SelectionString;
use mqi::properties_options::{Attributes, Metadata, Name};
use mqi::types::{MessageFormat, MessageId, QueueManagerName, QueueName};
use mqi::{get, Properties};
use mqi::{attribute, sys, Object};

#[test]
fn object() {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    let qm = mqi::connect_lib::<ThreadNoBlock, _>(mq_library(), credentials_app())
        .warn_as_error()
        .expect("connection should be established");

    thread::spawn(move || {
        let mut props = Properties::new(qm.connection_ref(), values::MQCMHO::default()).expect("property creation");
        props
            .set_property("my_property", "valuex2", values::MQSMPO::default())
            .warn_as_error()
            .expect("property set should not fail");
        qm.put_message(QUEUE, &mut props, "Hello")
            .warn_as_error()
            .expect("message put should not fail");
    })
    .join()
    .expect("thread join should not fail");
}

#[test]
fn get_message() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));
    let sel = String::from("my_property = 'valuex2'");
    let qm = mqi::connect_lib::<ThreadNone, _>(mq_library(), credentials_app()).warn_as_error()?;

    let object = Object::open(
        &qm,
        (
            QUEUE,
            SelectionString(&*sel),
            values::MQOO(sys::MQOO_BROWSE | sys::MQOO_INPUT_AS_Q_DEF),
        ),
    )?;
    let mut properties = Properties::new(&qm, values::MQCMHO::default())?;

    let buffer = vec![0; 4 * 1024]; // Use and consume a vector for the buffer
    let msg = object.get_as(
        (
            values::MQGMO(sys::MQGMO_BROWSE_FIRST), // Browse it
            get::GetConvert::ConvertTo(CCSID(500), values::MQENC(sys::MQENC_NORMAL)),
            &mut properties,          // Get some properties
            get::GetWait::Wait(2000), // Wait for 2 seconds
        ),
        buffer,
    )?;

    if let Some((rc, verb)) = msg.warning() {
        println!("Warning: {rc} on {verb}");
    }
    let msg: Option<(Cow<[u8]>, MessageId, MessageFormat, get::Headers)> = msg.discard_warning();

    match &msg {
        Some((_msg, msgid, format, headers)) => {
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
            for v in properties.property_iter("%", values::MQIMPO::default()) {
                let (value, Name(name), attr, meta): (String, Name<String>, Attributes, Metadata) = v.warn_as_error()?;
                println!("Property: {name} = {value}, {attr:?}, {meta:?}");
            }
            println!("Format: \"{}\"", format.fmt);
            println!("MessageId: \"{msgid}\"");
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
        // Hmmm... this works. Not documented for MQINQ though.
        #[expect(clippy::cast_possible_truncation)]
        unsafe {
            AttributeType::new(values::MQXA(sys::MQCA_VERSION), sys::MQ_VERSION_LENGTH as u32)
        },
        attribute::MQIA_COMMAND_LEVEL,
    ];

    let connection = mqi::connect_lib::<ThreadNone, _>(mq_library(), credentials_app()).warn_as_error()?;
    let (object, qm) = Object::open_with::<Option<QueueManagerName>>(
        connection,
        (QueueManagerName(mqstr!("QM1")), values::MQOO(sys::MQOO_INQUIRE)),
    )
    .warn_as_error()?;

    println!("{qm:?}");
    let result = object.inq(INQ)?;
    if let Some((rc, verb)) = result.warning() {
        eprintln!("MQRC warning: {verb} {rc}");
    }

    let values: HashMap<_, _> = result.iter().map(InqResItem::into_tuple).collect();

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

    let connection = mqi::connect_lib::<ThreadNone, _>(mq_library(), credentials_app()).warn_as_error()?;
    let object = Object::open(connection, (QUEUE, values::MQOO(sys::MQOO_OUTPUT))).warn_as_error()?;

    object.put_message((), "message").warn_as_error()?;

    Ok(())
}
