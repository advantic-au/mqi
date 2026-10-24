use std::{
    borrow::Cow,
    io::{self, Read as _},
};

use anyhow::Context as _;
use clap::Parser;
use mqi::{
    Object,
    connection::{ThreadNone, Tls},
    constants,
    get::GetWait,
    mqstr,
    prelude::*,
    types::{ApplName, CipherSpec, QueueManagerName, QueueName, ReplyToQueueName},
};

const APP_NAME: ApplName = ApplName(mqstr!("request_reply"));
const DEFAULT_CIPHER: CipherSpec = CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256")); // TLS 1.3 cipher
const DEFAULT_MAX_MSG: usize = 1024 * 1024; // 1Mb

mod args;

#[derive(Parser, Debug)]
struct Args {
    #[command(flatten)]
    connection: args::ConnectionArgs,

    #[arg(short, long)]
    queue: QueueName,

    #[arg(short = 'm', long, requires("queue"))]
    queue_manager: Option<QueueManagerName>,

    #[arg(short, long)]
    reply_to_queue: QueueName,
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

    // let reply_queue = QueueName(MqStr::from_str(&args.reply_to_queue).context("Parse reply queue name")?);

    // Open the reply to queue. If this is a model queue, then a temporary queue is created.
    let (reply_object, resolved_queue_name) =
        Object::open_with::<Option<QueueName>>(&qm, &(args.reply_to_queue, constants::MQOO_INPUT_AS_Q_DEF))
            .warn_as_error()
            .context("Opening the reply to queue")?;
    let reply_queue_name = ReplyToQueueName(*resolved_queue_name.context("Resolved queue name")?);

    // Read the message from stdin
    let mut stdin = io::stdin();
    let mut message = Vec::new();
    stdin.read_to_end(&mut message)?;

    qm.put_message(
        &(args.queue, args.queue_manager),
        &(reply_queue_name),
        &*String::from_utf8(message).context("Input not valid UTF-8")?,
    )
    .warn_as_error()?;

    let mut buffer = vec![0u8; DEFAULT_MAX_MSG];

    if let Some(response) = reply_object
        .get_string(&(GetWait::Wait(5000)), &mut *buffer)
        .warn_as_error()?
    {
        let response: Cow<str> = response.try_into().context("Response string")?;
        print!("{response}");
    } else {
        eprintln!("No response message received");
    }

    anyhow::Ok(())
}
