use std::error::Error;

use libmqm_sys::link::LinkedMQ;
use mqi::{sys, ConnectionOptions, Credentials, Message, MqStruct, MqValue, QueueManager, ResultCompExt};

#[test]
fn message_handle() -> Result<(), Box<dyn Error>> {
    let (conn, ..) = QueueManager::new(
        None,
        &ConnectionOptions::default_binding().credentials(Credentials::user("app", "app")),
    )
    .warn_as_error()?;
    let handle = &conn.handle();

    let message = Message::new(&LinkedMQ, handle, MqValue::from(sys::MQCMHO_DEFAULT_VALIDATION))?;
    message.set_property("property", "hello", MqValue::from(sys::MQSMPO_NONE), &MqStruct::<sys::MQPD>::default()).warn_as_error()?;
    let prop = message.inq_properties("property").next().map(mqi::ResultCompExt::warn_as_error).transpose()?;
    println!("{prop:?}");

    Ok(())
}
