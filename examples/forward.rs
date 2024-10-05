use std::str::FromStr;

mod args;

use anyhow::Context as _;
use clap::{Parser, ValueEnum};
use mqi::{
    connect_options::ApplName,
    prelude::*,
    put_options::{Context, PropertyAction},
    sys,
    types::{MessageFormat, QueueManagerName, QueueName},
    values::{MQCMHO, MQGMO, MQOO, MQPMO},
    MqStruct, Object, Properties, Syncpoint, ThreadNone,
};

const APP_NAME: ApplName = ApplName(mqstr!("forward"));

#[derive(Parser, Debug)]
struct Cli {
    #[command(flatten)]
    connection: args::ConnectionArgs,

    #[arg(short = 'x', long, value_enum, default_value_t=ContextArg::Default)]
    context: ContextArg,

    #[arg(short, long, default_value_t = false)]
    dry_run: bool,

    #[arg(short, long)]
    source_queue: String,

    #[arg(short, long)]
    queue: String,

    #[arg(short = 'm', long, requires("queue"))]
    queue_manager: Option<String>,
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

    let client_method = args.connection.method.connect_option()?;
    let qm_name = args
        .connection
        .queue_manager_name()
        .context("Connection queue manager name is invalid")?;
    let creds = args.connection.credentials();
    let cno = args.connection.cno().context("MQCNO option is not valid")?;

    let source_queue = QueueName::from_str(&args.source_queue)?;
    let target_queue = QueueName::from_str(&args.queue)?;
    let target_qm = args
        .queue_manager
        .as_deref()
        .map(QueueManagerName::from_str)
        .transpose()
        .context("Target queue manager name is invalid")?;

    // Connect to the queue manager using the supplied optional arguments. Fail on any warning.
    let qm = mqi::connect::<ThreadNone>((APP_NAME, qm_name, creds, cno, client_method))
        .warn_as_error()
        .context("Unable to connect to the queue manager")?;
    let qm_ref = qm.connection_ref();
    let obj = Object::open(
        qm_ref,
        (source_queue, MQOO(sys::MQOO_INPUT_AS_Q_DEF | sys::MQOO_SAVE_ALL_CONTEXT)),
    )
    .warn_as_error() // Fail on any warnings
    .context("Unable to open the object")?;

    let mut buffer: [u8; 20 * 1024] = [0; 20 * 1024]; // 20kb

    let syncpoint = Syncpoint::new(qm_ref);

    let mut properties = Properties::new(&qm, MQCMHO::default())?;
    let message = obj
        .get_data_with::<MqStruct<sys::MQMD2> /* MQMD2 */, _ /* buffer */>(
            (
                MQGMO(sys::MQGMO_SYNCPOINT), // Must use the syncpoint option
                &mut properties,             // Retrieve the message properties
            ),
            buffer.as_mut_slice(), // Provide a buffer for the message
        )
        .warn_as_error() // Fail on any warnings
        .context("Unable to get a messsage")?;

    if let Some((msg_data, md)) = message {
        let mut target_properties = Properties::new(&qm, MQCMHO::default())?; // Create a placeholder for target properties
        let fmt = MessageFormat::from_mqmd2(&md);
        qm_ref
            .put_message(
                // Equivalent to MQPUT1
                (
                    // Options used when opening the queue
                    MQPMO(sys::MQPMO_SYNCPOINT), // Syncpoint - final execution on commit.
                    MQPMO(match args.context {
                        ContextArg::Default => sys::MQPMO_DEFAULT_CONTEXT,
                        ContextArg::None => sys::MQPMO_NO_CONTEXT,
                        ContextArg::Identity => sys::MQPMO_PASS_IDENTITY_CONTEXT,
                        ContextArg::All => sys::MQPMO_PASS_ALL_CONTEXT,
                    }),
                    target_qm,    // Target queue manager
                    target_queue, // Target queue
                ),
                (
                    // Options used when putting to the queue
                    md,                                                           // Original MQMD2
                    Context(&obj),                                                // Source object as context
                    PropertyAction::Forward(&properties, &mut target_properties), // Forward the properties
                ),
                &(msg_data, fmt), // Set the message content and format
            )
            .warn_as_error() // Fail on any warnings
            .context("Unable to put a message")?;
    }

    if args.dry_run {
        syncpoint.backout().warn_as_error().context("Unable to backout")?; // Backout any changes
    } else {
        syncpoint.commit().warn_as_error().context("Unabel to commit")?; // Commit both the MQ get and MQ put.
    }

    Ok(())
}
