mod helpers;

use helpers::mq_library;
use mqi::{
    core::MqFunctions,
    prelude::*,
    values::{self, CCSID},
    Error,
};

#[test]
fn mqxcnvc() -> Result<(), Error> {
    let mq = MqFunctions(mq_library());
    let buffer: [u8; 1024] = [0; 1024];
    let mut target: [u8; 1024] = [0; 1024];

    let length = mq
        .mqxcnvc(None, values::MQDCC::default(), CCSID(1208), &buffer, CCSID(500), &mut target)
        .warn_as_error()?;

    assert_eq!(length, 1024);

    Ok(())
}
