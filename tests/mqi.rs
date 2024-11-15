#![cfg(feature = "mock")]

use mqi::{
    core::MqFunctions,
    prelude::*,
    test::mock::MockFunctions,
    values::{self, CCSID},
    Error,
};

#[test]
fn mqxcnvc() -> Result<(), Error> {
    let mut mock = MockFunctions::new();
    mock.expect_MQXCNVC().returning(|_, _, _, length, _, _, _, _, _, cc, rc| {
        assert_eq!(length, 1024);
        MockFunctions::mqi_outcome_ok(cc, rc);
    });

    let mq = MqFunctions(mock);
    let buffer: [u8; 1024] = [0; 1024];
    let mut target: [u8; 1024] = [0; 1024];

    let _ = mq
        .mqxcnvc(None, values::MQDCC::default(), CCSID(1208), &buffer, CCSID(500), &mut target)
        .warn_as_error()?;

    Ok(())
}
