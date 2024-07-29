use libmqm_sys::link::LINKED;
use mqi::{core::MQFunctions, Error, MqMask, ResultCompExt};


#[test]
fn mqxcnvc() -> Result<(), Error> {
    let mq = MQFunctions(&LINKED);
    let buffer: [u8; 1024] = [0; 1024];
    let mut target: [u8; 1024] = [0; 1024];

    let length = mq.mqxcnvc(None, MqMask::default(), 1208, &buffer, 500, &mut target).warn_as_error()?;

    assert_eq!(length, 1024);

    Ok(())
}