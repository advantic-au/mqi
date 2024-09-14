use std::str::FromStr;

use clap::Args;
use mqi::{
    connect_options::{Binding, Ccdt, ClientDefinition, ConnectOption, Credentials},
    core::values,
    sys,
    types::QueueManagerName,
};

#[derive(clap::Parser, Debug)]
pub struct ConnectionArgs {
    #[command(flatten)]
    pub method: MethodArgs,

    #[arg(short, long)]
    cno: Vec<String>,

    #[arg(long)]
    connect_queue_manager: Option<String>,
    #[arg(short, long)]
    username: Option<String>,
    #[arg(short, long, requires("username"))]
    password: Option<String>,
}

#[derive(Args, Debug)]
#[group(required = false, multiple = false)]
pub struct MethodArgs {
    #[arg(long)]
    mqserver: Option<String>,

    #[arg(long)]
    ccdt: Option<String>,

    #[arg(long)]
    local: bool,
}

impl MethodArgs {
    pub fn connect_option(&self) -> anyhow::Result<impl ConnectOption> {
        Ok((
            self.mqserver.as_deref().map(ClientDefinition::from_mqserver).transpose()?,
            self.ccdt.as_deref().map(Ccdt),
            if self.local { Binding::Local } else { Binding::Default },
        ))
    }
}

impl ConnectionArgs {
    pub fn cno(&self) -> Result<values::MQCNO, std::num::ParseIntError> {
        let mut cno_all = values::MQCNO(sys::MQCNO_NONE);
        for cno in &self.cno {
            cno_all |= values::MQCNO::from_str(cno)?;
        }
        Ok(cno_all)
    }

    pub fn queue_manager_name(&self) -> Result<Option<QueueManagerName>, mqi::MQStrError> {
        self.connect_queue_manager
            .as_deref()
            .map(QueueManagerName::from_str) // Convert to QueueManagerName which has 48 character length
            .transpose() // Option<Result> -> Result<Option>
    }

    pub fn credentials(&self) -> Option<Credentials<&str>> {
        if self.username.is_some() | self.password.is_some() {
            Some(Credentials::user(
                self.username.as_deref().unwrap_or(""),
                self.password.as_deref().unwrap_or(""),
            ))
        } else {
            None
        }
    }
}
