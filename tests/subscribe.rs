#![cfg(feature = "mock")]

use mqi::{open_options::ObjectString, sys, values, Object, Subscription};
use mqi::{prelude::*, test};

#[test]
fn subscribe() -> Result<(), Box<dyn std::error::Error>> {
    let connection = test::mock::connect_ok(|mock_library| {
        let mut seq = mockall::Sequence::new();
        mock_library.open_ok(0x0101_0101, 1, &mut seq);
        mock_library.subscribe_managed_ok(0x0505, 0x5b5b, 1, &mut seq);
    });

    let object = Object::open(connection.connection_ref(), &()).warn_as_error()?;
    let (sub, obj) = Subscription::subscribe_managed(
        connection.connection_ref(),
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
