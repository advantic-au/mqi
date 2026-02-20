mod args;

use anyhow::Context as _;
use clap::{Parser, ValueEnum};
use mqi::{
    Object, Properties, Syncpoint,
    connection::{ThreadNone, Tls},
    constants,
    prelude::*,
    put::{Context, PropertyAction},
    structs,
    types::{ApplName, CipherSpec, MQCMHO, MessageFormat, QueueManagerName, QueueName},
};

const APP_NAME: ApplName = ApplName(mqstr!("forward"));
const DEFAULT_CIPHER: CipherSpec = CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256")); // TLS 1.3 cipher

#[derive(Parser, Debug)]
struct Cli {
    #[command(flatten)]
    connection: args::ConnectionArgs,

    #[arg(short = 'x', long, value_enum, default_value_t=ContextArg::Default)]
    context: ContextArg,

    #[arg(short, long, default_value_t = false)]
    dry_run: bool,

    #[arg(short, long)]
    source_queue: QueueName,

    #[arg(short, long)]
    queue: QueueName,

    #[arg(short = 'm', long, requires("queue"))]
    queue_manager: Option<QueueManagerName>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ContextArg {
    #[default]
    Default,
    None,
    Identity,
    All,
}

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Cli::parse();

    let connection_options = args.connection.connection_option()?;

    // Set up the tls connection parameters from the arguments
    let tls = args.connection.tls(&DEFAULT_CIPHER).context("TLS options are not valid")?;
    let tls_connect = tls
        .as_ref()
        .map(|(repo, cipher, label)| Tls::new(repo, label.as_ref(), cipher));

    // Connect to the queue manager using the supplied optional arguments. Fail on any warning.
    let qm = mqi::connect::<ThreadNone>(&(APP_NAME, tls_connect, connection_options))
        .already_connected_ref()
        .discard_warning()
        .context("Unable to connect to the queue manager")?;
    let qm_ref = qm.connection_ref();
    let obj = Object::open(
        qm_ref.clone(),
        &(
            args.source_queue,
            constants::MQOO_INPUT_AS_Q_DEF | constants::MQOO_SAVE_ALL_CONTEXT,
        ),
    )
    .warn_as_error() // Fail on any warnings
    .context("Unable to open the object")?;

    let mut buffer = Vec::<u8>::with_capacity(20 * 1024); // 20kb
    let buf_write = buffer.spare_capacity_mut();
    let syncpoint = Syncpoint::new(qm_ref.clone());

    let mut properties = Properties::new(&qm, MQCMHO::default())?;
    let message: Option<(_, structs::MQMD)> = obj
        .get_data_with(
            &(
                constants::MQGMO_SYNCPOINT, // Must use the syncpoint option
                &mut properties,            // Retrieve the message properties
            ),
            buf_write, // Provide a buffer for the message
        )
        .warn_as_error() // Fail on any warnings
        .context("Unable to get a messsage")?;

    if let Some((msg_data, md)) = message {
        let len = msg_data.len();
        unsafe {
            buffer.set_len(len);
        }
        let mut target_properties = Properties::new(&qm, MQCMHO::default())?; // Create a placeholder for target properties
        let fmt = MessageFormat::from_mqmd(&md);
        qm_ref
            .put_message(
                // Equivalent to MQPUT1
                &(
                    // Options used when opening the queue
                    constants::MQPMO_SYNCPOINT, // Syncpoint - final execution on commit.
                    match args.context {
                        ContextArg::Default => constants::MQPMO_DEFAULT_CONTEXT,
                        ContextArg::None => constants::MQPMO_NO_CONTEXT,
                        ContextArg::Identity => constants::MQPMO_PASS_IDENTITY_CONTEXT,
                        ContextArg::All => constants::MQPMO_PASS_ALL_CONTEXT,
                    },
                    args.queue_manager, // Target queue manager
                    args.queue,         // Target queue
                ),
                &(
                    // Options used when putting to the queue
                    md,                                                           // Original MQMD
                    Context(&obj),                                                // Source object as context
                    PropertyAction::Forward(&properties, &mut target_properties), // Forward the properties
                ),
                &(buffer, fmt), // Set the message content and format
            )
            .warn_as_error() // Fail on any warnings
            .context("Unable to put a message")?;
    }

    let _ = if args.dry_run {
        syncpoint.backout().warn_as_error().context("Unable to backout") // Backout any changes
    } else {
        syncpoint.commit().warn_as_error().context("Unabel to commit") // Commit both the MQ get and MQ put.
    }?;

    Ok(())
}
