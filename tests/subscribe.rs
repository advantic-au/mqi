use mqi::{
    connect_options::Credentials, core::values::MQSO, mqstr, open_options::ObjectString, sys, types::QueueName, MqMask, Object,
    QueueManager, ResultCompExt as _, Subscription,
};

#[test]
fn subscribe() -> Result<(), Box<dyn std::error::Error>> {
    const QUEUE: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    let qm: QueueManager<_> = QueueManager::connect(None, &Credentials::user("app", "app")).warn_as_error()?;

    let object = Object::open::<Object<_>>(&qm, QUEUE, MqMask::from(sys::MQOO_INPUT_AS_Q_DEF)).warn_as_error()?;

    let (sub, obj) = Subscription::subscribe::<(Subscription<_>, Option<Object<_>>)>(
        &qm,
        (
            MqMask::<MQSO>::from(sys::MQSO_CREATE | sys::MQSO_NON_DURABLE),
            &object,
            ObjectString("dev/"),
        ),
    )
    .warn_as_error()?;

    println!("{sub:?}");
    println!("{obj:?}");
    println!("{object:?}");

    sub.close().warn_as_error()?;
    if let Some(obj) = obj {
        obj.close().warn_as_error()?;
    }

    Ok(())
}
