use clap::Args;
use mqi::{
    connect_options::{Ccdt, ClientDefinition, Credentials, MqServerSyntaxError},
    types::{ObjectName, QueueManagerName},
};

#[derive(clap::Parser, Debug)]
pub struct ConnectionArgs {
    #[command(flatten)]
    method: MethodArgs,

    #[arg(long)]
    pub queue_manager: Option<String>,
    #[arg(short, long)]
    pub username: Option<String>,
    #[arg(short, long)]
    pub password: Option<String>,
}

#[derive(Args, Debug)]
#[group(required = false, multiple = false)]
pub struct MethodArgs {
    #[arg(long)]
    pub mqserver: Option<String>,

    #[arg(long)]
    pub ccdt: Option<String>,
}

impl ConnectionArgs {
    pub fn client_method(&self) -> Result<(Option<ClientDefinition>, Option<Ccdt>), MqServerSyntaxError> {
        Ok((
            self.method
                .mqserver
                .as_deref()
                .map(ClientDefinition::from_mqserver)
                .transpose()?,
            self.method.ccdt.as_deref().map(Ccdt),
        ))
    }

    pub fn queue_manager_name(&self) -> Result<Option<QueueManagerName>, mqi::MQStrError> {
        self.queue_manager
            .as_deref()
            .map(ObjectName::try_from) // Convert to ObjectName which has 48 character length
            .transpose() // Option<Result> -> Result<Option>
            .map(|m| m.map(QueueManagerName))
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
