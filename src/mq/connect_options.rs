#![expect(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]
#![allow(non_snake_case)]

use std::any;

use libmqm_default as default;
use libmqm_sys::lib as sys;

use crate::{
    constants, conversion,
    macros::{all_multi_tuples, reverse_ident},
    prelude::*,
    structs, types, MqStr,
};

use super::{
    impl_min_version,
    types::{CertificateLabel, ChannelName, CipherSpec, ConnectionName, CryptoHardware, KeyRepo, QueueManagerName},
    ConnTag, ConnectParam, ConnectionId,
};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    derive_more::BitAnd,
    derive_more::BitAndAssign,
    derive_more::BitOr,
    derive_more::BitXorAssign,
)]
pub struct ConnectStructFlags(usize);

pub const CONNECT_HAS_NONE: ConnectStructFlags = ConnectStructFlags(0b00000);

/// A [`MQCNO`](libmqm_sys::lib::MQCNO) structure is required for the connection option
pub const CONNECT_HAS_CNO: ConnectStructFlags = ConnectStructFlags(0b00000);

/// A [`MQSCO`](libmqm_sys::lib::MQSCO) structure is required for the connection option
pub const CONNECT_HAS_SCO: ConnectStructFlags = ConnectStructFlags(0b00010);

/// A [`MQSCD`](libmqm_sys::lib::MQCD) structure is required for the connection option
pub const CONNECT_HAS_CD: ConnectStructFlags = ConnectStructFlags(0b00100);

/// A [`MQSCSP`](libmqm_sys::lib::MQCSP) structure is required for the connection option
pub const CONNECT_HAS_CSP: ConnectStructFlags = ConnectStructFlags(0b01000);

#[cfg(feature = "mqc_9_3_0_0")]
/// A [`MQBNO`](libmqm_sys::lib::MQBNO) structure is required for the connection option
pub const CONNECT_HAS_BNO: ConnectStructFlags = ConnectStructFlags(0b10000);

/// A collection of MQ structures used by MQ at connection time
#[derive(Debug, Clone)]
pub struct ConnectStructs<'ptr> {
    pub cno: structs::MQCNO<'ptr>,
    pub sco: structs::MQSCO<'ptr>,
    pub csp: structs::MQCSP<'ptr>,
    pub cd: structs::MQCD<'ptr>,
    #[cfg(feature = "mqc_9_3_0_0")]
    pub bno: structs::MQBNO,
}

/// A trait that manipulates the parameters to the [`mqconnx`](`crate::core::MqFunctions::mqconnx`) function
#[expect(unused_variables)]
#[diagnostic::on_unimplemented(
    message = "{Self} does not implement `ConnectOption` so it can't be used as an argument for MQI connect"
)]
/// # Safety
/// This trait can directly manipulate the [`MQCNO`](libmqm_sys::lib::MQCNO) structure which is used by [`MQCONNX`](libmqm_sys::Mqi::MQCONNX).
/// Incorrect values in the [`MQCONNX`](libmqm_sys::lib::MQCONNX) can lead to undefined behaviour.
///
/// Implementations of [`ConnectOption`] must ensure that pointers and offsets contained in the structure point to active data.
pub unsafe trait ConnectOption<'a> {
    /// Returns the queue manager name to connect to, or `None` to use the default queue manager name.
    #[inline]
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        None
    }

    /// Applies the type to to the structures contained in [`ConnectStructs`].
    ///
    /// Returns a mask indicating which structures are used by the type.
    #[inline]
    fn apply_param(&self, structs: &mut ConnectStructs<'a>) -> ConnectStructFlags {
        CONNECT_HAS_CNO
    }
}

#[expect(unused_parens)]
mod connect_impl {
    use crate::{ConnectValue, ConnectAttr, ConnectParam};
    use crate::ResultComp;
    use crate::prelude::*;
    use crate::macros::all_multi_tuples;

    macro_rules! impl_connectvalue_tuple {
        ([$first:ident, $($ty:ident),*]) => {
            #[expect(non_snake_case)]
            #[diagnostic::do_not_recommend]
            impl<S, $first, $($ty),*> ConnectValue<S> for ($first, $($ty),*)
            where
                $first: ConnectValue<S>,
                $($ty: ConnectAttr<S>),*
            {
                #[inline]
                fn connect_consume<'a, F>(param: &mut ConnectParam<'a>, connect: F) -> ResultComp<Self>
                where
                    F: FnOnce(&mut ConnectParam<'a>) -> ResultComp<S>,
                {
                    let mut rest_outer = None;
                    $first::connect_consume(param, |param| {
                        <($($ty),*) as ConnectAttr<S>>::connect_extract(param, connect).map_completion(|(rest, state)| {
                            rest_outer = Some(rest);
                            state
                        })
                    })
                    .map_completion(|a| {
                        let ($($ty),*) = rest_outer.expect("rest_outer should be set by the extract closure");
                        (a, $($ty),*)
                    })
                }
            }

        }
    }

    macro_rules! impl_connectattr_tuple {
        ([$first:ident, $($ty:ident),*]) => {
            #[expect(non_snake_case)]
            #[diagnostic::do_not_recommend]
            impl<S, $first, $($ty),*> ConnectAttr<S> for ($first, $($ty),*)
            where
                $first: ConnectAttr<S>,
                $($ty: ConnectAttr<S>),*
            {
                #[inline]
                fn connect_extract<'a, F>(param: &mut ConnectParam<'a>, mqi: F) -> ResultComp<(Self, S)>
                where
                    F: FnOnce(&mut ConnectParam<'a>) -> ResultComp<S>
                {
                    let mut rest_outer = None;
                    $first::connect_extract(param, |param| {
                        <($($ty),*) as ConnectAttr<S>>::connect_extract(param, mqi).map_completion(|(rest, state)| {
                            rest_outer = Some(rest);
                            state
                        })
                    })
                    .map_completion(|(a, s)| {
                        let ($($ty),*) = rest_outer.expect("rest_outer should be set by extract closure");
                        ((a, $($ty),*), s)
                    })
                }
            }
        }
    }

    all_multi_tuples!(impl_connectvalue_tuple);
    all_multi_tuples!(impl_connectattr_tuple);
}

unsafe impl<'a, O: ConnectOption<'a>> ConnectOption<'a> for Option<O> {
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        self.as_ref().and_then(|o| o.queue_manager_name())
    }

    fn apply_param(&self, structs: &mut ConnectStructs<'a>) -> ConnectStructFlags {
        self.as_ref().map_or(ConnectStructFlags(0), |o| o.apply_param(structs))
    }
}

impl Default for ConnectStructs<'_> {
    fn default() -> Self {
        Self {
            cno: structs::MQCNO::new(default::MQCNO_DEFAULT),
            sco: structs::MQSCO::new(default::MQSCO_DEFAULT),
            csp: structs::MQCSP::new(default::MQCSP_DEFAULT),
            cd: structs::MQCD::new(default::MQCD_CLIENT_CONN_DEFAULT),
            #[cfg(feature = "mqc_9_3_0_0")]
            bno: structs::MQBNO::new(default::MQBNO_DEFAULT),
        }
    }
}

impl_min_version!(['a], structs::MQSCO<'a>);
impl_min_version!(['a], structs::MQCD<'a>);
impl_min_version!(['a], structs::MQCNO<'a>);

/// Client Channel Definition Table URL connection option. Sets the connection as `MQCNO_CLIENT_BINDING`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::From)]
pub struct Ccdt<'url>(pub &'url str);

#[derive(Debug, Clone, Copy)]
pub struct MqServer<'m> {
    channel_name: &'m str,
    connection_name: &'m str,
    transport: types::MQXPT,
}

impl<'m> TryFrom<&'m str> for MqServer<'m> {
    type Error = MqServerSyntaxError;

    fn try_from(server: &'m str) -> Result<Self, Self::Error> {
        #[allow(clippy::unwrap_used)]
        let server_pattern = regex_lite::Regex::new(r"^(.{1,20}?)/(.+?)/(.{1,264}?)$").unwrap();

        match server_pattern.captures(server).map(|v| v.extract()) {
            Some((_, [channel, transport, connection_name])) => Ok(Self {
                channel_name: if channel.len() <= 20 {
                    Ok(channel)
                } else {
                    Err(MqServerSyntaxError::ChannelFormat(channel.to_string()))
                }?,
                connection_name: if connection_name.len() <= 264 {
                    Ok(connection_name)
                } else {
                    Err(MqServerSyntaxError::ConnectionNameFormat(connection_name.to_string()))
                }?,
                transport: match transport {
                    "TCP" => Ok(constants::MQXPT_TCP),
                    "LU62" => Ok(constants::MQXPT_LU62),
                    "NETBIOS" => Ok(constants::MQXPT_NETBIOS),
                    "SPX" => Ok(constants::MQXPT_SPX),
                    other => Err(MqServerSyntaxError::UnrecognizedTransport(other.to_string())),
                }?,
            }),
            _ => Err(MqServerSyntaxError::InvalidFormat),
        }
    }
}

unsafe impl<'m> ConnectOption<'m> for MqServer<'m> {
    fn apply_param(&self, ConnectStructs { cno, cd, .. }: &mut ConnectStructs<'m>) -> ConnectStructFlags {
        assert!(MqStr::assign(
            cd.ChannelName.as_mut(),
            conversion::slice_byte_to_mqchar(self.channel_name.as_bytes())
        ));

        assert!(MqStr::assign(
            cd.ConnectionName.as_mut(),
            conversion::slice_byte_to_mqchar(self.connection_name.as_bytes())
        ));
        *cd.TransportType.as_mut() = self.transport;
        let cno_options: &mut types::MQCNO = cno.Options.as_mut();
        cno_options.remove(constants::MQCNO_LOCAL_BINDING);
        cno_options.insert(constants::MQCNO_CLIENT_BINDING);
        CONNECT_HAS_CD
    }
}

/// Connection binding mode connection option. Represents the `MQCNO_*_BINDING` constants.
#[derive(Debug, Clone, Copy, Default)]
pub enum Binding {
    #[default]
    /// MQI default binding
    Default,
    /// Attempt a local connection (`MQCNO_LOCAL_BINDING`)
    Local,
    /// Attempt a client connection (`MQCNO_CLIENT_BINDING`)
    Client,
}

unsafe impl ConnectOption<'_> for Binding {
    fn apply_param(&self, structs: &mut ConnectStructs<'_>) -> ConnectStructFlags {
        let cno_options: &mut types::MQCNO = structs.cno.Options.as_mut();
        cno_options.remove(constants::MQCNO_CLIENT_BINDING | constants::MQCNO_LOCAL_BINDING);
        cno_options.insert(match self {
            Self::Default => constants::MQCNO_NONE,
            Self::Local => constants::MQCNO_LOCAL_BINDING,
            Self::Client => constants::MQCNO_CLIENT_BINDING,
        });
        CONNECT_HAS_CNO
    }
}

unsafe impl ConnectOption<'_> for QueueManagerName {
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        Some(self)
    }
}

#[cfg(feature = "mqc_9_3_0_0")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut)]
#[repr(transparent)]
pub struct InitialKeySecret<S>(S);

pub type InitialKey<S> = InitialKeySecret<ProtectedSecret<S>>;

#[derive(Default, Debug, Clone, Copy)]
pub enum CredentialsSecret<'cred, S> {
    #[default]
    Default,
    User(&'cred str, S),
    #[cfg(feature = "mqc_9_3_4_0")]
    Token(S),
}

pub type Credentials<'cred, S> = CredentialsSecret<'cred, ProtectedSecret<S>>;

#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct ProtectedSecret<T: ?Sized>(T);

impl<T> ProtectedSecret<T> {
    pub const fn new(secret: T) -> Self {
        Self(secret)
    }
}

/// Holds TLS parameters for use with [`connect`](crate::connect).
///
/// It is a wrapper around the [`MQSCO`](libmqm_sys::lib::MQSCO) structure.
#[derive(Debug, Clone)]
#[must_use]
pub struct Tls<'pw>(structs::MQSCO<'pw>, CipherSpec);

impl Default for Tls<'_> {
    fn default() -> Self {
        Self(structs::MQSCO::new(default::MQSCO_DEFAULT), CipherSpec::default())
    }
}

pub enum SuiteB {
    None,
    Min(usize),
}

impl From<SuiteB> for [types::MQ_SUITE; 4] {
    fn from(value: SuiteB) -> Self {
        const SIZED: &[(usize, types::MQ_SUITE)] = &[(128, constants::MQ_SUITE_B_128_BIT), (192, constants::MQ_SUITE_B_192_BIT)];
        match value {
            SuiteB::None => [
                constants::MQ_SUITE_B_NONE,
                constants::MQ_SUITE_B_NOT_AVAILABLE,
                constants::MQ_SUITE_B_NOT_AVAILABLE,
                constants::MQ_SUITE_B_NOT_AVAILABLE,
            ],
            SuiteB::Min(min_size) => {
                let mut result = [
                    constants::MQ_SUITE_B_NOT_AVAILABLE,
                    constants::MQ_SUITE_B_NOT_AVAILABLE,
                    constants::MQ_SUITE_B_NOT_AVAILABLE,
                    constants::MQ_SUITE_B_NOT_AVAILABLE,
                ];
                for (i, (.., suite)) in SIZED.iter().filter(|(size, ..)| *size >= min_size).enumerate() {
                    result[i] = *suite;
                }
                result
            }
        }
    }
}

#[allow(
    unknown_lints,
    clippy::needless_lifetimes,
    clippy::elidable_lifetime_names,
    reason = "pw lifetime is required for feature mqc_9_3_0_0"
)]
impl<'pw> Tls<'pw> {
    /// Create a TLS connection option for use with [`connect`](crate::connect) family of functions.
    ///
    /// # Example
    /// Create a TLS connection
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use mqi::types::{KeyRepo, CipherSpec};
    /// use mqi::{ThreadNone, mqstr};
    /// use mqi::connect_options::{MqServer, Tls};
    ///
    /// // Set up the Tls connection options
    /// let tls_options = Tls::new(
    ///     &KeyRepo(mqstr!("tls.kdb")), // Key repository for TLS
    ///     None, // No certificate label
    ///     &CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256")) // Cipher spec
    /// );
    /// // Connect to a remote server with TLS
    /// let connection = mqi::connect::<ThreadNone>(&(tls_options, MqServer::try_from("DEV.APP.SVRCONN/TCP/mq.example.com")?))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(repo: &KeyRepo, label: Option<&CertificateLabel>, cipher: &CipherSpec) -> Self {
        let mut tls = Self::default();
        tls.key_repo(repo);
        tls.certificate_label(label);
        cipher.clone_into(&mut tls.1);
        tls
    }

    pub fn crypto_hardware(&mut self, hardware: Option<&CryptoHardware>) -> &mut Self {
        match hardware {
            Some(ch) => ch.as_mqchar().clone_into(&mut self.0.CryptoHardware),
            None => MqStr::empty().as_mqchar().clone_into(&mut self.0.CryptoHardware),
        }
        self
    }

    pub fn certificate_label(&mut self, label: Option<&CertificateLabel>) -> &mut Self {
        self.0.set_min_version(sys::MQSCO_VERSION_5);
        match label {
            Some(cl) => cl.as_mqchar().clone_into(&mut self.0.CertificateLabel),
            None => MqStr::empty().as_mqchar().clone_into(&mut self.0.CertificateLabel),
        }
        self
    }

    pub fn fips_required(&mut self, is_required: bool) -> &mut Self {
        self.0.set_min_version(sys::MQSCO_VERSION_2);
        self.0.FipsRequired = if is_required {
            sys::MQSSL_FIPS_YES
        } else {
            sys::MQSSL_FIPS_NO
        };
        self
    }

    pub fn suite_b_policy(&mut self, policy: [types::MQLONG; 4]) -> &mut Self {
        self.0.set_min_version(sys::MQSCO_VERSION_3);
        self.0.EncryptionPolicySuiteB = policy;
        self
    }

    pub fn cert_val_policy(&mut self, policy: types::MQLONG) -> &mut Self {
        self.0.set_min_version(sys::MQSCO_VERSION_4);
        self.0.CertificateValPolicy = policy;
        self
    }

    pub fn key_reset_count(&mut self, count: types::MQLONG) -> &mut Self {
        self.0.set_min_version(sys::MQSCO_VERSION_2);
        self.0.KeyResetCount = count;
        self
    }

    #[cfg(feature = "mqc_9_3_0_0")]
    pub fn key_repo_password<S: Secret<'pw, str> + Copy>(&mut self, password: Option<S>) -> &mut Self {
        self.0.attach_repo_password(password);
        self
    }

    pub fn key_repo(&mut self, repo: &KeyRepo) -> &mut Self {
        repo.as_mqchar().clone_into(&mut self.0.KeyRepository);
        self
    }
}

unsafe impl ConnectOption<'_> for CipherSpec {
    fn apply_param(&self, structs: &mut ConnectStructs<'_>) -> ConnectStructFlags {
        structs.cd.set_min_version(sys::MQCD_VERSION_7);
        self.as_mqchar().clone_into(&mut structs.cd.SSLCipherSpec);
        CONNECT_HAS_CD
    }
}

#[cfg(feature = "mqc_9_3_0_0")]
unsafe impl<'a, T: Secret<'a, str> + Copy> ConnectOption<'a> for types::KeyRepoPassword<T> {
    fn apply_param(&self, structs: &mut ConnectStructs<'a>) -> ConnectStructFlags {
        structs.sco.attach_repo_password(Some(self.0));
        CONNECT_HAS_SCO
    }
}

unsafe impl<'tls> ConnectOption<'tls> for Tls<'tls> {
    fn apply_param(&self, structs: &mut ConnectStructs<'tls>) -> ConnectStructFlags {
        self.0.clone_into(&mut structs.sco);
        CONNECT_HAS_SCO | self.1.apply_param(structs)
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
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

#[cfg(feature = "mqc_9_3_0_0")]
unsafe impl<'cred, S: Secret<'cred, str>> ConnectOption<'cred> for InitialKeySecret<S> {
    fn apply_param(&self, structs: &mut ConnectStructs<'cred>) -> ConnectStructFlags {
        let initial_key = self.expose_secret();
        structs.csp.attach_initial_key(initial_key);

        CONNECT_HAS_CSP
    }
}

unsafe impl<'cred, S: Secret<'cred, str>> ConnectOption<'cred> for CredentialsSecret<'cred, S> {
    fn apply_param(&self, structs: &mut ConnectStructs<'cred>) -> ConnectStructFlags {
        let auth_type = structs.csp.AuthenticationType.as_mut();
        match &self {
            CredentialsSecret::Default => {
                // No authentication
                *auth_type = constants::MQCSP_AUTH_NONE;
            }
            CredentialsSecret::User(user, password) => {
                // UserId and Password authentication
                let password = password.expose_secret();
                *auth_type = constants::MQCSP_AUTH_USER_ID_AND_PWD;
                structs.csp.attach_password(password);
                structs.csp.attach_userid(user);
            }
            #[cfg(feature = "mqc_9_3_4_0")]
            CredentialsSecret::Token(token) => {
                // JWT authentication
                let token = token.expose_secret();
                *auth_type = constants::MQCSP_AUTH_ID_TOKEN;
                structs.csp.attach_token(token);
            }
        }

        CONNECT_HAS_CSP
    }
}

unsafe impl ConnectOption<'_> for types::MQCNO {
    fn apply_param(&self, structs: &mut ConnectStructs<'_>) -> ConnectStructFlags {
        let cno_options: &mut Self = structs.cno.Options.as_mut();
        cno_options.insert(*self);
        CONNECT_HAS_CNO
    }
}

unsafe impl ConnectOption<'_> for () {}

macro_rules! impl_connectoptions {
    ([$($ty:ident),*]) => {
        // reverse_ident macro is used to ensure right to left application of options
        #[allow(non_snake_case,unused_variables)]
        #[diagnostic::do_not_recommend]
        unsafe impl<'r, $($ty),*> ConnectOption<'r> for ($($ty),*)
        where
            $($ty: ConnectOption<'r>),*
        {
            fn queue_manager_name(&self) -> Option<&QueueManagerName> {
                let ($($ty),*) = self;
                $(
                    if let name @ Some(_) = $ty.queue_manager_name() {
                        return name;
                    }
                )*

                None
            }

            #[inline]
            fn apply_param(&self, structs: &mut ConnectStructs<'r>) -> ConnectStructFlags
            {
                let reverse_ident!($($ty),*) = self; // first is last, last is first
                $($ty.apply_param(structs))|*
            }
        }
    }
}

all_multi_tuples!(impl_connectoptions);

unsafe impl ConnectOption<'_> for types::ApplName {
    fn apply_param(&self, structs: &mut ConnectStructs<'_>) -> ConnectStructFlags {
        structs.cno.set_min_version(sys::MQCNO_VERSION_7);
        self.0.as_mqchar().clone_into(&mut structs.cno.ApplName);
        CONNECT_HAS_CNO
    }
}

unsafe impl<'url> ConnectOption<'url> for Ccdt<'url> {
    fn apply_param(&self, structs: &mut ConnectStructs<'url>) -> ConnectStructFlags {
        let cno_options: &mut types::MQCNO = structs.cno.Options.as_mut();
        cno_options.remove(constants::MQCNO_LOCAL_BINDING);
        cno_options.insert(constants::MQCNO_CLIENT_BINDING);
        structs.cno.attach_ccdt(self.0);

        CONNECT_HAS_CNO
    }
}

#[cfg(feature = "mqc_9_3_0_0")]
unsafe impl ConnectOption<'_> for structs::MQBNO {
    fn apply_param(&self, structs: &mut ConnectStructs<'_>) -> ConnectStructFlags {
        self.clone_into(&mut structs.bno);
        structs.cno.set_min_version(sys::MQCNO_VERSION_8);
        CONNECT_HAS_BNO
    }
}

unsafe impl<'csp> ConnectOption<'csp> for structs::MQCSP<'csp> {
    fn apply_param(&self, structs: &mut ConnectStructs<'csp>) -> ConnectStructFlags {
        self.clone_into(&mut structs.csp);
        structs.cno.set_min_version(sys::MQCNO_VERSION_5);
        CONNECT_HAS_CSP
    }
}

unsafe impl<'sco> ConnectOption<'sco> for structs::MQSCO<'sco> {
    fn apply_param(&self, structs: &mut ConnectStructs<'sco>) -> ConnectStructFlags {
        self.clone_into(&mut structs.sco);
        structs.cno.set_min_version(sys::MQCNO_VERSION_4);
        CONNECT_HAS_SCO
    }
}

unsafe impl<'cd> ConnectOption<'cd> for structs::MQCD<'cd> {
    fn apply_param(&self, structs: &mut ConnectStructs<'cd>) -> ConnectStructFlags {
        self.clone_into(&mut structs.cd);
        structs.cno.set_min_version(sys::MQCNO_VERSION_2);
        let cno_options: &mut types::MQCNO = structs.cno.Options.as_mut();
        cno_options.remove(constants::MQCNO_LOCAL_BINDING);
        cno_options.insert(constants::MQCNO_CLIENT_BINDING);
        CONNECT_HAS_CD
    }
}

impl<S> super::ConnectAttr<S> for ConnectionId {
    #[inline]
    fn connect_extract<'b, F>(param: &mut ConnectParam<'b>, connect: F) -> crate::ResultComp<(Self, S)>
    where
        F: FnOnce(&mut ConnectParam<'b>) -> crate::ResultComp<S>,
    {
        param.set_min_version(sys::MQCNO_VERSION_5);
        connect(param).map_completion(|state| (Self(param.ConnectionId), state))
    }
}

impl<S> super::ConnectAttr<S> for ConnTag {
    #[inline]
    fn connect_extract<'b, F>(param: &mut ConnectParam<'b>, connect: F) -> crate::ResultComp<(Self, S)>
    where
        F: FnOnce(&mut ConnectParam<'b>) -> crate::ResultComp<S>,
    {
        let cno_options: &mut types::MQCNO = param.Options.as_mut();
        cno_options.insert(constants::MQCNO_GENERATE_CONN_TAG);
        param.set_min_version(sys::MQCNO_VERSION_3);
        connect(param).map_completion(|state| (Self(param.ConnTag), state))
    }
}

pub fn mqserver(server: &str) -> Result<(ChannelName, ConnectionName, types::MQXPT), MqServerSyntaxError> {
    #[expect(clippy::unwrap_used)]
    let server_pattern = regex_lite::Regex::new(r"^(.+)/(.+)/(.+)$").unwrap();

    match server_pattern.captures(server).map(|v| v.extract()) {
        Some((_, [channel, transport, connection_name])) => {
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
                "TCP" => Ok(constants::MQXPT_TCP),
                "LU62" => Ok(constants::MQXPT_LU62),
                "NETBIOS" => Ok(constants::MQXPT_NETBIOS),
                "SPX" => Ok(constants::MQXPT_SPX),
                other => Err(MqServerSyntaxError::UnrecognizedTransport(other.to_string())),
            }?;
            Ok((channel, connection_name, transport))
        }
        _ => Err(MqServerSyntaxError::InvalidFormat),
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::types::MQXPT;
    use crate::constants;

    const CLIENT_MASK: types::MQCNO = types::MQCNO(sys::MQCNO_CLIENT_BINDING | sys::MQCNO_LOCAL_BINDING);

    const VALID: &[(&str, MQXPT, &str, &str)] = &[
        ("a", constants::MQXPT_TCP, "b", "a/TCP/b"),
        ("a", constants::MQXPT_SPX, "c", "a/SPX/c"),
        ("a", constants::MQXPT_LU62, "d", "a/LU62/d"),
        ("a", constants::MQXPT_NETBIOS, "e", "a/NETBIOS/e"),
    ];

    #[test]
    fn secret() {
        let x: ProtectedSecret<&str> = "hello".into();
        let _secret = x.expose_secret();
    }

    #[test]
    fn mqserver_parse() -> Result<(), MqServerSyntaxError> {
        for (channel, transport, connection, server) in VALID {
            let (m_channel, m_connection, m_transport) = mqserver(server)?;
            assert!(m_transport == *transport);
            assert!(m_channel.0 == *channel);
            assert!(m_connection.0 == *connection);
        }

        assert!(mqserver("a/BAD/c").is_err_and(|e| matches!(e, MqServerSyntaxError::UnrecognizedTransport(_))));
        assert!(mqserver("invalid").is_err_and(|e| matches!(e, MqServerSyntaxError::InvalidFormat)));

        Ok(())
    }

    #[test]
    fn mqserver_try_from() -> Result<(), MqServerSyntaxError> {
        for (v_channel, v_transport, v_connection, v_server) in VALID {
            let MqServer {
                channel_name,
                connection_name,
                transport,
            } = MqServer::try_from(*v_server)?;
            assert!(*v_transport == transport);
            assert!(channel_name == *v_channel);
            assert!(connection_name == *v_connection);
        }

        assert!(MqServer::try_from("a/BAD/c").is_err_and(|e| matches!(e, MqServerSyntaxError::UnrecognizedTransport(_))));
        assert!(MqServer::try_from("invalid").is_err_and(|e| matches!(e, MqServerSyntaxError::InvalidFormat)));
        Ok(())
    }

    #[test]
    fn connect_option_option() {
        struct NoExecuteConnectOptions;

        unsafe impl ConnectOption<'_> for NoExecuteConnectOptions {
            fn apply_param(&self, _structs: &mut ConnectStructs<'_>) -> ConnectStructFlags {
                panic!("Should not be called");
            }
            fn queue_manager_name(&self) -> Option<&QueueManagerName> {
                panic!("Should not be called");
            }
        }

        let mut cs = ConnectStructs::default();
        // Test that apply_param is not executed
        let none_options = None::<NoExecuteConnectOptions>;
        ConnectOption::apply_param(&none_options, &mut cs);
        ConnectOption::queue_manager_name(&none_options);
        // Test that apply_param is executed
        test_co(&Some(constants::MQCNO_RECONNECT), |_, _, cs| {
            assert!(types::MQCNO(cs.cno.Options).contains(constants::MQCNO_RECONNECT));
        });
    }

    #[test]
    fn binding() {
        test_co(&Binding::Client, |_, _, cs| {
            assert_eq!(
                types::MQCNO(cs.cno.Options).intersection(CLIENT_MASK),
                constants::MQCNO_CLIENT_BINDING
            );
        });
        test_co(&Binding::Default, |_, _, cs| {
            assert_eq!(types::MQCNO(cs.cno.Options).intersection(CLIENT_MASK), 0);
        });
        test_co(&Binding::Local, |_, _, cs| {
            assert_eq!(
                types::MQCNO(cs.cno.Options).intersection(CLIENT_MASK),
                constants::MQCNO_LOCAL_BINDING
            );
        });
    }

    #[test]
    fn cipher_spec() {
        const CIPHER: CipherSpec = CipherSpec(mqstr!("TLS_RSA_WITH_AES_128_CBC_SHA256"));
        test_co(&CIPHER, |bf, _, cs| {
            assert!(cs.cd.Version >= sys::MQCD_VERSION_7);
            assert_eq!(&cs.cd.SSLCipherSpec, CIPHER.as_mqchar());
            assert_eq!(bf & CONNECT_HAS_CD, CONNECT_HAS_CD);
        });
    }

    #[test]
    fn appl_name() {
        const APP: types::ApplName = types::ApplName(mqstr!("MYAPP"));
        test_co(&APP, |bf, _, cs| {
            assert!(cs.cno.Version >= sys::MQCNO_VERSION_7);
            assert_eq!(&cs.cno.ApplName, APP.as_mqchar());
            assert_eq!(bf & CONNECT_HAS_CNO, CONNECT_HAS_CNO);
        });
    }

    #[test]
    fn ccdt() {
        const CCDT: Ccdt = Ccdt("url");
        test_co(&CCDT, |bf, _, cs| {
            assert!(cs.cno.Version >= sys::MQCNO_VERSION_6);
            assert_eq!(
                types::MQCNO(cs.cno.Options).intersection(CLIENT_MASK),
                constants::MQCNO_CLIENT_BINDING
            );
            assert_eq!(bf & CONNECT_HAS_CNO, CONNECT_HAS_CNO);
            assert_eq!(cs.cno.CCDTUrlLength, 3);
            assert!(!cs.cno.CCDTUrlPtr.is_null());
        });
    }

    #[cfg(feature = "mqc_9_3_0_0")]
    #[test]
    fn initial_key() {
        let initial_key: InitialKey<_> = InitialKeySecret("key".into());
        test_co(&initial_key, |bf, _, cs| {
            assert_eq!(cs.csp.InitialKeyLength, 3);
            assert!(!cs.csp.InitialKeyPtr.is_null());
            assert_eq!(bf & CONNECT_HAS_CSP, CONNECT_HAS_CSP);
        });
    }

    #[test]
    fn credentials() {
        test_co(&Credentials::<'_, &str>::Default, |bf, _, cs| {
            assert_eq!(cs.csp.AuthenticationType, sys::MQCSP_AUTH_NONE);
            assert_eq!(bf & CONNECT_HAS_CSP, CONNECT_HAS_CSP);
        });
        test_co(&Credentials::User("user", "password".into()), |bf, _, cs| {
            assert_eq!(types::MQCSP(cs.csp.AuthenticationType), constants::MQCSP_AUTH_USER_ID_AND_PWD);
            assert_eq!(cs.csp.CSPUserIdLength, 4);
            assert!(!cs.csp.CSPUserIdPtr.is_null());
            assert_eq!(cs.csp.CSPPasswordLength, 8);
            assert!(!cs.csp.CSPPasswordPtr.is_null());
            assert_eq!(bf & CONNECT_HAS_CSP, CONNECT_HAS_CSP);
        });

        #[cfg(feature = "mqc_9_3_4_0")]
        {
            test_co(&Credentials::Token("token".into()), |bf, _, cs| {
                assert_eq!(cs.csp.AuthenticationType, sys::MQCSP_AUTH_ID_TOKEN);
                assert_eq!(cs.csp.TokenLength, 5);
                assert!(!cs.csp.TokenPtr.is_null());
                assert_eq!(bf & CONNECT_HAS_CSP, CONNECT_HAS_CSP);
            });
        }
    }

    #[test]
    fn mqserver_co() {
        const QM: QueueManagerName = QueueManagerName(mqstr!("MYQM"));
        test_co(&QM, |_, coqm, _| {
            assert!(coqm.is_some_and(|name| name == &QM));
        });
    }

    /// Test a `ConnectionOption`
    fn test_co<'a, F: FnOnce(ConnectStructFlags, Option<&QueueManagerName>, &ConnectStructs<'_>)>(
        co: &impl ConnectOption<'a>,
        f: F,
    ) {
        let mut cs = ConnectStructs::default();
        f(co.apply_param(&mut cs), co.queue_manager_name(), &cs);
    }
}
