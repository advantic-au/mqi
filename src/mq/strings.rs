use std::num::{NonZero, NonZeroI32};
use thiserror::Error;

#[derive(Debug, Clone, Hash)]
pub struct StringCcsid<T> {
    pub(crate) ccsid: Option<std::num::NonZeroI32>,
    pub(crate) data: T,
}

pub type StrCcsid<'a> = StringCcsid<&'a [u8]>;
pub type OwnedStrCcsid = StringCcsid<Vec<u8>>;

#[derive(Error, Debug)]
pub enum FromStringCcsidError {
    #[error(transparent)]
    NonUtf8Ccsid(#[from] CcsidError),
    #[error("UTF-8 conversion")]
    Utf8Convert(#[from] std::string::FromUtf8Error),
}

#[derive(Error, Debug)]
#[error("{} is not a UTF-8 CCSID", .str.ccsid.map_or(0, NonZeroI32::get))]
pub struct CcsidError {
    str: OwnedStrCcsid,
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
        Self::from_utf8(value.data.into()).map_err(FromStringCcsidError::Utf8Convert)
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
        Self { ccsid: NonZero::new(1208), data: Default::default() }
    }
}
