#![cfg(feature = "mock")]

use mqi::{Object, Subscription, constants, open_options::ObjectString, prelude::*, test::mock};

#[test]
fn subscribe() -> Result<(), Box<dyn std::error::Error>> {
    let connection = mock::connect_ok(|mock_library| {
        let mut seq = mockall::Sequence::new();
        mock::open_ok(mock_library, 0x0101_0101, 1, &mut seq);
        mock::subscribe_managed_ok(mock_library, 0x0505, 0x5b5b, 1, &mut seq);
    });

    let object = Object::open(connection.connection_ref(), &()).warn_as_error()?;
    let (sub, obj) = Subscription::subscribe_managed(
        connection.connection_ref(),
        (
            constants::MQSO_CREATE | constants::MQSO_NON_DURABLE,
            &object,
            ObjectString("dev/"),
        ),
    )
    .warn_as_error()?;

    sub.close().warn_as_error()?;
    obj.close().warn_as_error()?;

    Ok(())
}
