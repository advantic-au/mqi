use std::num::NonZeroI32;
use thiserror::Error;

#[derive(Debug, Clone)]
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

impl<'a> TryFrom<StrCcsid<'a>> for String {
    type Error = FromStringCcsidError;

    fn try_from(value: StrCcsid) -> Result<Self, Self::Error> {
        if value.ccsid != NonZeroI32::new(1208) {
            return Err(FromStringCcsidError::NonUtf8Ccsid(CcsidError { str: StringCcsid { ccsid: value.ccsid, data: value.data.into() } }));
        }
        Self::from_utf8(value.data.into()).map_err(FromStringCcsidError::Utf8Convert)
    }
}

// impl<'a> FromStr for StringCcsid<'a> {
//     type Err = Infallible;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         Ok(Self {
//             ccsid: NonZeroI32::new(1208),
//             data: s.as_bytes().into(),
//         })
//     }
// }

pub trait EncodedString: std::fmt::Debug {
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

impl EncodedString for StrCcsid<'_> {
    fn ccsid(&self) -> Option<NonZeroI32> {
        self.ccsid
    }

    fn data(&self) -> &[u8] {
        self.data
    }
}
