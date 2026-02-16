use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::Context as _;
use clap::Parser;
use mqi::{
    MqStr, Object, Properties,
    connection::{ThreadNone, Tls},
    constants,
    get::GetWait,
    mqstr,
    prelude::*,
    put::PropertyAction,
    structs,
    types::{ApplName, CipherSpec, MessageFormat, QueueManagerName, QueueName},
};

mod args;

const APP_NAME: ApplName = ApplName(mqstr!("open_put"));
const DEFAULT_CIPHER: CipherSpec = CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256")); // TLS 1.3 cipher
const DEFAULT_MAX_MSG: usize = 1024 * 1024; // 1Mb

#[derive(clap::Parser, Debug)]
struct Args {
    #[command(flatten)]
    connection: args::ConnectionArgs,

    #[arg(short, long)]
    queue: QueueName,
}

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    let connection_option = args.connection.connection_option()?;

    // Set up the tls connection parameters from the arguments
    let tls = args.connection.tls(&DEFAULT_CIPHER).context("TLS options are not valid")?;
    let tls_connect = tls
        .as_ref()
        .map(|(repo, cipher, label)| Tls::new(repo, label.as_ref(), cipher));

    // Connect to the queue manager using the supplied optional arguments. Fail on any warning.
    let qm = mqi::connect::<ThreadNone>(&(APP_NAME, tls_connect, connection_option))
        .warn_as_error()
        .context("Unable to connect to the queue manager")?;

    let queue = Object::open(&qm, &(args.queue, constants::MQOO_INPUT_AS_Q_DEF))
        .warn_as_error()
        .context("Open Queue")?;

    // Interrupt handler to stop the MQGET loop
    let running = Arc::new(AtomicBool::new(true));
    let running_check = running.clone();
    ctrlc::set_handler(move || running.store(false, Ordering::Relaxed))?;

    let mut buffer = vec![0; DEFAULT_MAX_MSG];

    let mut reply_properties = Properties::new(&qm, constants::MQCMHO_NO_VALIDATION)?;
    let mut properties = Properties::new(&qm, constants::MQCMHO_NO_VALIDATION)?;

    // Continually poll the queue until a break signal is received
    while running_check.load(Ordering::Relaxed) {
        // Get the data with the MQMD and message format from the queue
        let message: Option<(_, (structs::MQMD, MessageFormat))> = queue
            .get_data_with(&(GetWait::Wait(500), &mut properties), &mut buffer)
            .warn_as_error()
            .context("Get message")?;

        if let Some((message, (mut mqmd, mf))) = message {
            let target_queue = QueueName(mqmd.ReplyToQ.into());
            let target_queue_manager = QueueManagerName(mqmd.ReplyToQMgr.into());
            // Set the MQMD fields to be suitable for reply
            *mqmd.ReplyToQ.as_mut() = MqStr::empty();
            *mqmd.ReplyToQMgr.as_mut() = MqStr::empty();
            mqmd.CorrelId = mqmd.MsgId;
            *mqmd.MsgType.as_mut() = constants::MQMT_REPLY;
            // Perform the PUT
            qm.put_message(
                &(target_queue, target_queue_manager),
                &(
                    constants::MQPMO_NEW_MSG_ID,
                    mqmd,
                    PropertyAction::Reply(&properties, &mut reply_properties),
                ),
                &(message, mf),
            )
            .warn_as_error()
            .context("Put reply")?;
        }
    }

    anyhow::Ok(())
}
