use super::{InqReqType, InqReqItem};
use crate::sys;

pub const MQCA_ALTERATION_DATE: InqReqType = (sys::MQCA_ALTERATION_DATE, InqReqItem::Str(sys::MQ_DATE_LENGTH));
pub const MQCA_ALTERATION_TIME: InqReqType = (sys::MQCA_ALTERATION_TIME, InqReqItem::Str(sys::MQ_TIME_LENGTH));
pub const MQIA_CODED_CHAR_SET_ID: InqReqType = (sys::MQIA_CODED_CHAR_SET_ID, InqReqItem::Long);
pub const MQCA_Q_MGR_NAME: InqReqType = (sys::MQCA_Q_MGR_NAME, InqReqItem::Str(sys::MQ_Q_MGR_NAME_LENGTH));
pub const MQCA_DEF_XMIT_Q_NAME: InqReqType = (sys::MQCA_DEF_XMIT_Q_NAME, InqReqItem::Str(sys::MQ_Q_NAME_LENGTH));
