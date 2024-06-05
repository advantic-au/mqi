use crate::{sys, MqStr, MqValue, QMName};
use super::Selector;

pub const MQCA_Q_NAME: Selector<QMName> = Selector::new(MqValue::new(sys::MQCA_Q_NAME));
pub const MQCA_APPL_ID: Selector<MqStr<256>> = Selector::new(MqValue::new(sys::MQCA_APPL_ID));
pub const MQCA_VERSION: Selector<MqStr<8>> = Selector::new(MqValue::new(sys::MQCA_VERSION));
pub const MQCA_ALTERATION_DATE: Selector<MqStr<12>> = Selector::new(MqValue::new(sys::MQCA_ALTERATION_DATE));
pub const MQCA_ALTERATION_TIME: Selector<MqStr<8>> = Selector::new(MqValue::new(sys::MQCA_ALTERATION_TIME));
pub const MQCA_Q_DESC: Selector<MqStr<64>> = Selector::new(MqValue::new(sys::MQCA_Q_DESC));
pub const MQCA_Q_MGR_DESC: Selector<MqStr<64>> = Selector::new(MqValue::new(sys::MQCA_Q_MGR_DESC)); // TODO: CCSID from qmgr?
pub const MQIA_CURRENT_Q_DEPTH: Selector<sys::MQLONG> = Selector::new(MqValue::new(sys::MQIA_CURRENT_Q_DEPTH));
pub const MQIA_CODED_CHAR_SET_ID: Selector<sys::MQLONG> = Selector::new(MqValue::new(sys::MQIA_CODED_CHAR_SET_ID));
pub const MQIA_Q_TYPE: Selector<sys::MQLONG> = Selector::new(MqValue::new(sys::MQIA_Q_TYPE));
pub const MQIA_PAGESET_ID: Selector<sys::MQLONG> = Selector::new(MqValue::new(sys::MQIA_PAGESET_ID));
