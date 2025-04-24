#![cfg(any(feature = "link", feature = "dlopen2"))]

use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;

use mqi::test;
use mqi::headers::fmt;
use mqi::open_options::SelectionString;
use mqi::prelude::*;
use mqi::attribute::{AttributeType, AttributeValue, InqResItem};
use mqi::constants;
use mqi::types::{MQOO, MQCMHO, MQXA};
use mqi::types::{MessageFormat, MessageId, QueueManagerName, QueueName};
use mqi::{get, Properties};
use mqi::{attribute, sys, Object};

#[cfg(not(feature = "mock"))]
use mqi::{connect_options::Credentials, ThreadNone};

#[test]
fn no_message() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));
    #[allow(clippy::allow_attributes, unused_mut)]
    let mut connection;
    #[cfg(feature = "mock")]
    {
        connection = test::mock::connect_ok(|mock_library| {
            let mut seq = mockall::Sequence::new();
            mock_library.open_ok(0x0c0c, 1, &mut seq);
            mock_library.get_error(constants::MQRC_NO_MSG_AVAILABLE, 1, &mut seq);
        });
    }
    #[cfg(not(feature = "mock"))]
    {
        let creds = test::credentials();
        let cred_options: Credentials<_> = creds.as_ref().into();
        connection = mqi::connect_lib::<ThreadNone, _>(test::mq_library(), &cred_options).warn_as_error()?;
    }
    let object = Object::open(
        &connection,
        &(
            QUEUE,
            constants::MQOO_INPUT_AS_Q_DEF,
            SelectionString("Root.MQMD.CorrelId = 0x0c0c0c0c"), // This should not exist
        ),
    )?;

    let mut buffer = vec![0; 4 * 1024]; // Use and consume a vector for the buffer
    let msg = object.get_data(&(), &mut buffer)?;

    assert_eq!(msg.warning(), None);
    assert_eq!(msg.discard_warning(), None);

    Ok(())
}

#[test]
fn put_get_message() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));
    #[allow(clippy::allow_attributes, unused_mut)]
    let mut connection;
    #[cfg(feature = "mock")]
    {
        connection = test::mock::connect_ok(|mock_library| {
            let mut seq = mockall::Sequence::new();
            mock_library.open_ok(0x0c0c, 1, &mut seq);
            mock_library.expect_MQPUT().returning(|_, _, _, _, _, _, cc, rc| {
                // TODO: add assertions here
                test::mock::MockFunctions::mqi_outcome_ok(cc, rc);
            });
            mock_library.properties_ok(0x0d0d, 1, &mut seq);
            mock_library.get_ok("put_get_message test", 1, &mut seq);
        });
    }
    #[cfg(not(feature = "mock"))]
    {
        let creds = test::credentials();
        let cred_options: Credentials<_> = creds.as_ref().into();
        connection = mqi::connect_lib::<ThreadNone, _>(test::mq_library(), &cred_options).warn_as_error()?;
    }

    let object = Object::open(&connection, &(QUEUE, MQOO(sys::MQOO_INPUT_SHARED | sys::MQOO_OUTPUT)))?;

    let mid = object
        .put_message_with::<MessageId>(&(), "put_get_message test")
        .warn_as_error()?;

    let mut properties = Properties::new(&connection, MQCMHO::default())?;

    let buffer = vec![0; 4 * 1024]; // Use and consume a vector for the buffer
    let msg = object.get_as(
        &(
            &mut properties, // Get some properties
            mid,             // Only the message matching the output of put
        ),
        buffer,
    )?;

    let (msg, _msgid, format, headers): (Cow<[u8]>, MessageId, MessageFormat, get::Headers) =
        msg.discard_warning().expect("Message to be present");

    assert!(headers.all_headers().next().is_none());
    assert!(headers.error().is_none());
    assert_eq!(String::from_utf8_lossy(&msg), "put_get_message test");
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
            AttributeType::new(MQXA(sys::MQCA_VERSION), sys::MQ_VERSION_LENGTH as u32)
        },
        attribute::MQIA_COMMAND_LEVEL,
    ];

    #[allow(clippy::allow_attributes, unused_mut)]
    let mut connection;
    #[cfg(feature = "mock")]
    {
        connection = test::mock::connect_ok(|mock_library| {
            let mut seq = mockall::Sequence::new();
            mock_library.open_ok(0x0c0c, 1, &mut seq);
            mock_library
                .expect_MQINQ()
                .returning(|_, _, _, _, int_len, ints, char_len, chars, cc, rc| {
                    use std::slice;

                    let char_slice =
                        unsafe { slice::from_raw_parts_mut(chars, char_len.try_into().expect("char_len should be positive")) };
                    char_slice.fill(32);
                    let int_slice =
                        unsafe { slice::from_raw_parts_mut(ints, int_len.try_into().expect("int_len should be positive")) };
                    int_slice.fill(0);
                    test::mock::MockFunctions::mqi_outcome_ok(cc, rc);
                });
        });
    }
    #[cfg(not(feature = "mock"))]
    {
        let creds = test::credentials();
        let cred_options: Credentials<_> = creds.as_ref().into();

        connection = mqi::connect_lib::<ThreadNone, _>(test::mq_library(), &cred_options).warn_as_error()?;
    }

    let object = Object::open(connection, &(QueueManagerName(mqstr!("")), constants::MQOO_INQUIRE)).warn_as_error()?;

    let result = object.inq(INQ)?;
    if let Some((rc, verb)) = result.warning() {
        eprintln!("MQRC warning: {verb} {rc}");
    }

    let values: HashMap<_, _> = result.iter().map(InqResItem::into_tuple).collect();

    for (attr, value) in values {
        match value {
            AttributeValue::Text(value) => println!("{attr}: {value:?}"),
            AttributeValue::Long(value) => println!("{attr}: {value}"),
        }
    }

    let r = object.inq_item(attribute::MQCA_DEF_XMIT_Q_NAME).warn_as_error()?;
    println!("{r:?}");

    Ok(())
}

#[test]
fn put_message() -> Result<(), Box<dyn Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));
    #[allow(clippy::allow_attributes, unused_mut)]
    let mut connection;
    #[cfg(feature = "mock")]
    {
        connection = test::mock::connect_ok(|mock_library| {
            let mut seq = mockall::Sequence::new();
            mock_library.open_ok(0x0c0c, 1, &mut seq);
            mock_library.expect_MQPUT().returning(|_, _, _, _, _, _, cc, rc| {
                // TODO: add assertions here
                test::mock::MockFunctions::mqi_outcome_ok(cc, rc);
            });
        });
    }
    #[cfg(not(feature = "mock"))]
    {
        let creds = test::credentials();
        let cred_options: Credentials<_> = creds.as_ref().into();
        connection = mqi::connect_lib::<ThreadNone, _>(test::mq_library(), &cred_options).warn_as_error()?;
    }

    let object = Object::open(connection, &(QUEUE, constants::MQOO_OUTPUT)).warn_as_error()?;

    object.put_message(&(), "message").warn_as_error()?;

    Ok(())
}
