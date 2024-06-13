use crate::{impl_constant_lookup, mapping, sys, RawValue};

pub struct OpenOptions;
impl_constant_lookup!(OpenOptions, mapping::MQOO_CONST);

pub struct CloseOptions;
impl_constant_lookup!(CloseOptions, mapping::MQCO_CONST);

#[derive(Clone, Copy)]
pub struct CallbackOperation;
impl_constant_lookup!(CallbackOperation, mapping::MQOP_CONST);
impl RawValue for CallbackOperation {
    type ValueType = sys::MQLONG;
}
