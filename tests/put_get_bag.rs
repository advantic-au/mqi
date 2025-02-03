use mqi::{admin::Bag, connect_options, open_options, prelude::*, sys, test, types, values, Object, ThreadNone};

#[test]
fn bag_message() -> Result<(), Box<dyn std::error::Error>> {
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
