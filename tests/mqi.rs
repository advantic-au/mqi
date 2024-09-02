use mqi::{
    core::{values, MQFunctions},
    Error, ResultCompExt as _,
};

#[test]
fn mqxcnvc() -> Result<(), Error> {
    let mq = MQFunctions::linked();
    let buffer: [u8; 1024] = [0; 1024];
    let mut target: [u8; 1024] = [0; 1024];

    let length = mq
        .mqxcnvc(None, values::MQDCC::default(), 1208, &buffer, 500, &mut target)
        .warn_as_error()?;

    assert_eq!(length, 1024);

    Ok(())
}
