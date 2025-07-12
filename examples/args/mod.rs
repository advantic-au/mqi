use std::str::FromStr;

use clap::Args;
use mqi::{
    MqStr,
    connection::{Binding, Ccdt, ConnectOption, Credentials, MqServer},
    constants,
    types::{CertificateLabel, CipherSpec, KeyRepo, MQCNO, QueueManagerName},
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

    #[arg(short, long)]
    tls_key_repo: Option<String>,
    #[arg(short, long, requires("tls_key_repo"))]
    tls_cipher_spec: Option<String>,
    #[arg(short, long, requires("tls_key_repo"))]
    cert_label: Option<String>,
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
    pub fn cno(&self) -> Result<MQCNO, std::num::ParseIntError> {
        let mut cno_all = constants::MQCNO_NONE;
        for cno in &self.cno {
            cno_all.insert(MQCNO::from_str(cno)?);
        }
        Ok(cno_all)
    }

    pub fn queue_manager_name(&self) -> Result<Option<QueueManagerName>, mqi::MqStrError> {
        self.connect_queue_manager
            .as_deref()
            .map(QueueManagerName::from_str) // Convert to QueueManagerName which has 48 character length
            .transpose() // Option<Result> -> Result<Option>
    }

    pub fn credentials(&self) -> Option<Credentials<'_, &str>> {
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
    ) -> Result<Option<(KeyRepo, CipherSpec, Option<CertificateLabel>)>, mqi::MqStrError> {
        let cipher = self
            .tls_cipher_spec
            .as_ref()
            .map(|cipher_arg| Ok(CipherSpec(MqStr::from_str(cipher_arg)?)))
            .transpose()?
            .unwrap_or(*default_cipher);

        let label = self
            .cert_label
            .as_ref()
            .map(|label_arg| Ok(CertificateLabel(MqStr::from_str(label_arg)?)))
            .transpose()?;

        self.tls_key_repo
            .as_ref()
            .map(|repo_arg| Ok((KeyRepo(MqStr::from_str(repo_arg)?), cipher, label)))
            .transpose()
    }
}
