#![cfg(feature = "mock")]

use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::thread;

use mqi::headers::fmt;
use mqi::{prelude::*, test, ThreadNone};
use mqi::attribute::{AttributeType, AttributeValue, InqResItem};
use mqi::values::{self, CCSID};
use mqi::types::{MessageFormat, MessageId, QueueManagerName};
use mqi::{get, Properties};
use mqi::{attribute, sys, Object};

#[test]
fn object() {
    let mut mock = test::mock::connect_ok();
    let mut seq = mockall::Sequence::new();
    mock.properties_ok(0x0c0c, 1, &mut seq);

    mock.expect_MQSETMP().returning(|_, _, _, _, _, _, _, _, cc, rc| {
        // TODO: assert values set
        test::mock::MockFunctions::mqi_outcome_ok(cc, rc);
    });
    mock.expect_MQPUT1().returning(|_, _, _, _, _, _, cc, rc| {
        // TODO: assert values set
        test::mock::MockFunctions::mqi_outcome_ok(cc, rc);
    });

    let qm = Arc::new(
        mqi::connect_lib::<mqi::ThreadBlock, _>(mock, ())
            .warn_as_error()
            .expect("connection should be established"),
    );

    thread::spawn(move || {
        let mut props = Properties::new(qm.clone(), values::MQCMHO::default()).expect("property creation");
        props
            .set_property("my_property", "valuex2", values::MQSMPO::default())
            .warn_as_error()
            .expect("property set should not fail");
        qm.put_message((), &mut props, "Hello")
            .warn_as_error()
            .expect("message put should not fail");
    })
    .join()
    .expect("thread join should not fail");
}

#[test]
fn get_message() -> Result<(), Box<dyn std::error::Error>> {
    let mut mock = test::mock::connect_ok();
    let mut seq = mockall::Sequence::new();
    mock.open_ok(0x0c0c, 1, &mut seq);
    mock.properties_ok(0x0d0d, 1, &mut seq);
    mock.get_ok("test message", 1, &mut seq);

    let qm = mqi::connect_lib::<ThreadNone, _>(mock, ()).warn_as_error()?;
    let object = Object::open(&qm, ())?;
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

    let (msg, _msgid, format, headers): (Cow<[u8]>, MessageId, MessageFormat, get::Headers) = msg.discard_warning().expect("Message to be present");

    assert!(headers.all_headers().next().is_none());
    assert!(headers.error().is_none());
    assert_eq!(String::from_utf8_lossy(&msg), "test message");
    assert_eq!(format.fmt, fmt::MQFMT_STRING);

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

    let mut mock = test::mock::connect_ok();
    let mut seq = mockall::Sequence::new();
    mock.open_ok(0x0c0c, 1, &mut seq);
    mock.expect_MQINQ().returning(|_, _, _, _, _, _, _, _, cc, rc| {
        // TODO: Add some return data
        test::mock::MockFunctions::mqi_outcome_ok(cc, rc);
    });

    let connection = mqi::connect_lib::<ThreadNone, _>(mock, ()).warn_as_error()?;
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
fn put_message() -> Result<(), Box<dyn Error>> {
    let mut mock = test::mock::connect_ok();
    let mut seq = mockall::Sequence::new();
    mock.open_ok(0x0c0c, 1, &mut seq);
    mock.expect_MQPUT().returning(|_, _, _, _, _, _, cc, rc| {
        // TODO: add assertions here
        test::mock::MockFunctions::mqi_outcome_ok(cc, rc);
    });

    let connection = mqi::connect_lib::<ThreadNone, _>(mock, ()).warn_as_error()?;
    let object = Object::open(connection, ()).warn_as_error()?;

    object.put_message((), "message").warn_as_error()?;

    Ok(())
}
