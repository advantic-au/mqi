use std::{
    error::Error,
    io::{self, Read},
};
use clap::{Args, Parser};

mod args;

use mqi::{
    prelude::*,
    connect_options::ApplName,
    core::values::{MQENC, MQOO},
    headers::TextEnc,
    mqstr,
    open_options::ObjectString,
    sys,
    types::{MessageFormat, ObjectName, QueueName},
    MqStr, Object, QueueManager,
};
use tracing::Level;

const APP_NAME: ApplName = ApplName(mqstr!("open_put"));

#[derive(Parser, Debug)]
struct Cli {
    #[command(flatten)]
    connection: args::ConnectionArgs,

    #[arg(long)]
    format: Option<String>,

    #[command(flatten)]
    target: Target,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct Target {
    #[arg(short, long)]
    topic: Option<String>,
    #[arg(short, long)]
    queue: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt().compact().with_max_level(Level::DEBUG).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Cli::parse();

    let client_definition = args.connection.client_definition()?;
    let qm_name = args.connection.queue_manager_name()?;
    let creds = args.connection.credentials();

    // It will be either queue or topic but not both
    let (queue, topic) = (
        args.target
            .queue
            .as_deref()
            .map(ObjectName::try_from)
            .transpose()? // Option<Result> -> Result<Option>
            .map(QueueName),
        args.target.topic.as_deref().map(ObjectString),
    );

    /* TODO: conversion from str -> TextEnc::Ascii is clunky */
    let fmt: MqStr<8> = (*args.format.unwrap_or_default()).try_into()?;
    let msg_fmt = MessageFormat {
        ccsid: 1208,
        encoding: MQENC::default(),
        fmt: TextEnc::Ascii(*fmt.as_bytes()),
    };

    // Connect to the queue manager using the supplied optional arguments. Fail on any warning.
    let qm = QueueManager::connect((APP_NAME, qm_name, creds, client_definition)).warn_as_error()?;

    // Open the queue or topic with MQOO_OUTPUT option
    let object = Object::open(qm, (queue, topic, MQOO(sys::MQOO_OUTPUT))).warn_as_error()?;

    // Read the message from stdin
    let mut stdin = io::stdin();
    let mut message = Vec::new();
    stdin.read_to_end(&mut message)?;

    // Put a message to the object from the data from stdin
    object.put_message((), &(message, msg_fmt)).warn_as_error()?;

    Ok(())
}
