#![cfg(all(feature = "mock", feature = "exits"))]

use libmqm_sys::mock::MockMq;
use mqi::{MqChar, MqFunctions, prelude::*, result::Error, string::CCSID, test::mock, types::MQDCC};

#[test]
fn mqxcnvc() -> Result<(), Error> {
    let mut mock = MockMq::new();
    mock.expect_MQXCNVC().returning(|_, _, _, length, _, _, _, _, _, cc, rc| {
        assert_eq!(length, 1024);
        mock::mqi_outcome_ok(cc, rc);
    });

    let mq = MqFunctions(mock);
    let buffer: MqChar<1024> = [0; 1024];
    let mut target: MqChar<1024> = [0; 1024];

    let _ = mq
        .mqxcnvc(None, MQDCC::default(), CCSID(1208), &buffer, CCSID(500), &mut target)
        .warn_as_error()?;

    Ok(())
}
