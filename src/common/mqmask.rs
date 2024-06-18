use std::borrow::Cow;

use crate::{sys, ConstLookup, ConstantItem, HasConstLookup};

#[derive(Eq)]
#[repr(transparent)]
pub struct MqMask<T>(pub sys::MQLONG, std::marker::PhantomData<T>);

impl<T: HasConstLookup> MqMask<T> {
    pub fn masked_list(&self) -> (impl Iterator<Item = ConstantItem<'static>>, sys::MQLONG) {
        let &Self(val, ..) = self;
        let cl = T::const_lookup();
        let source = cl.all();
        let mut mask_list = Vec::new();
        let residual = source
            .into_iter()
            .filter(|&(value, name)| value != 0 && !name.ends_with("_MASK"))
            .fold(val, |acc, item @ (value, ..)| {
                let masked = val & value;
                if masked == value {
                    mask_list.push(item);
                    acc & !masked
                } else {
                    acc
                }
            });
        (mask_list.into_iter(), residual)
    }

    fn mask_str<'a>(list: impl Iterator<Item = ConstantItem<'a>>, residual: sys::MQLONG) -> Option<Cow<'a, str>> {
        let res_cow = (residual != 0).then(|| Cow::from(format!("{residual:#X}")));
        let list = list.map(|(.., name)| Cow::from(name)).chain(res_cow);
        list.reduce(|mut acc, name| {
            let acc_mut = acc.to_mut();
            acc_mut.push('|');
            acc_mut.push_str(&name);
            acc
        })
        .or_else(|| T::const_lookup().by_value(residual).next().map(Cow::from))
    }
}

impl<T> MqMask<T> {
    #[must_use]
    pub const fn from(value: sys::MQLONG) -> Self {
        Self(value, std::marker::PhantomData)
    }
}

impl<T> PartialEq for MqMask<T> {
    // Implemented PartialEq for *any* T
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> std::hash::Hash for MqMask<T> {
    // Implement Hash for *any* T
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Clone for MqMask<T> {
    // Implemented Clone for *any* T
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for MqMask<T> {} // Implement Copy for *any* T

impl<T> std::ops::BitOr for MqMask<T> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0, self.1)
    }
}

impl<T, Y: Into<sys::MQLONG>> std::ops::BitOr<Y> for MqMask<T> {
    type Output = Self;

    fn bitor(self, rhs: Y) -> Self::Output {
        Self(self.0 | rhs.into(), self.1)
    }
}

impl<T> std::ops::BitOrAssign for MqMask<T> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl<T, Y: Into<sys::MQLONG>> std::ops::BitOrAssign<Y> for MqMask<T> {
    fn bitor_assign(&mut self, rhs: Y) {
        self.0 |= rhs.into();
    }
}

impl<T> std::ops::BitAnd for MqMask<T> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0, self.1)
    }
}

impl<T, Y: Into<sys::MQLONG>> std::ops::BitAnd<Y> for MqMask<T> {
    type Output = Self;

    fn bitand(self, rhs: Y) -> Self::Output {
        Self(self.0 & rhs.into(), self.1)
    }
}

impl<T> std::ops::BitAndAssign for MqMask<T> {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl<T, Y: Into<sys::MQLONG>> std::ops::BitAndAssign<Y> for MqMask<T> {
    fn bitand_assign(&mut self, rhs: Y) {
        self.0 &= rhs.into();
    }
}

// Format of Display is 'CONSTANT_A|CONSTANT_B|(residual number))'
impl<T: HasConstLookup> std::fmt::Display for MqMask<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (list_iter, residual) = self.masked_list();
        match Self::mask_str(list_iter, residual) {
            Some(mask_str) => f.write_str(&mask_str),
            None => f.write_str(&format!("{:#X}", self.0)),
        }
    }
}

impl<T: HasConstLookup> std::fmt::Debug for MqMask<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (list, residual) = self.masked_list();
        if residual == self.0 && residual != 0 {
            f.debug_tuple("MqMask").field(&format_args!("{:#X}", self.0)).finish()
        } else if let Some(mask_str) = Self::mask_str(list, residual) {
            f.debug_tuple("MqMask")
                .field(&format_args!("{} = {:#X}", mask_str, self.0))
                .finish()
        } else {
            f.debug_tuple("MqMask").field(&format_args!("{:#X}", self.0)).finish()
        }
    }
}

impl<T> From<sys::MQLONG> for MqMask<T> {
    fn from(value: sys::MQLONG) -> Self {
        Self(value, std::marker::PhantomData)
    }
}

#[cfg(test)]
mod test {
    use crate::{impl_constant_lookup, ConstantItem, MqMask};

    const ONEB: &[ConstantItem] = &[
        (0, "ZERO"),
        (0, "ZERO_ALIAS"),
        (0b1, "ONE"),
        (0b1, "ONEB"),
        (0b1, "ONE_MASK"),
        (0b10, "TWO"),
    ];

    struct OneSource;
    impl_constant_lookup!(OneSource, ONEB);
    type MaskOne = MqMask<OneSource>;

    struct NoZeroSource;
    impl_constant_lookup!(NoZeroSource, NO_ZERO);
    const NO_ZERO: &[ConstantItem] = &[(1, "ONE")];


    #[test]
    fn mask_type() {
        let mut one = MaskOne::from(1);
        let two = (one & MqMask::from(2)) | 7;
        one |= MqMask::from(2);
        one |= 2;

        let one_copy = one;
        assert_eq!(one, one_copy);
        assert_eq!(two, MqMask::from(7));
    }

    #[test]
    fn mask_debug() {
        assert_eq!(format!("{:?}", MaskOne::from(1)), "MqMask(ONE|ONEB = 0x1)");
        assert_eq!(format!("{:?}", MaskOne::from(0)), "MqMask(ZERO = 0x0)");
        assert_eq!(format!("{:?}", MaskOne::from(0b101)), "MqMask(ONE|ONEB|0x4 = 0x5)");
        assert_eq!(format!("{:?}", MaskOne::from(0b100)), "MqMask(0x4)");
        assert_eq!(format!("{:?}", MqMask::<NoZeroSource>::from(0)), "MqMask(0x0)");
    }
}
