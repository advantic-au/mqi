use std::marker::PhantomPinned;
use std::ops::Deref;
use std::pin::Pin;
use std::{convert::Into, fmt::Debug};

use crate::core::Library;
use crate::{sys, CipherSpec, MqStructSelfRef, NoStruct, StructBuilder, StructOptionBuilder, StructType};
use crate::{ApplName, ChannelName, ConnectionName, MqStr, MqStruct, QMName, ResultComp};
use thiserror::Error;

use super::{ConnectionShare, HandleShare};

pub trait Secret<T: Deref = String> {
    fn expose_secret(&self) -> &T::Target;
}

#[derive(Clone, Default)]
pub struct ProtectedSecret<T = String>(T);

impl<T> ProtectedSecret<T> {
    pub const fn new(secret: T) -> Self {
        Self(secret)
    }
}

impl<T: Deref> Secret<T> for ProtectedSecret<T> {
    #[must_use]
    fn expose_secret(&self) -> &T::Target {
        let Self(secret) = self;
        secret
    }
}

impl Debug for ProtectedSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ProtectedSecret").field(&"[redacted]").finish()
    }
}

impl<T: Into<String>> From<T> for ProtectedSecret {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, Default)]
pub struct ExternalConfig<const BINDING: sys::MQLONG>;
pub type LocalBinding = ExternalConfig<0x400 /* sys::MQCNO_LOCAL_BINDING */>;
pub type DefaultBinding = ExternalConfig<0 /* sys::MQCNO_NONE> */>;

impl<const N: sys::MQLONG> DefinitionMethod for ExternalConfig<N> {
    type ScoBuilder = NoStruct;
    type CdBuilder = NoStruct;
    type BnoBuilder = NoStruct;

    fn apply_cno<'ptr>(&'ptr self, cno: &mut MqStruct<'ptr, sys::MQCNO>) {
        cno.Options &= !(sys::MQCNO_LOCAL_BINDING | sys::MQCNO_CLIENT_BINDING); // Clear the BINDING bits
        cno.Options |= N;
    }

    fn sco(&self) -> &Self::ScoBuilder {
        &NoStruct
    }

    fn cd(&self) -> &Self::CdBuilder {
        &NoStruct
    }

    fn bno(&self) -> &Self::BnoBuilder {
        &NoStruct
    }
}

pub struct ClientDefinition<C, T> {
    config: C,
    tls: T,
    balance: Option<MqStruct<'static, sys::MQBNO>>,
}

impl<C: DefinitionMethod, T: StructOptionBuilder<sys::MQSCO>> DefinitionMethod for ClientDefinition<C, T> {
    type ScoBuilder = T;
    type CdBuilder = C::CdBuilder;
    type BnoBuilder = NoStruct;

    fn apply_cno<'ptr>(&'ptr self, cno: &mut MqStruct<'ptr, sys::MQCNO>) {
        self.config.apply_cno(cno);
    }

    fn sco(&self) -> &Self::ScoBuilder {
        &self.tls
    }

    fn cd(&self) -> &Self::CdBuilder {
        self.config.cd()
    }

    fn bno(&self) -> &Self::BnoBuilder {
        &NoStruct
    }
}

#[derive(Debug, Clone)]
pub struct Ccdt {
    url: String,
}

impl Ccdt {
    pub fn new(url: impl Into<String>) -> ClientDefinition<Self, NoStruct> {
        ClientDefinition {
            config: Self { url: url.into() },
            tls: NoStruct,
            balance: None,
        }
    }
}

impl DefinitionMethod for Ccdt {
    type ScoBuilder = NoStruct;
    type CdBuilder = NoStruct;
    type BnoBuilder = NoStruct;

    fn apply_cno<'ptr>(&'ptr self, cno: &mut MqStruct<'ptr, sys::MQCNO>) {
        cno.Options &= !(sys::MQCNO_LOCAL_BINDING | sys::MQCNO_CLIENT_BINDING); // Clear the BINDING bits
        cno.Options |= sys::MQCNO_CLIENT_BINDING;
        cno.attach_ccdt(self.url.as_str());
    }

    fn sco(&self) -> &Self::ScoBuilder {
        &NoStruct
    }

    fn cd(&self) -> &Self::ScoBuilder {
        &NoStruct
    }

    fn bno(&self) -> &Self::ScoBuilder {
        &NoStruct
    }
}

impl<S> StructType<sys::MQCSP> for CredentialsSecret<S> {
    type Struct<'a> = MqStruct<'a, sys::MQCSP> where Self: 'a;
}

#[allow(clippy::field_reassign_with_default)]
impl<S: Secret> StructBuilder<sys::MQCSP> for CredentialsSecret<S> {
    fn build(&self) -> Self::Struct<'_> {
        let mut csp = MqStruct::<sys::MQCSP>::default();
        csp.Version = sys::MQCSP_VERSION_3;

        match self {
            Self::Default => {
                // No authentication
                csp.AuthenticationType = sys::MQCSP_AUTH_NONE;
            }
            Self::User(ref user, ref password, ..) => {
                // UserId and Password authentication
                let password = password.expose_secret();
                csp.AuthenticationType = sys::MQCSP_AUTH_USER_ID_AND_PWD;
                csp.attach_password(password);
                csp.attach_userid(user);
            }
            Self::Token(ref token, ..) => {
                // JWT authentication
                let token = token.expose_secret();
                csp.AuthenticationType = sys::MQCSP_AUTH_ID_TOKEN;
                csp.attach_token(token);
            }
        }

        // Populate the initial key
        if let Self::User(.., Some(initial_key)) | Self::Token(.., Some(initial_key)) = &self {
            let initial_key = initial_key.expose_secret();
            csp.attach_initial_key(initial_key);
        }

        csp
    }
}

impl<S: Secret> StructOptionBuilder<sys::MQCSP> for CredentialsSecret<S> {
    fn option_build(&self) -> Option<Self::Struct<'_>> {
        Some(self.build())
    }
}

/// Defines how the `MQCNO` is populated for the connection method
pub trait DefinitionMethod {
    type ScoBuilder: StructOptionBuilder<sys::MQSCO>;
    type CdBuilder: StructOptionBuilder<sys::MQCD>;
    type BnoBuilder: StructOptionBuilder<sys::MQBNO>;

    /// Update the provided `MQCNO` with method details
    fn apply_cno<'ptr>(&'ptr self, cno: &mut MqStruct<'ptr, sys::MQCNO>);

    /// Create and populate an optional `MQSCO` structure for the method
    fn sco(&self) -> &Self::ScoBuilder;

    /// Create and populate an optional `MQCD` structure for the method
    fn cd(&self) -> &Self::CdBuilder;

    /// Create and populate an optional `MQBNO` structure for the method
    fn bno(&self) -> &Self::BnoBuilder;
}

pub type Credentials = CredentialsSecret<ProtectedSecret>;

/// Provides the credentials at connection time
#[derive(Default, Debug, Clone)]
pub enum CredentialsSecret<S> {
    #[default]
    Default,
    User(String, S, Option<S>),
    Token(S, Option<S>),
}

impl<S> CredentialsSecret<S> {
    pub fn user(user: impl Into<String>, password: impl Into<S>) -> Self {
        Self::User(user.into(), password.into(), None)
    }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct TlsSecret<S> {
    sco: sys::MQSCO,
    key_repo_password: Option<S>,
}
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

impl<S> StructType<sys::MQSCO> for TlsSecret<S> {
    type Struct<'a> = MqStruct<'a, sys::MQSCO> where Self: 'a;
}

impl<S: Secret> StructBuilder<sys::MQSCO> for TlsSecret<S> {
    fn build(&self) -> Self::Struct<'_> {
        let mut sco = MqStruct::new(self.sco);

        if let Some(pwd) = &self.key_repo_password {
            let secret = pwd.expose_secret();
            sco.attach_repo_password(secret);
        }

        sco
    }
}

impl<S: Secret> StructOptionBuilder<sys::MQSCO> for TlsSecret<S> {
    fn option_build(&self) -> Option<Self::Struct<'_>> {
        Some(self.build())
    }
}

impl<S> Default for TlsSecret<S> {
    fn default() -> Self {
        Self {
            sco: sys::MQSCO {
                Version: sys::MQSCO_VERSION_6,
                ..sys::MQSCO::default()
            },
            key_repo_password: Option::default(),
        }
    }
}

pub type Tls = TlsSecret<ProtectedSecret<String>>;

pub type KeyRepo = MqStr<256>;
pub type CryptoHardware = MqStr<256>;
pub type CertificateLabel = MqStr<64>;

impl<S> TlsSecret<S> {
    pub fn new(repo: &KeyRepo, password: Option<impl Into<S>>, label: Option<&CertificateLabel>) -> Self {
        let mut tls = Self::default();
        tls.key_repo(repo);
        tls.certificate_label(label);
        tls.key_repo_password(password.map(Into::into));
        tls
    }

    pub fn crypto_hardware(&mut self, hardware: Option<&CryptoHardware>) -> &mut Self {
        hardware
            .unwrap_or(&MqStr::empty())
            .copy_into_mqchar(&mut self.sco.CryptoHardware);
        self
    }

    pub fn certificate_label(&mut self, label: Option<&CertificateLabel>) -> &mut Self {
        label
            .unwrap_or(&MqStr::empty())
            .copy_into_mqchar(&mut self.sco.CertificateLabel);
        self
    }

    pub fn fips_required(&mut self, is_required: bool) -> &mut Self {
        self.sco.FipsRequired = if is_required {
            sys::MQSSL_FIPS_YES
        } else {
            sys::MQSSL_FIPS_NO
        };
        self
    }

    pub fn suite_b_policy(&mut self, policy: impl Into<[sys::MQLONG; 4]>) -> &mut Self {
        self.sco.EncryptionPolicySuiteB = policy.into();
        self
    }

    pub fn cert_val_policy(&mut self, policy: sys::MQLONG) -> &mut Self {
        self.sco.CertificateValPolicy = policy;
        self
    }

    pub fn key_reset_count(&mut self, count: sys::MQLONG) -> &mut Self {
        self.sco.KeyResetCount = count;
        self
    }

    pub fn key_repo_password(&mut self, password: Option<S>) -> &mut Self {
        self.key_repo_password = password;
        self
    }

    pub fn key_repo(&mut self, repo: &KeyRepo) -> &mut Self {
        repo.copy_into_mqchar(&mut self.sco.KeyRepository);
        self
    }
}

/// A builder for creating parameters to connect to an IBM MQ queue manager
#[derive(Debug, Clone, Default)]
#[must_use]
pub struct ConnectionOptions<C, D> {
    method: D,
    credentials: C,
    app_name: Option<ApplName>,
}

#[derive(Debug, Error)]
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

pub fn mqserver(server: &str) -> Result<(ChannelName, ConnectionName, sys::MQLONG), MqServerSyntaxError> {
    #[allow(clippy::unwrap_used)]
    let server_pattern = regex::Regex::new(r"^(.+)/(.+)/(.+)$").unwrap();

    if let Some((_, [channel, transport, connection_name])) = server_pattern.captures(server).map(|v| v.extract()) {
        let channel = channel
            .try_into()
            .ok()
            .filter(ChannelName::has_value)
            .ok_or_else(|| MqServerSyntaxError::ChannelFormat(channel.to_string()))?;
        let connection_name = connection_name
            .try_into()
            .ok()
            .filter(ConnectionName::has_value)
            .ok_or_else(|| MqServerSyntaxError::ConnectionNameFormat(connection_name.to_string()))?;
        let transport = match transport {
            "TCP" => Ok(sys::MQXPT_TCP),
            "LU62" => Ok(sys::MQXPT_LU62),
            "NETBIOS" => Ok(sys::MQXPT_NETBIOS),
            "SPX" => Ok(sys::MQXPT_SPX),
            other => Err(MqServerSyntaxError::UnrecognizedTransport(other.to_string())),
        }?;
        Ok((channel, connection_name, transport))
    } else {
        Err(MqServerSyntaxError::InvalidFormat)
    }
}

impl DefinitionMethod for MqStruct<'_, sys::MQCD> {
    type ScoBuilder = NoStruct;
    type CdBuilder = Self;
    type BnoBuilder = NoStruct;

    fn apply_cno<'ptr>(&'ptr self, cno: &mut MqStruct<'ptr, sys::MQCNO>) {
        cno.Options &= !(sys::MQCNO_LOCAL_BINDING | sys::MQCNO_CLIENT_BINDING); // Clear the BINDING bits
        cno.Options |= sys::MQCNO_CLIENT_BINDING;
    }

    fn sco(&self) -> &Self::ScoBuilder {
        &NoStruct
    }

    fn cd(&self) -> &Self::CdBuilder {
        self
    }

    fn bno(&self) -> &Self::BnoBuilder {
        &NoStruct
    }
}

impl<T> ClientDefinition<MqStruct<'_, sys::MQCD>, T> {
    pub fn cipher_spec(&mut self, spec: Option<&CipherSpec>) -> &mut Self {
        spec.unwrap_or(&CipherSpec::default())
            .copy_into_mqchar(&mut self.config.SSLCipherSpec);
        self
    }
}

impl<C, A> ClientDefinition<C, A> {
    pub fn tls_options<T: StructOptionBuilder<sys::MQSCO>>(self, options: T) -> ClientDefinition<C, T> {
        let Self { config, balance, .. } = self;
        ClientDefinition {
            config,
            tls: options,
            balance,
        }
    }

    #[must_use]
    pub fn balance_options<B: StructOptionBuilder<sys::MQBNO>>(
        self,
        options: Option<MqStruct<'static, sys::MQBNO>>,
    ) -> Self {
        let Self { config, tls, .. } = self;
        Self {
            config,
            tls,
            balance: options,
        }
    }
}

pub type AppDefinedClient<'ptr, T> = ClientDefinition<MqStruct<'ptr, sys::MQCD>, T>;

impl<'a, T: StructOptionBuilder<sys::MQSCO>> AppDefinedClient<'a, T> {
    #[must_use]
    pub fn tls(self, cipher: Option<&CipherSpec>, tls: T) -> Self {
        let mut mself = self;
        mself.cipher_spec(cipher);
        Self {
            config: mself.config,
            tls,
            balance: mself.balance,
        }
    }
}

impl<'ptr> AppDefinedClient<'ptr, NoStruct> {
    #[must_use]
    fn default_client() -> Self {
        Self::from_mqcd(MqStruct::new(sys::MQCD {
            Version: sys::MQCD_VERSION_12,
            ..sys::MQCD::client_conn_default()
        }))
    }

    #[must_use]
    pub const fn from_mqcd(config: MqStruct<'ptr, sys::MQCD>) -> Self {
        ClientDefinition {
            config,
            tls: NoStruct,
            balance: None,
        }
    }

    /// Create a channel definition (MQCD) from the minimal channel name, connection name and optional transport type.
    #[must_use]
    pub fn new_client(
        channel_name: &ChannelName,
        connection_name: &ConnectionName,
        transport: Option<sys::MQLONG>,
    ) -> Self {
        let mut outcome = Self::default_client();
        let mqcd = &mut outcome.config;
        if let Some(transport) = transport {
            mqcd.TransportType = transport;
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

#[derive(Debug, Clone)]
pub struct ConnectionOptionsOwned<Csp, Sco, Cd, Bno> {
    pub csp: Option<Csp>,
    pub sco: Option<Sco>,
    pub cd: Option<Cd>,
    pub bno: Option<Bno>,
    _pinned: PhantomPinned
}

// The outcome of a ConnectionOptions build needs to hold the SCO and CSP in a stable memory location - use PhantomPinned + Pin
type ConnectionOptionsBuild<Csp, Sco, Cd, Bno> =
    MqStructSelfRef<sys::MQCNO, Pin<Box<ConnectionOptionsOwned<Csp, Sco, Cd, Bno>>>>;

impl<C: StructOptionBuilder<sys::MQCSP>, D: DefinitionMethod> StructType<sys::MQCNO> for ConnectionOptions<C, D> {
    type Struct<'a> = ConnectionOptionsBuild<
        C::Struct<'a>,
        <D::ScoBuilder as StructType<sys::MQSCO>>::Struct<'a>,
        <D::CdBuilder as StructType<sys::MQCD>>::Struct<'a>,
        <D::BnoBuilder as StructType<sys::MQBNO>>::Struct<'a>
    > where Self: 'a;
}

impl<C: StructOptionBuilder<sys::MQCSP>, D: DefinitionMethod> StructBuilder<sys::MQCNO> for ConnectionOptions<C, D> {
    fn build(&self) -> Self::Struct<'_> {
        // Construct the MQCNO
        let mut cno = MqStruct::<sys::MQCNO>::default();
        cno.Version = sys::MQCNO_VERSION_8;
        cno.Options |= sys::MQCNO_GENERATE_CONN_TAG;

        let referee = Box::pin(ConnectionOptionsOwned {
            csp: self.credentials.option_build(),
            sco: self.method.sco().option_build(),
            cd: self.method.cd().option_build(),
            bno: self.method.bno().option_build(),
            _pinned: PhantomPinned,
        });
        // let (csp, sco, cd, bno, ..) = &*referee;
        if let Some(csp) = referee.csp.as_deref() {
            cno.attach_csp(csp);
        }
        if let Some(sco) = referee.sco.as_deref() {
            cno.attach_sco(sco);
        }
        if let Some(cd) =  referee.cd.as_deref() {
            cno.attach_cd(cd);
        }
        if let Some(bno) = referee.bno.as_deref() {
            cno.attach_bno(bno);
        }
        self.app_name
            .unwrap_or(MqStr::empty())
            .copy_into_mqchar(&mut cno.ApplName);
        self.method.apply_cno(&mut cno);

        MqStructSelfRef::new(*cno, referee)
    }
}

impl<D> ConnectionOptions<NoStruct, D> {
    pub const fn from_definition(method: D) -> Self {
        Self {
            method,
            credentials: NoStruct,
            app_name: None,
        }
    }
}

impl ConnectionOptions<NoStruct, DefaultBinding> {
    pub const fn default_binding() -> Self {
        Self::from_definition(ExternalConfig)
    }
}

impl ConnectionOptions<NoStruct, LocalBinding> {
    pub const fn local_binding() -> Self {
        Self::from_definition(ExternalConfig)
    }
}

impl ConnectionOptions<NoStruct, AppDefinedClient<'_, NoStruct>> {
    pub fn from_mqserver(mqserver: &str) -> Result<Self, MqServerSyntaxError> {
        Ok(Self::from_definition(ClientDefinition::from_mqserver(mqserver)?))
    }
}

impl<'a, C, T> ConnectionOptions<C, AppDefinedClient<'a, T>> {
    pub fn tls<Y: StructOptionBuilder<sys::MQSCO>>(
        self,
        cipher: &CipherSpec,
        options: Y,
    ) -> ConnectionOptions<C, AppDefinedClient<'a, Y>> {
        let Self {
            method,
            credentials,
            app_name,
        } = self;
        let mut method = method.tls_options(options);
        method.cipher_spec(Some(cipher));
        ConnectionOptions {
            method,
            credentials,
            app_name,
        }
    }
}

impl<C, D: DefinitionMethod, A> ConnectionOptions<C, ClientDefinition<D, A>> {
    pub fn tls_options<T: StructOptionBuilder<sys::MQSCO>>(
        self,
        options: T,
    ) -> ConnectionOptions<C, ClientDefinition<D, T>> {
        {
            ConnectionOptions {
                method: self.method.tls_options(options),
                credentials: self.credentials,
                app_name: self.app_name,
            }
        }
    }
}

impl<C, D> ConnectionOptions<C, D> {
    pub fn definition<E: DefinitionMethod>(self, method: E) -> ConnectionOptions<C, E> {
        let Self {
            credentials, app_name, ..
        } = self;
        ConnectionOptions {
            method,
            credentials,
            app_name,
        }
    }

    /// Use the supplied credentials to authenticate to the queue manager
    pub fn credentials<A: StructOptionBuilder<sys::MQCSP>>(self, credentials: A) -> ConnectionOptions<A, D> {
        let Self { method, app_name, .. } = self;
        ConnectionOptions {
            method,
            credentials,
            app_name,
        }
    }

    /// Set the application name for the connection. Setting `None` uses the default application name
    /// assigned by the IBM MQ libraries.
    pub fn application_name(self, name: Option<ApplName>) -> Self {
        Self { app_name: name, ..self }
    }
}

impl<C: StructOptionBuilder<sys::MQCSP>, D: DefinitionMethod> ConnectionOptions<C, D> {
    /// Execute a connection to MQ using the provided `qm_name` and the `ConnectionOptions` settings
    pub fn connect_lib<L: Library, H: HandleShare>(
        &self,
        lib: L,
        qm_name: Option<&QMName>,
    ) -> ResultComp<ConnectionShare<L, H>> {
        ConnectionShare::new_lib(lib, qm_name, self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{sys, Ccdt, DefinitionMethod, ProtectedSecret, Secret};

    use super::MqStruct;

    #[test]
    fn mqstructure() {
        #[derive(Default, Debug)]
        struct Test(u32);
        fn lt(_param: &Test) -> MqStruct<Test> {
            MqStruct::new(Test(1))
        }
        let b = Test(2);
        let c = lt(&b);
        assert_eq!(c.0, 1);
    }

    #[test]
    fn apply_cno() {
        let mut cno = MqStruct::<sys::MQCNO>::default();
        let b = Ccdt::new("bla");
        b.apply_cno(&mut cno);
        dbg!(cno);
    }

    #[test]
    fn protected_secret() {
        let c = ProtectedSecret::new("hello".to_string());
        assert_eq!(c.expose_secret(), "hello");
    }
}
