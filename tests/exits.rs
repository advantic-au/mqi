#![cfg(all(feature = "mock", feature = "exits"))]

use mqi::{CCSID, Error, MqChar, MqFunctions, prelude::*, test::mock::MockFunctions, types::MQDCC};

#[test]
fn mqxcnvc() -> Result<(), Error> {
    let mut mock = MockFunctions::new();
    mock.expect_MQXCNVC().returning(|_, _, _, length, _, _, _, _, _, cc, rc| {
        assert_eq!(length, 1024);
        MockFunctions::mqi_outcome_ok(cc, rc);
    });

    let mq = MqFunctions(mock);
    let buffer: MqChar<1024> = [0; 1024];
    let mut target: MqChar<1024> = [0; 1024];

    let _ = mq
        .mqxcnvc(None, MQDCC::default(), CCSID(1208), &buffer, CCSID(500), &mut target)
        .warn_as_error()?;

    Ok(())
}
