use crate::sys;

trait Sealed {}
#[expect(private_bounds, reason = "sealed trait pattern")]
pub trait MQMD: Sealed + std::fmt::Debug {}
impl Sealed for sys::MQMD {}
impl Sealed for sys::MQMD2 {}

impl MQMD for sys::MQMD {}
impl MQMD for sys::MQMD2 {}
