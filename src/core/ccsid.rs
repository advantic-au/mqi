use crate::{encoding, sys};

#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
#[repr(transparent)]
pub struct CCSID(pub sys::MQLONG);

impl CCSID {
    #[must_use]
    pub fn name(self) -> Option<&'static str> {
        #[expect(clippy::cast_sign_loss)]
        encoding::ccsid_lookup(self.0 as u32).map(|(.., name)| *name)
    }

    #[must_use]
    pub fn is_ebcdic(self) -> Option<bool> {
        #[expect(clippy::cast_sign_loss)]
        encoding::ccsid_lookup(self.0 as u32).map(|(_, encoding, _)| *encoding == 1)
    }
}

impl AsRef<CCSID> for sys::MQLONG {
    fn as_ref(&self) -> &CCSID {
        // SAFETY: CCSID is repr(transparent)
        unsafe { &*(std::ptr::from_ref(self).cast()) }
    }
}

impl AsMut<CCSID> for sys::MQLONG {
    fn as_mut(&mut self) -> &mut CCSID {
        // SAFETY: CCSID is repr(transparent)
        unsafe { &mut *(std::ptr::from_mut(self).cast()) }
    }
}

impl std::fmt::Display for CCSID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        if let Some(name) = self.name() {
            write!(f, " ({name})")?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for CCSID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[expect(clippy::cast_sign_loss)]
        let encoding = encoding::ccsid_lookup(self.0 as u32);
        match encoding {
            Some(&(.., name)) => f.debug_tuple("CCSID").field(&format_args!("{}: {name}", self.0)).finish(),
            None => f.debug_tuple("CCSID").field(&format_args!("{}", self.0)).finish(),
        }
    }
}

impl Default for CCSID {
    fn default() -> Self {
        Self(sys::MQCCSI_UNDEFINED)
    }
}

impl PartialEq<sys::MQLONG> for CCSID {
    fn eq(&self, other: &sys::MQLONG) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use std::convert::identity;

    use super::*;

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
