use std::{borrow::Cow, str::FromStr};

use crate::{sys, ConstLookup as _, HasConstLookup, HasMqNames, MQConstant};

pub trait RawValue {
    type ValueType: Copy;
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
#[repr(transparent)]
pub struct MqValue<T: RawValue>(pub T::ValueType);

impl<T: RawValue> MqValue<T> {
    pub const fn from(value: T::ValueType) -> Self {
        Self(value)
    }
}

impl<T: HasConstLookup + RawValue<ValueType = sys::MQLONG>> FromStr for MqValue<T> {
    type Err = <sys::MQLONG as FromStr>::Err;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            T::const_lookup()
                .by_name(name)
                .map_or_else(|| FromStr::from_str(name), Ok)?,
        ))
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
        let code = self
            .mq_primary_name()
            .map_or_else(|| Cow::from(attribute.to_string()), Cow::from);
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

impl<T: RawValue<ValueType = sys::MQLONG>> From<sys::MQLONG> for MqValue<T> {
    fn from(value: T::ValueType) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod test {
    use std::{error::Error, str::FromStr};

    use crate::{impl_constant_lookup, sys, ConstantItem, MqValue, RawValue};

    const LOOKUP: &[ConstantItem] = &[(1, "ONE")];
    #[derive(PartialEq)]
    struct L;
    impl RawValue for L {
        type ValueType = sys::MQLONG;
    }
    impl_constant_lookup!(L, LOOKUP);

    #[test]
    fn from_str() -> Result<(), Box<dyn Error>> {
        assert_eq!(MqValue::<L>::from_str("0")?, MqValue::from(0));
        assert_eq!(MqValue::<L>::from_str("ONE")?, MqValue::from(1));
        assert!(MqValue::<L>::from_str("TWO").is_err());

        Ok(())
    }
}
