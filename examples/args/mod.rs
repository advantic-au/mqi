use anyhow::Context;
use clap::Args;
use mqi::{
    connection::{Binding, Ccdt, ConnectOption, Credentials, MqServer},
    string,
    types::{CertificateLabel, CipherSpec, KeyRepo, MQCNO, QueueManagerName},
};

#[derive(clap::Parser, Debug)]
pub struct ConnectionArgs {
    #[command(flatten)]
    pub method: MethodArgs,

    #[arg(short, long)]
    cno: Vec<MQCNO>,

    #[arg(long)]
    connect_queue_manager: Option<QueueManagerName>,
    #[arg(short, long)]
    username: Option<String>,
    #[arg(short, long, requires("username"))]
    password: Option<String>,

    #[arg(short = 'k', long)]
    tls_key_repo: Option<KeyRepo>,
    #[arg(short = 's', long, requires("tls_key_repo"))]
    tls_cipher_spec: Option<CipherSpec>,
    #[arg(short = 'l', long, requires("tls_key_repo"))]
    cert_label: Option<CertificateLabel>,
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
    pub fn connect_option(&self) -> anyhow::Result<impl ConnectOption<'_>> {
        Ok((
            self.mqserver.as_deref().map(MqServer::try_from).transpose()?,
            self.ccdt.as_deref().map(Ccdt),
            if self.local { Binding::Local } else { Binding::Default },
        ))
    }
}

impl ConnectionArgs {
    fn cno(&self) -> MQCNO {
        self.cno.iter().copied().collect()
    }

    const fn queue_manager_name(&self) -> Option<QueueManagerName> {
        self.connect_queue_manager
    }

    fn credentials(&self) -> Option<Credentials<'_, &str>> {
        if self.username.is_some() | self.password.is_some() {
            Some(Credentials::User(
                self.username.as_deref().unwrap_or(""),
                self.password.as_deref().unwrap_or("").into(),
            ))
        } else {
            None
        }
    }

    pub fn tls(
        &self,
        default_cipher: &CipherSpec,
    ) -> Result<Option<(KeyRepo, CipherSpec, Option<CertificateLabel>)>, string::MqStrError> {
        let cipher = self
            .tls_cipher_spec
            .as_ref()
            .map(|cipher_arg| Ok(*cipher_arg))
            .transpose()?
            .unwrap_or(*default_cipher);

        let label = self.cert_label.as_ref().map(|label_arg| Ok(*label_arg)).transpose()?;

        self.tls_key_repo
            .as_ref()
            .map(|repo_arg| Ok((*repo_arg, cipher, label)))
            .transpose()
    }

    pub fn connection_option(&self) -> anyhow::Result<impl ConnectOption<'_>> {
        let creds = self.credentials();
        let qm = self.queue_manager_name().context("Queue Manager argument")?;
        let cno = self.cno();

        anyhow::Ok((self.method.connect_option()?, cno, creds, qm))
    }
}
