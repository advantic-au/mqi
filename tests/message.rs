use std::error::Error;

use mqi::{
    property::{self, Attributes, OwnedRawMeta},
    ConnectionOptions, Credentials, Message, MqMask, MqValue, OwnedStrCcsid, QueueManager, ResultCompExt,
};

#[test]
fn message_handle() -> Result<(), Box<dyn Error>> {
    const PROPS: &[(&str, &str)] = &[("usr.b.x", "B"), ("usr.p.x", "A"), ("usr.c", "By"), ("usr.p.y", "C")];

    let (conn, ..) = QueueManager::new(
        None,
        &ConnectionOptions::default_binding().credentials(Credentials::user("app", "app")),
    )
    .warn_as_error()?;

    let message = Message::new(conn, MqValue::default())?;

    for &(name, value) in PROPS {
        message.set_property(name, value, MqValue::default()).warn_as_error()?;
    }

    for v in message.property_iter(property::INQUIRE_ALL, MqMask::default()) {
        let (name, value): (OwnedStrCcsid, Attributes<OwnedRawMeta>) = v.warn_as_error()?;
        println!("{name:?}, {value:?}");
    }

    Ok(())
}
