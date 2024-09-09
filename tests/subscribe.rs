use mqi::{
    connect_options::Credentials,
    core::values::{self, MQOO, MQSO},
    mqstr,
    open_options::ObjectString,
    sys,
    types::QueueName,
    Object, QueueManager, ResultCompExt as _, Subscription,
};

#[test]
fn publish() -> Result<(), Box<dyn std::error::Error>> {
    const TOPIC: ObjectString<&str> = ObjectString("dev/");
    let qm = QueueManager::connect(Credentials::user("app", "app")).warn_as_error()?;
    let object = Object::open(qm, (TOPIC, MQOO(sys::MQOO_OUTPUT))).warn_as_error()?;
    object.put_message((), "Hello").warn_as_error()?;
    Ok(())
}

#[test]
fn subscribe() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    let qm = QueueManager::connect(Credentials::user("app", "app")).warn_as_error()?;
    let object = Object::open(&qm, (QUEUE, values::MQOO(sys::MQOO_INPUT_AS_Q_DEF))).warn_as_error()?;
    let (sub, obj) = Subscription::subscribe_managed(
        &qm,
        (MQSO(sys::MQSO_CREATE | sys::MQSO_NON_DURABLE), &object, ObjectString("dev/")),
    )
    .warn_as_error()?;

    println!("{sub:?}");
    println!("{obj:?}");
    println!("{object:?}");

    sub.close().warn_as_error()?;
    obj.close().warn_as_error()?;

    Ok(())
}
