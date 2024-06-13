use std::borrow::Cow;

use crate::{sys, ConstLookup, ConstantItem, HasConstLookup};


#[derive(Eq)]
#[repr(transparent)]
pub struct Mask<T>(pub sys::MQLONG, std::marker::PhantomData<T>);

impl<T: HasConstLookup> Mask<T> {
    pub fn masked_list(&self) -> (impl Iterator<Item = ConstantItem<'static>>, sys::MQLONG) {
        mask_list(T::const_lookup(), self.0)
    }
}

impl<T> Mask<T> {
    #[must_use]
    pub const fn from(value: sys::MQLONG) -> Self {
        Self(value, std::marker::PhantomData)
    }
}

impl<T> PartialEq for Mask<T> {
    // Implemented Hash for any T
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> std::hash::Hash for Mask<T> {
    // Implement Hash for any T
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Clone for Mask<T> {
    // Implemented Clone for *any* T
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Mask<T> {} // Implement Copy for *any* T

impl<T> std::ops::BitOr for Mask<T> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0, self.1)
    }
}

impl<T> std::ops::BitOrAssign for Mask<T> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl<T> std::ops::BitAnd for Mask<T> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0, self.1)
    }
}

impl<T> std::ops::BitAndAssign for Mask<T> {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl<T: HasConstLookup> std::fmt::Display for Mask<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (list_iter, residual) = self.masked_list();
        let residual = (residual != 0).then(|| Cow::from(format!("{residual:#X}")));
        let list = list_iter.map(|(.., name)| Cow::from(name)).chain(residual);
        let list_str = list
            .reduce(|mut acc, name| {
                let acc_mut = acc.to_mut();
                acc_mut.push('|');
                acc_mut.push_str(&name);
                acc
            })
            .unwrap_or_else(|| Cow::from(format!("{:#X}", self.0)));
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
    let residual = source
        .into_iter()
        .filter(|&(value, ..)| value != 0)
        .fold(input, |acc, f| {
            let masked = input & f.0;
            if masked == f.0 {
                mask_list.push(f);
                acc & !masked
            } else {
                acc
            }
        });
    (mask_list.into_iter(), residual)
}

impl<T> From<sys::MQLONG> for Mask<T> {
    fn from(value: sys::MQLONG) -> Self {
        Self(value, std::marker::PhantomData)
    }
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

#[cfg(test)]
mod test {
    use super::mq_mask;
    use crate::{impl_constant_lookup, ConstantItem, Mask};

    const ONEB: &[ConstantItem] = &[(0b1, "ONE"), (0b1, "ONEB"), (0b10, "TWO")];

    struct OneSource;
    impl_constant_lookup!(OneSource, ONEB);

    #[test]
    fn mask() {
        let (mask_list, residual) = mq_mask(ONEB.iter().copied(), 0b101);

        assert_eq!(mask_list.into_iter().collect::<Vec<_>>(), &[(0b1, "ONE"), (0b1, "ONEB")]);
        assert_eq!(residual, 0b100);
    }

    #[test]
    fn mask_type() {
        let mut one = Mask::<OneSource>::from(1);
        let two = (one & Mask::from(2)) | Mask::from(7);
        one |= Mask::from(2);

        let one_copy = one;
        assert_eq!(one, one_copy);
        assert_eq!(two, Mask::from(7));
    }
}