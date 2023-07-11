use crate::sys;

pub mod mapping;

pub fn mq_mask<'a>(
    source: impl Iterator<Item = ConstantItem<'a>>,
    input: sys::MQLONG,
) -> impl Iterator<Item = ConstantItem<'a>> {
    source.filter(move |&(value, ..)| value != 0 && (value & input) == value)
}

/// Implements `HasConstLookup` using the provided `ConstSource` static instance
#[macro_export]
macro_rules! impl_constant_lookup {
    ($t:ty, $source:path) => {
        impl constants::HasConstLookup for $t {
            fn const_lookup<'a>() -> &'a (impl constants::ConstLookup + 'static) {
                &$source
            }
        }
    };
}
pub trait MQConstant {
    fn mq_value(&self) -> sys::MQLONG;
}

type ConstantItem<'a> = (sys::MQLONG, &'a str);

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
        assert_eq!(mq_mask(ONEB.all(), 1).collect::<Vec<_>>(), &[(1, "ONE"), (1, "ONEB")]);
    }
}
