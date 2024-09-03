use std::{env, error::Error};

use clap::{ArgGroup, Parser};
use mqi::{
    connect_options::{Binding, ClientDefinition, Credentials},
    types::{ObjectName, QueueManagerName},
    QueueManager, ResultCompExt,
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    mqserver: Option<String>,

    #[arg(short, long)]
    queue_manager: Option<String>,

    #[arg(short, long)]
    username: Option<String>,

    #[arg(short, long)]
    password: Option<String>
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let cd = args
        .mqserver
        .or_else(|| env::var("MQSERVER").ok())
        .map(|server| ClientDefinition::from_mqserver(&server))
        .transpose()?;

    let qm_name = args
        .queue_manager
        .map(|name| ObjectName::try_from(&*name))
        .transpose()?
        .map(QueueManagerName);

    let creds = if args.username.is_some() | args.password.is_some() {
        Some(Credentials::user(args.username.as_deref().unwrap_or(""), args.password.as_deref().unwrap_or("")))
    }
    else {
        None
    };

    let qm: QueueManager<_> = QueueManager::connect(qm_name.as_ref(), &(creds, cd, Binding::Local)).warn_as_error()?;

    Ok(())
}
