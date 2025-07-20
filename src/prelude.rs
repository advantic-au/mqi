#[cfg(feature = "dlopen2")]
pub use libmqm_sys::dlopen2::LoadMqmExt as _;

pub use crate::{
    connection::AsConnection as _,
    mqstr,
    result::{ResultCompErrExt as _, ResultCompExt as _, WithMqError as _},
};
