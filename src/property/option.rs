use std::{borrow::Cow, num::NonZero};

use crate::{
    result::{Error, ResultComp, ResultCompErr},
    structs,
    traits::ReadRaw,
    types,
};

#[derive(Debug, Clone)]
pub struct PropertyState<'s> {
    pub name: Option<Cow<'s, [types::MQCHAR]>>,
    pub value: Cow<'s, [u8]>,
}

#[derive(Clone, Debug)]
pub struct PropertyParam<'p> {
    pub value_type: types::MQTYPE,
    pub impo: structs::MQIMPO<'p>,
    pub mqpd: structs::MQPD,
    pub name_required: NameUsage,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NameUsage {
    #[default]
    Ignored,
    MaxLength(NonZero<usize>),
    AnyLength,
}

/// # Safety
/// This trait can directly manipulate the [`MQIMPO`](structs::MQIMPO) structure which is used by [`MQINQMP`](libmqm_sys::MQINQMP) function.
/// Incorrect values in the [`MQIMPO`](structs::MQIMPO) can lead to undefined behaviour.
///
/// Implementations of the [`PropertyValue`] trait must ensure that pointers and offsets contained in the structure point to active data.
pub unsafe trait PropertyValue {
    type Error: From<Error> + Into<Error> + std::fmt::Debug;

    fn property_consume<'p, 's, F>(param: &mut PropertyParam<'p>, mqi: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
        Self: std::marker::Sized;

    #[must_use]
    fn max_value_size() -> Option<NonZero<usize>> {
        None
    }
}

/// # Safety
/// This trait can directly manipulate the [`MQIMPO`](structs::MQIMPO) structure which is used by [`MQINQMP`](libmqm_sys::MQINQMP).
/// Incorrect values in the [`MQIMPO`](structs::MQIMPO) can lead to undefined behaviour.
///
/// Implementations of the [`PropertyAttr`] trait must ensure that pointers and offsets contained in the structure point to active data.
pub unsafe trait PropertyAttr {
    fn property_extract<'p, 's, F>(param: &mut PropertyParam<'p>, mqi: F) -> ResultComp<(Self, PropertyState<'s>)>
    where
        F: FnOnce(&mut PropertyParam<'p>) -> ResultComp<PropertyState<'s>>,
        Self: Sized;
}

pub trait SetProperty {
    type Data: ReadRaw + ?Sized;
    fn apply_mqsetmp(&self, pd: &mut structs::MQPD, smpo: &mut structs::MQSMPO) -> (&Self::Data, types::MQTYPE);
}

pub trait SetPropertyAttr {
    fn apply_mqsetmp(&self, pd: &mut structs::MQPD, smpo: &mut structs::MQSMPO);
}
