use std::{borrow::Cow, marker::PhantomData};
use crate::sys;

pub mod mapping;

pub trait RawValue {
    type ValueType: Copy;
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
#[repr(transparent)]
pub struct MqValue<T: RawValue>(pub(super) T::ValueType);

impl<T: RawValue> MqValue<T> {
    pub const fn new(value: T::ValueType) -> Self {
        Self(value)
    }
}

impl<T: HasConstLookup + RawValue<ValueType = sys::MQLONG>> HasMqNames for MqValue<T> {
    fn mq_names(&self) -> impl Iterator<Item = &'static str> {
        T::const_lookup().by_value(self.0)
    }
    fn mq_primary_name(&self) -> Option<&'static str> {
        self.mq_names().next()
    }
}

impl<T: RawValue<ValueType = sys::MQLONG>> MQConstant for MqValue<T> {
    fn mq_value(&self) -> sys::MQLONG {
        let Self(value) = self;
        *value
    }
}

impl<T: RawValue<ValueType = sys::MQLONG> + HasConstLookup> std::fmt::Display for MqValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(attribute) = self;
        let code = self.mq_primary_name().map_or_else(|| Cow::from(attribute.to_string()), Cow::from);
        f.write_str(&code)
    }
}

impl<T: RawValue<ValueType = sys::MQLONG> + HasConstLookup> std::fmt::Debug for MqValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(attribute) = self;
        let code = format!("{} = {attribute}", self.mq_primary_name().unwrap_or("Unknown"));
        f.debug_tuple("MqValue").field(&code).finish()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
#[repr(transparent)]
pub struct Mask<T>(pub(super) sys::MQLONG, PhantomData<T>);

impl<T: HasConstLookup> Mask<T> {
    pub fn masked_list(&self) -> (impl Iterator<Item = ConstantItem<'static>>, sys::MQLONG) {
        mask_list(T::const_lookup(), self.0)
    }
}

impl<T: HasConstLookup> std::fmt::Display for Mask<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (list_iter, residual) = self.masked_list();
        let residual = (residual != 0).then(|| Cow::from(format!("{residual:#X}")));
        let list = list_iter.map(|(.., name)| Cow::from(name)).chain(residual);
        let list_str = list.reduce(|mut acc, name| {
            let acc_mut = acc.to_mut();
            acc_mut.push('|');
            acc_mut.push_str(&name);
            acc
        }).unwrap_or_else(|| Cow::from(format!("{:#X}", self.0)));
        f.write_str(&list_str)
    }
}

impl<T: HasConstLookup> std::fmt::Debug for Mask<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Mask").field(&format!("{self} = {:#X}", self.0)).finish()
    }
}


fn mq_mask<'a>(
    source: impl IntoIterator<Item = ConstantItem<'a>>,
    input: sys::MQLONG,
) -> (impl Iterator<Item = ConstantItem<'a>>, sys::MQLONG) {
    let mut mask_list = Vec::new();
    let residual = source.into_iter().filter(|&(value, ..)| value != 0).fold(input, |acc, f| {
        let masked = input & f.0;
        if masked == f.0 {
            mask_list.push(f);
            acc & !masked
        }
        else {
            acc
        }
    });
    (mask_list.into_iter(), residual)
}

impl<T> From<sys::MQLONG> for Mask<T> {
    fn from(value: sys::MQLONG) -> Self {
        Self(value, PhantomData)
    }
}

impl<T: RawValue<ValueType = sys::MQLONG>> From<sys::MQLONG> for MqValue<T> {
    fn from(value: T::ValueType) -> Self {
        Self(value)
    }
}

/// Implements `HasConstLookup` using the provided `ConstSource` static instance
#[macro_export]
macro_rules! impl_constant_lookup {
    ($t:ty, $source:path) => {
        impl $crate::constants::HasConstLookup for $t {
            fn const_lookup<'a>() -> &'a (impl $crate::constants::ConstLookup + 'static) {
                &$source
            }
        }
    };
}
pub trait MQConstant {
    fn mq_value(&self) -> sys::MQLONG;
}

fn mask_list(cl: &impl ConstLookup, value: sys::MQLONG) -> (impl Iterator<Item = ConstantItem>, sys::MQLONG) {
    let (list, residual) = mq_mask(cl.all(), value);
    (
        list.chain(
            (value == 0)
                .then(|| cl.by_value(value).take(1))
                .into_iter()
                .flatten()
                .map(|name| (0, name)),
        ),
        residual,
    )
}

pub type ConstantItem<'a> = (sys::MQLONG, &'a str);

/// Provides an MQ lookup functions to a type
pub trait ConstLookup {
    /// All the constant names for the provided value.
    /// The first value returned by the iterator is the primary constant for the value.
    fn by_value(&self, value: sys::MQLONG) -> impl Iterator<Item = &'static str>;
    /// The constant value for the provided name.
    fn by_name(&self, name: &str) -> Option<sys::MQLONG>;
    /// The complete list of value and name constants
    fn all(&self) -> impl Iterator<Item = ConstantItem>;
}

/// MQ constant repository with primary and secondary constants
pub struct ConstSource<P, S>(P, S);
pub type PhfSource<'a> = ConstSource<&'a ::phf::Map<sys::MQLONG, &'a str>, &'a [ConstantItem<'a>]>;
pub type LinearSource<'a> = ConstSource<&'a [ConstantItem<'a>], &'a [ConstantItem<'a>]>;
pub type BinarySearchSource<'a> = ConstSource<BinarySearch<'a>, &'a [ConstantItem<'a>]>;
pub struct BinarySearch<'a>(&'a [ConstantItem<'a>]);

/// Associated constant lookup table with a type
pub trait HasConstLookup {
    /// Retrieve the static constant lookup table
    fn const_lookup<'a>() -> &'a (impl ConstLookup + 'static);
}

impl<P: ConstLookup, S: ConstLookup> ConstLookup for ConstSource<P, S> {
    fn by_value(&self, value: sys::MQLONG) -> impl Iterator<Item = &'static str> {
        let Self(source, extra) = self;
        source.by_value(value).chain(extra.by_value(value))
    }

    fn by_name(&self, name: &str) -> Option<sys::MQLONG> {
        let Self(source, extra) = self;
        source.by_name(name).or_else(|| extra.by_name(name))
    }

    fn all(&self) -> impl Iterator<Item = ConstantItem> {
        let Self(source, extra) = self;
        source.all().chain(extra.all())
    }
}

impl ConstLookup for BinarySearch<'static> {
    fn by_value(&self, value: sys::MQLONG) -> impl Iterator<Item = &'static str> {
        let list = &self.0;
        list.binary_search_by_key(&value, |&(value, ..)| value)
            .map(|index| list[index].1)
            .into_iter()
    }

    fn by_name(&self, name: &str) -> Option<sys::MQLONG> {
        let list = &self.0;
        list.binary_search_by_key(&name, |&(.., name)| name)
            .map(|index| list[index].0)
            .ok()
    }

    fn all(&self) -> impl Iterator<Item = ConstantItem> {
        let list = &self.0;
        list.iter().copied()
    }
}

impl ConstLookup for &::phf::Map<sys::MQLONG, &'static str> {
    fn by_value(&self, value: sys::MQLONG) -> impl Iterator<Item = &'static str> {
        self.get(&value).copied().into_iter()
    }

    fn by_name(&self, name: &str) -> Option<sys::MQLONG> {
        mapping::MQI_BY_STRING
            .get(name)
            .copied()
            .filter(|v| self.get(v) == Some(&name))
    }

    fn all(&self) -> impl Iterator<Item = ConstantItem> {
        self.entries().map(|(&v, &n)| (v, n))
    }
}

impl ConstLookup for &[ConstantItem<'static>] {
    fn by_value(&self, value: sys::MQLONG) -> impl Iterator<Item = &'static str> {
        self.iter().filter_map(move |&(v, name)| (v == value).then_some(name))
    }

    fn by_name(&self, name: &str) -> Option<sys::MQLONG> {
        self.iter().find_map(|&(value, n)| (n == name).then_some(value))
    }

    fn all(&self) -> impl Iterator<Item = ConstantItem> {
        self.iter().copied()
    }
}

pub trait HasMqNames {
    fn mq_names(&self) -> impl Iterator<Item = &'static str>;
    fn mq_primary_name(&self) -> Option<&'static str>;
}

impl<T: MQConstant + HasConstLookup> HasMqNames for T {
    fn mq_names(&self) -> impl Iterator<Item = &'static str> {
        Self::const_lookup().by_value(self.mq_value())
    }
    fn mq_primary_name(&self) -> Option<&'static str> {
        self.mq_names().next()
    }
}

impl<T: AsRef<sys::MQLONG>> MQConstant for T {
    fn mq_value(&self) -> sys::MQLONG {
        *self.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::ConstLookup;

    use super::{mq_mask, ConstSource, LinearSource};

    const ZERO: LinearSource = ConstSource(&[], &[]);
    const ONE: LinearSource = ConstSource(&[(1, "ONE")], &[]);
    const ONEB: LinearSource = ConstSource(&[(1, "ONE")], &[(1, "ONEB")]);

    #[test]
    fn const_source() {
        assert_eq!(ZERO.by_name("TEST"), None);
        assert_eq!(ONE.by_name("ONE"), Some(1));
        assert_eq!(ONE.by_name("ZERO"), None);
        assert_eq!(ONEB.by_name("ONEB"), Some(1));
        assert_eq!(ONEB.by_name("THREE"), None);

        assert_eq!(ONEB.by_value(1).collect::<Vec<_>>(), &["ONE", "ONEB"]);
        assert_eq!(ONEB.by_value(0).collect::<Vec<_>>(), Vec::<&str>::new());
    }

    #[test]
    fn mask() {
        let (mask_list, residual) = mq_mask(ONEB.all(), 3);

        assert_eq!(mask_list.into_iter().collect::<Vec<_>>(), &[(1, "ONE"), (1, "ONEB")]);
        assert_eq!(residual, 2);
    }
}
