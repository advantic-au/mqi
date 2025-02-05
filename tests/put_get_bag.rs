use mqi::{
    admin::Bag,
    connect_options,
    headers::{fmt, TextEnc},
    open_options,
    prelude::*,
    sys, test, types, values, Object, ThreadNone,
};

#[test]
fn put_get_bag() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: types::QueueName = types::QueueName(mqstr!("DEV.QUEUE.1"));
    #[allow(clippy::allow_attributes, unused_mut)]
    let mut mq_lib;
    #[cfg(feature = "mock")]
    {
        mq_lib = test::mock::connect_ok();
        let mut seq = mockall::Sequence::new();
        mq_lib.open_ok(0x0c0c, 1, &mut seq);
        mq_lib
            .expect_mqCreateBag()
            .returning(move |_, bag_handle, cc, rc| {
                unsafe { *bag_handle = 0x0f0f };
                test::mock::MockFunctions::mqi_outcome_ok(cc, rc);
            })
            .times(1)
            .in_sequence(&mut seq);
        mq_lib
            .expect_mqSetInteger()
            .returning(move |_, _, _, _, cc, rc| test::mock::MockFunctions::mqi_outcome_ok(cc, rc))
            .times(1)
            .in_sequence(&mut seq);
        mq_lib
            .expect_mqPutBag()
            .returning(move |_, _, _, _, _, cc, rc| test::mock::MockFunctions::mqi_outcome_ok(cc, rc))
            .times(1)
            .in_sequence(&mut seq);

        mq_lib
            .expect_mqCreateBag()
            .returning(move |_, bag_handle, cc, rc| {
                unsafe { *bag_handle = 0x0d0d };
                test::mock::MockFunctions::mqi_outcome_ok(cc, rc);
            })
            .times(1)
            .in_sequence(&mut seq);
        mq_lib
            .expect_mqSetInteger()
            .returning(move |_, _, _, _, cc, rc| test::mock::MockFunctions::mqi_outcome_ok(cc, rc))
            .times(1)
            .in_sequence(&mut seq);
        mq_lib
            .expect_mqGetBag()
            .returning(move |_, _, _, _, _, cc, rc| test::mock::MockFunctions::mqi_outcome_ok(cc, rc))
            .times(1)
            .in_sequence(&mut seq);

        mq_lib
            .expect_mqDeleteBag()
            .returning(move |_, cc, rc| test::mock::MockFunctions::mqi_outcome_ok(cc, rc))
            .times(2)
            .in_sequence(&mut seq);
    }
    #[cfg(not(feature = "mock"))]
    {
        mq_lib = test::mq_library();
    }
    let creds = test::credentials();
    let cred_options: connect_options::Credentials<_> = creds.as_ref().into();
    let qm = mqi::connect_lib::<ThreadNone, _>(&mq_lib, &cred_options).warn_as_error()?;

    // Open the queue
    let object = Object::open(&qm, &(QUEUE, values::MQOO(sys::MQOO_INPUT_SHARED | sys::MQOO_OUTPUT)))?;

    // Put an empty bag on the queue, return the message id
    let bag = Bag::new_lib(&mq_lib, values::MQCBO(sys::MQCBO_NONE)).warn_as_error()?;
    let mid: types::MessageId = object
        .put_bag_with(&(), TextEnc::Ascii(fmt::MQFMT_ADMIN), &bag)
        .warn_as_error()?;

    // Retrieve the bag from the queue by message id
    let mut bag_from_queue = Bag::new_lib(&mq_lib, values::MQCBO(sys::MQCBO_NONE)).warn_as_error()?;
    assert!(object.get_bag(&mid, &mut bag_from_queue).warn_as_error()?);

    Ok(())
}

#[test]
fn bag_no_message() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: types::QueueName = types::QueueName(mqstr!("DEV.QUEUE.1"));
    #[allow(clippy::allow_attributes, unused_mut)]
    let mut mq_lib;
    #[cfg(feature = "mock")]
    {
        mq_lib = test::mock::connect_ok();
        let mut seq = mockall::Sequence::new();
        mq_lib.open_ok(0x0c0c, 1, &mut seq);
        mq_lib
            .expect_mqCreateBag()
            .returning(move |_, bag_handle, cc, rc| {
                unsafe { *bag_handle = 0x0f0f };
                test::mock::MockFunctions::mqi_outcome_ok(cc, rc);
            })
            .times(1)
            .in_sequence(&mut seq);
        mq_lib
            .expect_mqSetInteger()
            .returning(move |_, _, _, _, cc, rc| test::mock::MockFunctions::mqi_outcome_ok(cc, rc))
            .times(1)
            .in_sequence(&mut seq);
        mq_lib.get_bag_error(sys::MQRC_NO_MSG_AVAILABLE, 1, &mut seq);
        mq_lib
            .expect_mqDeleteBag()
            .returning(move |_, cc, rc| test::mock::MockFunctions::mqi_outcome_ok(cc, rc))
            .times(1)
            .in_sequence(&mut seq);
    }
    #[cfg(not(feature = "mock"))]
    {
        mq_lib = test::mq_library();
    }
    let creds = test::credentials();
    let cred_options: connect_options::Credentials<_> = creds.as_ref().into();
    let qm = mqi::connect_lib::<ThreadNone, _>(&mq_lib, &cred_options).warn_as_error()?;

    let object = Object::open(
        &qm,
        &(
            QUEUE,
            values::MQOO(sys::MQOO_INPUT_AS_Q_DEF),
            open_options::SelectionString("Root.MQMD.CorrelId = 0x0c0c0c0c"), // This should not exist
        ),
    )?;

    let mut bag = Bag::new_lib(&mq_lib, values::MQCBO(sys::MQCBO_NONE)).discard_warning()?;
    assert!(!object.get_bag(&(), &mut bag).warn_as_error()?);

    Ok(())
}
