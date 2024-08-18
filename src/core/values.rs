use crate::{define_mqvalue, impl_default_mqvalue, mapping, sys, MqMask, MqValue};

// Open Options mask
define_mqvalue!(pub MQOO, mapping::MQOO_CONST);

// Close Options mask
define_mqvalue!(pub MQCO, mapping::MQCO_CONST);
impl_default_mqvalue!(MqMask<MQCO>, sys::MQCO_NONE);
define_mqvalue!(pub MQSO, mapping::MQSO_CONST);
// Callback Operation mask/value
define_mqvalue!(pub MQOP, mapping::MQOP_CONST);
define_mqvalue!(pub MQSR, mapping::MQSR_CONST);
define_mqvalue!(pub MQTYPE, mapping::MQTYPE_CONST);
define_mqvalue!(pub MQENC, mapping::MQENC_CONST);
define_mqvalue!(pub MQGMO, mapping::MQGMO_CONST);
impl_default_mqvalue!(MqMask<MQGMO>, sys::MQGMO_NONE);
define_mqvalue!(pub MQPMO, mapping::MQGMO_CONST);
impl_default_mqvalue!(MqMask<MQPMO>, sys::MQPMO_NONE);
define_mqvalue!(pub MQSTAT, mapping::MQSTAT_CONST);
// Create bag options mask
define_mqvalue!(pub MQCBO, mapping::MQCBO_CONST);
define_mqvalue!(pub MQCMHO, mapping::MQCMHO_CONST);
impl_default_mqvalue!(MqValue<MQCMHO>, sys::MQCMHO_DEFAULT_VALIDATION);
define_mqvalue!(pub MQSMPO, mapping::MQSMPO_CONST);
impl_default_mqvalue!(MqValue<MQSMPO>, sys::MQSMPO_SET_FIRST);
define_mqvalue!(pub MQDMPO, mapping::MQDMPO_CONST);
impl_default_mqvalue!(MqValue<MQDMPO>, sys::MQDMPO_DEL_FIRST);
define_mqvalue!(pub MQXA, mapping::MQXA_FULL_CONST);
// Callback options (`MQCBDO_*`)
define_mqvalue!(pub MQCBDO, mapping::MQCBDO_CONST);
define_mqvalue!(pub MQIMPO, mapping::MQIMPO_CONST);
impl_default_mqvalue!(MqMask<MQIMPO>, sys::MQIMPO_NONE);
define_mqvalue!(pub MQPD, mapping::MQPD_CONST);
define_mqvalue!(pub MQCOPY, mapping::MQCOPY_CONST);
define_mqvalue!(pub MQRC, mapping::MQRC_FULL_CONST);
define_mqvalue!(pub MQCC, mapping::MQCC_CONST);
define_mqvalue!(pub MQDCC, mapping::MQDCC_CONST);
impl_default_mqvalue!(MqMask<MQDCC>, sys::MQDCC_NONE);

define_mqvalue!(pub MQCNO, mapping::MQCNO_CONST);
define_mqvalue!(pub MQXPT, mapping::MQXPT_CONST);

define_mqvalue!(pub MQOT, mapping::MQOT_CONST);
