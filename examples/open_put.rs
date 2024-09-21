use std::{
    error::Error,
    io::{self, Read},
    str::FromStr,
};
use clap::{Args, Parser};

mod args;

use mqi::{
    connect_options::ApplName,
    headers::TextEnc,
    mqstr,
    open_options::ObjectString,
    prelude::*,
    sys,
    types::{MessageFormat, QueueManagerName, QueueName},
    values::{CCSID, MQENC, MQOO, MQPMO},
    MqStr, Object, ThreadNone,
};
use tracing::Level;

const APP_NAME: ApplName = ApplName(mqstr!("open_put"));

#[derive(Parser, Debug)]
struct Cli {
    #[command(flatten)]
    connection: args::ConnectionArgs,

    #[arg(long)]
    format: Option<String>,

    #[arg(long)]
    oo: Vec<String>,

    #[arg(long)]
    pmo: Vec<String>,

    #[command(flatten)]
    target: Target,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = true)]
struct Target {
    #[arg(short, long, conflicts_with("queue"), conflicts_with("queue_manager"))]
    topic: Option<String>,

    #[arg(short, long)]
    queue: Option<String>,

    #[arg(short = 'm', long, requires("queue"))]
    queue_manager: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt().compact().with_max_level(Level::TRACE).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Cli::parse();

    let client_method = args.connection.method.connect_option()?;
    let qm_name = args.connection.queue_manager_name()?;
    let creds = args.connection.credentials();
    let cno = args.connection.cno()?;

    // It will be either queue or topic but not both
    let target_topic = args.target.topic.as_deref().map(ObjectString);
    let target_queue = args.target.queue.as_deref().map(QueueName::from_str).transpose()?;
    let target_qm = args
        .target
        .queue_manager
        .as_deref()
        .map(QueueManagerName::from_str)
        .transpose()?;

    // Additional MQOO options from the command line
    let mut oo = MQOO(sys::MQOO_OUTPUT);
    for o in &args.oo {
        oo |= MQOO::from_str(o)?;
    }

    // Additional MQPMO options from the command line
    let mut pmo = MQPMO(sys::MQPMO_NONE);
    for p in &args.pmo {
        pmo |= MQPMO::from_str(p)?;
    }

    /* TODO: conversion from str -> TextEnc::Ascii is clunky */
    let fmt: MqStr<8> = (*args.format.unwrap_or_default()).try_into()?;
    let msg_fmt = MessageFormat {
        ccsid: CCSID(1208),
        encoding: MQENC::default(),
        fmt: TextEnc::Ascii(*fmt.as_bytes()),
    };

    // Connect to the queue manager using the supplied optional arguments. Fail on any warning.
    let connection = mqi::connect::<ThreadNone>((APP_NAME, qm_name, creds, cno, client_method)).warn_as_error()?;

    // Open the queue or topic with MQOO_OUTPUT option
    let object = Object::open(connection, (target_queue, target_qm, target_topic, oo)).warn_as_error()?;

    // Read the message from stdin
    let mut stdin = io::stdin();
    let mut message = Vec::new();
    stdin.read_to_end(&mut message)?;

    // Put a message to the object from the data from stdin
    object.put_message(pmo, &(message, msg_fmt)).warn_as_error()?;

    Ok(())
}
