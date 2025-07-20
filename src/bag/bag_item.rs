use std::{fmt::Debug, mem};

use libmqm_sys::Mqai;

use super::{Bag, BagDrop, Filter};
use crate::{
    Library, MqStr, constants,
    prelude::*,
    result::{Completion, Error, ResultComp, ResultCompErr, WithMqError},
    string::{CCSID, EncodedString, StrCcsidOwned, StringCcsid},
    types::{MQBYTE, MQIND, MQINT64, MQITEM, MQLONG, Selector},
};

#[derive(derive_more::Error, derive_more::Display, derive_more::From, Debug)]
pub enum PutStringCcsidError {
    #[display("Provided CCSID = {}, bag CCSID = {}", _0, _1)]
    CcsidMismatch(CCSID, CCSID),
    #[from]
    Mqi(Error),
}

pub trait BagItemPut<L: Library<MQ: Mqai>> {
    type Error;

    fn add_to_bag(&self, selector: Selector, bag: &Bag<impl BagDrop, L>) -> ResultCompErr<(), Self::Error>;
    fn set_bag_item(&self, selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultCompErr<(), Self::Error>;
}

pub trait BagItemGet<L: Library<MQ: Mqai>>: Sized {
    type Error: WithMqError + Debug;
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultCompErr<Self, Self::Error>;
}

const STACK_BUFFER_SIZE: usize = 0x1000;

impl WithMqError for PutStringCcsidError {
    fn mqi_error(&self) -> Option<&Error> {
        match self {
            Self::Mqi(e) => Some(e),
            Self::CcsidMismatch(..) => None,
        }
    }
}

impl<L: Library<MQ: Mqai>> BagItemPut<L> for MQLONG {
    type Error = Error;

    fn add_to_bag(&self, selector: Selector, bag: &Bag<impl BagDrop, L>) -> ResultComp<()> {
        bag.mq.mq_add_integer(bag, selector, *self)
    }

    fn set_bag_item(&self, selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<()> {
        bag.mq.mq_set_integer(bag, selector, index, *self)
    }
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for MQLONG {
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer(bag, selector, index)
    }

    type Error = Error;
}

impl<L: Library<MQ: Mqai>> BagItemPut<L> for Filter<MQLONG> {
    type Error = Error;

    fn add_to_bag(&self, selector: Selector, bag: &Bag<impl BagDrop, L>) -> ResultComp<()> {
        bag.mq.mq_add_integer_filter(bag, selector, *self)
    }

    fn set_bag_item(&self, selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<()> {
        bag.mq.mq_set_integer_filter(bag, selector, index, *self)
    }
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for Filter<MQLONG> {
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer_filter(bag, selector, index)
    }

    type Error = Error;
}

impl<L: Library<MQ: Mqai>> BagItemPut<L> for MQINT64 {
    type Error = Error;

    fn add_to_bag(&self, selector: Selector, bag: &Bag<impl BagDrop, L>) -> ResultComp<()> {
        bag.mq.mq_add_integer64(bag, selector, *self)
    }

    fn set_bag_item(&self, selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<()> {
        bag.mq.mq_set_integer64(bag, selector, index, *self)
    }
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for MQINT64 {
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer64(bag, selector, index)
    }

    type Error = Error;
}

impl<L: Library<MQ: Mqai>> BagItemPut<L> for [MQBYTE] {
    type Error = Error;

    fn add_to_bag(&self, selector: Selector, bag: &Bag<impl BagDrop, L>) -> ResultComp<()> {
        bag.mq.mq_add_byte_string(bag, selector, self)
    }

    fn set_bag_item(&self, selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<()> {
        bag.mq.mq_set_byte_string(bag, selector, index, self)
    }
}

impl<T: EncodedString + ?Sized, L: Library<MQ: Mqai>> BagItemPut<L> for T {
    type Error = PutStringCcsidError;

    fn add_to_bag(&self, selector: Selector, bag: &Bag<impl BagDrop, L>) -> ResultCompErr<(), Self::Error> {
        let bag_ccsid = CCSID(
            bag.mq
                .mq_inquire_integer(bag, constants::MQIASY_CODED_CHAR_SET_ID.into(), MQIND::default())
                .warn_as_error()?,
        );
        if bag_ccsid != self.ccsid() {
            return Err(PutStringCcsidError::CcsidMismatch(self.ccsid(), bag_ccsid));
        }
        bag.mq.mq_add_string(bag, selector, self.data()).map_err(Into::into)
    }

    fn set_bag_item(&self, selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultCompErr<(), Self::Error> {
        let bag_ccsid = CCSID(
            bag.mq
                .mq_inquire_integer(bag, constants::MQIASY_CODED_CHAR_SET_ID.into(), MQIND::default())
                .warn_as_error()?,
        );
        if bag_ccsid != self.ccsid() {
            return Err(PutStringCcsidError::CcsidMismatch(self.ccsid(), bag_ccsid));
        }
        bag.mq.mq_set_string(bag, selector, index, self.data()).map_err(Into::into)
    }
}

impl<T: EncodedString, L: Library<MQ: Mqai>> BagItemPut<L> for Filter<T> {
    type Error = PutStringCcsidError;

    fn add_to_bag(&self, selector: Selector, bag: &Bag<impl BagDrop, L>) -> ResultCompErr<(), Self::Error> {
        let Self { operator, value } = self;
        let bag_ccsid = CCSID(
            bag.mq
                .mq_inquire_integer(bag, constants::MQIASY_CODED_CHAR_SET_ID.into(), MQIND::default())
                .warn_as_error()?,
        );
        if bag_ccsid != value.ccsid() {
            return Err(PutStringCcsidError::CcsidMismatch(value.ccsid(), bag_ccsid));
        }
        bag.mq
            .mq_add_string_filter(
                bag,
                selector,
                Filter {
                    operator: *operator,
                    value: value.data(),
                },
            )
            .map_err(Into::into)
    }

    fn set_bag_item(&self, selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultCompErr<(), Self::Error> {
        let Self { operator, value } = self;
        let bag_ccsid = CCSID(
            bag.mq
                .mq_inquire_integer(bag, constants::MQIASY_CODED_CHAR_SET_ID.into(), MQIND::default())
                .warn_as_error()?,
        );
        if bag_ccsid != value.ccsid() {
            return Err(PutStringCcsidError::CcsidMismatch(value.ccsid(), bag_ccsid));
        }
        bag.mq
            .mq_set_string_filter(
                bag,
                selector,
                index,
                Filter {
                    operator: *operator,
                    value: value.data(),
                },
            )
            .map_err(Into::into)
    }
}

impl<L: Library<MQ: Mqai>, const N: usize> BagItemGet<L> for MqStr<N> {
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultCompErr<Self, Self::Error> {
        let bag_ccsid = CCSID(
            bag.mq
                .mq_inquire_integer(bag, constants::MQIASY_CODED_CHAR_SET_ID.into(), MQIND::default())
                .warn_as_error()?,
        );
        if bag_ccsid != 1208 {
            return Err(PutStringCcsidError::CcsidMismatch(CCSID(1208), bag_ccsid));
        }

        let mut outcome = Self::empty();
        bag.mq
            .mq_inquire_string(bag, selector, index, outcome.as_mut())
            .map_completion(|_| outcome)
            .map_err(PutStringCcsidError::Mqi)
    }

    type Error = PutStringCcsidError;
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for StrCcsidOwned {
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<Self> {
        let mut data_s = [const { mem::MaybeUninit::uninit() }; STACK_BUFFER_SIZE];
        let Completion((length, ccsid), mut warning) = bag.mq.mq_inquire_string(bag, selector, index, &mut data_s)?;
        let str_length: usize = length.try_into().expect("mq_inquire_string should not return negative");
        let mut data = Vec::with_capacity(str_length);
        let data_write = data.spare_capacity_mut();
        if matches!(warning, Some((constants::MQRC_STRING_TRUNCATED, _))) {
            let Completion((_, _), new_warning) = bag.mq.mq_inquire_string(bag, selector, index, data_write)?;
            warning = new_warning;
        } else {
            data_write.copy_from_slice(&data_s[..str_length]);
        }
        unsafe {
            data.set_len(str_length);
        }

        Ok(Completion(Self::from_vec(data, ccsid), warning))
    }

    type Error = Error;
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for Filter<StrCcsidOwned> {
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<Self> {
        let mut data_s = [const { mem::MaybeUninit::uninit() }; STACK_BUFFER_SIZE];
        let Completion((length, ccsid, operator), mut warning) =
            bag.mq.mq_inquire_string_filter(bag, selector, index, &mut data_s)?;
        let str_length: usize = length
            .try_into()
            .expect("mq_inquire_string_filter should not return a negative length");
        let mut data = Vec::with_capacity(str_length);
        let data_write = data.spare_capacity_mut();
        if matches!(warning, Some((constants::MQRC_STRING_TRUNCATED, _))) {
            let Completion(_, new_warning) = bag.mq.mq_inquire_string_filter(bag, selector, index, data_write)?;
            warning = new_warning;
        } else {
            data_write.copy_from_slice(&data_s[..str_length]);
        }
        unsafe {
            data.set_len(str_length);
        }

        Ok(Completion(Self::new(StringCcsid::from_vec(data, ccsid), operator), warning))
    }

    type Error = Error;
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for Vec<MQBYTE> {
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<Self> {
        let mut data_s = [const { mem::MaybeUninit::uninit() }; STACK_BUFFER_SIZE];
        let Completion(length, mut warning) = bag.mq.mq_inquire_byte_string(bag, selector, index, &mut data_s)?;
        let byte_str_length: usize = length
            .try_into()
            .expect("mq_inquire_string_filter should not return a negative length");
        let mut data = Self::with_capacity(byte_str_length);
        let data_write = data.spare_capacity_mut();
        if matches!(warning, Some((constants::MQRC_STRING_TRUNCATED, _))) {
            let Completion(_, new_warning) = bag.mq.mq_inquire_byte_string(bag, selector, index, data_write)?;
            warning = new_warning;
        } else {
            data_write.copy_from_slice(&data_s[..byte_str_length]);
        }
        unsafe {
            data.set_len(byte_str_length);
        }
        Ok(Completion(data, warning))
    }

    type Error = Error;
}

impl<L: Library<MQ: Mqai>> BagItemPut<L> for Filter<&[MQBYTE]> {
    type Error = Error;

    fn add_to_bag(&self, selector: Selector, bag: &Bag<impl BagDrop, L>) -> ResultComp<()> {
        bag.mq.mq_add_byte_string_filter(bag, selector, *self)
    }

    fn set_bag_item(&self, selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<()> {
        bag.mq.mq_set_byte_string_filter(bag, selector, index, *self)
    }
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for Filter<Vec<MQBYTE>> {
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<Self> {
        let mut data_s = [const { mem::MaybeUninit::uninit() }; STACK_BUFFER_SIZE];
        let Completion((length, operator), mut warning) =
            bag.mq.mq_inquire_byte_string_filter(bag, selector, index, &mut data_s)?;
        let byte_str_length: usize = length
            .try_into()
            .expect("mq_inquire_byte_string_filter should not return a negative length");
        let mut data = Vec::with_capacity(byte_str_length);
        let data_write = data.spare_capacity_mut();
        if matches!(warning, Some((constants::MQRC_STRING_TRUNCATED, _))) {
            let Completion(_, new_warning) = bag.mq.mq_inquire_byte_string_filter(bag, selector, index, data_write)?;
            warning = new_warning;
        } else {
            data_write.copy_from_slice(&data_s[..byte_str_length]);
        }
        unsafe {
            data.set_len(byte_str_length);
        }
        Ok(Completion(Self::new(data, operator), warning))
    }

    type Error = Error;
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for (Selector, MQITEM) {
    type Error = Error;

    #[inline]
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultCompErr<Self, Self::Error> {
        bag.mq.mq_inquire_item_info(bag, selector, index)
    }
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for MQITEM {
    type Error = Error;

    #[inline]
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultCompErr<Self, Self::Error> {
        BagItemGet::inq_bag_item(selector, index, bag).map_completion(|(_, item)| item)
    }
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for Selector {
    type Error = Error;

    #[inline]
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultCompErr<Self, Self::Error> {
        BagItemGet::inq_bag_item(selector, index, bag).map_completion(|(selector, _)| selector)
    }
}

#[cfg(all(test, any(feature = "link", feature = "dlopen2")))]
mod tests {
    use super::*;
    use crate::{string::StrCcsidOwned, test::mq_library};

    #[allow(
        clippy::allow_attributes,
        clippy::needless_borrows_for_generic_args,
        reason = "Borrow required when built with dlopen2"
    )]
    #[test]
    fn put_inq_bag_item_types() -> Result<(), Box<dyn std::error::Error>> {
        const BYTES: [u8; 2] = [0x0, 0x1];
        const STR: &str = "test";
        let long_s: String = vec!['a'; 2usize.pow(14)].into_iter().collect();
        let large_bytes = vec![127u8; 2usize.pow(14)];

        let lib = mq_library();

        // StrCcsidOwned
        test_put_inq_bag_item(STR, &lib, |s: StrCcsidOwned| assert!(s == STR))?;
        test_put_inq_bag_item(&*long_s, &lib, |s: StrCcsidOwned| assert!(s == &*long_s))?;

        // Vec<MQBYTE>
        test_put_inq_bag_item(BYTES.as_slice(), &lib, |subject: Vec<MQBYTE>| assert!(subject == BYTES))?;
        test_put_inq_bag_item(large_bytes.as_slice(), &lib, |subject: Vec<MQBYTE>| {
            assert!(subject == large_bytes);
        })?;

        // Filter<StrCcsidOwned>
        test_put_inq_bag_item(&Filter::greater(STR), &lib, |subject: Filter<StrCcsidOwned>| {
            assert!(subject == Filter::greater(STR));
        })?;
        test_put_inq_bag_item(&Filter::greater(&*long_s), &lib, |subject: Filter<StrCcsidOwned>| {
            assert!(subject == Filter::greater(&*long_s));
        })?;

        // (Selector, MQITEM)
        test_put_inq_bag_item(&99i32, &lib, |subject: (Selector, MQITEM)| {
            assert_eq!(subject, (Selector(0), constants::MQITEM_INTEGER));
        })?;

        // Selector
        test_put_inq_bag_item(&88i32, &lib, |subject: Selector| {
            assert_eq!(subject, Selector(0));
        })?;

        // MQITEM
        test_put_inq_bag_item(STR, &lib, |subject: MQITEM| {
            assert_eq!(subject, constants::MQITEM_STRING);
        })?;

        // MQLONG
        test_put_inq_bag_item(&69i32, &lib, |subject: MQLONG| assert_eq!(subject, 69))?;

        // MQINT64
        test_put_inq_bag_item(&169i64, &lib, |subject: MQINT64| assert_eq!(subject, 169))?;

        // Filter<MQLONG>
        test_put_inq_bag_item(&Filter::greater(69i32), &lib, |subject: Filter<MQLONG>| {
            assert_eq!(subject, Filter::greater(69));
        })?;

        // MqStr<N>
        test_put_inq_bag_item(STR, &lib, |subject: MqStr<20>| {
            assert_eq!(subject, STR);
        })?;

        Ok(())
    }

    fn test_put_inq_bag_item<T, I, L, F>(item: &I, lib: L, assert: F) -> Result<(), Box<dyn std::error::Error>>
    where
        T: super::BagItemGet<L> + Debug,
        I: super::BagItemPut<L> + Debug + ?Sized,
        T::Error: std::error::Error + 'static,
        I::Error: std::error::Error + 'static,
        L: Library<MQ: Mqai>,
        F: Fn(T),
    {
        let bag = Bag::new_lib(lib, constants::MQCBO_NONE).discard_warning()?;

        let not_present: Result<Option<T>, T::Error> = bag.inquire(Selector(0)).discard_warning();
        assert!(matches!(not_present, Ok(None)));

        bag.add(Selector(0), item).discard_warning()?;
        assert(
            bag.inquire(Selector(0))
                .discard_warning()?
                .expect("Inquire on value should exist"),
        );
        bag.set(Selector(0), item).discard_warning()?;
        assert(
            bag.inquire(Selector(0))
                .discard_warning()?
                .expect("Inquire on value should exist"),
        );
        Ok(())
    }
}
