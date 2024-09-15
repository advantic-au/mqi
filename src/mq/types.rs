use crate::{core::values, headers::TextEnc, sys, MqStr, ReasonCode};
use std::{mem, str};

use super::{headers::fmt::MQFMT_NONE, MqStruct};

#[derive(Debug, Clone, Copy)]
pub struct CorrelationId(pub [u8; sys::MQ_CORREL_ID_LENGTH]);
#[derive(Debug, Clone, Copy)]
pub struct MessageId(pub [u8; sys::MQ_MSG_ID_LENGTH]);
#[derive(Debug, Clone, Copy)]
pub struct GroupId(pub [u8; sys::MQ_GROUP_ID_LENGTH]);
#[derive(Debug, Clone, Copy)]
pub struct MsgToken(pub [u8; sys::MQ_MSG_TOKEN_LENGTH]);

/// Delegates `FromStr` to wrapped type implementation
macro_rules! impl_from_str {
    ($i:ident, $ty:ty) => {
        impl std::str::FromStr for $i {
            type Err = <$ty as std::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(<$ty as std::str::FromStr>::from_str(s)?))
            }
        }
    };
}

pub(crate) use impl_from_str;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct UserIdentifier(pub MqStr<12>);
impl_from_str!(UserIdentifier, MqStr<12>);

pub type StrucId = [u8; 4];
pub type Fmt = [u8; 8];

pub type Warning = (ReasonCode, &'static str);

#[derive(Clone, Copy, Debug)]
pub struct MessageFormat {
    pub ccsid: sys::MQLONG,
    pub encoding: values::MQENC,
    pub fmt: TextEnc<Fmt>,
}

impl MessageFormat {
    #[must_use]
    pub fn from_mqmd2(md: &MqStruct<sys::MQMD2>) -> Self {
        Self {
            ccsid: md.CodedCharSetId,
            encoding: values::MQENC(md.Encoding),
            fmt: TextEnc::Ascii(unsafe { mem::transmute::<[sys::MQCHAR; 8], [u8; 8]>(md.Format) }),
        }
    }
}

pub const FORMAT_NONE: MessageFormat = MessageFormat {
    ccsid: 1208,
    encoding: values::MQENC(sys::MQENC_NATIVE),
    fmt: TextEnc::Ascii(MQFMT_NONE),
};

impl From<MessageId> for CorrelationId {
    #[inline]
    fn from(value: MessageId) -> Self {
        Self(value.0)
    }
}

impl CorrelationId {
    #[must_use]
    pub fn new(source: [u8; sys::MQ_CORREL_ID_LENGTH]) -> Option<Self> {
        if source == sys::MQCI_NONE[..sys::MQ_CORREL_ID_LENGTH] {
            None
        } else {
            Some(Self(source))
        }
    }
}

impl UserIdentifier {
    #[must_use]
    pub fn new(source: [sys::MQCHAR; sys::MQ_USER_ID_LENGTH]) -> Option<Self> {
        Some(MqStr::from(source)).filter(MqStr::has_value).map(UserIdentifier)
    }
}

pub type ObjectName = MqStr<48>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct ConnectionName(pub MqStr<264>);
impl_from_str!(ConnectionName, MqStr<264>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct ChannelName(pub MqStr<20>);
impl_from_str!(ChannelName, MqStr<20>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct QueueName(pub ObjectName);
impl_from_str!(QueueName, ObjectName);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct QueueManagerName(pub ObjectName);
impl_from_str!(QueueManagerName, ObjectName);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct CipherSpec(pub MqStr<32>);
impl_from_str!(CipherSpec, MqStr<32>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct KeyRepo(pub MqStr<256>);
impl_from_str!(KeyRepo, MqStr<256>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct CryptoHardware(pub MqStr<256>);
impl_from_str!(CryptoHardware, MqStr<256>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct CertificateLabel(pub MqStr<64>);
impl_from_str!(CertificateLabel, MqStr<64>);
