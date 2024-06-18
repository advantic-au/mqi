use crate::{impl_constant_lookup, mapping, sys, RawValue};

/// Close Options mask
pub struct MQOO;
impl_constant_lookup!(MQOO, mapping::MQOO_CONST);

/// Open Options mask
pub struct MQCO;
impl_constant_lookup!(MQCO, mapping::MQCO_CONST);

/// Callback Operation mask/value
#[derive(Clone, Copy)]
pub struct MQOP;
impl_constant_lookup!(MQOP, mapping::MQOP_CONST);
impl RawValue for MQOP {
    type ValueType = sys::MQLONG;
}
