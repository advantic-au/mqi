use crate::{core::values, headers::TextEnc, sys, MqMask, MqStr, ReasonCode};
use std::str;

use super::put::PutResult;

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
    pub format: TextEnc<Fmt>,
}

impl From<MessageId> for CorrelationId {
    #[inline]
    fn from(value: MessageId) -> Self {
        Self(value.0)
    }
}

impl PutResult for MessageId {
    #[inline]
    fn create_from(md: &super::MqStruct<'static, sys::MQMD2>, _pmo: &super::MqStruct<'static, sys::MQPMO>, _warning: Option<Warning>) -> Self {
        Self(md.MsgId)
    }
}

impl PutResult for Option<CorrelationId> {
    #[inline]
    fn create_from(md: &super::MqStruct<'static, sys::MQMD2>, _pmo: &super::MqStruct<'static, sys::MQPMO>, _warning: Option<Warning>) -> Self {
        CorrelationId::new(md.CorrelId)
    }
}

impl PutResult for Option<UserIdentifier> {
    fn create_from(md: &super::MqStruct<'static, sys::MQMD2>, _pmo: &super::MqStruct<'static, sys::MQPMO>, _warning: Option<Warning>) -> Self {
        UserIdentifier::new(md.UserIdentifier)
    }
}

impl CorrelationId {
    #[must_use]
    pub fn new(source: [u8; sys::MQ_CORREL_ID_LENGTH]) -> Option<Self> {
        if source == sys::MQCI_NONE[..sys::MQ_CORREL_ID_LENGTH] {
            None
        }
        else {
            Some(Self(source))
        }
    }
}

impl UserIdentifier {
    #[must_use]
    pub fn new(source: [sys::MQCHAR; sys::MQ_USER_ID_LENGTH]) -> Option<Self > {
        Some(MqStr::from(source)).filter(MqStr::has_value).map(UserIdentifier)
    }
}
