use std::{
    borrow::Cow,
    num::{NonZero, NonZeroI32},
};
use thiserror::Error;

#[derive(Debug, Clone, Hash)]
pub struct StringCcsid<T> {
    pub(crate) ccsid: Option<std::num::NonZeroI32>,
    pub(crate) data: T,
}

pub type StrCcsid<'a> = StringCcsid<&'a [u8]>;
pub type StrCcsidOwned = StringCcsid<Vec<u8>>;
pub type StrCcsidCow<'a> = StringCcsid<Cow<'a, [u8]>>;

#[derive(Error, Debug)]
pub enum FromStringCcsidError {
    #[error(transparent)]
    NonUtf8Ccsid(#[from] CcsidError),
    #[error("UTF-8 conversion")]
    Utf8Convert(#[from] std::str::Utf8Error),
}

#[derive(Error, Debug)]
#[error("{} is not a UTF-8 CCSID", .str.ccsid.map_or(0, NonZeroI32::get))]
pub struct CcsidError {
    str: StrCcsidOwned,
}

impl<'a> From<&'a str> for StrCcsid<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            ccsid: NonZero::new(1208),
            data: value.as_bytes(),
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
        }
    }
}

impl<T: ToString> From<T> for StrCcsidOwned {
    fn from(value: T) -> Self {
        Self {
            ccsid: NonZero::new(1208),
            data: value.to_string().into_bytes(),
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

impl<T: Default> Default for StringCcsid<T> {
    fn default() -> Self {
        Self {
            ccsid: NonZero::new(1208),
            data: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::Cow, num::NonZero};

    use crate::{StrCcsid, StrCcsidCow, StringCcsid};

    const NON_UTF8_COW: StrCcsidCow = StrCcsidCow {
        ccsid: NonZero::new(450),
        data: Cow::Borrowed(b"Hello".as_slice()),
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
