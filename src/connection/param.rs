#![expect(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]
#![allow(non_snake_case)]

use libmqm_default as default;
use libmqm_sys as mq;

use super::option;
use crate::{
    MqStr, constants, conversion,
    macros::{all_multi_tuples, reverse_ident},
    prelude::*,
    result::ResultComp,
    structs,
    traits::Secret,
    types::{self, CertificateLabel, CipherSpec, CryptoHardware, KeyRepo, ProtectedSecret, QueueManagerName},
};

#[expect(unused_parens)]
mod connect_impl {
    use super::option::{ConnectAttr, ConnectParam, ConnectValue};
    use crate::{macros::all_multi_tuples, prelude::*, result::ResultComp};

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

unsafe impl<'a, O: option::ConnectOption<'a>> option::ConnectOption<'a> for Option<O> {
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        self.as_ref().and_then(|o| o.queue_manager_name())
    }

    fn apply_param(&self, structs: &mut option::ConnectStructs<'a>) -> option::ConnectStructFlags {
        self.as_ref().map_or(option::CONNECT_HAS_NONE, |o| o.apply_param(structs))
    }
}

impl Default for option::ConnectStructs<'_> {
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

structs::impl_min_version!(['a], structs::MQSCO<'a>);
structs::impl_min_version!(['a], structs::MQCD<'a>);
structs::impl_min_version!(['a], structs::MQCNO<'a>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Deref)]
pub struct ConnectionId(pub types::Identifier<24>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Deref)]
pub struct ConnTag(pub [types::MQBYTE; mq::MQ_CONN_TAG_LENGTH]);

/// Client Channel Definition Table URL connection option. Sets the connection as `MQCNO_CLIENT_BINDING`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::From)]
pub struct Ccdt<'url>(pub &'url str);

#[derive(Debug, Clone, Copy)]
pub struct MqServer<'m> {
    channel_name: &'m [types::MQCHAR],
    connection_name: &'m [types::MQCHAR],
    transport: types::MQXPT,
}

// Split a MQSERVER into its components
fn mqserver_parse(server: &str) -> Result<(&[types::MQCHAR], types::MQXPT, &[types::MQCHAR]), MqServerSyntaxError> {
    use conversion::slice_byte_to_mqchar as mqchar;
    let split: Vec<_> = server.split('/').take(4).collect();
    let (&[channel, transport, connection], rest) = split.split_first_chunk().ok_or(MqServerSyntaxError::InvalidFormat)?;
    if !rest.is_empty() {
        Err(MqServerSyntaxError::InvalidFormat)?;
    }

    if channel.len() > mq::MQ_CHANNEL_NAME_LENGTH {
        Err(MqServerSyntaxError::ChannelFormat(channel.to_string()))?;
    }

    if connection.len() > mq::MQ_CONN_NAME_LENGTH {
        Err(MqServerSyntaxError::ConnectionNameFormat(connection.to_string()))?;
    }

    let transport = match transport {
        "TCP" => Ok(constants::MQXPT_TCP),
        "LU62" => Ok(constants::MQXPT_LU62),
        "NETBIOS" => Ok(constants::MQXPT_NETBIOS),
        "SPX" => Ok(constants::MQXPT_SPX),
        other => Err(MqServerSyntaxError::UnrecognizedTransport(other.to_string())),
    }?;

    Ok((mqchar(channel.as_bytes()), transport, mqchar(connection.as_bytes())))
}

impl<'m> TryFrom<&'m str> for MqServer<'m> {
    type Error = MqServerSyntaxError;

    fn try_from(server: &'m str) -> Result<Self, Self::Error> {
        let (channel_name, transport, connection_name) = mqserver_parse(server)?;
        Ok(Self {
            channel_name,
            connection_name,
            transport,
        })
    }
}

impl std::fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(AsRef::<types::DisplayId<24>>::as_ref(&self.0), f)
    }
}

unsafe impl<'m> option::ConnectOption<'m> for MqServer<'m> {
    fn apply_param(&self, option::ConnectStructs { cno, cd, .. }: &mut option::ConnectStructs<'m>) -> option::ConnectStructFlags {
        assert!(MqStr::assign(cd.ChannelName.as_mut(), self.channel_name));

        assert!(MqStr::assign(cd.ConnectionName.as_mut(), self.connection_name));
        *cd.TransportType.as_mut() = self.transport;
        let cno_options: &mut types::MQCNO = cno.Options.as_mut();
        cno_options.remove(constants::MQCNO_LOCAL_BINDING);
        cno_options.insert(constants::MQCNO_CLIENT_BINDING);
        option::CONNECT_HAS_CD
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

unsafe impl option::ConnectOption<'_> for Binding {
    fn apply_param(&self, structs: &mut option::ConnectStructs<'_>) -> option::ConnectStructFlags {
        let cno_options: &mut types::MQCNO = structs.cno.Options.as_mut();
        cno_options.remove(constants::MQCNO_CLIENT_BINDING | constants::MQCNO_LOCAL_BINDING);
        cno_options.insert(match self {
            Self::Default => constants::MQCNO_NONE,
            Self::Local => constants::MQCNO_LOCAL_BINDING,
            Self::Client => constants::MQCNO_CLIENT_BINDING,
        });
        option::CONNECT_HAS_CNO
    }
}

unsafe impl option::ConnectOption<'_> for QueueManagerName {
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        Some(self)
    }
}

#[cfg(feature = "mqc_9_3_0_0")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut)]
#[repr(transparent)]
pub struct InitialKeySecret<S>(S);

#[cfg(feature = "mqc_9_3_0_0")]
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

/// Holds TLS parameters for use with [`connect`](crate::connection).
///
/// It is a wrapper around the [`MQSCO`](libmqm_sys::MQSCO) structure.
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
    /// Create a TLS connection option for use with [`connect`](crate::connection) family of functions.
    ///
    /// ## Example
    /// Create a TLS connection
    /// ```
    /// use mqi::types::{KeyRepo, CipherSpec};
    /// use mqi::connection::{ThreadNone, MqServer, Tls};
    /// use mqi::mqstr;
    ///
    /// // Set up the Tls connection options
    /// let tls_options = Tls::new(
    ///     &KeyRepo(mqstr!("tls.kdb")), // Key repository for TLS
    ///     None, // No certificate label
    ///     &CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256")) // Cipher spec
    /// );
    /// ```
    /// ```ignore
    /// // Example connect to a remote server with TLS
    /// let connection = mqi::connect::<ThreadNone>(&(
    ///     tls_options, // Apply it in the ConnectonOption tuple
    ///     MqServer::try_from("DEV.APP.SVRCONN/TCP/mq.example.com")?
    /// ))?;
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
        self.0.set_min_version(mq::MQSCO_VERSION_5);
        match label {
            Some(cl) => cl.as_mqchar().clone_into(&mut self.0.CertificateLabel),
            None => MqStr::empty().as_mqchar().clone_into(&mut self.0.CertificateLabel),
        }
        self
    }

    pub fn fips_required(&mut self, is_required: bool) -> &mut Self {
        self.0.set_min_version(mq::MQSCO_VERSION_2);
        self.0.FipsRequired = if is_required { mq::MQSSL_FIPS_YES } else { mq::MQSSL_FIPS_NO };
        self
    }

    pub fn suite_b_policy(&mut self, policy: [types::MQLONG; 4]) -> &mut Self {
        self.0.set_min_version(mq::MQSCO_VERSION_3);
        self.0.EncryptionPolicySuiteB = policy;
        self
    }

    pub fn cert_val_policy(&mut self, policy: types::MQLONG) -> &mut Self {
        self.0.set_min_version(mq::MQSCO_VERSION_4);
        self.0.CertificateValPolicy = policy;
        self
    }

    pub fn key_reset_count(&mut self, count: types::MQLONG) -> &mut Self {
        self.0.set_min_version(mq::MQSCO_VERSION_2);
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

unsafe impl option::ConnectOption<'_> for CipherSpec {
    fn apply_param(&self, structs: &mut option::ConnectStructs<'_>) -> option::ConnectStructFlags {
        structs.cd.set_min_version(mq::MQCD_VERSION_7);
        self.as_mqchar().clone_into(&mut structs.cd.SSLCipherSpec);
        option::CONNECT_HAS_CD
    }
}

#[cfg(feature = "mqc_9_3_0_0")]
unsafe impl<'a, T: Secret<'a, str> + Copy> option::ConnectOption<'a> for types::KeyRepoPassword<T> {
    fn apply_param(&self, structs: &mut option::ConnectStructs<'a>) -> option::ConnectStructFlags {
        structs.sco.attach_repo_password(Some(self.0));
        option::CONNECT_HAS_SCO
    }
}

unsafe impl<'tls> option::ConnectOption<'tls> for Tls<'tls> {
    fn apply_param(&self, structs: &mut option::ConnectStructs<'tls>) -> option::ConnectStructFlags {
        self.0.clone_into(&mut structs.sco);
        option::CONNECT_HAS_SCO | self.1.apply_param(structs)
    }
}

#[cfg(feature = "mqc_9_3_0_0")]
unsafe impl<'cred, S: Secret<'cred, str>> option::ConnectOption<'cred> for InitialKeySecret<S> {
    fn apply_param(&self, structs: &mut option::ConnectStructs<'cred>) -> option::ConnectStructFlags {
        let initial_key = self.expose_secret();
        structs.csp.attach_initial_key(initial_key);

        option::CONNECT_HAS_CSP
    }
}

unsafe impl<'cred, S: Secret<'cred, str>> option::ConnectOption<'cred> for CredentialsSecret<'cred, S> {
    fn apply_param(&self, structs: &mut option::ConnectStructs<'cred>) -> option::ConnectStructFlags {
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

        option::CONNECT_HAS_CSP
    }
}

unsafe impl option::ConnectOption<'_> for types::MQCNO {
    fn apply_param(&self, structs: &mut option::ConnectStructs<'_>) -> option::ConnectStructFlags {
        let cno_options: &mut Self = structs.cno.Options.as_mut();
        cno_options.insert(*self);
        option::CONNECT_HAS_CNO
    }
}

unsafe impl option::ConnectOption<'_> for () {}

macro_rules! impl_connectoptions {
    ([$($ty:ident),*]) => {
        // reverse_ident macro is used to ensure right to left application of options
        #[allow(non_snake_case,unused_variables)]
        #[diagnostic::do_not_recommend]
        unsafe impl<'r, $($ty),*> option::ConnectOption<'r> for ($($ty),*)
        where
            $($ty: option::ConnectOption<'r>),*
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
            fn apply_param(&self, structs: &mut option::ConnectStructs<'r>) -> option::ConnectStructFlags
            {
                let reverse_ident!($($ty),*) = self; // first is last, last is first
                $($ty.apply_param(structs))|*
            }
        }
    }
}

all_multi_tuples!(impl_connectoptions);

unsafe impl option::ConnectOption<'_> for types::ApplName {
    fn apply_param(&self, structs: &mut option::ConnectStructs<'_>) -> option::ConnectStructFlags {
        structs.cno.set_min_version(mq::MQCNO_VERSION_7);
        self.0.as_mqchar().clone_into(&mut structs.cno.ApplName);
        option::CONNECT_HAS_CNO
    }
}

unsafe impl<'url> option::ConnectOption<'url> for Ccdt<'url> {
    fn apply_param(&self, structs: &mut option::ConnectStructs<'url>) -> option::ConnectStructFlags {
        let cno_options: &mut types::MQCNO = structs.cno.Options.as_mut();
        cno_options.remove(constants::MQCNO_LOCAL_BINDING);
        cno_options.insert(constants::MQCNO_CLIENT_BINDING);
        structs.cno.attach_ccdt(self.0);

        option::CONNECT_HAS_CNO
    }
}

#[cfg(feature = "mqc_9_3_0_0")]
unsafe impl option::ConnectOption<'_> for structs::MQBNO {
    fn apply_param(&self, structs: &mut option::ConnectStructs<'_>) -> option::ConnectStructFlags {
        self.clone_into(&mut structs.bno);
        structs.cno.set_min_version(mq::MQCNO_VERSION_8);
        option::CONNECT_HAS_BNO
    }
}

unsafe impl<'csp> option::ConnectOption<'csp> for structs::MQCSP<'csp> {
    fn apply_param(&self, structs: &mut option::ConnectStructs<'csp>) -> option::ConnectStructFlags {
        self.clone_into(&mut structs.csp);
        structs.cno.set_min_version(mq::MQCNO_VERSION_5);
        option::CONNECT_HAS_CSP
    }
}

unsafe impl<'sco> option::ConnectOption<'sco> for structs::MQSCO<'sco> {
    fn apply_param(&self, structs: &mut option::ConnectStructs<'sco>) -> option::ConnectStructFlags {
        self.clone_into(&mut structs.sco);
        structs.cno.set_min_version(mq::MQCNO_VERSION_4);
        option::CONNECT_HAS_SCO
    }
}

unsafe impl<'cd> option::ConnectOption<'cd> for structs::MQCD<'cd> {
    fn apply_param(&self, structs: &mut option::ConnectStructs<'cd>) -> option::ConnectStructFlags {
        self.clone_into(&mut structs.cd);
        structs.cno.set_min_version(mq::MQCNO_VERSION_2);
        let cno_options: &mut types::MQCNO = structs.cno.Options.as_mut();
        cno_options.remove(constants::MQCNO_LOCAL_BINDING);
        cno_options.insert(constants::MQCNO_CLIENT_BINDING);
        option::CONNECT_HAS_CD
    }
}

impl<S> option::ConnectAttr<S> for ConnectionId {
    #[inline]
    fn connect_extract<'b, F>(param: &mut option::ConnectParam<'b>, connect: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut option::ConnectParam<'b>) -> ResultComp<S>,
    {
        param.set_min_version(mq::MQCNO_VERSION_5);
        connect(param).map_completion(|state| (Self(param.ConnectionId), state))
    }
}

impl<S> option::ConnectAttr<S> for ConnTag {
    #[inline]
    fn connect_extract<'b, F>(param: &mut option::ConnectParam<'b>, connect: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut option::ConnectParam<'b>) -> ResultComp<S>,
    {
        let cno_options: &mut types::MQCNO = param.Options.as_mut();
        cno_options.insert(constants::MQCNO_GENERATE_CONN_TAG);
        param.set_min_version(mq::MQCNO_VERSION_3);
        connect(param).map_completion(|state| (Self(param.ConnTag), state))
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
    use crate::{constants, types::MQXPT};

    const CLIENT_MASK: types::MQCNO = types::MQCNO(mq::MQCNO_CLIENT_BINDING | mq::MQCNO_LOCAL_BINDING);

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
    fn mqserver_try_from() -> Result<(), MqServerSyntaxError> {
        use conversion::slice_byte_to_mqchar as mqchar;
        for (v_channel, v_transport, v_connection, v_server) in VALID {
            let MqServer {
                channel_name,
                connection_name,
                transport,
            } = MqServer::try_from(*v_server)?;
            assert_eq!(*v_transport, transport);
            assert_eq!(channel_name, mqchar(v_channel.as_bytes()));
            assert_eq!(connection_name, mqchar(v_connection.as_bytes()));
        }

        assert!(MqServer::try_from("a/BAD/c").is_err_and(|e| matches!(e, MqServerSyntaxError::UnrecognizedTransport(_))));
        assert!(MqServer::try_from("a/b/c/d").is_err_and(|e| matches!(e, MqServerSyntaxError::InvalidFormat)));
        Ok(())
    }

    #[test]
    fn connect_option() {
        struct NoExecuteConnectOptions;

        unsafe impl option::ConnectOption<'_> for NoExecuteConnectOptions {
            fn apply_param(&self, _structs: &mut option::ConnectStructs<'_>) -> option::ConnectStructFlags {
                panic!("Should not be called");
            }
            fn queue_manager_name(&self) -> Option<&QueueManagerName> {
                panic!("Should not be called");
            }
        }

        let mut cs = option::ConnectStructs::default();
        // Test that apply_param is not executed
        let none_options = None::<NoExecuteConnectOptions>;
        option::ConnectOption::apply_param(&none_options, &mut cs);
        option::ConnectOption::queue_manager_name(&none_options);
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
            assert!(cs.cd.Version >= mq::MQCD_VERSION_7);
            assert_eq!(&cs.cd.SSLCipherSpec, CIPHER.as_mqchar());
            assert_eq!(bf & option::CONNECT_HAS_CD, option::CONNECT_HAS_CD);
        });
    }

    #[test]
    fn appl_name() {
        const APP: types::ApplName = types::ApplName(mqstr!("MYAPP"));
        test_co(&APP, |bf, _, cs| {
            assert!(cs.cno.Version >= mq::MQCNO_VERSION_7);
            assert_eq!(&cs.cno.ApplName, APP.as_mqchar());
            assert_eq!(bf & option::CONNECT_HAS_CNO, option::CONNECT_HAS_CNO);
        });
    }

    #[test]
    fn ccdt() {
        const CCDT: Ccdt = Ccdt("url");
        test_co(&CCDT, |bf, _, cs| {
            assert!(cs.cno.Version >= mq::MQCNO_VERSION_6);
            assert_eq!(
                types::MQCNO(cs.cno.Options).intersection(CLIENT_MASK),
                constants::MQCNO_CLIENT_BINDING
            );
            assert_eq!(bf & option::CONNECT_HAS_CNO, option::CONNECT_HAS_CNO);
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
            assert_eq!(bf & option::CONNECT_HAS_CSP, option::CONNECT_HAS_CSP);
        });
    }

    #[test]
    fn credentials() {
        test_co(&Credentials::<'_, &str>::Default, |bf, _, cs| {
            assert_eq!(cs.csp.AuthenticationType, mq::MQCSP_AUTH_NONE);
            assert_eq!(bf & option::CONNECT_HAS_CSP, option::CONNECT_HAS_CSP);
        });
        test_co(&Credentials::User("user", "password".into()), |bf, _, cs| {
            assert_eq!(types::MQCSP(cs.csp.AuthenticationType), constants::MQCSP_AUTH_USER_ID_AND_PWD);
            assert_eq!(cs.csp.CSPUserIdLength, 4);
            assert!(!cs.csp.CSPUserIdPtr.is_null());
            assert_eq!(cs.csp.CSPPasswordLength, 8);
            assert!(!cs.csp.CSPPasswordPtr.is_null());
            assert_eq!(bf & option::CONNECT_HAS_CSP, option::CONNECT_HAS_CSP);
        });

        #[cfg(feature = "mqc_9_3_4_0")]
        {
            test_co(&Credentials::Token("token".into()), |bf, _, cs| {
                assert_eq!(cs.csp.AuthenticationType, mq::MQCSP_AUTH_ID_TOKEN);
                assert_eq!(cs.csp.TokenLength, 5);
                assert!(!cs.csp.TokenPtr.is_null());
                assert_eq!(bf & option::CONNECT_HAS_CSP, option::CONNECT_HAS_CSP);
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
    fn test_co<'a, F: FnOnce(option::ConnectStructFlags, Option<&QueueManagerName>, &option::ConnectStructs<'_>)>(
        co: &impl option::ConnectOption<'a>,
        f: F,
    ) {
        let mut cs = option::ConnectStructs::default();
        f(co.apply_param(&mut cs), co.queue_manager_name(), &cs);
    }
}
