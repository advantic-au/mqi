#![cfg(any(feature = "link", feature = "dlopen2"))]

use std::{borrow::Cow, collections::HashMap, error::Error};

#[cfg(not(feature = "mock"))]
use mqi::connection::{Credentials, ThreadNone};
use mqi::{
    Object, Properties, attribute,
    attribute::{AttributeType, AttributeValue, InqResItem},
    constants,
    get::Headers,
    header::fmt,
    open::SelectionString,
    prelude::*,
    test,
    types::{MQCMHO, MQXA, MessageFormat, MessageId, QueueManagerName, QueueName},
};

#[test]
fn no_message() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));
    #[allow(clippy::allow_attributes, unused_mut)]
    let mut connection;
    #[cfg(feature = "mock")]
    {
        connection = test::mock::connect_ok(|mock_library| {
            let mut seq = mockall::Sequence::new();
            test::mock::open_ok(mock_library, 0x0c0c, 1, &mut seq);
            test::mock::get_error(mock_library, constants::MQRC_NO_MSG_AVAILABLE, 1, &mut seq);
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
            test::mock::open_ok(mock_library, 0x0c0c, 1, &mut seq);
            mock_library.expect_MQPUT().returning(|_, _, _, _, _, _, cc, rc| {
                // TODO: add assertions here
                test::mock::mqi_outcome_ok(cc, rc);
            });
            test::mock::properties_ok(mock_library, 0x0d0d, 1, &mut seq);
            test::mock::get_ok(mock_library, "put_get_message test", 1, &mut seq);
        });
    }
    #[cfg(not(feature = "mock"))]
    {
        let creds = test::credentials();
        let cred_options: Credentials<_> = creds.as_ref().into();
        connection = mqi::connect_lib::<ThreadNone, _>(test::mq_library(), &cred_options).warn_as_error()?;
    }

    let object = Object::open(&connection, &(QUEUE, constants::MQOO_INPUT_SHARED | constants::MQOO_OUTPUT))?;

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

    let (msg, _msgid, format, headers): (Cow<[u8]>, MessageId, MessageFormat, Headers) =
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
            AttributeType::new(MQXA(constants::MQCA_VERSION.0), libmqm_sys::MQ_VERSION_LENGTH as u32)
        },
        attribute::MQIA_COMMAND_LEVEL,
    ];

    #[allow(clippy::allow_attributes, unused_mut)]
    let mut connection;
    #[cfg(feature = "mock")]
    {
        connection = test::mock::connect_ok(|mock_library| {
            let mut seq = mockall::Sequence::new();
            test::mock::open_ok(mock_library, 0x0c0c, 1, &mut seq);
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
                    test::mock::mqi_outcome_ok(cc, rc);
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

    let result = object.inquire(INQ)?;
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

    let r = object.inquire_item(attribute::MQCA_DEF_XMIT_Q_NAME).warn_as_error()?;
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
            test::mock::open_ok(mock_library, 0x0c0c, 1, &mut seq);
            mock_library.expect_MQPUT().returning(|_, _, _, _, _, _, cc, rc| {
                // TODO: add assertions here
                test::mock::mqi_outcome_ok(cc, rc);
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

#[cfg(feature = "mock")]
#[test]
fn open_resolved() -> Result<(), Box<dyn Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));
    let connection = test::mock::connect_ok(|mock_library| {
        mock_library
            .expect_MQOPEN()
            .returning(|_, mqod, _, hobj, cc, rc| {
                let resolved_queue: QueueName = QueueName(mqstr!("RESOLVED.QUEUE"));
                let resolved_qmgr: QueueManagerName = QueueManagerName(mqstr!("QM1"));
                mqod.ResolvedQName = *resolved_queue.0.as_mqchar();
                mqod.ResolvedQMgrName = *resolved_qmgr.0.as_mqchar();
                *hobj = 0x0c0c;
                test::mock::mqi_outcome_ok(cc, rc);
            })
            .times(1);
        mock_library
            .expect_MQCLOSE()
            .returning(|_, obj, _, cc, rc| {
                *obj = -1;
                test::mock::mqi_outcome_ok(cc, rc);
            })
            .times(1);
    });

    let (object, resolved_queue, resolved_qmgr) =
        Object::open_resolved(connection, &(QUEUE, constants::MQOO_INPUT_AS_Q_DEF)).warn_as_error()?;
    assert_eq!(resolved_queue, Some(QueueName(mqstr!("RESOLVED.QUEUE"))));
    assert_eq!(resolved_qmgr, Some(QueueManagerName(mqstr!("QM1"))));
    drop(object);
    Ok(())
}

#[cfg(feature = "mock")]
#[test]
fn inquire_integer() -> Result<(), Box<dyn Error>> {
    let connection = test::mock::connect_ok(|mock_library| {
        mock_library
            .expect_MQOPEN()
            .returning(|_, _, _, hobj, cc, rc| {
                *hobj = 0x0c0c;
                test::mock::mqi_outcome_ok(cc, rc);
            })
            .times(1);
        mock_library
            .expect_MQINQ()
            .returning(|_, _, _, _, int_len, ints, char_len, chars, cc, rc| {
                use std::slice;

                if int_len > 0 {
                    let int_slice =
                        unsafe { slice::from_raw_parts_mut(ints, int_len.try_into().expect("int_len should be positive")) };
                    int_slice.fill(1024);
                }
                if char_len > 0 {
                    let char_slice =
                        unsafe { slice::from_raw_parts_mut(chars, char_len.try_into().expect("char_len should be positive")) };
                    char_slice.fill(0x20);
                }
                test::mock::mqi_outcome_ok(cc, rc);
            })
            .times(1);
        mock_library
            .expect_MQCLOSE()
            .returning(|_, obj, _, cc, rc| {
                *obj = -1;
                test::mock::mqi_outcome_ok(cc, rc);
            })
            .times(1);
    });

    let object = Object::open(connection, &(QueueManagerName(mqstr!("")), constants::MQOO_INQUIRE)).warn_as_error()?;
    let value = object.inquire_integer(attribute::MQIA_MAX_MSG_LENGTH).warn_as_error()?;
    assert_eq!(value, Some(1024));
    drop(object);
    Ok(())
}

#[cfg(feature = "mock")]
#[test]
fn inquire_text() -> Result<(), Box<dyn Error>> {
    let connection = test::mock::connect_ok(|mock_library| {
        mock_library
            .expect_MQOPEN()
            .returning(|_, _, _, hobj, cc, rc| {
                *hobj = 0x0c0c;
                test::mock::mqi_outcome_ok(cc, rc);
            })
            .times(1);
        mock_library
            .expect_MQINQ()
            .returning(|_, _, _, _, int_len, ints, char_len, chars, cc, rc| {
                use std::slice;

                if int_len > 0 {
                    let int_slice =
                        unsafe { slice::from_raw_parts_mut(ints, int_len.try_into().expect("int_len should be positive")) };
                    int_slice.fill(0);
                }
                if char_len > 0 {
                    let char_slice =
                        unsafe { slice::from_raw_parts_mut(chars, char_len.try_into().expect("char_len should be positive")) };
                    char_slice.fill(0x20);
                }
                test::mock::mqi_outcome_ok(cc, rc);
            })
            .times(1);
        mock_library
            .expect_MQCLOSE()
            .returning(|_, obj, _, cc, rc| {
                *obj = -1;
                test::mock::mqi_outcome_ok(cc, rc);
            })
            .times(1);
    });

    let object = Object::open(connection, &(QueueManagerName(mqstr!("")), constants::MQOO_INQUIRE)).warn_as_error()?;
    let value = object.inquire_text(attribute::MQCA_DEF_XMIT_Q_NAME).warn_as_error()?;
    let value = value.expect("text attribute should be present");
    assert_eq!(value.len(), 48);
    assert!(value.iter().all(|c| *c == 0x20));
    drop(object);
    Ok(())
}

#[cfg(feature = "mock")]
#[test]
fn inquire_mixed_attributes() -> Result<(), Box<dyn Error>> {
    let connection = test::mock::connect_ok(|mock_library| {
        mock_library
            .expect_MQOPEN()
            .returning(|_, _, _, hobj, cc, rc| {
                *hobj = 0x0c0c;
                test::mock::mqi_outcome_ok(cc, rc);
            })
            .times(1);
        mock_library
            .expect_MQINQ()
            .returning(|_, _, _, _, int_len, ints, char_len, chars, cc, rc| {
                use std::slice;

                assert_eq!(int_len, 1);
                assert_eq!(
                    char_len,
                    (libmqm_sys::MQ_DATE_LENGTH + libmqm_sys::MQ_Q_MGR_NAME_LENGTH)
                        .try_into()
                        .expect("text buffer length should fit MQLONG")
                );
                let int_slice =
                    unsafe { slice::from_raw_parts_mut(ints, int_len.try_into().expect("int_len should be positive")) };
                int_slice[0] = 1024;
                let char_slice =
                    unsafe { slice::from_raw_parts_mut(chars, char_len.try_into().expect("char_len should be positive")) };
                let (date, qmgr_name) = char_slice.split_at_mut(libmqm_sys::MQ_DATE_LENGTH);
                date.fill(0x31);
                qmgr_name.fill(0x32);
                test::mock::mqi_outcome_ok(cc, rc);
            })
            .times(1);
        mock_library
            .expect_MQCLOSE()
            .returning(|_, obj, _, cc, rc| {
                *obj = -1;
                test::mock::mqi_outcome_ok(cc, rc);
            })
            .times(1);
    });

    let object = Object::open(connection, &(QueueManagerName(mqstr!("")), constants::MQOO_INQUIRE)).warn_as_error()?;
    let result = object
        .inquire(&[
            attribute::MQCA_ALTERATION_DATE,
            attribute::MQIA_MAX_MSG_LENGTH,
            attribute::MQCA_Q_MGR_NAME,
        ])
        .warn_as_error()?;
    let mut items = result.iter();

    let (selector, value) = items.next().expect("date attribute should be present").into_tuple();
    assert_eq!(selector, MQXA(constants::MQCA_ALTERATION_DATE.0));
    match value {
        AttributeValue::Text(date) => {
            assert_eq!(date.len(), libmqm_sys::MQ_DATE_LENGTH);
            assert!(date.iter().all(|c| *c == 0x31));
        }
        AttributeValue::Long(_) => panic!("date attribute should be text"),
    }

    let (selector, value) = items.next().expect("integer attribute should be present").into_tuple();
    assert_eq!(selector, MQXA(constants::MQIA_MAX_MSG_LENGTH.0));
    assert!(matches!(value, AttributeValue::Long(1024)));

    let (selector, value) = items.next().expect("queue manager name should be present").into_tuple();
    assert_eq!(selector, MQXA(constants::MQCA_Q_MGR_NAME.0));
    match value {
        AttributeValue::Text(qmgr_name) => {
            assert_eq!(qmgr_name.len(), libmqm_sys::MQ_Q_MGR_NAME_LENGTH);
            assert!(qmgr_name.iter().all(|c| *c == 0x32));
        }
        AttributeValue::Long(_) => panic!("queue manager name should be text"),
    }
    assert!(items.next().is_none());
    drop(object);
    Ok(())
}
