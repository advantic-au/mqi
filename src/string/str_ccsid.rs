use std::{borrow::Cow, ptr};

use libmqm_default as default;
use libmqm_sys::MQCHARV;

use crate::{constants, conversion, string::CCSID, structs, types};

#[derive(Debug, Clone, Copy, Hash)]
pub struct StringCcsid<T> {
    pub(crate) ccsid: CCSID,
    pub(crate) le: bool,
    pub(crate) data: T,
}

impl<T> StringCcsid<T> {
    pub const fn new(data: T, ccsid: CCSID, le: bool) -> Self {
        Self { ccsid, le, data }
    }
}

pub type StrCcsid<'a> = StringCcsid<&'a [types::MQCHAR]>;
pub type StrCcsidOwned = StringCcsid<Vec<types::MQCHAR>>;
pub type StrCcsidCow<'a> = StringCcsid<Cow<'a, [types::MQCHAR]>>;

const NATIVE_IS_LE: bool = constants::MQENC_NATIVE.contains(constants::MQENC_INTEGER_REVERSED);

#[derive(derive_more::Error, derive_more::Display, derive_more::From, Debug)]
pub enum FromStringCcsidError {
    NonUtf8Ccsid(CcsidError),
    Utf8Convert(std::str::Utf8Error),
}

#[derive(derive_more::Error, derive_more::Display, Debug)]
#[display("{} is not a UTF-8 CCSID", str.ccsid)]
pub struct CcsidError {
    str: StrCcsidOwned,
}

impl StrCcsidOwned {
    #[must_use]
    pub const fn from_vec(data: Vec<types::MQCHAR>, ccsid: CCSID) -> Self {
        Self {
            ccsid,
            le: NATIVE_IS_LE,
            data,
        }
    }
}

impl<'a> From<&'a str> for StrCcsid<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            ccsid: CCSID(1208),
            data: conversion::slice_byte_to_mqchar(value.as_bytes()),
            le: NATIVE_IS_LE,
        }
    }
}

impl<A: AsRef<[types::MQCHAR]>, B: AsRef<[types::MQCHAR]>> PartialEq<StringCcsid<B>> for StringCcsid<A> {
    fn eq(&self, other: &StringCcsid<B>) -> bool {
        self.ccsid == other.ccsid && self.data.as_ref() == other.data.as_ref()
    }
}

impl<A: AsRef<[types::MQCHAR]>> PartialEq<&str> for StringCcsid<A> {
    fn eq(&self, other: &&str) -> bool {
        self.ccsid == 1208 && self.data.as_ref() == other.data()
    }
}

impl<'a, T: Into<Cow<'a, str>>> From<T> for StrCcsidCow<'a> {
    fn from(value: T) -> Self {
        Self {
            ccsid: CCSID(1208),
            data: match value.into() {
                Cow::Borrowed(str_val) => Cow::Borrowed(conversion::slice_byte_to_mqchar(str_val.as_bytes())),
                Cow::Owned(str_val) => Cow::Owned(conversion::vec_byte_to_mqchar(str_val.into_bytes())),
            },
            le: NATIVE_IS_LE,
        }
    }
}

impl<T: ToString> From<T> for StrCcsidOwned {
    fn from(value: T) -> Self {
        Self {
            ccsid: CCSID(1208),
            data: conversion::vec_byte_to_mqchar(value.to_string().into_bytes()),
            le: NATIVE_IS_LE,
        }
    }
}

impl<T: Into<Vec<types::MQCHAR>>> TryFrom<StringCcsid<T>> for String {
    type Error = FromStringCcsidError;

    fn try_from(value: StringCcsid<T>) -> Result<Self, Self::Error> {
        if value.ccsid != 1208 {
            return Err(FromStringCcsidError::NonUtf8Ccsid(CcsidError {
                str: StringCcsid {
                    ccsid: value.ccsid,
                    data: value.data.into(),
                    le: NATIVE_IS_LE,
                },
            }));
        }
        Self::from_utf8(conversion::vec_mqchar_to_byte(value.data.into()))
            .map_err(|e| FromStringCcsidError::Utf8Convert(e.utf8_error()))
    }
}

impl<'a, T: Into<Cow<'a, [types::MQCHAR]>>> TryFrom<StringCcsid<T>> for Cow<'a, str> {
    type Error = FromStringCcsidError;

    fn try_from(value: StringCcsid<T>) -> Result<Self, Self::Error> {
        if value.ccsid != 1208 {
            return Err(FromStringCcsidError::NonUtf8Ccsid(CcsidError {
                str: StringCcsid {
                    ccsid: value.ccsid,
                    data: value.data.into().into_owned(),
                    le: NATIVE_IS_LE,
                },
            }));
        }

        Ok(match value.data.into() {
            Cow::Borrowed(chars) => Cow::Borrowed(std::str::from_utf8(conversion::slice_mqchar_to_byte(chars))?),
            Cow::Owned(chars) => {
                Cow::Owned(String::from_utf8(conversion::vec_mqchar_to_byte(chars)).map_err(|e| e.utf8_error())?)
            }
        })
    }
}

#[cfg(feature = "exits")]
impl<T: AsRef<[types::MQCHAR]>> StringCcsid<T> {
    pub fn try_mq_convert<'a, C>(
        &self,
        ccsid: CCSID,
        conn: &C,
        target_le: bool,
        buffer: &'a mut [types::MQCHAR],
    ) -> crate::result::ResultComp<StrCcsid<'a>>
    where
        C::Lib: crate::Library<MQ: libmqm_sys::Exits>,
        C: crate::Conn,
    {
        use crate::{constants, prelude::*};

        let mut mqdcc = if self.le {
            constants::MQDCC_SOURCE_ENC_REVERSED
        } else {
            constants::MQDCC_SOURCE_ENC_NORMAL
        };

        mqdcc.insert(if target_le {
            constants::MQDCC_TARGET_ENC_REVERSED
        } else {
            constants::MQDCC_TARGET_ENC_NORMAL
        });
        conn.mq()
            .mqxcnvc(Some(conn.handle()), mqdcc, self.ccsid, self.data.as_ref(), ccsid, buffer)
            .map_completion(|length| StringCcsid {
                ccsid,
                le: target_le,
                data: &buffer[..length
                    .try_into()
                    .expect("length should not exceed maximum positive MQLONG for MQCHARV")],
            })
    }
}

pub trait EncodedString {
    fn ccsid(&self) -> CCSID;
    fn data(&self) -> &[types::MQCHAR];
}

impl EncodedString for &str {
    fn ccsid(&self) -> CCSID {
        EncodedString::ccsid(*self)
    }

    fn data(&self) -> &[types::MQCHAR] {
        EncodedString::data(*self)
    }
}

impl EncodedString for str {
    fn ccsid(&self) -> CCSID {
        CCSID(1208) // = UTF-8 CCSID. str types are _always_ UTF-8
    }

    fn data(&self) -> &[types::MQCHAR] {
        unsafe { &*(std::ptr::from_ref(self) as *const _) }
    }
}

impl<T: AsRef<[types::MQCHAR]>> EncodedString for StringCcsid<T> {
    fn ccsid(&self) -> CCSID {
        self.ccsid
    }

    fn data(&self) -> &[types::MQCHAR] {
        self.data.as_ref()
    }
}

impl<'a> structs::MQCHARV<'a> {
    pub fn from_encoded_str(value: &'a (impl EncodedString + ?Sized)) -> Self {
        let data = value.data();
        let len = data
            .len()
            .try_into()
            .expect("string length should not exceed maximum positive MQLONG for MQCHARV");
        structs::MQCHARV::new(MQCHARV {
            VSPtr: ptr::from_ref(data).cast_mut().cast(),
            VSLength: len,
            VSBufSize: len,
            VSCCSID: value.ccsid().0,
            ..default::MQCHARV_DEFAULT
        })
    }
}

impl<T: Default> Default for StringCcsid<T> {
    fn default() -> Self {
        Self {
            ccsid: CCSID(1208),
            data: T::default(),
            le: NATIVE_IS_LE,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use std::{borrow::Cow, mem};

    use super::NATIVE_IS_LE;
    use crate::{
        string::{CCSID, StrCcsid, StrCcsidCow, StringCcsid},
        types,
    };

    const NON_UTF8_COW: StrCcsidCow = StrCcsidCow {
        ccsid: CCSID(450),
        data: Cow::Borrowed(unsafe { mem::transmute::<&[u8], &[types::MQCHAR]>(b"Hello".as_slice()) }),
        le: NATIVE_IS_LE,
    };

    #[test]
    fn strccsidcow() {
        let basic_cow: StrCcsidCow = StringCcsid::from("Hello");
        let basic_ref: StrCcsid = StringCcsid::from("Hello");
        assert!(
            TryInto::<String>::try_into(basic_cow.clone()).is_ok(),
            "Convert must be successful when CCSID = 1208"
        );
        assert!(
            TryInto::<Cow<str>>::try_into(basic_ref).is_ok(),
            "Convert must be successful from ref"
        );

        assert!(
            TryInto::<String>::try_into(NON_UTF8_COW).is_err(),
            "Convert must fail when CCSID != 1208"
        );

        assert!(
            TryInto::<Cow<str>>::try_into(basic_cow.clone()).is_ok(),
            "Convert from Cow to Cow"
        );
    }
}
