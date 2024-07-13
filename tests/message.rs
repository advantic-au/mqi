use std::error::Error;

use libmqm_sys::link::LinkedMQ;
use mqi::{
    property::{self, Attributes, RawMeta},
    sys, ConnectionOptions, Credentials, Message, MqMask, MqValue, OwnedStrCcsid, QueueManager, ResultCompExt,
};

#[test]
fn message_handle() -> Result<(), Box<dyn Error>> {
    let (conn, ..) = QueueManager::new(
        None,
        &ConnectionOptions::default_binding().credentials(Credentials::user("app", "app")),
    )
    .warn_as_error()?;

    let message = Message::new(&LinkedMQ, Some(conn.handle()), MqValue::from(sys::MQCMHO_DEFAULT_VALIDATION))?;
    message
        .set_property("usr.b.x", "B", MqValue::from(sys::MQSMPO_NONE))
        .warn_as_error()?;
    message
        .set_property("usr.p.x", "A", MqValue::from(sys::MQSMPO_NONE))
        .warn_as_error()?;

    message
        .set_property("usr.c", "By", MqValue::from(sys::MQSMPO_NONE))
        .warn_as_error()?;

    message
        .set_property("usr.p.y", "C", MqValue::from(sys::MQSMPO_NONE))
        .warn_as_error()?;

    let v = message.property::<String>("usr.p.y", MqMask::default()).warn_as_error()?;

    for v in message.property_iter(property::INQUIRE_ALL, MqMask::default()) {
        let (name, value): (OwnedStrCcsid, Attributes<RawMeta<Vec<u8>>>)  = v.warn_as_error()?;
        println!("{name:?}, {value:?}");
    }

    Ok(())
}
