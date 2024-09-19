#![expect(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]
#![allow(non_snake_case)]

use std::{any, cmp};

use crate::{
    macros::{all_multi_tuples, reverse_ident},
    prelude::*,
    values, sys, MqStr, MqiAttr,
};

use super::{
    types::{
        impl_from_str, CertificateLabel, ChannelName, CipherSpec, ConnectionName, CryptoHardware, KeyRepo, QueueManagerName,
    },
    ConnTag, ConnectParam, ConnectionId, MqStruct,
};

pub const HAS_CNO: i32 = 0b00000;
pub const HAS_SCO: i32 = 0b00010;
pub const HAS_CD: i32 = 0b00100;
pub const HAS_CSP: i32 = 0b01000;
pub const HAS_BNO: i32 = 0b10000;

#[derive(Debug, Clone, Default)]
pub struct ConnectStructs<'ptr> {
    pub cno: MqStruct<'ptr, sys::MQCNO>,
    pub sco: MqStruct<'ptr, sys::MQSCO>,
    pub csp: MqStruct<'ptr, sys::MQCSP>,
    pub cd: MqStruct<'ptr, sys::MQCD>,
    pub bno: MqStruct<'ptr, sys::MQBNO>,
}

/*
 TODO: I don't believe I have this interface 100% correct. Lifetimes are not conducive
 to the goals I'm trying to achieve. Borrowing self may be better on apply_param.
*/
#[expect(unused_variables)]
pub trait ConnectOption<'a>: Sized {
    #[inline]
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        None
    }

    #[inline]
    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'a: 'ptr,
    {
        HAS_CNO
    }
}

impl<'a, T: ConnectOption<'a> + Copy> ConnectOption<'a> for &T {
    #[inline]
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        T::queue_manager_name(self)
    }

    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'a: 'ptr,
    {
        T::apply_param(*self, structs)
    }
}

impl<'a, O: ConnectOption<'a>> ConnectOption<'a> for Option<O> {
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        self.as_ref().and_then(|o| o.queue_manager_name())
    }

    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'a: 'ptr,
    {
        self.map_or(0, |o| o.apply_param(structs))
    }
}

#[derive(Clone, Debug, derive_more::Deref)]
pub struct ClientDefinition<'ptr> {
    cd: MqStruct<'ptr, sys::MQCD>,
}

impl<'ptr> ClientDefinition<'ptr> {
    #[must_use]
    pub fn default_client() -> Self {
        Self::from_mqcd(MqStruct::new(sys::MQCD {
            Version: sys::MQCD_VERSION_12,
            ..sys::MQCD::client_conn_default()
        }))
    }
    #[must_use]
    pub const fn from_mqcd(cd: MqStruct<'ptr, sys::MQCD>) -> Self {
        Self { cd }
    }

    /// Create a client definition from the minimal channel name, connection name and optional transport type.
    #[must_use]
    pub fn new_client(channel_name: &ChannelName, connection_name: &ConnectionName, transport: Option<values::MQXPT>) -> Self {
        let mut outcome = Self::default_client();
        let mqcd = &mut outcome.cd;
        if let Some(transport) = transport {
            mqcd.TransportType = transport.value();
        }
        channel_name.copy_into_mqchar(&mut mqcd.ChannelName);
        connection_name.copy_into_mqchar(&mut mqcd.ConnectionName);

        outcome
    }

    pub fn from_mqserver(server: &str) -> Result<Self, MqServerSyntaxError> {
        let (channel_name, connection_name, transport) = mqserver(server)?;
        Ok(Self::new_client(&channel_name, &connection_name, Some(transport)))
    }
}

impl<'cd> ConnectOption<'cd> for ClientDefinition<'cd> {
    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'cd: 'ptr,
    {
        self.cd.clone_into(&mut structs.cd);
        HAS_CD
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct ApplName(pub MqStr<28>);
impl_from_str!(ApplName, MqStr<28>);

/// Client Channel Definition Table URL. Sets the connection as `MQCNO_CLIENT_BINDING`.
///
/// Implements the [`ConnectOption`] trait as a parameter to the [`QueueManager::connect`](crate::QueueManager::connect) function.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::From)]
pub struct Ccdt<'url>(pub &'url str);

/// Connection binding mode. Represents the `MQCNO_*_BINDING` constants.
///
/// Implements the [`ConnectOption`] trait as a parameter to the [`QueueManager::connect`](crate::QueueManager::connect) function.
#[derive(Debug, Clone, Copy, Default)]
pub enum Binding {
    #[default]
    /// MQI default binding
    Default,
    /// Attempt a server connection (`MQCNO_LOCAL_BINDING`)
    Local,
    /// Attempt a client connection (`MQCNO_CLIENT_BINDING`)
    Client,
}

impl ConnectOption<'_> for Binding {
    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'static: 'ptr,
    {
        structs.cno.Options &= !(sys::MQCNO_CLIENT_BINDING | sys::MQCNO_LOCAL_BINDING);
        structs.cno.Options |= match self {
            Self::Default => sys::MQCNO_NONE,
            Self::Local => sys::MQCNO_LOCAL_BINDING,
            Self::Client => sys::MQCNO_CLIENT_BINDING,
        };
        HAS_CNO
    }
}

impl ConnectOption<'_> for QueueManagerName {
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        Some(self)
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum CredentialsSecret<'cred, S> {
    #[default]
    Default,
    User(&'cred str, S, Option<S>),
    Token(S, Option<S>),
}

impl<'cred, S> CredentialsSecret<'cred, S> {
    pub fn user(user: &'cred str, password: impl Into<S>) -> Self {
        Self::User(user, password.into(), None)
    }
}

pub type Credentials<'cred, S> = CredentialsSecret<'cred, ProtectedSecret<S>>;

#[derive(Clone, Copy, Default)]
pub struct ProtectedSecret<T>(T);

impl<T> ProtectedSecret<T> {
    pub const fn new(secret: T) -> Self {
        Self(secret)
    }
}

#[derive(Debug, Clone, Default)]
#[must_use]
pub struct Tls<'pw>(MqStruct<'pw, sys::MQSCO>, CipherSpec);

pub enum SuiteB {
    None,
    Min(usize),
}

impl From<SuiteB> for [sys::MQLONG; 4] {
    fn from(value: SuiteB) -> Self {
        const SIZED: &[(usize, sys::MQLONG)] = &[(128, sys::MQ_SUITE_B_128_BIT), (192, sys::MQ_SUITE_B_192_BIT)];
        match value {
            SuiteB::None => [
                sys::MQ_SUITE_B_NONE,
                sys::MQ_SUITE_B_NOT_AVAILABLE,
                sys::MQ_SUITE_B_NOT_AVAILABLE,
                sys::MQ_SUITE_B_NOT_AVAILABLE,
            ],
            SuiteB::Min(min_size) => {
                let mut result = [
                    sys::MQ_SUITE_B_NOT_AVAILABLE,
                    sys::MQ_SUITE_B_NOT_AVAILABLE,
                    sys::MQ_SUITE_B_NOT_AVAILABLE,
                    sys::MQ_SUITE_B_NOT_AVAILABLE,
                ];
                for (i, (.., suite)) in SIZED.iter().filter(|(size, ..)| *size >= min_size).enumerate() {
                    result[i] = *suite;
                }
                result
            }
        }
    }
}

impl<'pw> Tls<'pw> {
    pub fn new(repo: &KeyRepo, password: Option<&'pw str>, label: Option<&CertificateLabel>, cipher: &CipherSpec) -> Self {
        let mut tls = Self::default();
        tls.key_repo(repo);
        tls.certificate_label(label);
        tls.key_repo_password(password);
        cipher.clone_into(&mut tls.1);
        tls
    }

    pub fn crypto_hardware(&mut self, hardware: Option<&CryptoHardware>) -> &mut Self {
        hardware
            .unwrap_or(&CryptoHardware::default())
            .copy_into_mqchar(&mut self.0.CryptoHardware);
        self
    }

    pub fn certificate_label(&mut self, label: Option<&CertificateLabel>) -> &mut Self {
        self.0.Version = cmp::max(sys::MQSCO_VERSION_5, self.0.Version);
        label
            .unwrap_or(&CertificateLabel::default())
            .copy_into_mqchar(&mut self.0.CertificateLabel);
        self
    }

    pub fn fips_required(&mut self, is_required: bool) -> &mut Self {
        self.0.Version = cmp::max(sys::MQSCO_VERSION_2, self.0.Version);
        self.0.FipsRequired = if is_required {
            sys::MQSSL_FIPS_YES
        } else {
            sys::MQSSL_FIPS_NO
        };
        self
    }

    pub fn suite_b_policy(&mut self, policy: impl Into<[sys::MQLONG; 4]>) -> &mut Self {
        self.0.Version = cmp::max(sys::MQSCO_VERSION_3, self.0.Version);
        self.0.EncryptionPolicySuiteB = policy.into();
        self
    }

    pub fn cert_val_policy(&mut self, policy: sys::MQLONG) -> &mut Self {
        self.0.Version = cmp::max(sys::MQSCO_VERSION_4, self.0.Version);
        self.0.CertificateValPolicy = policy;
        self
    }

    pub fn key_reset_count(&mut self, count: sys::MQLONG) -> &mut Self {
        self.0.Version = cmp::max(sys::MQSCO_VERSION_2, self.0.Version);
        self.0.KeyResetCount = count;
        self
    }

    pub fn key_repo_password(&mut self, password: Option<&'pw str>) -> &mut Self {
        self.0.attach_repo_password(password);
        self
    }

    pub fn key_repo(&mut self, repo: &KeyRepo) -> &mut Self {
        repo.copy_into_mqchar(&mut self.0.KeyRepository);
        self
    }
}

impl ConnectOption<'_> for CipherSpec {
    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'static: 'ptr,
    {
        structs.cd.Version = cmp::max(sys::MQCD_VERSION_7, structs.cd.Version);
        self.copy_into_mqchar(&mut structs.cd.SSLCipherSpec);
        HAS_CD
    }
}

impl<'tls> ConnectOption<'tls> for Tls<'tls> {
    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'tls: 'ptr,
    {
        self.0.clone_into(&mut structs.sco);
        HAS_SCO | self.1.apply_param(structs)
    }
}

pub trait Secret<'y, Y: ?Sized> {
    #[must_use]
    fn expose_secret(&self) -> &'y Y;
}

impl<'t, T: ?Sized> Secret<'t, T> for ProtectedSecret<&'t T> {
    fn expose_secret(&self) -> &'t T {
        let Self(secret) = self;
        secret
    }
}

impl<T> std::fmt::Debug for ProtectedSecret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ProtectedSecret")
            .field(&format_args!("{} <REDACTED>", any::type_name::<T>()))
            .finish()
    }
}

impl<T> From<T> for ProtectedSecret<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<'cred, S: Secret<'cred, str>> ConnectOption<'cred> for CredentialsSecret<'cred, S> {
    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'cred: 'ptr,
    {
        match &self {
            CredentialsSecret::Default => {
                // No authentication
                structs.csp.AuthenticationType = sys::MQCSP_AUTH_NONE;
            }
            CredentialsSecret::User(user, password, ..) => {
                // UserId and Password authentication
                let password = password.expose_secret();
                structs.csp.AuthenticationType = sys::MQCSP_AUTH_USER_ID_AND_PWD;
                structs.csp.attach_password(password);
                structs.csp.attach_userid(user);
            }
            CredentialsSecret::Token(token, ..) => {
                // JWT authentication
                let token = token.expose_secret();
                structs.csp.AuthenticationType = sys::MQCSP_AUTH_ID_TOKEN;
                structs.csp.attach_token(token);
            }
        }

        // Populate the initial key
        if let CredentialsSecret::User(.., Some(initial_key)) | CredentialsSecret::Token(.., Some(initial_key)) = self {
            let initial_key = initial_key.expose_secret();
            structs.csp.attach_initial_key(initial_key);
        }

        HAS_CSP
    }
}

impl ConnectOption<'_> for values::MQCNO {
    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'static: 'ptr,
    {
        structs.cno.Options |= self.value();
        HAS_CNO
    }
}

impl ConnectOption<'_> for () {}

macro_rules! impl_connectoptions {
    ($first:ident, [$($ty:ident),*]) => {
        // reverse_ident macro is used to ensure right to left application of options
        #[allow(non_snake_case,unused_variables)]
        impl<'r, $first $(, $ty)*> ConnectOption<'r> for ($first $(, $ty)*)
        where
            $first: ConnectOption<'r>,
            $($ty: ConnectOption<'r>),*
        {
            fn queue_manager_name(&self) -> Option<&QueueManagerName> {
                let ( $first, $($ty),*) = self;
                if let name @ Some(_) = $first.queue_manager_name() {
                    return name;
                }

                $(
                    if let name @ Some(_) = $ty.queue_manager_name() {
                        return name;
                    }
                )*

                None
            }

            #[inline]
            fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
            where
                'r: 'ptr,
            {
                let reverse_ident!($first, $($ty),*) = self; // first is last, last is first
                $first.apply_param(structs) | $($ty.apply_param(structs))|*
            }
        }
    }
}

all_multi_tuples!(impl_connectoptions);

impl ConnectOption<'_> for ApplName {
    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'static: 'ptr,
    {
        structs.cno.Version = cmp::max(sys::MQCNO_VERSION_7, structs.cno.Version);
        self.0.copy_into_mqchar(&mut structs.cno.ApplName);
        HAS_CNO
    }
}

impl<'url> ConnectOption<'url> for Ccdt<'url> {
    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'url: 'ptr,
    {
        structs.cno.Options &= !sys::MQCNO_LOCAL_BINDING;
        structs.cno.Options |= sys::MQCNO_CLIENT_BINDING;
        structs.cno.attach_ccdt(self.0);

        HAS_CNO
    }
}

impl<'bno> ConnectOption<'bno> for MqStruct<'bno, sys::MQBNO> {
    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'bno: 'ptr,
    {
        self.clone_into(&mut structs.bno);
        structs.cno.Version = cmp::max(sys::MQCNO_VERSION_8, structs.cno.Version);
        HAS_BNO
    }
}

impl<'csp> ConnectOption<'csp> for MqStruct<'csp, sys::MQCSP> {
    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'csp: 'ptr,
    {
        self.clone_into(&mut structs.csp);
        structs.cno.Version = cmp::max(sys::MQCNO_VERSION_5, structs.cno.Version);
        HAS_CSP
    }
}

impl<'sco> ConnectOption<'sco> for MqStruct<'sco, sys::MQSCO> {
    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'sco: 'ptr,
    {
        self.clone_into(&mut structs.sco);
        structs.cno.Version = cmp::max(sys::MQCNO_VERSION_4, structs.cno.Version);
        HAS_SCO
    }
}

impl<'cd> ConnectOption<'cd> for MqStruct<'cd, sys::MQCD> {
    fn apply_param<'ptr>(self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'cd: 'ptr,
    {
        self.clone_into(&mut structs.cd);
        structs.cno.Version = cmp::max(sys::MQCNO_VERSION_2, structs.cno.Version);
        structs.cno.Options &= !sys::MQCNO_LOCAL_BINDING;
        structs.cno.Options |= sys::MQCNO_CLIENT_BINDING;
        HAS_CD
    }
}

impl<'b, S> MqiAttr<ConnectParam<'b>, S> for ConnectionId {
    #[inline]
    fn extract<F>(param: &mut ConnectParam<'b>, connect: F) -> crate::ResultComp<(Self, S)>
    where
        F: FnOnce(&mut ConnectParam<'b>) -> crate::ResultComp<S>,
    {
        param.Version = cmp::max(sys::MQCNO_VERSION_5, param.Version);
        connect(param).map_completion(|state| (Self(param.ConnectionId), state))
    }
}

impl<'b, S> MqiAttr<ConnectParam<'b>, S> for ConnTag {
    #[inline]
    fn extract<F>(param: &mut ConnectParam<'b>, connect: F) -> crate::ResultComp<(Self, S)>
    where
        F: FnOnce(&mut ConnectParam<'b>) -> crate::ResultComp<S>,
    {
        param.Options |= sys::MQCNO_GENERATE_CONN_TAG;
        param.Version = cmp::max(sys::MQCNO_VERSION_3, param.Version);
        connect(param).map_completion(|state| (Self(param.ConnTag), state))
    }
}

pub fn mqserver(server: &str) -> Result<(ChannelName, ConnectionName, values::MQXPT), MqServerSyntaxError> {
    #[expect(clippy::unwrap_used)]
    let server_pattern = regex_lite::Regex::new(r"^(.+)/(.+)/(.+)$").unwrap();

    if let Some((_, [channel, transport, connection_name])) = server_pattern.captures(server).map(|v| v.extract()) {
        let channel: ChannelName = channel
            .try_into()
            .ok()
            .filter(MqStr::has_value)
            .map(ChannelName)
            .ok_or_else(|| MqServerSyntaxError::ChannelFormat(channel.to_string()))?;
        let connection_name = connection_name
            .try_into()
            .ok()
            .filter(MqStr::has_value)
            .map(ConnectionName)
            .ok_or_else(|| MqServerSyntaxError::ConnectionNameFormat(connection_name.to_string()))?;
        let transport = match transport {
            "TCP" => Ok(values::MQXPT(sys::MQXPT_TCP)),
            "LU62" => Ok(values::MQXPT(sys::MQXPT_LU62)),
            "NETBIOS" => Ok(values::MQXPT(sys::MQXPT_NETBIOS)),
            "SPX" => Ok(values::MQXPT(sys::MQXPT_SPX)),
            other => Err(MqServerSyntaxError::UnrecognizedTransport(other.to_string())),
        }?;
        Ok((channel, connection_name, transport))
    } else {
        Err(MqServerSyntaxError::InvalidFormat)
    }
}

#[derive(Debug, derive_more::Error, derive_more::Display)]
pub enum MqServerSyntaxError {
    #[display("Invalid Format")]
    InvalidFormat,
    #[display("Channel \"{_0}\" invalid format")]
    #[error(ignore)]
    ChannelFormat(String),
    #[display("Connection Name \"{_0}\" invalid format")]
    #[error(ignore)]
    ConnectionNameFormat(String),
    #[display("Transport \"{_0}\" not recognized")]
    #[error(ignore)]
    UnrecognizedTransport(String),
}

#[cfg(test)]
mod tests {
    use super::{ProtectedSecret, Secret as _};

    #[test]
    fn secret() {
        let x: ProtectedSecret<&str> = "hello".into();
        let _secret = x.expose_secret();
    }
}
