use std::fmt::Display;

use crate::types::MQCFOP;
use crate::constants;

#[derive(Debug, Clone, Copy, Eq)]
pub struct Filter<T> {
    pub operator: MQCFOP,
    pub value: T,
}

impl<A, B> PartialEq<Filter<A>> for Filter<B>
where
    B: PartialEq<A>,
{
    fn eq(&self, other: &Filter<A>) -> bool {
        self.operator == other.operator && self.value == other.value
    }
}

impl<T> Filter<T> {
    pub const fn value(&self) -> &T {
        &self.value
    }

    pub const fn operator(&self) -> MQCFOP {
        let &Self { operator, .. } = self;
        operator
    }

    pub const fn new(value: T, operator: MQCFOP) -> Self {
        Self { operator, value }
    }

    pub const fn less(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_LESS,
            value,
        }
    }

    pub const fn equal(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_EQUAL,
            value,
        }
    }

    pub const fn not_greater(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_NOT_GREATER,
            value,
        }
    }

    pub const fn greater(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_GREATER,
            value,
        }
    }

    pub const fn not_equal(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_NOT_EQUAL,
            value,
        }
    }

    pub const fn not_less(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_NOT_LESS,
            value,
        }
    }

    pub const fn contains(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_CONTAINS,
            value,
        }
    }

    pub const fn excludes(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_EXCLUDES,
            value,
        }
    }

    pub const fn like(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_LIKE,
            value,
        }
    }

    pub const fn not_like(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_NOT_LIKE,
            value,
        }
    }

    pub const fn contains_gen(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_CONTAINS_GEN,
            value,
        }
    }

    pub const fn excludes_gen(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_EXCLUDES_GEN,
            value,
        }
    }
}

impl<T> Filter<Vec<T>> {
    #[must_use]
    pub const fn as_slice(&self) -> Filter<&[T]> {
        Filter {
            operator: self.operator,
            value: self.value.as_slice(),
        }
    }
}

impl<T: Display> Display for Filter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} <{}>", self.value, self.operator)
    }
}
