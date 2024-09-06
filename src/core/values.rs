#![expect(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]

use crate::{define_mqmask, define_mqvalue, impl_default_mqvalue, mapping, sys};

// Open Options mask
define_mqmask!(pub MQOO, mapping::MQOO_CONST);

// Close Options mask
define_mqmask!(pub MQCO, mapping::MQCO_CONST);
impl_default_mqvalue!(MQCO, sys::MQCO_NONE);
define_mqmask!(pub MQSO, mapping::MQSO_CONST);
// Callback Operation mask/value
define_mqmask!(pub MQOP, mapping::MQOP_CONST);
define_mqvalue!(pub MQSR, mapping::MQSR_CONST);
define_mqvalue!(pub MQTYPE, mapping::MQTYPE_CONST);
impl_default_mqvalue!(MQTYPE, sys::MQTYPE_AS_SET);
define_mqmask!(pub MQENC, mapping::MQENC_CONST);
impl_default_mqvalue!(MQENC, sys::MQENC_NATIVE);
define_mqmask!(pub MQGMO, mapping::MQGMO_CONST);
impl_default_mqvalue!(MQGMO, sys::MQGMO_NONE);
define_mqmask!(pub MQPMO, mapping::MQGMO_CONST);
impl_default_mqvalue!(MQPMO, sys::MQPMO_NONE);
define_mqvalue!(pub MQSTAT, mapping::MQSTAT_CONST);
// Create bag options mask
define_mqmask!(pub MQCBO, mapping::MQCBO_CONST);
define_mqvalue!(pub MQCMHO, mapping::MQCMHO_CONST);
impl_default_mqvalue!(MQCMHO, sys::MQCMHO_DEFAULT_VALIDATION);
define_mqvalue!(pub MQSMPO, mapping::MQSMPO_CONST);
impl_default_mqvalue!(MQSMPO, sys::MQSMPO_SET_FIRST);
define_mqvalue!(pub MQDMPO, mapping::MQDMPO_CONST);
impl_default_mqvalue!(MQDMPO, sys::MQDMPO_DEL_FIRST);
define_mqvalue!(pub MQXA, mapping::MQXA_FULL_CONST);
// Callback options (`MQCBDO_*`)
define_mqmask!(pub MQCBDO, mapping::MQCBDO_CONST);
define_mqmask!(pub MQIMPO, mapping::MQIMPO_CONST);
impl_default_mqvalue!(MQIMPO, sys::MQIMPO_NONE);
define_mqvalue!(pub MQPD, mapping::MQPD_CONST);
define_mqmask!(pub MQCOPY, mapping::MQCOPY_CONST);
define_mqvalue!(pub MQRC, mapping::MQRC_FULL_CONST);
define_mqvalue!(pub MQCC, mapping::MQCC_CONST);
define_mqmask!(pub MQDCC, mapping::MQDCC_CONST);
impl_default_mqvalue!(MQDCC, sys::MQDCC_NONE);

define_mqmask!(pub MQCNO, mapping::MQCNO_CONST);
define_mqmask!(pub MQXPT, mapping::MQXPT_CONST);

define_mqvalue!(pub MQOT, mapping::MQOT_CONST);
