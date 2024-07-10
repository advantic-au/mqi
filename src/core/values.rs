use crate::{define_mqvalue, mapping, sys, MqMask};

// Close Options mask
define_mqvalue!(pub MQOO, mapping::MQOO_CONST);
// Open Options mask
define_mqvalue!(pub MQCO, mapping::MQCO_CONST);
// Callback Operation mask/value
define_mqvalue!(pub MQOP, mapping::MQOP_CONST);
define_mqvalue!(pub MQSR, mapping::MQSR_CONST);
define_mqvalue!(pub MQTYPE, mapping::MQTYPE_CONST);
define_mqvalue!(pub MQENC, mapping::MQENC_CONST);

impl Default for MqMask<MQENC> {
    fn default() -> Self {
        Self::from(sys::MQENC_NORMAL)
    }
}

define_mqvalue!(pub MQSTAT, mapping::MQSTAT_CONST);
// Create bag options mask
define_mqvalue!(pub MQCBO, mapping::MQCBO_CONST);
define_mqvalue!(pub MQCMHO, mapping::MQCMHO_CONST);
define_mqvalue!(pub MQSMPO, mapping::MQSMPO_CONST);
define_mqvalue!(pub MQXA, mapping::MQXA_FULL_CONST);
// Callback options (`MQCBDO_*`)
define_mqvalue!(pub MQCBDO, mapping::MQCBDO_CONST);

define_mqvalue!(pub MQIMPO, mapping::MQIMPO_CONST);
impl Default for MqMask<MQIMPO> {
    fn default() -> Self {
        Self::from(sys::MQIMPO_NONE)
    }
}

define_mqvalue!(pub MQPD, mapping::MQPD_CONST);
define_mqvalue!(pub MQCOPY, mapping::MQCOPY_CONST);

define_mqvalue!(pub MQRC, mapping::MQRC_FULL_CONST);
define_mqvalue!(pub MQCC, mapping::MQCC_CONST);
