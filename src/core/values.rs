#![expect(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]

use std::fmt::{Debug, Display};

use crate::{define_mqmask, define_mqvalue, encoding, impl_default_mqvalue, mapping, sys};

define_mqmask!(pub MQOO, mapping::MQOO_CONST, "Options mask to control the action of `MQOPEN`");
define_mqmask!(pub MQCO, mapping::MQCO_CONST, "Options mask to control the action of `MQCLOSE`");
impl_default_mqvalue!(MQCO, sys::MQCO_NONE);
define_mqmask!(pub MQSO, mapping::MQSO_CONST, "Options mask to control the action of `MQSUB`");
define_mqmask!(pub MQOP, mapping::MQOP_CONST, "Operation codes for `MQCTL` and `MQCB`");
define_mqvalue!(pub MQCBCT, mapping::MQCBCT_CONST, "Callback control and message delivery call types");
define_mqvalue!(pub MQCBF, mapping::MQCBCF_CONST, "Flags containing information about the callback consumer");
define_mqvalue!(pub MQCS, mapping::MQCS_CONST, "Callback consumer state");
define_mqvalue!(pub MQRD, mapping::MQRD_CONST, "Reconnect delay");
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
define_mqmask!(pub MQCBO, mapping::MQCBO_CONST, "Create-Bag options mask for `mqCreateBag`");
define_mqvalue!(pub MQCMHO, mapping::MQCMHO_CONST, "Create message handle options for `MQCRTMH`");
impl_default_mqvalue!(MQCMHO, sys::MQCMHO_DEFAULT_VALIDATION);
define_mqvalue!(pub MQSMPO, mapping::MQSMPO_CONST, "Set message property options");
impl_default_mqvalue!(MQSMPO, sys::MQSMPO_SET_FIRST);
define_mqvalue!(pub MQDMPO, mapping::MQDMPO_CONST, "Delete message property options");
impl_default_mqvalue!(MQDMPO, sys::MQDMPO_DEL_FIRST);
define_mqvalue!(pub MQXA, mapping::MQXA_FULL_CONST, "Integer and Character attribute selectors");
define_mqmask!(pub MQCBDO, mapping::MQCBDO_CONST, "Options mask to control the action of `MQCB`");
define_mqmask!(pub MQIMPO, mapping::MQIMPO_CONST, "Options mask to control the action of `MQINQMP`");
impl_default_mqvalue!(MQIMPO, sys::MQIMPO_NONE);
define_mqvalue!(pub MQPD, mapping::MQPD_CONST, "Property descriptor, support and context");
define_mqmask!(pub MQCOPY, mapping::MQCOPY_CONST, "Property copy options mask");
define_mqvalue!(pub MQRC, mapping::MQRC_FULL_CONST, "Reason Code from an MQ function call");
define_mqvalue!(pub MQCC, mapping::MQCC_CONST, "Completion Code from an MQ function call");
define_mqmask!(pub MQDCC, mapping::MQDCC_CONST, "Options mask that control the action of `MQXCNVC`");
impl_default_mqvalue!(MQDCC, sys::MQDCC_NONE);

define_mqmask!(pub MQCNO, mapping::MQCNO_CONST, "Options mask that control the action of `MQCONNX`");
define_mqvalue!(pub MQXPT, mapping::MQXPT_CONST, "Transport Types");

define_mqvalue!(pub MQOT, mapping::MQOT_CONST, "Object Types and Extended Object Types");

#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
pub struct CCSID(pub sys::MQLONG);

impl CCSID {
    #[must_use]
    pub fn name(self) -> Option<&'static str> {
        encoding::ccsid_lookup(self.0).map(|&(.., name)| name)
    }

    #[must_use]
    pub fn is_ebcdic(self) -> Option<bool> {
        encoding::ccsid_lookup(self.0).map(|&(_, encoding, _)| encoding == 1)
    }
}

impl Display for CCSID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        if let Some(name) = self.name() {
            write!(f, " ({name})")?;
        }
        Ok(())
    }
}

impl Debug for CCSID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let encoding = encoding::ccsid_lookup(self.0);
        match encoding {
            Some(&(.., name)) => f.debug_tuple("CCSID").field(&format_args!("{}: {name}", self.0)).finish(),
            None => f.debug_tuple("CCSID").field(&format_args!("{}", self.0)).finish(),
        }
    }
}

impl_default_mqvalue!(CCSID, sys::MQCCSI_UNDEFINED);

impl PartialEq<sys::MQLONG> for CCSID {
    fn eq(&self, other: &sys::MQLONG) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod test {
    use std::convert::identity;

    use crate::values::CCSID;

    #[test]
    fn ccsid() {
        assert_eq!(format!("{:?}", CCSID(1208)), "CCSID(1208: UTF-8)");
        assert_eq!(format!("{}", CCSID(1208)), "1208 (UTF-8)");
        assert_eq!(format!("{}", CCSID(5050)), "5050 (EUC-JP)");
        assert!(CCSID(1208).is_ebcdic().is_some_and(|e| !e));
        assert!(CCSID(500).is_ebcdic().is_some_and(identity));
        assert!(CCSID(999).is_ebcdic().is_none());
        assert_eq!(format!("{}", CCSID(999)), "999");
        assert_eq!(format!("{:?}", CCSID(999)), "CCSID(999)");
    }
}
