use std::fmt::Display;

use crate::{sys, MqValue};

use super::values::MQCFOP;

#[derive(Debug, Clone, Copy)]
pub struct Filter<T> {
    pub operator: MqValue<MQCFOP>,
    pub value: T,
}

impl<T> Filter<T> {
    pub const fn value(&self) -> &T {
        &self.value
    }

    pub const fn operator(&self) -> MqValue<MQCFOP> {
        let &Self {
            operator, ..
        } = self;
        operator
    }

    pub const fn new(value: T, operator: MqValue<MQCFOP>) -> Self {
        Self { operator, value }
    }

    pub const fn less(value: T) -> Self {
        Self {
            operator: MqValue::from(sys::MQCFOP_LESS),
            value,
        }
    }

    pub const fn equal(value: T) -> Self {
        Self {
            operator: MqValue::from(sys::MQCFOP_EQUAL),
            value,
        }
    }

    pub const fn not_greater(value: T) -> Self {
        Self {
            operator: MqValue::from(sys::MQCFOP_NOT_GREATER),
            value,
        }
    }

    pub const fn greater(value: T) -> Self {
        Self {
            operator: MqValue::from(sys::MQCFOP_GREATER),
            value,
        }
    }

    pub const fn not_equal(value: T) -> Self {
        Self {
            operator: MqValue::from(sys::MQCFOP_NOT_EQUAL),
            value,
        }
    }

    pub const fn not_less(value: T) -> Self {
        Self {
            operator: MqValue::from(sys::MQCFOP_NOT_LESS),
            value,
        }
    }

    pub const fn contains(value: T) -> Self {
        Self {
            operator: MqValue::from(sys::MQCFOP_CONTAINS),
            value,
        }
    }

    pub const fn excludes(value: T) -> Self {
        Self {
            operator: MqValue::from(sys::MQCFOP_EXCLUDES),
            value,
        }
    }

    pub const fn like(value: T) -> Self {
        Self {
            operator: MqValue::from(sys::MQCFOP_LIKE),
            value,
        }
    }

    pub const fn not_like(value: T) -> Self {
        Self {
            operator: MqValue::from(sys::MQCFOP_NOT_LIKE),
            value,
        }
    }

    pub const fn contains_gen(value: T) -> Self {
        Self {
            operator: MqValue::from(sys::MQCFOP_CONTAINS_GEN),
            value,
        }
    }

    pub const fn excludes_gen(value: T) -> Self {
        Self {
            operator: MqValue::from(sys::MQCFOP_EXCLUDES_GEN),
            value,
        }
    }
}

impl<T: Display> Display for Filter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <{}>", self.value, self.operator)
    }
}
