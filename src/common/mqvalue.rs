#![allow(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]

#[macro_export]
macro_rules! define_mqvalue {
    ($vis:vis $i:ident, $source:path) => {
        #[allow(unused_imports)]
        use $crate::constants::HasConstLookup as _;
        #[allow(unused_imports)]
        use $crate::constants::ConstLookup as _;
        #[allow(unused_imports)]
        use $crate::constants::HasMqNames as _;

        #[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
        #[repr(transparent)]
        $vis struct $i(pub $crate::sys::MQLONG);

        #[allow(dead_code)]
        impl $i {
            #[must_use]
            pub const fn value(&self) -> $crate::sys::MQLONG {
                self.0
            }
        }

        impl $crate::HasConstLookup for $i {
            fn const_lookup<'a>() -> &'a (impl $crate::ConstLookup + 'static) {
                &$source
            }
        }

        impl std::str::FromStr for $i {
            type Err = <$crate::sys::MQLONG as std::str::FromStr>::Err;

            fn from_str(name: &str) -> Result<Self, Self::Err> {
                Ok(Self(
                    Self::const_lookup().by_name(name).map_or_else(|| std::str::FromStr::from_str(name), Ok)?,
                ))
            }
        }

        impl $crate::MQConstant for $i {
            fn mq_value(&self) -> $crate::sys::MQLONG {
                let Self(value) = self;
                *value
            }
        }

        impl std::fmt::Display for $i {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let Self(attribute) = self;
                let code = self
                    .mq_primary_name()
                    .map_or_else(|| std::borrow::Cow::from(attribute.to_string()), std::borrow::Cow::from);
                f.write_str(&code)
            }
        }

        impl std::fmt::Debug for $i {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let Self(attribute) = self;
                let names = self
                    .mq_names()
                    .map(std::borrow::Cow::from)
                    .reduce(|acc, name| std::borrow::Cow::from(format!("{acc}|{name}")));

                if let Some(name_str) = names {
                    f.debug_tuple(stringify!($i))
                        .field(&format_args!("{name_str} = {attribute}"))
                        .finish()
                } else {
                    f.debug_tuple(stringify!($i)).field(&format_args!("{attribute}")).finish()
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use std::{error::Error, str::FromStr};

    use crate::{ConstantItem, HasMqNames};

    const LOOKUP: &[ConstantItem] = &[(0, "ZERO"), (0, "ZERO_ALIAS"), (1, "ONE"), (1, "ONE_ALIAS")];

    define_mqvalue!(pub X, LOOKUP);

    #[test]
    fn from_str() -> Result<(), Box<dyn Error>> {
        assert_eq!(X::from(0).mq_names().collect::<Vec<_>>(), &["ZERO", "ZERO_ALIAS"]);
        assert_eq!(X::from_str("0")?, X::from(0));
        assert_eq!(X::from_str("ONE")?, X::from(1));
        assert!(X::from_str("TWO").is_err());

        Ok(())
    }

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", X::from(1)), "X(ONE|ONE_ALIAS = 1)");
        assert_eq!(format!("{:?}", X::from(0)), "X(ZERO|ZERO_ALIAS = 0)");
    }

    #[test]
    fn to_string() {
        assert_eq!(format!("{}", X::from(1)), "ONE");
        assert_eq!(format!("{}", X::from(0)), "ZERO");
        assert_eq!(format!("{}", X::from(2)), "2");
    }
}
