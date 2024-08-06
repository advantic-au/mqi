use std::ops::Deref;

use crate::{core::values, sys, MqMask, MqStr};

use super::MqStruct;

pub trait CnoOptions<'a> {
    fn apply_mqconnx<'ptr>(self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'a: 'ptr;
}

pub struct ApplName(pub MqStr<28>);
pub struct Ccdt<'url>(pub &'url str);

#[derive(Debug, Clone, Copy)]
pub enum Binding {
    Default,
    Local,
    Client,
}

impl CnoOptions<'_> for Binding {
    fn apply_mqconnx<'ptr>(self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
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

#[derive(Debug, Clone)]
#[must_use]
pub struct Tls<'pw>(MqStruct<'pw, sys::MQSCO>);

impl Default for Tls<'_> {
    fn default() -> Self {
        Self(MqStruct::new(sys::MQSCO {
            Version: sys::MQSCO_VERSION_6,
            ..sys::MQSCO::default()
        }))
    }
}

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
    pub fn new(repo: &KeyRepo, password: Option<&'pw str>, label: Option<&CertificateLabel>) -> Self {
        let mut tls = Self::default();
        tls.key_repo(repo);
        tls.certificate_label(label);
        tls.key_repo_password(password);
        tls
    }

    pub fn crypto_hardware(&mut self, hardware: Option<&CryptoHardware>) -> &mut Self {
        hardware
            .unwrap_or(&MqStr::empty())
            .copy_into_mqchar(&mut self.0.CryptoHardware);
        self
    }

    pub fn certificate_label(&mut self, label: Option<&CertificateLabel>) -> &mut Self {
        label
            .unwrap_or(&MqStr::empty())
            .copy_into_mqchar(&mut self.0.CertificateLabel);
        self
    }

    pub fn fips_required(&mut self, is_required: bool) -> &mut Self {
        self.0.FipsRequired = if is_required {
            sys::MQSSL_FIPS_YES
        } else {
            sys::MQSSL_FIPS_NO
        };
        self
    }

    pub fn suite_b_policy(&mut self, policy: impl Into<[sys::MQLONG; 4]>) -> &mut Self {
        self.0.EncryptionPolicySuiteB = policy.into();
        self
    }

    pub fn cert_val_policy(&mut self, policy: sys::MQLONG) -> &mut Self {
        self.0.CertificateValPolicy = policy;
        self
    }

    pub fn key_reset_count(&mut self, count: sys::MQLONG) -> &mut Self {
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

impl<'tls> CnoOptions<'tls> for &'tls Tls<'tls> {
    fn apply_mqconnx<'ptr>(self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'tls: 'ptr,
    {
        mqcno.attach_sco(&self.0);
    }
}

pub trait Secret<T: Deref> {
    #[must_use]
    fn expose_secret(&self) -> &T::Target;
}

impl<T: Deref> Secret<T> for ProtectedSecret<T> {
    fn expose_secret(&self) -> &T::Target {
        let Self(secret) = self;
        secret
    }
}

impl<T> std::fmt::Debug for ProtectedSecret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ProtectedSecret").field(&format_args!("[redacted]")).finish()
    }
}

impl<T> From<T> for ProtectedSecret<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[allow(clippy::field_reassign_with_default)]
impl<'cred, S> CredentialsSecret<'cred, S> {
    pub fn build_csp<T: Deref<Target = str>>(&self) -> MqStruct<sys::MQCSP>
    where
        S: Secret<T>,
    {
        let mut csp = MqStruct::<sys::MQCSP>::default();
        csp.Version = sys::MQCSP_VERSION_3;

        match self {
            Self::Default => {
                // No authentication
                csp.AuthenticationType = sys::MQCSP_AUTH_NONE;
            }
            Self::User(user, password, ..) => {
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

impl CnoOptions<'_> for MqMask<values::MQCNO> {
    fn apply_mqconnx<'ptr>(self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'static: 'ptr,
    {
        mqcno.Options |= self.value();
    }
}

impl CnoOptions<'_> for () {
    fn apply_mqconnx<'ptr>(self, _mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'static: 'ptr,
    {
    }
}

impl<'r, A: CnoOptions<'r>> CnoOptions<'r> for (A,) {
    fn apply_mqconnx<'ptr>(self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'r: 'ptr,
    {
        A::apply_mqconnx(self.0, mqcno);
    }
}

impl<'r, A: CnoOptions<'r>, B: CnoOptions<'r>> CnoOptions<'r> for (A, B) {
    fn apply_mqconnx<'ptr>(self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'r: 'ptr,
    {
        self.0.apply_mqconnx(mqcno);
        self.1.apply_mqconnx(mqcno);
    }
}

impl CnoOptions<'static> for &ApplName {
    fn apply_mqconnx<'ptr>(self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'static: 'ptr,
    {
        self.0.copy_into_mqchar(&mut mqcno.ApplName);
    }
}

impl<'url> CnoOptions<'url> for Ccdt<'url> {
    fn apply_mqconnx<'ptr>(self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'url: 'ptr,
    {
        mqcno.Options &= !sys::MQCNO_LOCAL_BINDING;
        mqcno.Options |= sys::MQCNO_CLIENT_BINDING;
        mqcno.attach_ccdt(self.0);
    }
}

impl<'bno> CnoOptions<'bno> for &'bno MqStruct<'bno, sys::MQBNO> {
    fn apply_mqconnx<'ptr>(self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'bno: 'ptr,
    {
        mqcno.attach_bno(self);
    }
}

impl<'data> CnoOptions<'data> for &'data MqStruct<'data, sys::MQCSP> {
    fn apply_mqconnx<'ptr>(self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'data: 'ptr,
    {
        mqcno.attach_csp(self);
    }
}

impl<'data> CnoOptions<'data> for &'data MqStruct<'data, sys::MQSCO> {
    fn apply_mqconnx<'ptr>(self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'data: 'ptr,
    {
        mqcno.attach_sco(self);
    }
}

impl<'data> CnoOptions<'data> for &'data MqStruct<'data, sys::MQCD> {
    fn apply_mqconnx<'ptr>(self, mqcno: &mut MqStruct<'ptr, sys::MQCNO>)
    where
        'data: 'ptr,
    {
        mqcno.Options &= !sys::MQCNO_LOCAL_BINDING;
        mqcno.Options |= sys::MQCNO_CLIENT_BINDING;
        mqcno.attach_cd(self);
    }
}
