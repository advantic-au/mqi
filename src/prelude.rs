#[cfg(feature = "dlopen2")]
pub use libmqm_sys::dlopen2::LoadMqmExt as _;

pub use crate::{Conn as _, ResultCompErrExt as _, ResultCompExt as _, WithMqError as _, mqstr};
