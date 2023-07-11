use std::{fmt::Display, num::NonZeroI32};

use crate::constants::{self, mapping};
use crate::{impl_constant_lookup, sys, HasMqNames};

#[derive(Default, Debug, Clone, Copy)]
pub struct Filter<T> {
    pub operator: Op,
    pub value: T,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Op(pub sys::MQLONG);

impl constants::MQConstant for Op {
    fn mq_value(&self) -> sys::MQLONG {
        let &Self(value) = self;
        value
    }
}

impl<T> Filter<T> {
    pub const fn value(&self) -> &T {
        &self.value
    }

    pub const fn operator(&self) -> sys::MQLONG {
        let &Self {
            operator: Op(op_value), ..
        } = self;
        op_value
    }

    pub const fn new(value: T, operator: sys::MQLONG) -> Self {
        Self {
            value,
            operator: Op(operator),
        }
    }

    pub const fn less(value: T) -> Self {
        Self {
            operator: Op(sys::MQCFOP_LESS),
            value,
        }
    }

    pub const fn equal(value: T) -> Self {
        Self {
            operator: Op(sys::MQCFOP_EQUAL),
            value,
        }
    }

    pub const fn not_greater(value: T) -> Self {
        Self {
            operator: Op(sys::MQCFOP_NOT_GREATER),
            value,
        }
    }

    pub const fn greater(value: T) -> Self {
        Self {
            operator: Op(sys::MQCFOP_GREATER),
            value,
        }
    }

    pub const fn not_equal(value: T) -> Self {
        Self {
            operator: Op(sys::MQCFOP_NOT_EQUAL),
            value,
        }
    }

    pub const fn not_less(value: T) -> Self {
        Self {
            operator: Op(sys::MQCFOP_NOT_LESS),
            value,
        }
    }

    pub const fn contains(value: T) -> Self {
        Self {
            operator: Op(sys::MQCFOP_CONTAINS),
            value,
        }
    }

    pub const fn excludes(value: T) -> Self {
        Self {
            operator: Op(sys::MQCFOP_EXCLUDES),
            value,
        }
    }

    pub const fn like(value: T) -> Self {
        Self {
            operator: Op(sys::MQCFOP_LIKE),
            value,
        }
    }

    pub const fn not_like(value: T) -> Self {
        Self {
            operator: Op(sys::MQCFOP_NOT_LIKE),
            value,
        }
    }

    pub const fn contains_gen(value: T) -> Self {
        Self {
            operator: Op(sys::MQCFOP_CONTAINS_GEN),
            value,
        }
    }

    pub const fn excludes_gen(value: T) -> Self {
        Self {
            operator: Op(sys::MQCFOP_EXCLUDES_GEN),
            value,
        }
    }
}

impl<T: Display> Display for Filter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.value, self.operator)
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(mq_value) = self;
        match self.mq_primary_name() {
            Some(name) => write!(f, "{name}"),
            None => write!(f, "UNKNOWN({mq_value})"),
        }
    }
}

impl_constant_lookup!(Op, mapping::MQCFOP_CONST);

pub trait EncodedString {
    fn ccsid(&self) -> Option<NonZeroI32>;
    fn data(&self) -> &[u8];
}
