use mqi::{
    connect_options::Credentials, open_options::ObjectString, prelude::*, sys, types::QueueName, values, Connection, Object,
    ShareBlock, Subscription,
};

#[test]
fn publish() -> Result<(), Box<dyn std::error::Error>> {
    const TOPIC: ObjectString<&str> = ObjectString("dev/");
    let connection = Connection::<_, ShareBlock>::connect(Credentials::user("app", "app")).warn_as_error()?;
    let object = Object::open(connection, (TOPIC, values::MQOO(sys::MQOO_OUTPUT))).warn_as_error()?;
    object.put_message((), "Hello").warn_as_error()?;
    Ok(())
}

#[test]
fn subscribe() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    let connection = Connection::<_, ShareBlock>::connect(Credentials::user("app", "app")).warn_as_error()?;
    let object = Object::open(&connection, (QUEUE, values::MQOO(sys::MQOO_INPUT_AS_Q_DEF))).warn_as_error()?;
    let (sub, obj) = Subscription::subscribe_managed(
        &connection,
        (
            values::MQSO(sys::MQSO_CREATE | sys::MQSO_NON_DURABLE),
            &object,
            ObjectString("dev/"),
        ),
    )
    .warn_as_error()?;

    println!("{sub:?}");
    println!("{obj:?}");
    println!("{object:?}");

    sub.close().warn_as_error()?;
    obj.close().warn_as_error()?;

    Ok(())
}
