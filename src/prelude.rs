#[cfg(feature = "dlopen2")]
pub use libmqm_sys::dlopen2::LoadMqmExt as _;

pub use crate::{Conn as _, result::ResultCompErrExt as _, result::ResultCompExt as _, result::WithMqError as _, mqstr};
