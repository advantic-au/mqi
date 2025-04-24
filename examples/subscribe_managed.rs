use std::{
    env,
    sync::{atomic, Arc},
};

mod args;

use anyhow::Context as _;
use clap::Parser;
use mqi::{
    connect_options::ApplName, get::GetWait, open_options::ObjectString, prelude::*, types::MessageFormat, constants, ThreadNone,
    Subscription,
};
use tracing::Level;

#[derive(Parser, Debug)]
struct Args {
    #[command(flatten)]
    connection: args::ConnectionArgs,

    #[arg(short, long)]
    topic: Option<String>,
}

const APP_NAME: ApplName = ApplName(mqstr!("subscribe_managed"));

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt().compact().with_max_level(Level::TRACE).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    let topic_str = args.topic.or_else(|| env::var("TOPIC").ok());
    let topic = topic_str.as_deref().map(ObjectString);

    let client_method = args.connection.method.connect_option()?;
    let qm_name = args
        .connection
        .queue_manager_name()
        .context("Connection queue manager name is invalid")?;
    let creds = args.connection.credentials();
    let cno = args.connection.cno().context("MQCNO options are invalid")?;

    // Connect to the queue manager using the supplied optional arguments. Fail on any warning.
    let qm = mqi::connect::<ThreadNone>(&(APP_NAME, qm_name, creds, cno, client_method))
        .warn_as_error()
        .context("Unable to connect to the queue manager")?;

    // Create a managed, non-durable subscription to the topic. Fail on any warning.
    // The subscription will persist until `_subscription` is descoped.
    let (_subscription, queue) = Subscription::subscribe_managed(
        qm.connection_ref(),
        (constants::MQSO_CREATE | constants::MQSO_NON_DURABLE, topic),
    )
    .warn_as_error()
    .context("Unable to subscribe to topic")?;

    let mut buffer = vec![0u8; 20 * 1024].into_boxed_slice(); // 20kb

    // Interrupt handler to stop the MQGET loop
    let running = Arc::new(atomic::AtomicBool::new(true));
    let running_check = running.clone();
    ctrlc::set_handler(move || running.store(false, atomic::Ordering::Relaxed))?;

    while running_check.load(atomic::Ordering::Relaxed) {
        let message: Option<(_, MessageFormat)> = queue
            .get_data_with(&GetWait::Wait(500), &mut buffer)
            .warn_as_error()
            .context("Unable to get message")?;

        if let Some((data, _format)) = message {
            println!("{data:?}");
            // TODO: demonstrate some simple message handling
        }
    }

    Ok(())
}
