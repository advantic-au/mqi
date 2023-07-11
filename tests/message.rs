use libmqm_sys::link::LinkedMQ;
use mqi::{Connection, ConnectionOptions, Credentials, Message, MessageHandleOptions};

#[test]
fn message_handle() {
    let conn = Connection::new(
        None,
        &ConnectionOptions::default_binding().credentials(Credentials::user("app", "app")),
    )
    .expect("Could not establish connection");
    let handle = &conn.handle();

    // let l = unsafe { MqmContainer::load_mqm_default() }.expect("Could not load library");
    let result = Message::new(&LinkedMQ, handle, MessageHandleOptions::default())
        .expect("Unable to create message handle");
    let prop = result.inq_properties("property").next();
    println!("{prop:?}");
}
