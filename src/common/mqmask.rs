#![allow(clippy::allow_attributes, reason = "Macro include 'allow' for generation purposes")]

use std::borrow::Cow;

use crate::{sys, ConstLookup, ConstantItem};

#[macro_export]
macro_rules! define_mqmask {
    ($vis:vis $i:ident, $source:path) => {
        define_mqmask!($vis $i, $source, "");
    };
    ($vis:vis $i:ident, $source:path, $lit:literal) => {
        #[allow(unused_imports)]
        use $crate::constants::HasConstLookup as _;
        #[allow(unused_imports)]
        use $crate::constants::ConstLookup as _;
        #[allow(unused_imports)]
        use $crate::constants::HasMqNames as _;

        #[allow(clippy::empty_docs)]
        #[doc = $lit]
        #[derive(
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
            derive_more::From,
            derive_more::BitOr,
            derive_more::BitOrAssign,
            derive_more::BitAnd,
            derive_more::BitAndAssign,
        )]
        #[repr(transparent)]
        pub struct $i(pub $crate::sys::MQLONG);

        impl $crate::HasConstLookup for $i {
            fn const_lookup<'a>() -> &'a (impl $crate::ConstLookup + 'static) {
                &$source
            }
        }

        impl std::str::FromStr for $i {
            type Err = <$crate::sys::MQLONG as std::str::FromStr>::Err;

            fn from_str(name: &str) -> Result<Self, Self::Err> {
                Ok(Self(
                    Self::const_lookup()
                        .by_name(name)
                        .map_or_else(|| std::str::FromStr::from_str(name), Ok)?,
                ))
            }
        }

        impl $i {
            pub fn masked_list(&self) -> (impl Iterator<Item = $crate::ConstantItem<'static>>, $crate::sys::MQLONG) {
                let &Self(val) = self;
                $crate::mqmask::masked_list(val, Self::const_lookup().all())
            }

            fn mask_str<'a>(
                list: impl Iterator<Item = $crate::ConstantItem<'a>>,
                residual: $crate::sys::MQLONG,
            ) -> Option<std::borrow::Cow<'a, str>> {
                $crate::mqmask::mask_str(Self::const_lookup(), list, residual)
            }
        }

        #[allow(dead_code)]
        impl $i {
            #[must_use]
            pub const fn value(&self) -> $crate::sys::MQLONG {
                self.0
            }
        }

        impl PartialEq<$crate::sys::MQLONG> for $i {
            fn eq(&self, other: &$crate::sys::MQLONG) -> bool {
                self.0 == *other
            }
        }

        impl<Y: Into<$crate::sys::MQLONG>> std::ops::BitOr<Y> for $i {
            type Output = Self;

            fn bitor(self, rhs: Y) -> Self::Output {
                Self(self.0 | rhs.into())
            }
        }

        impl<Y: Into<$crate::sys::MQLONG>> std::ops::BitOrAssign<Y> for $i {
            fn bitor_assign(&mut self, rhs: Y) {
                self.0 |= rhs.into();
            }
        }

        impl<Y: Into<$crate::sys::MQLONG>> std::ops::BitAnd<Y> for $i {
            type Output = Self;

            fn bitand(self, rhs: Y) -> Self::Output {
                Self(self.0 & rhs.into())
            }
        }

        impl<Y: Into<$crate::sys::MQLONG>> std::ops::BitAndAssign<Y> for $i {
            fn bitand_assign(&mut self, rhs: Y) {
                self.0 &= rhs.into();
            }
        }

        // Format of Display is 'CONSTANT_A|CONSTANT_B|(residual number))'
        impl std::fmt::Display for $i {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let (list_iter, residual) = self.masked_list();
                match Self::mask_str(list_iter, residual) {
                    Some(mask_str) => f.write_str(&mask_str),
                    None => f.write_str(&format!("{:#X}", self.0)),
                }
            }
        }

        impl std::fmt::Debug for $i {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                $crate::mqmask::mask_debug(stringify!($i), self.0, Self::const_lookup(), f)
            }
        }
    };
}

pub(crate) fn mask_debug(
    type_name: &str,
    value: sys::MQLONG,
    lookup: &impl ConstLookup,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let (list, residual) = masked_list(value, lookup.all());
    if residual == value && residual != 0 {
        f.debug_tuple(type_name).field(&format_args!("{value:#X}")).finish()
    } else if let Some(mask_str) = mask_str(lookup, list, residual) {
        f.debug_tuple(type_name)
            .field(&format_args!("{mask_str} = {value:#X}"))
            .finish()
    } else {
        f.debug_tuple(type_name).field(&format_args!("{value:#X}")).finish()
    }
}

pub(crate) fn masked_list<'a>(
    value: sys::MQLONG,
    source: impl Iterator<Item = ConstantItem<'a>>,
) -> (impl Iterator<Item = ConstantItem<'a>>, sys::MQLONG) {
    let mut mask_list = Vec::new();
    let residual = source
        .into_iter()
        .filter(|&(value, name)| value != 0 && !name.ends_with("_MASK"))
        .fold(value, |acc, item @ (val, ..)| {
            let masked = value & val;
            if masked == val {
                mask_list.push(item);
                acc & !masked
            } else {
                acc
            }
        });
    (mask_list.into_iter(), residual)
}

pub(crate) fn mask_str<'a>(
    lookup: &'a impl ConstLookup,
    list: impl Iterator<Item = ConstantItem<'a>>,
    residual: sys::MQLONG,
) -> Option<Cow<'a, str>> {
    let res_cow = (residual != 0).then(|| Cow::from(format!("{residual:#X}")));
    let list = list.map(|(.., name)| Cow::from(name)).chain(res_cow);
    list.reduce(|mut acc, name| {
        let acc_mut = acc.to_mut();
        acc_mut.push('|');
        acc_mut.push_str(&name);
        acc
    })
    .or_else(|| lookup.by_value(residual).next().map(Cow::from))
}

#[cfg(test)]
mod test {
    use crate::ConstantItem;

    const ONEB: &[ConstantItem] = &[
        (0, "ZERO"),
        (0, "ZERO_ALIAS"),
        (0b1, "ONE"),
        (0b1, "ONEB"),
        (0b1, "ONE_MASK"),
        (0b10, "TWO"),
    ];
    define_mqmask!(MaskOne, ONEB);
    const NO_ZERO: &[ConstantItem] = &[(1, "ONE")];
    define_mqmask!(NoZero, NO_ZERO);

    #[test]
    fn mask_type() {
        let mut one = MaskOne::from(1);
        let two = (one & MaskOne::from(2)) | 7;
        one |= MaskOne::from(2);
        one |= 2;

        let one_copy = one;
        assert_eq!(one, one_copy);
        assert_eq!(two, MaskOne::from(7));
    }

    #[test]
    fn mask_debug() {
        assert_eq!(format!("{:?}", MaskOne::from(1)), "MaskOne(ONE|ONEB = 0x1)");
        assert_eq!(format!("{:?}", MaskOne::from(0)), "MaskOne(ZERO = 0x0)");
        assert_eq!(format!("{:?}", MaskOne::from(0b101)), "MaskOne(ONE|ONEB|0x4 = 0x5)");
        assert_eq!(format!("{:?}", MaskOne::from(0b100)), "MaskOne(0x4)");
        assert_eq!(format!("{:?}", NoZero::from(0)), "NoZero(0x0)");
    }
}
