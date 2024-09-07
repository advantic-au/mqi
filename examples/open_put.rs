use std::{
    error::Error,
    io::{self, Read},
};

use clap::{Args, Parser};
use mqi::{
    connect_options::{ApplName, ClientDefinition, Credentials},
    core::values::{MQENC, MQOO},
    headers::TextEnc,
    mqstr,
    open_options::ObjectString,
    sys,
    types::{MessageFormat, ObjectName, QueueManagerName, QueueName},
    MqStr, Object, QueueManager, ResultCompExt as _,
};
use tracing::Level;

const APP_NAME: ApplName = ApplName(mqstr!("open_put"));

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    mqserver: Option<String>,
    #[arg(long)]
    queue_manager: Option<String>,
    #[arg(short, long)]
    username: Option<String>,
    #[arg(short, long)]
    password: Option<String>,
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

    // One of queue or topic will be supplied
    let (queue, topic) = (
        args.target
            .queue
            .as_deref()
            .map(ObjectName::try_from)
            .transpose()? // Option<Result> -> Result<Option>
            .map(QueueName),
        args.target.topic.as_deref().map(ObjectString),
    );
    // It will be either queue or topic but not both
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

    /* TODO: conversion from str -> TextEnc::Ascii is clunky */
    let fmt: MqStr<8> = (*args.format.unwrap_or_default()).try_into()?;
    let msg_fmt = MessageFormat {
        ccsid: 1208,
        encoding: MQENC::default(),
        fmt: TextEnc::Ascii(*fmt.as_bytes()),
    };

    let qm = QueueManager::connect(&(APP_NAME, qm_name, creds, client_definition)).warn_as_error()?;
    let object = Object::open(qm, (queue, topic), MQOO(sys::MQOO_OUTPUT)).warn_as_error()?;

    // Read the message from stdin
    let mut stdin = io::stdin();
    let mut message = Vec::new();
    stdin.read_to_end(&mut message)?;

    object.put_message((), &(message, msg_fmt)).warn_as_error()?;

    Ok(())
}
