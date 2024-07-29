use crate::{core::values, sys, MqMask, ReasonCode};
use std::str;

pub type CorrelationId = [u8; sys::MQ_CORREL_ID_LENGTH];
pub type MessageId = [u8; sys::MQ_MSG_ID_LENGTH];
pub type GroupId = [u8; sys::MQ_GROUP_ID_LENGTH];
pub type MsgToken = [u8; sys::MQ_MSG_TOKEN_LENGTH];

pub type StrucId = [u8; 4];
pub type Fmt = [u8; 8];

pub type Warning = (ReasonCode, &'static str);

#[derive(Clone, Debug)]
pub struct MessageFormat<T> {
    pub ccsid: sys::MQLONG,
    pub encoding: MqMask<values::MQENC>,
    pub format: T,
}