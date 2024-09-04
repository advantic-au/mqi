use std::{
    borrow::Cow,
    env,
    error::Error,
    sync::{atomic, Arc},
};

use clap::Parser;
use mqi::{
    connect_options::{ClientDefinition, Credentials},
    core::values::MQSO,
    get::GetWait,
    open_options::ObjectString,
    sys,
    types::{MessageFormat, ObjectName, QueueManagerName},
    Object, QueueManager, ResultCompExt, Subscription,
};
use tracing::Level;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    mqserver: Option<String>,

    #[arg(short, long)]
    queue_manager: Option<String>,

    #[arg(short, long)]
    username: Option<String>,

    #[arg(short, long)]
    password: Option<String>,

    #[arg(short, long)]
    topic: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt().compact().with_max_level(Level::DEBUG).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Interrupt handler to stop the MQGET loop
    let running = Arc::new(atomic::AtomicBool::new(true));
    let running_check = running.clone();
    ctrlc::set_handler(move || running.store(false, atomic::Ordering::Relaxed))?;

    let args = Args::parse();
    
    let topic_str = args.topic.or_else(|| env::var("TOPIC").ok());
    let topic = topic_str.as_deref().map(ObjectString);

    let cd = args
        .mqserver
        .or_else(|| env::var("MQSERVER").ok())
        .map(|server| ClientDefinition::from_mqserver(&server))
        .transpose()?;

    let qm_name = args
        .queue_manager
        .map(|name| ObjectName::try_from(&*name))
        .transpose()?
        .map(QueueManagerName);

    let creds = if args.username.is_some() | args.password.is_some() {
        Some(Credentials::user(
            args.username.as_deref().unwrap_or(""),
            args.password.as_deref().unwrap_or(""),
        ))
    } else {
        None
    };

    let qm: QueueManager<_> = QueueManager::connect(qm_name.as_ref(), &(creds, cd)).warn_as_error()?;
    let (_subscription, queue) = Subscription::subscribe::<(Subscription<_>, Option<Object<_>>)>(
        &qm,
        (MQSO(sys::MQSO_CREATE | sys::MQSO_NON_DURABLE | sys::MQSO_MANAGED), topic),
    )
    .warn_as_error()?;
    let queue = queue.expect("Subscription queue always returned when one is not provided");

    let mut buffer: [u8; 20 * 1024] = [0; 20 * 1024]; // 20kb

    while running_check.load(atomic::Ordering::Relaxed) {
        if let Some((_data, _format)) = queue
            .get_message::<(Cow<[u8]>, MessageFormat), _>(GetWait::Wait(500), buffer.as_mut_slice())
            .warn_as_error()? { }
    }

    Ok(())
}
