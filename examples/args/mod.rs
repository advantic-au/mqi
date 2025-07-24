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

#[cfg(test)]
mod tests {
    use super::*;
    use mqi::{connection::ConnectStructs, constants, mqstr, traits::Secret};

    fn method_default() -> MethodArgs {
        MethodArgs {
            mqserver: None,
            ccdt: None,
            local: false,
        }
    }

    fn args_default() -> ConnectionArgs {
        ConnectionArgs {
            method: method_default(),
            cno: Vec::new(),
            connect_queue_manager: Some(QueueManagerName(mqstr!("QM1"))),
            username: None,
            password: None,
            tls_key_repo: None,
            tls_cipher_spec: None,
            cert_label: None,
        }
    }

    #[test]
    fn method_connect_option_local() {
        let method = MethodArgs {
            local: true,
            ..method_default()
        };
        let mut structs = ConnectStructs::default();
        let _ = method.connect_option().unwrap().apply_param(&mut structs);
        let options: &mut MQCNO = structs.cno.Options.as_mut();
        assert!(options.contains(constants::MQCNO_LOCAL_BINDING));
    }

    #[test]
    fn method_connect_option_mqserver() {
        let method = MethodArgs {
            mqserver: Some("QM1/TCP/localhost(1414)".into()),
            ..method_default()
        };
        let mut structs = ConnectStructs::default();
        let _ = method.connect_option().unwrap().apply_param(&mut structs);
        let options: &mut MQCNO = structs.cno.Options.as_mut();
        assert!(options.contains(constants::MQCNO_CLIENT_BINDING));
    }

    #[test]
    fn method_connect_option_ccdt() {
        let method = MethodArgs {
            ccdt: Some("file:/path/to/ccdt.tab".into()),
            ..method_default()
        };
        let mut structs = ConnectStructs::default();
        let _ = method.connect_option().unwrap().apply_param(&mut structs);
        let options: &mut MQCNO = structs.cno.Options.as_mut();
        assert!(options.contains(constants::MQCNO_CLIENT_BINDING));
    }

    #[test]
    fn method_connect_option_invalid_server() {
        let method = MethodArgs {
            mqserver: Some("invalid".into()),
            ..method_default()
        };
        assert!(method.connect_option().is_err());
    }

    #[test]
    fn tls_none() {
        let args = args_default();
        let default_cipher = CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256"));
        assert_eq!(args.tls(&default_cipher).unwrap(), None);
    }

    #[test]
    fn tls_with_key_repo() {
        let key_repo = KeyRepo(mqstr!("/path/to/keyrepo"));
        let args = ConnectionArgs {
            tls_key_repo: Some(key_repo),
            ..args_default()
        };
        let default_cipher = CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256"));
        let expected = Some((key_repo, default_cipher, None));
        assert_eq!(args.tls(&default_cipher).unwrap(), expected);
    }

    #[test]
    fn tls_with_cipher_and_label() {
        let key_repo = KeyRepo(mqstr!("/path/to/keyrepo"));
        let cipher = CipherSpec(mqstr!("TLS_AES_256_GCM_SHA384"));
        let label = CertificateLabel(mqstr!("my-label"));
        let args = ConnectionArgs {
            tls_key_repo: Some(key_repo),
            tls_cipher_spec: Some(cipher),
            cert_label: Some(label),
            ..args_default()
        };
        let default_cipher = CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256"));
        let expected = Some((key_repo, cipher, Some(label)));
        assert_eq!(args.tls(&default_cipher).unwrap(), expected);
    }

    #[test]
    fn connection_option_requires_queue_manager() {
        let args = ConnectionArgs {
            connect_queue_manager: None,
            ..args_default()
        };
        match args.connection_option() {
            Ok(_) => panic!("expected error"),
            Err(err) => assert!(err.to_string().contains("Queue Manager argument")),
        }
    }

    #[test]
    fn connection_option_returns_queue_manager() {
        let args = args_default();
        let options = args.connection_option().unwrap();
        assert_eq!(options.queue_manager_name(), Some(&QueueManagerName(mqstr!("QM1"))));
    }

    #[test]
    fn credentials_none() {
        let args = args_default();
        assert!(args.credentials().is_none());
    }

    #[test]
    fn credentials_user() {
        let args = ConnectionArgs {
            username: Some("user".into()),
            password: Some("pass".into()),
            ..args_default()
        };
        match args.credentials() {
            Some(Credentials::User(user, password)) => {
                assert_eq!(user, "user");
                assert_eq!(password.expose_secret(), "pass");
            }
            _ => panic!("expected user credentials"),
        }
    }

    #[test]
    fn queue_manager_name() {
        let args = args_default();
        assert_eq!(args.queue_manager_name(), Some(QueueManagerName(mqstr!("QM1"))));
    }
}
