use libmqm_sys::lib as sys;

use crate::MqStruct;

pub type MQMD = MqStruct<'static, sys::MQMD>;
pub type MQMD1 = MqStruct<'static, sys::MQMD1>;
pub type MQMD2 = MqStruct<'static, sys::MQMD2>;

pub type MQGMO = MqStruct<'static, sys::MQGMO>;
pub type MQPMO<'a> = MqStruct<'a, sys::MQPMO>;

pub type MQCNO<'a> = MqStruct<'a, sys::MQCNO>;
pub type MQSCO<'a> = MqStruct<'a, sys::MQSCO>;
pub type MQCSP<'a> = MqStruct<'a, sys::MQCSP>;
pub type MQCD<'a> = MqStruct<'a, sys::MQCD>;
#[cfg(feature = "mqc_9_3_0_0")]
pub type MQBNO = MqStruct<'static, sys::MQBNO>;

pub type MQSD<'a> = MqStruct<'a, sys::MQSD>;
pub type MQOD<'a> = MqStruct<'a, sys::MQOD>;
pub type MQAIR<'a> = MqStruct<'a, sys::MQAIR>;

pub type MQCHARV<'a> = MqStruct<'a, sys::MQCHARV>;

pub type MQIMPO<'a> = MqStruct<'a, sys::MQIMPO>;
pub type MQDMPO = MqStruct<'static, sys::MQDMPO>;
pub type MQSMPO = MqStruct<'static, sys::MQSMPO>;

pub type MQMHBO = MqStruct<'static, sys::MQMHBO>;
pub type MQBMHO = MqStruct<'static, sys::MQBMHO>;

pub type MQPD = MqStruct<'static, sys::MQPD>;

pub type MQSRO = MqStruct<'static, sys::MQSRO>;

pub type MQSTS<'a> = MqStruct<'a, sys::MQSTS>;

pub type MQCBC<'a> = MqStruct<'a, sys::MQCBC>;
pub type MQCBD<'a> = MqStruct<'a, sys::MQCBD>;

pub type MQBO = MqStruct<'static, sys::MQBO>;
pub type MQCTLO<'a> = MqStruct<'a, sys::MQCTLO>;
