use super::Selector;
use crate::{core::MQOO, mapping, sys, MqStr, MqValue, QMName};

type SelectorStr<const N: usize> = Selector<MqStr<N>>;
type SelectorValue<T> = Selector<MqValue<T>>;
type SelectorLong = Selector<sys::MQLONG>;


#[macro_export]
macro_rules! define_mqvalue {
    ($i:ident, $source:path) => {
        #[derive(Copy, Clone)]
		pub struct $i;
		$crate::impl_constant_lookup!($i, $source);
    };
}

define_mqvalue!(MQQT, mapping::MQQT_CONST);
define_mqvalue!(MQAT, mapping::MQAT_CONST);

pub const MQIA_DEF_INPUT_OPEN_OPTION: SelectorValue<MQOO> = Selector::from(MqValue::from(sys::MQIA_DEF_INPUT_OPEN_OPTION));

pub const MQCA_Q_NAME: Selector<QMName> = Selector::from(MqValue::from(sys::MQCA_Q_NAME));
pub const MQCA_APPL_ID: SelectorStr<256> = Selector::from(MqValue::from(sys::MQCA_APPL_ID));
pub const MQCA_VERSION: SelectorStr<8> = Selector::from(MqValue::from(sys::MQCA_VERSION));
pub const MQCA_ALTERATION_DATE: SelectorStr<12> = Selector::from(MqValue::from(sys::MQCA_ALTERATION_DATE));
pub const MQCA_ALTERATION_TIME: SelectorStr<8> = Selector::from(MqValue::from(sys::MQCA_ALTERATION_TIME));
pub const MQCA_Q_DESC: SelectorStr<64> = Selector::from(MqValue::from(sys::MQCA_Q_DESC));
pub const MQCA_Q_MGR_DESC: SelectorStr<64> = Selector::from(MqValue::from(sys::MQCA_Q_MGR_DESC)); // TODO: CCSID from qmgr?

pub const MQIA_CURRENT_Q_DEPTH: SelectorLong = Selector::from(MqValue::from(sys::MQIA_CURRENT_Q_DEPTH));
pub const MQIA_CODED_CHAR_SET_ID: SelectorLong = Selector::from(MqValue::from(sys::MQIA_CODED_CHAR_SET_ID));
pub const MQIA_Q_TYPE: SelectorValue<MQQT> = Selector::from(MqValue::from(sys::MQIA_Q_TYPE));
pub const MQIA_PAGESET_ID: SelectorLong = Selector::from(MqValue::from(sys::MQIA_PAGESET_ID));