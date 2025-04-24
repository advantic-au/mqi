#![expect(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]
#![allow(non_snake_case)]

use std::any;

use libmqm_default as default;

use crate::{
    conversion,
    macros::{all_multi_tuples, reverse_ident},
    prelude::*,
    constants, sys, MqStr, types,
};

use super::{
    impl_mqstruct_min_version,
    types::{
        impl_from_str, CertificateLabel, ChannelName, CipherSpec, ConnectionName, CryptoHardware, KeyRepo, QueueManagerName,
    },
    ConnTag, ConnectParam, ConnectionId, MqStruct,
};

/// A [`MQCNO`](sys::MQCNO) structure is required for the connection option
pub const HAS_CNO: i32 = 0b00000;
/// A [`MQSCO`](sys::MQSCO) structure is required for the connection option
pub const HAS_SCO: i32 = 0b00010;
/// A [`MQSCD`](sys::MQCD) structure is required for the connection option
pub const HAS_CD: i32 = 0b00100;
/// A [`MQSCSP`](sys::MQCSP) structure is required for the connection option
pub const HAS_CSP: i32 = 0b01000;
/// A [`MQBNO`](sys::MQBNO) structure is required for the connection option
#[cfg(feature = "mqc_9_3_0_0")]
pub const HAS_BNO: i32 = 0b10000;

/// A collection of MQ structures used by MQ at connection time
#[derive(Debug, Clone)]
pub struct ConnectStructs<'ptr> {
    pub cno: MqStruct<'ptr, sys::MQCNO>,
    pub sco: MqStruct<'ptr, sys::MQSCO>,
    pub csp: MqStruct<'ptr, sys::MQCSP>,
    pub cd: MqStruct<'ptr, sys::MQCD>,
    #[cfg(feature = "mqc_9_3_0_0")]
    pub bno: MqStruct<'ptr, sys::MQBNO>,
}

/// A trait that manipulates the parameters to the [`mqconnx`](`crate::core::MqFunctions::mqconnx`) function
#[expect(unused_variables)]
#[diagnostic::on_unimplemented(
    message = "{Self} does not implement `ConnectOption` so it can't be used as an argument for MQI connect"
)]
/*
 TODO: I don't believe I have this interface 100% correct. Lifetimes are not conducive
 to the goals I'm trying to achieve. Borrowing self may be better on apply_param.
*/
pub trait ConnectOption<'a> {
    /// Returns the queue manager name to connect to, or `None` to use the default queue manager name.
    #[inline]
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        None
    }

    /// Applies the type to to the structures contained in [`ConnectStructs`].
    ///
    /// Returns a mask indicating which structures are used by the type.
    #[inline]
    fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'a: 'ptr,
    {
        HAS_CNO
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

impl<'a, O: ConnectOption<'a>> ConnectOption<'a> for Option<O> {
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        self.as_ref().and_then(|o| o.queue_manager_name())
    }

    fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'a: 'ptr,
    {
        self.as_ref().map_or(0, |o| o.apply_param(structs))
    }
}

impl Default for ConnectStructs<'_> {
    fn default() -> Self {
        Self {
            cno: MqStruct::new(default::MQCNO_DEFAULT),
            sco: MqStruct::new(default::MQSCO_DEFAULT),
            csp: MqStruct::new(default::MQCSP_DEFAULT),
            cd: MqStruct::new(default::MQCD_CLIENT_CONN_DEFAULT),
            #[cfg(feature = "mqc_9_3_0_0")]
            bno: MqStruct::new(default::MQBNO_DEFAULT),
        }
    }
}

impl_mqstruct_min_version!(sys::MQSCO);
impl_mqstruct_min_version!(sys::MQCD);
impl_mqstruct_min_version!(sys::MQCNO);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct ApplName(pub MqStr<28>);
impl_from_str!(ApplName, MqStr<28>);

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

impl<'m> ConnectOption<'m> for MqServer<'m> {
    fn apply_param<'ptr>(&self, ConnectStructs { cno, cd, .. }: &mut ConnectStructs<'ptr>) -> i32
    where
        'm: 'ptr,
    {
        assert!(MqStr::assign(
            cd.ChannelName.as_mut(),
            conversion::slice_byte_to_mqchar(self.channel_name.as_bytes())
        ));

        assert!(MqStr::assign(
            cd.ConnectionName.as_mut(),
            conversion::slice_byte_to_mqchar(self.connection_name.as_bytes())
        ));
        cd.TransportType = self.transport.0;
        cno.Options &= !sys::MQCNO_LOCAL_BINDING;
        cno.Options |= sys::MQCNO_CLIENT_BINDING;
        HAS_CD
    }
}

/// Connection binding mode connection option. Represents the `MQCNO_*_BINDING` constants.
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
    fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
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
    #[cfg(feature = "mqc_9_3_4_0")]
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

#[derive(Debug, Clone)]
#[must_use]
pub struct Tls<'pw>(MqStruct<'pw, sys::MQSCO>, CipherSpec);

impl Default for Tls<'_> {
    fn default() -> Self {
        Self(MqStruct::new(default::MQSCO_DEFAULT), CipherSpec::default())
    }
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

#[allow(
    unknown_lints,
    clippy::needless_lifetimes,
    clippy::elidable_lifetime_names,
    reason = "pw lifetime is required for feature mqc_9_3_0_0"
)]
impl<'pw> Tls<'pw> {
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

    pub fn suite_b_policy(&mut self, policy: [sys::MQLONG; 4]) -> &mut Self {
        self.0.set_min_version(sys::MQSCO_VERSION_3);
        self.0.EncryptionPolicySuiteB = policy;
        self
    }

    pub fn cert_val_policy(&mut self, policy: sys::MQLONG) -> &mut Self {
        self.0.set_min_version(sys::MQSCO_VERSION_4);
        self.0.CertificateValPolicy = policy;
        self
    }

    pub fn key_reset_count(&mut self, count: sys::MQLONG) -> &mut Self {
        self.0.set_min_version(sys::MQSCO_VERSION_2);
        self.0.KeyResetCount = count;
        self
    }

    #[cfg(feature = "mqc_9_3_0_0")]
    pub fn key_repo_password(&mut self, password: Option<&'pw str>) -> &mut Self {
        self.0.attach_repo_password(password);
        self
    }

    pub fn key_repo(&mut self, repo: &KeyRepo) -> &mut Self {
        repo.as_mqchar().clone_into(&mut self.0.KeyRepository);
        self
    }
}

impl ConnectOption<'_> for CipherSpec {
    fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'static: 'ptr,
    {
        structs.cd.set_min_version(sys::MQCD_VERSION_7);
        self.as_mqchar().clone_into(&mut structs.cd.SSLCipherSpec);
        HAS_CD
    }
}

impl<'tls> ConnectOption<'tls> for Tls<'tls> {
    fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
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

impl<'cred, S: Secret<'cred, str>> ConnectOption<'cred> for CredentialsSecret<'cred, S> {
    fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
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
            #[cfg(feature = "mqc_9_3_4_0")]
            CredentialsSecret::Token(token, ..) => {
                // JWT authentication
                let token = token.expose_secret();
                structs.csp.AuthenticationType = sys::MQCSP_AUTH_ID_TOKEN;
                structs.csp.attach_token(token);
            }
        }

        // Populate the initial key
        #[cfg(feature = "mqc_9_3_0_0")]
        if let CredentialsSecret::User(.., Some(initial_key)) = &self {
            let initial_key = initial_key.expose_secret();
            structs.csp.attach_initial_key(initial_key);
        }

        #[cfg(feature = "mqc_9_3_4_0")]
        if let CredentialsSecret::Token(.., Some(initial_key)) = &self {
            let initial_key = initial_key.expose_secret();
            structs.csp.attach_initial_key(initial_key);
        }

        HAS_CSP
    }
}

impl ConnectOption<'_> for types::MQCNO {
    fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'static: 'ptr,
    {
        structs.cno.Options |= self.0;
        HAS_CNO
    }
}

impl ConnectOption<'_> for () {}

macro_rules! impl_connectoptions {
    ([$($ty:ident),*]) => {
        // reverse_ident macro is used to ensure right to left application of options
        #[allow(non_snake_case,unused_variables)]
        #[diagnostic::do_not_recommend]
        impl<'r, $($ty),*> ConnectOption<'r> for ($($ty),*)
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
            fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
            where
                'r: 'ptr,
            {
                let reverse_ident!($($ty),*) = self; // first is last, last is first
                $($ty.apply_param(structs))|*
            }
        }
    }
}

all_multi_tuples!(impl_connectoptions);

impl ConnectOption<'_> for ApplName {
    fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'static: 'ptr,
    {
        structs.cno.set_min_version(sys::MQCNO_VERSION_7);
        self.0.as_mqchar().clone_into(&mut structs.cno.ApplName);
        HAS_CNO
    }
}

impl<'url> ConnectOption<'url> for Ccdt<'url> {
    fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'url: 'ptr,
    {
        structs.cno.Options &= !sys::MQCNO_LOCAL_BINDING;
        structs.cno.Options |= sys::MQCNO_CLIENT_BINDING;
        structs.cno.attach_ccdt(self.0);

        HAS_CNO
    }
}

#[cfg(feature = "mqc_9_3_0_0")]
impl<'bno> ConnectOption<'bno> for MqStruct<'bno, sys::MQBNO> {
    fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'bno: 'ptr,
    {
        self.clone_into(&mut structs.bno);
        structs.cno.set_min_version(sys::MQCNO_VERSION_8);
        HAS_BNO
    }
}

impl<'csp> ConnectOption<'csp> for MqStruct<'csp, sys::MQCSP> {
    fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'csp: 'ptr,
    {
        self.clone_into(&mut structs.csp);
        structs.cno.set_min_version(sys::MQCNO_VERSION_5);
        HAS_CSP
    }
}

impl<'sco> ConnectOption<'sco> for MqStruct<'sco, sys::MQSCO> {
    fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'sco: 'ptr,
    {
        self.clone_into(&mut structs.sco);
        structs.cno.set_min_version(sys::MQCNO_VERSION_4);
        HAS_SCO
    }
}

impl<'cd> ConnectOption<'cd> for MqStruct<'cd, sys::MQCD> {
    fn apply_param<'ptr>(&self, structs: &mut ConnectStructs<'ptr>) -> i32
    where
        'cd: 'ptr,
    {
        self.clone_into(&mut structs.cd);
        structs.cno.set_min_version(sys::MQCNO_VERSION_2);
        structs.cno.Options &= !sys::MQCNO_LOCAL_BINDING;
        structs.cno.Options |= sys::MQCNO_CLIENT_BINDING;
        HAS_CD
    }
}

impl<S> super::ConnectAttr<S> for ConnectionId {
    #[inline]
    fn connect_extract<'b, F>(param: &mut ConnectParam<'b>, connect: F) -> crate::ResultComp<(Self, S)>
    where
        F: FnOnce(&mut ConnectParam<'b>) -> crate::ResultComp<S>,
    {
        param.set_min_version(sys::MQCNO_VERSION_5);
        connect(param).map_completion(|state| (Self(param.ConnectionId.into()), state))
    }
}

impl<S> super::ConnectAttr<S> for ConnTag {
    #[inline]
    fn connect_extract<'b, F>(param: &mut ConnectParam<'b>, connect: F) -> crate::ResultComp<(Self, S)>
    where
        F: FnOnce(&mut ConnectParam<'b>) -> crate::ResultComp<S>,
    {
        param.Options |= sys::MQCNO_GENERATE_CONN_TAG;
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
    use crate::types::MQXPT;
    use crate::constants;

    const CLIENT_MASK: sys::MQLONG = sys::MQCNO_CLIENT_BINDING | sys::MQCNO_LOCAL_BINDING;

    const VALID: &[(&str, MQXPT, &str, &str)] = &[
        ("a", constants::MQXPT_TCP, "b", "a/TCP/b"),
        ("a", constants::MQXPT_SPX, "c", "a/SPX/c"),
        ("a", constants::MQXPT_LU62, "d", "a/LU62/d"),
        ("a", constants::MQXPT_NETBIOS, "e", "a/NETBIOS/e"),
    ];

    use super::*;

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

        impl ConnectOption<'_> for NoExecuteConnectOptions {
            fn apply_param<'ptr>(&self, _structs: &mut ConnectStructs<'ptr>) -> i32
            where
                'static: 'ptr,
            {
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
            assert!(cs.cno.Options & sys::MQCNO_RECONNECT != 0);
        });
    }

    #[test]
    fn binding() {
        test_co(&Binding::Client, |_, _, cs| {
            assert_eq!(cs.cno.Options & CLIENT_MASK, sys::MQCNO_CLIENT_BINDING);
        });
        test_co(&Binding::Default, |_, _, cs| assert_eq!(cs.cno.Options & CLIENT_MASK, 0));
        test_co(&Binding::Local, |_, _, cs| {
            assert_eq!(cs.cno.Options & CLIENT_MASK, sys::MQCNO_LOCAL_BINDING);
        });
    }

    #[test]
    fn cipher_spec() {
        const CIPHER: CipherSpec = CipherSpec(mqstr!("TLS_RSA_WITH_AES_128_CBC_SHA256"));
        test_co(&CIPHER, |bf, _, cs| {
            assert!(cs.cd.Version >= sys::MQCD_VERSION_7);
            assert_eq!(&cs.cd.SSLCipherSpec, CIPHER.as_mqchar());
            assert_eq!(bf & HAS_CD, HAS_CD);
        });
    }

    #[test]
    fn appl_name() {
        const APP: ApplName = ApplName(mqstr!("MYAPP"));
        test_co(&APP, |bf, _, cs| {
            assert!(cs.cno.Version >= sys::MQCNO_VERSION_7);
            assert_eq!(&cs.cno.ApplName, APP.as_mqchar());
            assert_eq!(bf & HAS_CNO, HAS_CNO);
        });
    }

    #[test]
    fn ccdt() {
        const CCDT: Ccdt = Ccdt("url");
        test_co(&CCDT, |bf, _, cs| {
            assert!(cs.cno.Version >= sys::MQCNO_VERSION_6);
            assert_eq!(cs.cno.Options & CLIENT_MASK, sys::MQCNO_CLIENT_BINDING);
            assert_eq!(bf & HAS_CNO, HAS_CNO);
            assert_eq!(cs.cno.CCDTUrlLength, 3);
            assert!(!cs.cno.CCDTUrlPtr.is_null());
        });
    }

    #[test]
    fn credentials() {
        test_co(&Credentials::<'_, &str>::Default, |bf, _, cs| {
            assert_eq!(cs.csp.AuthenticationType, sys::MQCSP_AUTH_NONE);
            assert_eq!(bf & HAS_CSP, HAS_CSP);
        });
        test_co(&Credentials::user("user", "password"), |bf, _, cs| {
            assert_eq!(cs.csp.AuthenticationType, sys::MQCSP_AUTH_USER_ID_AND_PWD);
            assert_eq!(cs.csp.CSPUserIdLength, 4);
            assert!(!cs.csp.CSPUserIdPtr.is_null());
            assert_eq!(cs.csp.CSPPasswordLength, 8);
            assert!(!cs.csp.CSPPasswordPtr.is_null());
            assert_eq!(bf & HAS_CSP, HAS_CSP);
        });
        test_co(
            &Credentials::User("user", "password".into(), Some("key".into())),
            |_, _, cs| {
                assert_eq!(cs.csp.InitialKeyLength, 3);
                assert!(!cs.csp.InitialKeyPtr.is_null());
            },
        );
        #[cfg(feature = "mqc_9_3_4_0")]
        {
            test_co(&Credentials::Token("token".into(), None), |bf, _, cs| {
                assert_eq!(cs.csp.AuthenticationType, sys::MQCSP_AUTH_ID_TOKEN);
                assert_eq!(cs.csp.TokenLength, 5);
                assert!(!cs.csp.TokenPtr.is_null());
                assert_eq!(bf & HAS_CSP, HAS_CSP);
            });
            test_co(&Credentials::Token("token".into(), Some("key".into())), |_, _, cs| {
                assert_eq!(cs.csp.InitialKeyLength, 3);
                assert!(!cs.csp.InitialKeyPtr.is_null());
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
    fn test_co<'a, F: FnOnce(i32, Option<&QueueManagerName>, &ConnectStructs<'_>)>(co: &impl ConnectOption<'a>, f: F) {
        let mut cs = ConnectStructs::default();
        f(co.apply_param(&mut cs), co.queue_manager_name(), &cs);
    }
}
