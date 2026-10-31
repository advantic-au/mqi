use std::fmt::Display;

use crate::{constants, types::MQCFOP};

#[derive(Debug, Clone, Copy, Eq)]
/// A value and MQ comparison operator used to filter bag inquiries.
pub struct Filter<T> {
    /// MQ comparison operator applied to `value`.
    pub operator: MQCFOP,
    /// Value compared with the selected bag items.
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
    /// Borrow the comparison value.
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Return the MQ comparison operator.
    pub const fn operator(&self) -> MQCFOP {
        let &Self { operator, .. } = self;
        operator
    }

    /// Construct a filter with an explicit MQ comparison operator.
    pub const fn new(value: T, operator: MQCFOP) -> Self {
        Self { operator, value }
    }

    /// Match values less than `value`.
    pub const fn less(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_LESS,
            value,
        }
    }

    /// Match values equal to `value`.
    pub const fn equal(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_EQUAL,
            value,
        }
    }

    /// Match values that are not greater than `value`.
    pub const fn not_greater(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_NOT_GREATER,
            value,
        }
    }

    /// Match values greater than `value`.
    pub const fn greater(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_GREATER,
            value,
        }
    }

    /// Match values that are not equal to `value`.
    pub const fn not_equal(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_NOT_EQUAL,
            value,
        }
    }

    /// Match values that are not less than `value`.
    pub const fn not_less(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_NOT_LESS,
            value,
        }
    }

    /// Match values that contain `value`.
    pub const fn contains(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_CONTAINS,
            value,
        }
    }

    /// Match values that exclude `value`.
    pub const fn excludes(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_EXCLUDES,
            value,
        }
    }

    /// Match values using MQ's `LIKE` comparison.
    pub const fn like(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_LIKE,
            value,
        }
    }

    /// Match values using MQ's `NOT LIKE` comparison.
    pub const fn not_like(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_NOT_LIKE,
            value,
        }
    }

    /// Match values that contain the generic pattern `value`.
    pub const fn contains_gen(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_CONTAINS_GEN,
            value,
        }
    }

    /// Match values that exclude the generic pattern `value`.
    pub const fn excludes_gen(value: T) -> Self {
        Self {
            operator: constants::MQCFOP_EXCLUDES_GEN,
            value,
        }
    }
}

impl<T> Filter<Vec<T>> {
    /// Borrow the vector contents as a slice while preserving the operator.
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
