use std::{borrow::Cow, num::NonZero};

use crate::{
    result::{Error, ResultComp, ResultCompErr},
    structs,
    traits::ReadRaw,
    types,
};

/// A property value and the optional property name returned by MQ.
#[derive(Debug, Clone)]
pub struct PropertyState<'s> {
    /// Property name in MQ character bytes, when MQ returned it.
    pub name: Option<Cow<'s, [types::MQCHAR]>>,
    /// Property value as the raw bytes returned by MQ.
    pub value: Cow<'s, [u8]>,
}

/// MQ structures and decoding settings used for one property operation.
#[derive(Clone, Debug)]
pub struct PropertyParam<'p> {
    /// MQ property type expected for an inquiry or selected for a set.
    pub value_type: types::MQTYPE,
    /// Options controlling property inquiry.
    pub impo: structs::MQIMPO<'p>,
    /// Property attributes and context.
    pub mqpd: structs::MQPD,
    /// Whether and how the property name is requested from MQ.
    pub name_required: NameUsage,
}

/// Policy for requesting a property name from MQ.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NameUsage {
    /// Do not request the property name.
    #[default]
    Ignored,
    /// Request a name, allowing at most the specified nonzero number of characters.
    MaxLength(NonZero<usize>),
    /// Request the full property name without a caller-specified length limit.
    AnyLength,
}

/// ## Safety
/// This trait can directly manipulate the [`MQIMPO`](structs::MQIMPO) structure which is used by [`MQINQMP`](libmqm_sys::MQINQMP) function.
/// Incorrect values in the [`MQIMPO`](structs::MQIMPO) can lead to undefined behaviour.
///
/// Implementations of the [`PropertyValue`] trait must ensure that pointers and offsets contained in the structure point to active data.
pub unsafe trait PropertyValue {
    /// Error returned when the raw MQ property cannot be decoded into this type.
    type Error: From<Error> + Into<Error> + std::fmt::Debug;

    /// # Errors
    /// Returns an error if MQ cannot retrieve the property or its value cannot be decoded as `Self`.
    fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqi: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
        Self: std::marker::Sized;

    /// Maximum property value size this type can consume, if it is bounded.
    ///
    /// Returning `None` lets the property inquiry size its buffer from the MQ
    /// response; a fixed-capacity type can return its maximum here.
    #[must_use]
    fn max_value_size() -> Option<NonZero<usize>> {
        None
    }
}

/// ## Safety
/// This trait can directly manipulate the [`MQIMPO`](structs::MQIMPO) structure which is used by [`MQINQMP`](libmqm_sys::MQINQMP).
/// Incorrect values in the [`MQIMPO`](structs::MQIMPO) can lead to undefined behaviour.
///
/// Implementations of the [`PropertyAttr`] trait must ensure that pointers and offsets contained in the structure point to active data.
pub unsafe trait PropertyAttr {
    /// # Errors
    /// Returns an error if MQ cannot retrieve the property or the requested attribute.
    fn property_extract<'p, 's, F>(param: &mut PropertyParam<'p>, mqi: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
        Self: Sized;
}

/// Converts a Rust value into the raw representation MQ expects for a property.
pub trait SetProperty {
    /// Raw representation passed to MQ and the corresponding MQ property type.
    type Data: ReadRaw + ?Sized;

    /// Configure MQ property metadata and return the value bytes and MQ type.
    fn apply_mqsetmp(&self, pd: &mut structs::MQPD, smpo: &mut structs::MQSMPO) -> (&Self::Data, types::MQTYPE);
}

/// A property attribute applied alongside the property's value.
pub trait SetPropertyAttr {
    /// Update the property descriptor or set-property options.
    fn apply_mqsetmp(&self, pd: &mut structs::MQPD, smpo: &mut structs::MQSMPO);
}
