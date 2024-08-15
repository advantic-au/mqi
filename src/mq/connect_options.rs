use std::{any, cmp, ops::Deref};

use crate::{core::values, sys, MqMask, MqStr, MqValue};

use super::{
    types::{ChannelName, CipherSpec, ConnectionName},
    ConnTag, ConnectParam, ConnectionId, MqStruct, MqiAttr,
};

pub const HAS_SCO: i32 = 0b00010;
pub const HAS_CD: i32 = 0b00100;
pub const HAS_CSP: i32 = 0b01000;
pub const HAS_BNO: i32 = 0b10000;

#[allow(unused_variables)]
pub trait ConnectOptions<'a> {
    const STRUCTS: i32;

    fn apply_cno<'ptr>(&'ptr self, cno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'a: 'ptr,
    {
    }

    fn apply_sco<'ptr>(&'ptr self, sco: &mut MqStruct<'ptr, sys::MQSCO>)
    where
        'a: 'ptr,
    {
    }

    fn apply_csp<'ptr>(&'ptr self, csp: &mut MqStruct<'ptr, sys::MQCSP>)
    where
        'a: 'ptr,
    {
    }

    fn apply_cd<'ptr>(&'ptr self, cd: &mut MqStruct<'ptr, sys::MQCD>)
    where
        'a: 'ptr,
    {
    }

    fn apply_bno<'ptr>(&'ptr self, bno: &mut MqStruct<'ptr, sys::MQBNO>)
    where
        'a: 'ptr,
    {
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
    pub fn new_client(
        channel_name: &ChannelName,
        connection_name: &ConnectionName,
        transport: Option<MqValue<values::MQXPT>>,
    ) -> Self {
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

impl<'cd> ConnectOptions<'cd> for ClientDefinition<'cd> {
    const STRUCTS: i32 = HAS_CD;

    fn apply_cd<'ptr>(&'ptr self, cd: &mut MqStruct<'ptr, sys::MQCD>)
    where
        'cd: 'ptr,
    {
        self.cd.clone_into(cd);
    }
}

pub struct ApplName(pub MqStr<28>);
pub struct Ccdt<'url>(pub &'url str);

#[derive(Debug, Clone, Copy)]
pub enum Binding {
    Default,
    Local,
    Client,
}

impl ConnectOptions<'_> for Binding {
    const STRUCTS: i32 = 0;
    fn apply_cno<'ptr>(&self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'static: 'ptr,
    {
        mqcno.Options &= !(sys::MQCNO_CLIENT_BINDING | sys::MQCNO_LOCAL_BINDING);
        mqcno.Options |= match self {
            Self::Default => sys::MQCNO_NONE,
            Self::Local => sys::MQCNO_LOCAL_BINDING,
            Self::Client => sys::MQCNO_CLIENT_BINDING,
        }
    }
}

#[derive(Default, Debug, Clone)]
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

#[derive(Clone, Default)]
pub struct ProtectedSecret<T>(T);

impl<T> ProtectedSecret<T> {
    pub const fn new(secret: T) -> Self {
        Self(secret)
    }
}

#[derive(Debug, Clone, Default)]
#[must_use]
pub struct Tls<'pw>(MqStruct<'pw, sys::MQSCO>, CipherSpec);

pub type KeyRepo = MqStr<256>;
pub type CryptoHardware = MqStr<256>;
pub type CertificateLabel = MqStr<64>;

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
            .unwrap_or(&MqStr::empty())
            .copy_into_mqchar(&mut self.0.CryptoHardware);
        self
    }

    pub fn certificate_label(&mut self, label: Option<&CertificateLabel>) -> &mut Self {
        self.0.Version = cmp::max(sys::MQSCO_VERSION_5, self.0.Version);
        label
            .unwrap_or(&MqStr::empty())
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

impl<'tls> ConnectOptions<'tls> for Tls<'tls> {
    const STRUCTS: i32 = HAS_SCO | CipherSpec::STRUCTS;
    fn apply_sco<'ptr>(&self, sco: &mut MqStruct<'ptr, sys::MQSCO>)
    where
        'tls: 'ptr,
    {
        self.0.clone_into(sco);
    }

    fn apply_cno<'ptr>(&'ptr self, cno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'tls: 'ptr,
    {
        CipherSpec::apply_cno(&self.1, cno);
    }

    fn apply_csp<'ptr>(&'ptr self, csp: &mut MqStruct<'ptr, sys::MQCSP>)
    where
        'tls: 'ptr,
    {
        CipherSpec::apply_csp(&self.1, csp);
    }

    fn apply_cd<'ptr>(&'ptr self, cd: &mut MqStruct<'ptr, sys::MQCD>)
    where
        'tls: 'ptr,
    {
        CipherSpec::apply_cd(&self.1, cd);
    }

    fn apply_bno<'ptr>(&'ptr self, bno: &mut MqStruct<'ptr, sys::MQBNO>)
    where
        'tls: 'ptr,
    {
        CipherSpec::apply_bno(&self.1, bno);
    }
}

pub trait Secret<Y: ?Sized> {
    #[must_use]
    fn expose_secret(&self) -> &Y;
}

impl<T: Deref> Secret<T::Target> for ProtectedSecret<T> {
    fn expose_secret(&self) -> &T::Target {
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

impl<'cred, S: Secret<str>> ConnectOptions<'cred> for CredentialsSecret<'cred, S> {
    const STRUCTS: i32 = HAS_CSP;

    fn apply_csp<'ptr>(&'ptr self, csp: &mut MqStruct<'ptr, sys::MQCSP>)
    where
        'cred: 'ptr,
    {
        match self {
            CredentialsSecret::Default => {
                // No authentication
                csp.AuthenticationType = sys::MQCSP_AUTH_NONE;
            }
            CredentialsSecret::User(user, password, ..) => {
                // UserId and Password authentication
                let password = password.expose_secret();
                csp.AuthenticationType = sys::MQCSP_AUTH_USER_ID_AND_PWD;
                csp.attach_password(password);
                csp.attach_userid(user);
            }
            CredentialsSecret::Token(token, ..) => {
                // JWT authentication
                let token = token.expose_secret();
                csp.AuthenticationType = sys::MQCSP_AUTH_ID_TOKEN;
                csp.attach_token(token);
            }
        }

        // Populate the initial key
        if let CredentialsSecret::User(.., Some(initial_key)) | CredentialsSecret::Token(.., Some(initial_key)) = &self {
            let initial_key = initial_key.expose_secret();
            csp.attach_initial_key(initial_key);
        }
    }
}

impl ConnectOptions<'_> for MqMask<values::MQCNO> {
    const STRUCTS: i32 = 0;

    fn apply_cno<'ptr>(&self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'static: 'ptr,
    {
        mqcno.Options |= self.value();
    }
}

impl ConnectOptions<'_> for () {
    const STRUCTS: i32 = 0;
}

impl<'r, A: ConnectOptions<'r>, B: ConnectOptions<'r>> ConnectOptions<'r> for (A, B) {
    const STRUCTS: i32 = A::STRUCTS | B::STRUCTS;

    #[inline]
    fn apply_cno<'ptr>(&'ptr self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'r: 'ptr,
    {
        self.1.apply_cno(mqcno);
        self.0.apply_cno(mqcno);
    }

    #[inline]
    fn apply_sco<'ptr>(&'ptr self, sco: &mut MqStruct<'ptr, sys::MQSCO>)
    where
        'r: 'ptr,
    {
        self.1.apply_sco(sco);
        self.0.apply_sco(sco);
    }

    #[inline]
    fn apply_cd<'ptr>(&'ptr self, cd: &mut MqStruct<'ptr, sys::MQCD>)
    where
        'r: 'ptr,
    {
        self.1.apply_cd(cd);
        self.0.apply_cd(cd);
    }

    #[inline]
    fn apply_bno<'ptr>(&'ptr self, bno: &mut MqStruct<'ptr, sys::MQBNO>)
    where
        'r: 'ptr,
    {
        self.1.apply_bno(bno);
        self.0.apply_bno(bno);
    }
}

impl<'r, A: ConnectOptions<'r>, B: ConnectOptions<'r>, C: ConnectOptions<'r>> ConnectOptions<'r> for (A, B, C) {
    const STRUCTS: i32 = A::STRUCTS | B::STRUCTS | C::STRUCTS;

    #[inline]
    fn apply_cno<'ptr>(&'ptr self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'r: 'ptr,
    {
        self.2.apply_cno(mqcno);
        self.1.apply_cno(mqcno);
        self.0.apply_cno(mqcno);
    }

    #[inline]
    fn apply_sco<'ptr>(&'ptr self, sco: &mut MqStruct<'ptr, sys::MQSCO>)
    where
        'r: 'ptr,
    {
        self.2.apply_sco(sco);
        self.1.apply_sco(sco);
        self.0.apply_sco(sco);
    }

    #[inline]
    fn apply_cd<'ptr>(&'ptr self, cd: &mut MqStruct<'ptr, sys::MQCD>)
    where
        'r: 'ptr,
    {
        self.2.apply_cd(cd);
        self.1.apply_cd(cd);
        self.0.apply_cd(cd);
    }

    #[inline]
    fn apply_bno<'ptr>(&'ptr self, bno: &mut MqStruct<'ptr, sys::MQBNO>)
    where
        'r: 'ptr,
    {
        self.2.apply_bno(bno);
        self.1.apply_bno(bno);
        self.0.apply_bno(bno);
    }

    #[inline]
    fn apply_csp<'ptr>(&'ptr self, csp: &mut MqStruct<'ptr, sys::MQCSP>)
    where
        'r: 'ptr,
    {
        self.2.apply_csp(csp);
        self.1.apply_csp(csp);
        self.0.apply_csp(csp);
    }
}

impl ConnectOptions<'static> for ApplName {
    const STRUCTS: i32 = 0;
    fn apply_cno<'ptr>(&self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'static: 'ptr,
    {
        mqcno.Version = cmp::max(sys::MQCNO_VERSION_7, mqcno.Version);
        self.0.copy_into_mqchar(&mut mqcno.ApplName);
    }
}

impl<'url> ConnectOptions<'url> for Ccdt<'url> {
    const STRUCTS: i32 = 0;

    fn apply_cno<'ptr>(&self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'url: 'ptr,
    {
        mqcno.Options &= !sys::MQCNO_LOCAL_BINDING;
        mqcno.Options |= sys::MQCNO_CLIENT_BINDING;
        mqcno.attach_ccdt(self.0);
    }
}

impl<'bno> ConnectOptions<'bno> for MqStruct<'bno, sys::MQBNO> {
    const STRUCTS: i32 = HAS_BNO;

    fn apply_bno<'ptr>(&self, bno: &mut MqStruct<'ptr, sys::MQBNO>)
    where
        'bno: 'ptr,
    {
        self.clone_into(bno);
    }

    fn apply_cno<'ptr>(&'ptr self, cno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'bno: 'ptr,
    {
        cno.Version = cmp::max(sys::MQCNO_VERSION_8, cno.Version);
    }
}

impl<'data> ConnectOptions<'data> for MqStruct<'data, sys::MQCSP> {
    const STRUCTS: i32 = HAS_CSP;

    fn apply_csp<'ptr>(&self, csp: &mut MqStruct<'ptr, sys::MQCSP>)
    where
        'data: 'ptr,
    {
        self.clone_into(csp);
    }
}

impl<'data> ConnectOptions<'data> for MqStruct<'data, sys::MQSCO> {
    const STRUCTS: i32 = HAS_SCO;

    fn apply_sco<'ptr>(&self, sco: &mut MqStruct<'ptr, sys::MQSCO>)
    where
        'data: 'ptr,
    {
        self.clone_into(sco);
    }
}

impl<'data> ConnectOptions<'data> for MqStruct<'data, sys::MQCD> {
    const STRUCTS: i32 = HAS_CD;

    fn apply_cno<'ptr>(&self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'data: 'ptr,
    {
        mqcno.Options &= !sys::MQCNO_LOCAL_BINDING;
        mqcno.Options |= sys::MQCNO_CLIENT_BINDING;
    }

    fn apply_cd<'ptr>(&self, cd: &mut MqStruct<'ptr, sys::MQCD>)
    where
        'data: 'ptr,
    {
        (*self).clone_into(cd);
    }
}

impl<'b> MqiAttr<ConnectParam<'b>> for ConnectionId {
    #[inline]
    fn from_mqi<Y, F: FnOnce(&mut ConnectParam<'b>) -> Y>(param: &mut ConnectParam<'b>, connect: F) -> (Self, Y) {
        param.Version = cmp::max(sys::MQCNO_VERSION_5, param.Version);
        let connect_result = connect(param);
        (Self(param.ConnectionId), connect_result)
    }
}

impl<'b> MqiAttr<ConnectParam<'b>> for ConnTag {
    #[inline]
    fn from_mqi<Y, F: FnOnce(&mut ConnectParam<'b>) -> Y>(param: &mut ConnectParam<'b>, connect: F) -> (Self, Y) {
        param.Options |= sys::MQCNO_GENERATE_CONN_TAG;
        param.Version = cmp::max(sys::MQCNO_VERSION_3, param.Version);
        let connect_result = connect(param);
        (Self(param.ConnTag), connect_result)
    }
}

pub fn mqserver(server: &str) -> Result<(ChannelName, ConnectionName, MqValue<values::MQXPT>), MqServerSyntaxError> {
    #[allow(clippy::unwrap_used)]
    let server_pattern = regex::Regex::new(r"^(.+)/(.+)/(.+)$").unwrap();

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
            "TCP" => Ok(MqValue::from(sys::MQXPT_TCP)),
            "LU62" => Ok(MqValue::from(sys::MQXPT_LU62)),
            "NETBIOS" => Ok(MqValue::from(sys::MQXPT_NETBIOS)),
            "SPX" => Ok(MqValue::from(sys::MQXPT_SPX)),
            other => Err(MqServerSyntaxError::UnrecognizedTransport(other.to_string())),
        }?;
        Ok((channel, connection_name, transport))
    } else {
        Err(MqServerSyntaxError::InvalidFormat)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MqServerSyntaxError {
    #[error("Invalid Format")]
    InvalidFormat,
    #[error("Channel \"{}\" invalid format", .0)]
    ChannelFormat(String),
    #[error("Connection Name \"{}\" invalid format", .0)]
    ConnectionNameFormat(String),
    #[error("Transport \"{}\" not recognized", .0)]
    UnrecognizedTransport(String),
}
