use crate::{sys, ReasonCode};

pub type CorrelationId = [u8; sys::MQ_CORREL_ID_LENGTH];
pub type MessageId = [u8; sys::MQ_MSG_ID_LENGTH];
pub type GroupId = [u8; sys::MQ_GROUP_ID_LENGTH];
pub type MsgToken = [u8; sys::MQ_MSG_TOKEN_LENGTH];

pub type Warning = (ReasonCode, &'static str);
