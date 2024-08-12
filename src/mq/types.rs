use crate::{core::values, headers::TextEnc, sys, MqMask, MqStr, ReasonCode};
use std::{cmp, str};

use super::connect_options::{self, ConnectOptions};

#[derive(Debug, Clone, Copy)]
pub struct CorrelationId(pub [u8; sys::MQ_CORREL_ID_LENGTH]);
#[derive(Debug, Clone, Copy)]
pub struct MessageId(pub [u8; sys::MQ_MSG_ID_LENGTH]);
#[derive(Debug, Clone, Copy)]
pub struct GroupId(pub [u8; sys::MQ_GROUP_ID_LENGTH]);
#[derive(Debug, Clone, Copy)]
pub struct MsgToken(pub [u8; sys::MQ_MSG_TOKEN_LENGTH]);

#[derive(Debug, Clone, Copy)]
pub struct UserIdentifier(pub MqStr<12>);

pub type StrucId = [u8; 4];
pub type Fmt = [u8; 8];

pub type Warning = (ReasonCode, &'static str);

#[derive(Clone, Copy, Debug)]
pub struct MessageFormat {
    pub ccsid: sys::MQLONG,
    pub encoding: MqMask<values::MQENC>,
    pub fmt: TextEnc<Fmt>,
}

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

#[derive(Debug, Clone, Copy, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct ConnectionName(pub MqStr<264>);

#[derive(Debug, Clone, Copy, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct ChannelName(pub MqStr<20>);

#[derive(Debug, Clone, Copy, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct QueueName(pub ObjectName);

#[derive(Debug, Clone, Copy, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct QueueManagerName(pub ObjectName);

#[derive(Debug, Clone, Copy, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct CipherSpec(pub MqStr<32>);

impl ConnectOptions<'_> for CipherSpec {
    const STRUCTS: i32 = connect_options::HAS_CD;

    fn apply_cd<'ptr>(&'ptr self, cd: &mut super::MqStruct<'ptr, sys::MQCD>)
    where
        'static: 'ptr,
    {
        cd.Version = cmp::max(sys::MQCD_VERSION_7, cd.Version);
        self.copy_into_mqchar(&mut cd.SSLCipherSpec);
    }
}
