use mqi::{open_options::ObjectString, sys, values, Object, Subscription, ThreadNone};
use mqi::prelude::*;

mod helpers;

#[test]
fn put_message() -> Result<(), Box<dyn std::error::Error>> {
    let mut mock = helpers::mock::connect_ok();
    let mut seq = mockall::Sequence::new();
    mock.open_ok(0x0101_0101, 1, &mut seq);

    let qm = mqi::connect_lib::<ThreadNone, _>(&mock, ()).warn_as_error()?;
    let object = Object::open(qm, ()).warn_as_error()?;
    object.put_message((), "Hello").warn_as_error()?;
    Ok(())
}

#[test]
fn subscribe() -> Result<(), Box<dyn std::error::Error>> {
    let mut mock = helpers::mock::connect_ok();
    let mut seq = mockall::Sequence::new();
    mock.open_ok(0x0101_0101, 1, &mut seq);

    let qm = mqi::connect_lib::<ThreadNone, _>(&mock, ()).warn_as_error()?;
    let object = Object::open(qm.connection_ref(), ()).warn_as_error()?;
    let (sub, obj) = Subscription::subscribe_managed(
        qm.connection_ref(),
        (
            values::MQSO(sys::MQSO_CREATE | sys::MQSO_NON_DURABLE),
            &object,
            ObjectString("dev/"),
        ),
    )
    .warn_as_error()?;

    sub.close().warn_as_error()?;
    obj.close().warn_as_error()?;

    Ok(())
}
