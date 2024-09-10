use mqi::{connect_options::{ClientDefinition, Credentials, MqServerSyntaxError}, types::{ObjectName, QueueManagerName}};


#[derive(clap::Parser, Debug)]
pub struct ConnectionArgs {
    #[arg(long)]
    pub mqserver: Option<String>,
    #[arg(long)]
    pub queue_manager: Option<String>,
    #[arg(short, long)]
    pub username: Option<String>,
    #[arg(short, long)]
    pub password: Option<String>,
}

impl ConnectionArgs {
    pub fn client_definition(&self) -> Result<Option<ClientDefinition>, MqServerSyntaxError> {
        self
        .mqserver
        .as_deref()
        .map(ClientDefinition::from_mqserver)
        .transpose() // Option<Result> -> Result<Option>
    }

    pub fn queue_manager_name(&self) -> Result<Option<QueueManagerName>, mqi::MQStrError> {
        self
        .queue_manager
        .as_deref()
        .map(ObjectName::try_from) // Convert to ObjectName which has 48 character length
        .transpose() // Option<Result> -> Result<Option>
        .map(|m | m.map(QueueManagerName))
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
