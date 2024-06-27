use crate::{define_mqvalue, mapping};

// Close Options mask
define_mqvalue!(MQOO, mapping::MQOO_CONST);
// Open Options mask
define_mqvalue!(MQCO, mapping::MQCO_CONST);
// Callback Operation mask/value
define_mqvalue!(MQOP, mapping::MQOP_CONST);
define_mqvalue!(MQSR, mapping::MQSR_CONST);
define_mqvalue!(MQTYPE, mapping::MQTYPE_CONST);
define_mqvalue!(MQENC, mapping::MQENC_CONST);
define_mqvalue!(MQSTAT, mapping::MQSTAT_CONST);
// Create bag options mask
define_mqvalue!(MQCBO, mapping::MQCBO_CONST);
define_mqvalue!(MQCMHO, mapping::MQCMHO_CONST);
define_mqvalue!(MQSMPO, mapping::MQSMPO_CONST);
define_mqvalue!(MQXA, mapping::MQXA_FULL_CONST);
// Callback options (`MQCBDO_*`)
define_mqvalue!(MQCBDO, mapping::MQCBDO_CONST);

define_mqvalue!(MQRC, mapping::MQRC_FULL_CONST);
define_mqvalue!(MQCC, mapping::MQCC_CONST);
