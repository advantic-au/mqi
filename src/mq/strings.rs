use std::{convert::Infallible, num::NonZeroI32, str::FromStr};
use thiserror::Error;

use crate::mq;

#[derive(Debug, Clone)]
pub struct StringCcsid {
    pub(crate) ccsid: Option<std::num::NonZeroI32>,
    pub(crate) data: Vec<u8>,
}

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
    str: mq::StringCcsid,
}

impl TryFrom<mq::StringCcsid> for String {
    type Error = FromStringCcsidError;

    fn try_from(value: mq::StringCcsid) -> Result<Self, Self::Error> {
        if value.ccsid != NonZeroI32::new(1208) {
            return Err(FromStringCcsidError::NonUtf8Ccsid(CcsidError { str: value }));
        }
        Self::from_utf8(value.data).map_err(FromStringCcsidError::Utf8Convert)
    }
}

impl FromStr for mq::StringCcsid {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            ccsid: NonZeroI32::new(1208),
            data: s.as_bytes().into(),
        })
    }
}
