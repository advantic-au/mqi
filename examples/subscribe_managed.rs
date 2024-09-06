use std::{
    env,
    error::Error,
    sync::{atomic, Arc},
};

use clap::Parser;
use mqi::{
    connect_options::{ApplName, ClientDefinition, Credentials},
    core::values::MQSO,
    get::GetWait,
    mqstr,
    open_options::ObjectString,
    sys,
    types::{MessageFormat, ObjectName, QueueManagerName},
    QueueManager, ResultCompExt as _, Subscription,
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
    topic: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    const APP_NAME: ApplName = ApplName(mqstr!("subscribe_managed"));

    let subscriber = tracing_subscriber::fmt().compact().with_max_level(Level::TRACE).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Interrupt handler to stop the MQGET loop
    let running = Arc::new(atomic::AtomicBool::new(true));
    let running_check = running.clone();
    ctrlc::set_handler(move || running.store(false, atomic::Ordering::Relaxed))?;

    let args = Args::parse();

    let topic_str = args.topic.or_else(|| env::var("TOPIC").ok());
    let topic = topic_str.as_deref().map(ObjectString);

    let client_definition = args
        .mqserver
        .map(|server| ClientDefinition::from_mqserver(&server))
        .transpose()?; // Option<Result> -> Result<Option>

    let qm_name = args
        .queue_manager
        .map(|name| ObjectName::try_from(&*name)) // Convert to ObjectName which has 48 character length
        .transpose()? // Option<Result> -> Result<Option>
        .map(QueueManagerName);

    let creds = if args.username.is_some() | args.password.is_some() {
        Some(Credentials::user(
            args.username.as_deref().unwrap_or(""),
            args.password.as_deref().unwrap_or(""),
        ))
    } else {
        None
    };

    let qm = QueueManager::connect(&(APP_NAME, qm_name, creds, client_definition)).warn_as_error()?;
    let (_subscription, queue) =
        Subscription::subscribe_managed(&qm, (MQSO(sys::MQSO_CREATE | sys::MQSO_NON_DURABLE), topic)).warn_as_error()?;

    let mut buffer: [u8; 20 * 1024] = [0; 20 * 1024]; // 20kb

    while running_check.load(atomic::Ordering::Relaxed) {
        if let Some((_data, _format)) = queue
            .get_data_with::<MessageFormat, _>(GetWait::Wait(500), buffer.as_mut_slice())
            .warn_as_error()?
        {
            // TODO: demonstrate some simple message handling
        }
    }

    Ok(())
}
