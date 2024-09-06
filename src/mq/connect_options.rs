#![expect(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]

use std::{any, cmp, ops::Deref};

use crate::{
    macros::{all_multi_tuples, reverse_ident},
    core::values,
    sys, MqStr, ResultCompErrExt, MqiAttr,
};

use super::{
    types::{ChannelName, CipherSpec, ConnectionName, QueueManagerName},
    ConnTag, ConnectParam, ConnectionId, MqStruct,
};

pub const HAS_SCO: i32 = 0b00010;
pub const HAS_CD: i32 = 0b00100;
pub const HAS_CSP: i32 = 0b01000;
pub const HAS_BNO: i32 = 0b10000;

#[expect(unused_variables)]
pub trait ConnectOption<'a> {
    #[inline]
    fn struct_mask(&self) -> i32 {
        0
    }

    #[inline]
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        None
    }

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

impl<'b, T: ConnectOption<'b>> ConnectOption<'b> for &T {
    #[inline]
    fn struct_mask(&self) -> i32 {
        T::struct_mask(self)
    }

    #[inline]
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        T::queue_manager_name(self)
    }

    #[inline]
    fn apply_cno<'ptr>(&'ptr self, cno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'b: 'ptr,
    {
        T::apply_cno(self, cno);
    }

    #[inline]
    fn apply_sco<'ptr>(&'ptr self, sco: &mut MqStruct<'ptr, sys::MQSCO>)
    where
        'b: 'ptr,
    {
        T::apply_sco(self, sco);
    }

    #[inline]
    fn apply_csp<'ptr>(&'ptr self, csp: &mut MqStruct<'ptr, sys::MQCSP>)
    where
        'b: 'ptr,
    {
        T::apply_csp(self, csp);
    }

    #[inline]
    fn apply_cd<'ptr>(&'ptr self, cd: &mut MqStruct<'ptr, sys::MQCD>)
    where
        'b: 'ptr,
    {
        T::apply_cd(self, cd);
    }

    #[inline]
    fn apply_bno<'ptr>(&'ptr self, bno: &mut MqStruct<'ptr, sys::MQBNO>)
    where
        'b: 'ptr,
    {
        T::apply_bno(self, bno);
    }
}

impl<'b, O: ConnectOption<'b>> ConnectOption<'b> for Option<O> {
    #[inline]
    fn struct_mask(&self) -> i32 {
        self.as_ref().map_or(0, ConnectOption::struct_mask)
    }

    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        self.as_ref().and_then(|o| o.queue_manager_name())
    }

    fn apply_cno<'ptr>(&'ptr self, cno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'b: 'ptr,
    {
        if let Some(co) = self {
            co.apply_cno(cno);
        }
    }

    fn apply_sco<'ptr>(&'ptr self, sco: &mut MqStruct<'ptr, sys::MQSCO>)
    where
        'b: 'ptr,
    {
        if let Some(co) = self {
            co.apply_sco(sco);
        }
    }

    fn apply_csp<'ptr>(&'ptr self, csp: &mut MqStruct<'ptr, sys::MQCSP>)
    where
        'b: 'ptr,
    {
        if let Some(co) = self {
            co.apply_csp(csp);
        }
    }

    fn apply_cd<'ptr>(&'ptr self, cd: &mut MqStruct<'ptr, sys::MQCD>)
    where
        'b: 'ptr,
    {
        if let Some(co) = self {
            co.apply_cd(cd);
        }
    }

    fn apply_bno<'ptr>(&'ptr self, bno: &mut MqStruct<'ptr, sys::MQBNO>)
    where
        'b: 'ptr,
    {
        if let Some(co) = self {
            co.apply_bno(bno);
        }
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
    fn apply_cd<'ptr>(&self, cd: &mut MqStruct<'ptr, sys::MQCD>)
    where
        'cd: 'ptr,
    {
        self.cd.clone_into(cd);
    }

    #[inline]
    fn struct_mask(&self) -> i32 {
        HAS_CD
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

impl ConnectOption<'_> for Binding {
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

impl ConnectOption<'_> for QueueManagerName {
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        Some(self)
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

impl ConnectOption<'_> for CipherSpec {
    #[inline]
    fn struct_mask(&self) -> i32 {
        HAS_CD
    }

    fn apply_cd<'ptr>(&'ptr self, cd: &mut super::MqStruct<'ptr, sys::MQCD>)
    where
        'static: 'ptr,
    {
        cd.Version = cmp::max(sys::MQCD_VERSION_7, cd.Version);
        self.copy_into_mqchar(&mut cd.SSLCipherSpec);
    }
}

impl<'tls> ConnectOption<'tls> for Tls<'tls> {
    #[inline]
    fn struct_mask(&self) -> i32 {
        HAS_SCO | self.1.struct_mask()
    }

    fn apply_sco<'ptr>(&self, sco: &mut MqStruct<'ptr, sys::MQSCO>)
    where
        'tls: 'ptr,
    {
        self.0.clone_into(sco);
    }

    fn apply_cd<'ptr>(&'ptr self, cd: &mut MqStruct<'ptr, sys::MQCD>)
    where
        'tls: 'ptr,
    {
        CipherSpec::apply_cd(&self.1, cd);
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

impl<'cred, S: Secret<str>> ConnectOption<'cred> for CredentialsSecret<'cred, S> {
    #[inline]
    fn struct_mask(&self) -> i32 {
        HAS_CSP
    }

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

impl ConnectOption<'_> for values::MQCNO {
    fn apply_cno<'ptr>(&self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'static: 'ptr,
    {
        mqcno.Options |= self.value();
    }
}

impl ConnectOption<'_> for () {}

macro_rules! impl_connectoptions {
    ($first:ident, [$($ty:ident),*]) => {
        // reverse_ident macro is used to ensure right to left application of options
        #[allow(non_snake_case,unused_parens)]
        impl<'r, $first $(, $ty)*> ConnectOption<'r> for ($first $(, $ty)*)
        where
            $first: ConnectOption<'r>,
            $($ty: ConnectOption<'r>),*
        {
            #[inline]
            fn struct_mask(&self) -> i32 {
                let ( $first, $($ty),*) = self;
                $first.struct_mask() | $($ty.struct_mask())|*
            }


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
            fn apply_cno<'ptr>(&'ptr self, cno: &mut MqStruct<'ptr, sys::MQCNO>)
            where
                'r: 'ptr,
            {
                let reverse_ident!($first, $($ty),*) = self;
                $first.apply_cno(cno); // last is first now
                $($ty.apply_cno(cno);)* // first is last now

            }

            #[inline]
            fn apply_sco<'ptr>(&'ptr self, sco: &mut MqStruct<'ptr, sys::MQSCO>)
            where
                'r: 'ptr,
            {
                let reverse_ident!($first, $($ty),*) = self;
                $first.apply_sco(sco);
                $($ty.apply_sco(sco);)*
            }

            #[inline]
            fn apply_cd<'ptr>(&'ptr self, cd: &mut MqStruct<'ptr, sys::MQCD>)
            where
                'r: 'ptr,
            {
                let reverse_ident!($first, $($ty),*) = self;
                $first.apply_cd(cd);
                $($ty.apply_cd(cd);)*
            }

            #[inline]
            fn apply_bno<'ptr>(&'ptr self, bno: &mut MqStruct<'ptr, sys::MQBNO>)
            where
                'r: 'ptr,
            {
                let reverse_ident!($first, $($ty),*) = self;
                $($ty.apply_bno(bno);)*
                $first.apply_bno(bno);

            }

            #[inline]
            fn apply_csp<'ptr>(&'ptr self, csp: &mut MqStruct<'ptr, sys::MQCSP>)
            where
                'r: 'ptr,
            {
                let reverse_ident!($first, $($ty),*) = self;
                $first.apply_csp(csp);
                $($ty.apply_csp(csp);)*
            }
        }
    }
}

all_multi_tuples!(impl_connectoptions);

impl ConnectOption<'_> for ApplName {
    fn apply_cno<'ptr>(&self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'static: 'ptr,
    {
        mqcno.Version = cmp::max(sys::MQCNO_VERSION_7, mqcno.Version);
        self.0.copy_into_mqchar(&mut mqcno.ApplName);
    }
}

impl<'url> ConnectOption<'url> for Ccdt<'url> {
    fn apply_cno<'ptr>(&self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'url: 'ptr,
    {
        mqcno.Options &= !sys::MQCNO_LOCAL_BINDING;
        mqcno.Options |= sys::MQCNO_CLIENT_BINDING;
        mqcno.attach_ccdt(self.0);
    }
}

impl<'bno> ConnectOption<'bno> for MqStruct<'bno, sys::MQBNO> {
    #[inline]
    fn struct_mask(&self) -> i32 {
        HAS_BNO
    }

    fn apply_bno<'ptr>(&self, bno: &mut MqStruct<'ptr, sys::MQBNO>)
    where
        'bno: 'ptr,
    {
        self.clone_into(bno);
    }

    fn apply_cno<'ptr>(&self, cno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'bno: 'ptr,
    {
        cno.Version = cmp::max(sys::MQCNO_VERSION_8, cno.Version);
    }
}

impl<'data> ConnectOption<'data> for MqStruct<'data, sys::MQCSP> {
    #[inline]
    fn struct_mask(&self) -> i32 {
        HAS_CSP
    }

    fn apply_csp<'ptr>(&self, csp: &mut MqStruct<'ptr, sys::MQCSP>)
    where
        'data: 'ptr,
    {
        self.clone_into(csp);
    }
}

impl<'data> ConnectOption<'data> for MqStruct<'data, sys::MQSCO> {
    #[inline]
    fn struct_mask(&self) -> i32 {
        HAS_SCO
    }

    fn apply_sco<'ptr>(&self, sco: &mut MqStruct<'ptr, sys::MQSCO>)
    where
        'data: 'ptr,
    {
        self.clone_into(sco);
    }
}

impl<'data> ConnectOption<'data> for MqStruct<'data, sys::MQCD> {
    #[inline]
    fn struct_mask(&self) -> i32 {
        HAS_CD
    }

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
