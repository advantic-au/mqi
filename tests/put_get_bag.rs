#![cfg(feature = "mqai")]

use mqi::{
    Bag, Object, constants,
    headers::{TextEnc, fmt},
    open,
    prelude::*,
    test, types,
};
#[cfg(not(feature = "mock"))]
use mqi::{ThreadNone, connect_options::Credentials};

#[test]
fn put_get_bag() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: types::QueueName = types::QueueName(mqstr!("DEV.QUEUE.1"));
    #[allow(clippy::allow_attributes, unused_mut)]
    let mut connection;
    #[cfg(feature = "mock")]
    {
        connection = test::mock::connect_ok(|mock_library| {
            let mut seq = mockall::Sequence::new();
            test::mock::open_ok(mock_library, 0x0c0c, 1, &mut seq);
            mock_library
                .expect_mqCreateBag()
                .returning(move |_, bag_handle, cc, rc| {
                    *bag_handle = 0x0f0f;
                    test::mock::mqi_outcome_ok(cc, rc);
                })
                .times(1)
                .in_sequence(&mut seq);
            mock_library
                .expect_mqSetInteger()
                .returning(move |_, _, _, _, cc, rc| test::mock::mqi_outcome_ok(cc, rc))
                .times(1)
                .in_sequence(&mut seq);
            mock_library
                .expect_mqPutBag()
                .returning(move |_, _, _, _, _, cc, rc| test::mock::mqi_outcome_ok(cc, rc))
                .times(1)
                .in_sequence(&mut seq);
            mock_library
                .expect_mqCreateBag()
                .returning(move |_, bag_handle, cc, rc| {
                    *bag_handle = 0x0d0d;
                    test::mock::mqi_outcome_ok(cc, rc);
                })
                .times(1)
                .in_sequence(&mut seq);
            mock_library
                .expect_mqSetInteger()
                .returning(move |_, _, _, _, cc, rc| test::mock::mqi_outcome_ok(cc, rc))
                .times(1)
                .in_sequence(&mut seq);
            mock_library
                .expect_mqGetBag()
                .returning(move |_, _, _, _, _, cc, rc| test::mock::mqi_outcome_ok(cc, rc))
                .times(1)
                .in_sequence(&mut seq);
            mock_library
                .expect_mqDeleteBag()
                .returning(move |_, cc, rc| test::mock::mqi_outcome_ok(cc, rc))
                .times(2)
                .in_sequence(&mut seq);
        });
    }
    #[cfg(not(feature = "mock"))]
    {
        let lib = std::rc::Rc::from(test::mq_library());
        let creds = test::credentials();
        let cred_options: Credentials<_> = creds.as_ref().into();
        connection = mqi::connect_lib::<ThreadNone, _>(lib, &cred_options).warn_as_error()?;
    }

    // Open the queue
    let object = Object::open(&connection, &(QUEUE, constants::MQOO_INPUT_SHARED | constants::MQOO_OUTPUT))?;

    // Put an empty bag on the queue, return the message id
    let bag = Bag::new_lib(connection.library(), constants::MQCBO_NONE).warn_as_error()?;
    let mid: types::MessageId = object
        .put_bag_with(&(), TextEnc::Ascii(fmt::MQFMT_ADMIN), &bag)
        .warn_as_error()?;

    // Retrieve the bag from the queue by message id
    let mut bag_from_queue = Bag::new_lib(connection.library(), constants::MQCBO_NONE).warn_as_error()?;
    assert!(object.get_bag(&mid, &mut bag_from_queue).warn_as_error()?);

    Ok(())
}

#[test]
fn bag_no_message() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: types::QueueName = types::QueueName(mqstr!("DEV.QUEUE.1"));
    #[allow(clippy::allow_attributes, unused_mut)]
    let mut connection;
    #[cfg(feature = "mock")]
    {
        connection = test::mock::connect_ok(|mock_library| {
            let mut seq = mockall::Sequence::new();
            test::mock::open_ok(mock_library, 0x0c0c, 1, &mut seq);
            mock_library
                .expect_mqCreateBag()
                .returning(move |_, bag_handle, cc, rc| {
                    *bag_handle = 0x0f0f;
                    test::mock::mqi_outcome_ok(cc, rc);
                })
                .times(1)
                .in_sequence(&mut seq);
            mock_library
                .expect_mqSetInteger()
                .returning(move |_, _, _, _, cc, rc| test::mock::mqi_outcome_ok(cc, rc))
                .times(1)
                .in_sequence(&mut seq);
            test::mock::mqai::get_bag_error(mock_library, constants::MQRC_NO_MSG_AVAILABLE, 1, &mut seq);
            mock_library
                .expect_mqDeleteBag()
                .returning(move |_, cc, rc| test::mock::mqi_outcome_ok(cc, rc))
                .times(1)
                .in_sequence(&mut seq);
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
            open::SelectionString("Root.MQMD.CorrelId = 0x0c0c0c0c"), // This should not exist
        ),
    )?;

    let mut bag = Bag::new_lib(connection.library(), constants::MQCBO_NONE).discard_warning()?;
    assert!(!object.get_bag(&(), &mut bag).warn_as_error()?);

    Ok(())
}
