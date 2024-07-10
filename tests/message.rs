use std::error::Error;

use libmqm_sys::link::LinkedMQ;
use mqi::{prop, sys, ConnectionOptions, Credentials, Message, MqMask, MqStruct, MqValue, QueueManager, ResultCompExt};

#[test]
fn message_handle() -> Result<(), Box<dyn Error>> {
    let (conn, ..) = QueueManager::new(
        None,
        &ConnectionOptions::default_binding().credentials(Credentials::user("app", "app")),
    )
    .warn_as_error()?;
    let handle = &conn.handle();

    let message = Message::new(&LinkedMQ, handle, MqValue::from(sys::MQCMHO_DEFAULT_VALIDATION))?;
    message
        .set_property(
            "property",
            "-1z",
            MqValue::from(sys::MQSMPO_NONE),
            &MqStruct::<sys::MQPD>::default(),
        )
        .warn_as_error()?;

    message
        .set_property(
            "property2",
            "aa",
            MqValue::from(sys::MQSMPO_NONE),
            &MqStruct::<sys::MQPD>::default(),
        )
        .warn_as_error()?;

    message
        .set_property(
            "property3",
            "-1",
            MqValue::from(sys::MQSMPO_NONE),
            &MqStruct::<sys::MQPD>::default(),
        )
        .warn_as_error()?;

    // if let Some((name, my_str)) = message.inq::<(String, PropDetails<Conversion<i32>>), _>("%", MqMask::default()).discard_completion()? {
    //     println!("{name} = \"{:?}\" (support: {}, context: {}, copy: {})", my_str, my_str.support(), my_str.context(), my_str.copy_options());
    // }

    for v in message.inq_iter::<(String, i32), _>(prop::INQUIRE_ALL, MqMask::default()) {
        println!("{v:?}");
    }

    Ok(())
}
