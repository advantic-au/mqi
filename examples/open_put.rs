use std::io::{self, Read};

use anyhow::Context as _;
use clap::{Args, Parser};

mod args;

use mqi::{
    MqStr, Object,
    connection::{ThreadNone, Tls},
    constants,
    header::fmt::MQFMT_STRING,
    open::ObjectString,
    prelude::*,
    types::{ApplName, CipherSpec, MQOO, MQPMO, MessageFormat, QueueManagerName, QueueName},
};
use tracing::Level;

const APP_NAME: ApplName = ApplName(mqstr!("open_put"));
const DEFAULT_CIPHER: CipherSpec = CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256")); // TLS 1.3 cipher

#[derive(Parser, Debug)]
struct Cli {
    #[command(flatten)]
    connection: args::ConnectionArgs,

    #[arg(long)]
    format: Option<MessageFormat>,

    #[arg(long)]
    oo: Vec<MQOO>,

    #[arg(long)]
    pmo: Vec<MQPMO>,

    #[command(flatten)]
    target: Target,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = true)]
struct Target {
    #[arg(short, long, conflicts_with("queue"), conflicts_with("queue_manager"))]
    topic: Option<String>,

    #[arg(short, long)]
    queue: Option<QueueName>,

    #[arg(short = 'm', long, requires("queue"))]
    queue_manager: Option<QueueManagerName>,
}

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt().compact().with_max_level(Level::TRACE).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Cli::parse();

    let connection_option = args.connection.connection_option()?;

    // Set up the tls connection parameters from the arguments
    let tls = args.connection.tls(&DEFAULT_CIPHER).context("TLS options are not valid")?;
    let tls_connect = tls
        .as_ref()
        .map(|(repo, cipher, label)| Tls::new(repo, label.as_ref(), cipher));

    // It will be either queue or topic but not both
    let target_topic = args.target.topic.as_deref().map(ObjectString);

    // Additional MQOO options from the command line
    let oo = constants::MQOO_OUTPUT | args.oo.into_iter().collect();

    // Additional MQPMO options from the command line
    let pmo: MQPMO = args.pmo.into_iter().collect();

    let msg_fmt = args.format.unwrap_or_else(|| MqStr::from(MQFMT_STRING).into());

    // Connect to the queue manager using the supplied optional arguments. Fail on any warning.
    let qm = mqi::connect::<ThreadNone>(&(APP_NAME, tls_connect, connection_option))
        .warn_as_error()
        .context("Unable to connect to the queue manager")?;

    // Open the queue or topic with MQOO_OUTPUT option
    let object = Object::open(qm, &(args.target.queue, args.target.queue_manager, target_topic, oo))
        .warn_as_error()
        .context("Unable to open the object")?;

    // Read the message from stdin
    let mut stdin = io::stdin();
    let mut message = Vec::new();
    stdin.read_to_end(&mut message)?;

    // Put a message to the object from the data from stdin
    object
        .put_message(&pmo, &(message, msg_fmt))
        .warn_as_error()
        .context("Unable to put the message")?;

    Ok(())
}
