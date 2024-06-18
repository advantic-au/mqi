use libmqm_sys::link::LinkedMQ;
use mqi::{sys, QueueManager, ConnectionOptions, Credentials, Message, MqValue, ResultCompExt};

#[test]
fn message_handle() {
    let (conn, ..) = QueueManager::new(
        None,
        &ConnectionOptions::default_binding().credentials(Credentials::user("app", "app")),
    )
    .warn_as_error()
    .expect("Could not establish connection");
    let handle = &conn.handle();

    // let l = unsafe { MqmContainer::load_mqm_default() }.expect("Could not load library");
    let result = Message::new(&LinkedMQ, handle, MqValue::from(sys::MQCMHO_DEFAULT_VALIDATION))
        .expect("Unable to create message handle");
    let prop = result.inq_properties("property").next();
    println!("{prop:?}");
}
