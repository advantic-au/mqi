#![expect(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]

use crate::{define_mqmask, define_mqvalue, impl_default_mqvalue, mapping, sys};

define_mqmask!(pub MQOO, mapping::MQOO_CONST, "Options mask to control the action of `MQOPEN`");
define_mqmask!(pub MQCO, mapping::MQCO_CONST, "Options mask to control the action of `MQCLOSE`");
impl_default_mqvalue!(MQCO, sys::MQCO_NONE);
define_mqmask!(pub MQSO, mapping::MQSO_CONST, "Options mask to control the action of `MQSUB`");
define_mqmask!(pub MQOP, mapping::MQOP_CONST, "Operation codes for `MQCTL` and `MQCB`");
define_mqvalue!(pub MQSR, mapping::MQSR_CONST, "Value describing action for `MQSUBRQ`");
define_mqvalue!(pub MQTYPE, mapping::MQTYPE_CONST, "Property data types");
impl_default_mqvalue!(MQTYPE, sys::MQTYPE_AS_SET);
define_mqmask!(pub MQENC, mapping::MQENC_CONST, "Mask describing data encoding");
impl_default_mqvalue!(MQENC, sys::MQENC_NATIVE);
define_mqmask!(pub MQGMO, mapping::MQGMO_CONST, "Options mask to control the action of `MQGET`");
impl_default_mqvalue!(MQGMO, sys::MQGMO_NONE);
define_mqmask!(pub MQPMO, mapping::MQPMO_CONST, "Options mask to control the action of `MQPUT` and `MQPUT1`");
impl_default_mqvalue!(MQPMO, sys::MQPMO_NONE);
define_mqvalue!(pub MQSTAT, mapping::MQSTAT_CONST, "Value describing the MQSTAT outcome");
// Create bag options mask
define_mqmask!(pub MQCBO, mapping::MQCBO_CONST);
define_mqvalue!(pub MQCMHO, mapping::MQCMHO_CONST, "Create message handle options for `MQCRTMH`");
impl_default_mqvalue!(MQCMHO, sys::MQCMHO_DEFAULT_VALIDATION);
define_mqvalue!(pub MQSMPO, mapping::MQSMPO_CONST);
impl_default_mqvalue!(MQSMPO, sys::MQSMPO_SET_FIRST);
define_mqvalue!(pub MQDMPO, mapping::MQDMPO_CONST);
impl_default_mqvalue!(MQDMPO, sys::MQDMPO_DEL_FIRST);
define_mqvalue!(pub MQXA, mapping::MQXA_FULL_CONST);
// Callback options (`MQCBDO_*`)
define_mqmask!(pub MQCBDO, mapping::MQCBDO_CONST);
define_mqmask!(pub MQIMPO, mapping::MQIMPO_CONST, "Options mask to control the action of `MQINQMP`");
impl_default_mqvalue!(MQIMPO, sys::MQIMPO_NONE);
define_mqvalue!(pub MQPD, mapping::MQPD_CONST, "Property descriptor, support and context");
define_mqmask!(pub MQCOPY, mapping::MQCOPY_CONST);
define_mqvalue!(pub MQRC, mapping::MQRC_FULL_CONST, "Reason Code from an MQ function call");
define_mqvalue!(pub MQCC, mapping::MQCC_CONST, "Completion Code from an MQ function call");
define_mqmask!(pub MQDCC, mapping::MQDCC_CONST, "Options mask that control the action of `MQXCNVC`");
impl_default_mqvalue!(MQDCC, sys::MQDCC_NONE);

define_mqmask!(pub MQCNO, mapping::MQCNO_CONST, "Options mask that control the action of `MQCONNX`");
define_mqvalue!(pub MQXPT, mapping::MQXPT_CONST, "Transport Types");

define_mqvalue!(pub MQOT, mapping::MQOT_CONST, "Object Types and Extended Object Types");
