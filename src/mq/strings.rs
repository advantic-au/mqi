use std::{
    borrow::Cow,
    num::{NonZero, NonZeroI32},
    ptr,
};

use crate::sys;

use super::MqStruct;

#[derive(Debug, Clone, Copy, Hash)]
pub struct StringCcsid<T> {
    pub(crate) ccsid: Option<std::num::NonZeroI32>,
    pub(crate) le: bool,
    pub(crate) data: T,
}

impl<T> StringCcsid<T> {
    pub const fn new(data: T, ccsid: Option<std::num::NonZeroI32>, le: bool) -> Self {
        Self { ccsid, le, data }
    }
}

pub type StrCcsid<'a> = StringCcsid<&'a [u8]>;
pub type StrCcsidOwned = StringCcsid<Vec<u8>>;
pub type StrCcsidCow<'a> = StringCcsid<Cow<'a, [u8]>>;

pub const NATIVE_IS_LE: bool = (sys::MQENC_NATIVE & sys::MQENC_INTEGER_REVERSED) != 0;

#[derive(derive_more::Error, derive_more::Display, derive_more::From, Debug)]
pub enum FromStringCcsidError {
    NonUtf8Ccsid(CcsidError),
    Utf8Convert(std::str::Utf8Error),
}

#[derive(derive_more::Error, derive_more::Display, Debug)]
#[display("{} is not a UTF-8 CCSID", str.ccsid.map_or(0, NonZeroI32::get))]
pub struct CcsidError {
    str: StrCcsidOwned,
}

impl StrCcsidOwned {
    #[must_use]
    pub const fn from_vec(data: Vec<u8>, ccsid: Option<NonZero<i32>>) -> Self {
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
            ccsid: NonZero::new(1208),
            data: value.as_bytes(),
            le: NATIVE_IS_LE,
        }
    }
}

impl<'a, T: Into<Cow<'a, str>>> From<T> for StrCcsidCow<'a> {
    fn from(value: T) -> Self {
        Self {
            ccsid: NonZero::new(1208),
            data: match value.into() {
                Cow::Borrowed(str_val) => Cow::Borrowed(str_val.as_bytes()),
                Cow::Owned(str_val) => Cow::Owned(str_val.into()),
            },
            le: NATIVE_IS_LE,
        }
    }
}

impl<T: ToString> From<T> for StrCcsidOwned {
    fn from(value: T) -> Self {
        Self {
            ccsid: NonZero::new(1208),
            data: value.to_string().into_bytes(),
            le: NATIVE_IS_LE,
        }
    }
}

impl<T: Into<Vec<u8>>> TryFrom<StringCcsid<T>> for String {
    type Error = FromStringCcsidError;

    fn try_from(value: StringCcsid<T>) -> Result<Self, Self::Error> {
        if value.ccsid != NonZeroI32::new(1208) {
            return Err(FromStringCcsidError::NonUtf8Ccsid(CcsidError {
                str: StringCcsid {
                    ccsid: value.ccsid,
                    data: value.data.into(),
                    le: NATIVE_IS_LE,
                },
            }));
        }
        Self::from_utf8(value.data.into()).map_err(|e| FromStringCcsidError::Utf8Convert(e.utf8_error()))
    }
}

impl<'a, T: Into<Cow<'a, [u8]>>> TryFrom<StringCcsid<T>> for Cow<'a, str> {
    type Error = FromStringCcsidError;

    fn try_from(value: StringCcsid<T>) -> Result<Self, Self::Error> {
        if value.ccsid != NonZeroI32::new(1208) {
            return Err(FromStringCcsidError::NonUtf8Ccsid(CcsidError {
                str: StringCcsid {
                    ccsid: value.ccsid,
                    data: value.data.into().into_owned(),
                    le: NATIVE_IS_LE,
                },
            }));
        }

        Ok(match value.data.into() {
            Cow::Borrowed(bytes) => Cow::Borrowed(std::str::from_utf8(bytes)?),
            Cow::Owned(bytes) => Cow::Owned(String::from_utf8(bytes).map_err(|e| e.utf8_error())?),
        })
    }
}

pub trait EncodedString {
    fn ccsid(&self) -> Option<NonZeroI32>;
    fn data(&self) -> &[u8];
}

impl EncodedString for str {
    fn ccsid(&self) -> Option<NonZeroI32> {
        NonZeroI32::new(1208) // = UTF-8 CCSID. str types are _always_ UTF-8
    }

    fn data(&self) -> &[u8] {
        unsafe { &*(std::ptr::from_ref(self) as *const [u8]) }
    }
}

impl<T: AsRef<[u8]>> EncodedString for StringCcsid<T> {
    fn ccsid(&self) -> Option<NonZeroI32> {
        self.ccsid
    }

    fn data(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<'a> MqStruct<'a, sys::MQCHARV> {
    pub fn from_encoded_str(value: &'a (impl EncodedString + ?Sized)) -> Self {
        let data = value.data();
        let len = data
            .len()
            .try_into()
            .expect("string length exceeds maximum positive MQLONG for MQCHARV");
        MqStruct::new(sys::MQCHARV {
            VSPtr: ptr::from_ref(data).cast_mut().cast(),
            VSLength: len,
            VSBufSize: len,
            VSCCSID: value.ccsid().map_or(0, NonZero::into),
            ..sys::MQCHARV::default()
        })
    }
}

impl<T: Default> Default for StringCcsid<T> {
    fn default() -> Self {
        Self {
            ccsid: NonZero::new(1208),
            data: Default::default(),
            le: NATIVE_IS_LE,
        }
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::Cow, num::NonZero};

    use crate::{StrCcsid, StrCcsidCow, StringCcsid};

    use super::NATIVE_IS_LE;

    const NON_UTF8_COW: StrCcsidCow = StrCcsidCow {
        ccsid: NonZero::new(450),
        data: Cow::Borrowed(b"Hello".as_slice()),
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
        // assert_matches!(basic, Ok("Hello"));
    }
}
