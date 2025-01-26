use libmqm_sys::Mqai;
use std::fmt::Debug;
use std::mem;

use crate::core::mqai;
use crate::values::{self, MqaiSelector, CCSID, MQIND};
use crate::core::Library;
use crate::{prelude::*, MqStr, StrCcsidOwned, StringCcsid, NATIVE_IS_LE};
use crate::{sys, Completion, EncodedString, Error, ResultComp, ResultCompErr, WithMqError};

use super::{Bag, BagDrop};

#[derive(derive_more::Error, derive_more::Display, derive_more::From, Debug)]
pub enum PutStringCcsidError {
    #[display("Provided CCSID = {}, bag CCSID = {}", _0, _1)]
    CcsidMismatch(CCSID, CCSID),
    #[from]
    Mqi(Error),
}

pub trait BagItemPut<L: Library<MQ: Mqai>> {
    type Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultCompErr<(), Self::Error>;
    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultCompErr<(), Self::Error>;
}

pub trait BagItemGet<L: Library<MQ: Mqai>>: Sized {
    type Error: WithMqError + Debug;
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultCompErr<Self, Self::Error>;
}

const STACK_BUFFER_SIZE: usize = 0x1000;

impl<L: Library<MQ: Mqai>> BagItemPut<L> for sys::MQLONG {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_integer(bag, selector, *self)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_set_integer(bag, selector, index, *self)
    }
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for sys::MQLONG {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer(bag, selector, index)
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: Mqai>> BagItemPut<L> for mqai::Filter<sys::MQLONG> {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_integer_filter(bag, selector, *self)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_set_integer_filter(bag, selector, index, *self)
    }
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for mqai::Filter<sys::MQLONG> {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer_filter(bag, selector, index)
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: Mqai>> BagItemPut<L> for i64 {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_integer64(bag, selector, *self)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_set_integer64(bag, selector, index, *self)
    }
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for i64 {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer64(bag, selector, index)
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: Mqai>> BagItemPut<L> for [sys::MQBYTE] {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_byte_string(bag, selector, self)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_set_byte_string(bag, selector, index, self)
    }
}

impl<L: Library<MQ: Mqai>> BagItemPut<L> for &[sys::MQBYTE] {
    type Error = <[sys::MQBYTE] as BagItemPut<L>>::Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultCompErr<(), Self::Error> {
        BagItemPut::add_to_bag(*self, selector, bag)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultCompErr<(), Self::Error> {
        BagItemPut::set_bag_item(*self, selector, index, bag)
    }
}

impl<L: Library<MQ: Mqai>> BagItemPut<L> for Vec<sys::MQBYTE> {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_byte_string(bag, selector, self)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_set_byte_string(bag, selector, index, self)
    }
}

impl<T: EncodedString + ?Sized, L: Library<MQ: Mqai>> BagItemPut<L> for T {
    type Error = PutStringCcsidError;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultCompErr<(), Self::Error> {
        let bag_ccsid = CCSID(
            bag.mq
                .mq_inquire_integer(bag, MqaiSelector(sys::MQIASY_CODED_CHAR_SET_ID), MQIND::default())
                .warn_as_error()?,
        );
        if bag_ccsid != self.ccsid() {
            return Err(PutStringCcsidError::CcsidMismatch(self.ccsid(), bag_ccsid));
        }
        bag.mq.mq_add_string(bag, selector, self.data()).map_err(Into::into)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultCompErr<(), Self::Error> {
        let bag_ccsid = CCSID(
            bag.mq
                .mq_inquire_integer(bag, MqaiSelector(sys::MQIASY_CODED_CHAR_SET_ID), MQIND::default())
                .warn_as_error()?,
        );
        if bag_ccsid != self.ccsid() {
            return Err(PutStringCcsidError::CcsidMismatch(self.ccsid(), bag_ccsid));
        }
        bag.mq.mq_set_string(bag, selector, index, self.data()).map_err(Into::into)
    }
}

impl<T: EncodedString, L: Library<MQ: Mqai>> BagItemPut<L> for mqai::Filter<T> {
    type Error = PutStringCcsidError;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultCompErr<(), Self::Error> {
        let Self { operator, value } = self;
        let bag_ccsid = CCSID(
            bag.mq
                .mq_inquire_integer(bag, MqaiSelector(sys::MQIASY_CODED_CHAR_SET_ID), MQIND::default())
                .warn_as_error()?,
        );
        if bag_ccsid != value.ccsid() {
            return Err(PutStringCcsidError::CcsidMismatch(value.ccsid(), bag_ccsid));
        }
        bag.mq
            .mq_add_string_filter(
                bag,
                selector,
                mqai::Filter {
                    operator: *operator,
                    value: value.data(),
                },
            )
            .map_err(Into::into)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultCompErr<(), Self::Error> {
        let Self { operator, value } = self;
        let bag_ccsid = CCSID(
            bag.mq
                .mq_inquire_integer(bag, MqaiSelector(sys::MQIASY_CODED_CHAR_SET_ID), MQIND::default())
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
                mqai::Filter {
                    operator: *operator,
                    value: value.data(),
                },
            )
            .map_err(Into::into)
    }
}

impl<L: Library<MQ: Mqai>, const N: usize> BagItemGet<L> for MqStr<N> {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        let mut result = Self::default();
        bag.mq
            .mq_inquire_string(bag, selector, index, result.as_mut())
            .map_completion(|_| result) // TODO: This ignores CCSID
    }

    type Error = crate::Error;
}

// TODO: Handle warnings better here
impl<L: Library<MQ: Mqai>> BagItemGet<L> for StrCcsidOwned {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        let mut data_s = [const { mem::MaybeUninit::uninit() }; STACK_BUFFER_SIZE];
        let (length, ccsid) = bag.mq.mq_inquire_string(bag, selector, index, &mut data_s).warn_as_error()?; // TODO: warn_as_error is probably wrong
        let str_length: usize = length.try_into().expect("mq_inquire_string should not return negative");
        let mut data = Vec::with_capacity(str_length);
        let data_write = data.spare_capacity_mut();
        if str_length > data_s.len() {
            // TODO: warn_as_error is probably wrong
            _ = bag.mq.mq_inquire_string(bag, selector, index, data_write).warn_as_error()?;
        } else {
            data_write.copy_from_slice(&data_s[..str_length]);
        }
        unsafe {
            data.set_len(str_length);
        }

        Ok(Completion(
            Self {
                le: NATIVE_IS_LE,
                ccsid,
                data,
            },
            None,
        ))
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for mqai::Filter<StrCcsidOwned> {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        let mut data_s = [const { mem::MaybeUninit::uninit() }; STACK_BUFFER_SIZE];
        let (length, ccsid, operator) = bag
            .mq
            .mq_inquire_string_filter(bag, selector, index, &mut data_s)
            .warn_as_error()?; // TODO: warn_as_error is probably wrong

        let str_length: usize = length
            .try_into()
            .expect("mq_inquire_string_filter should not return a negative length");
        let mut data = Vec::with_capacity(str_length);
        let data_write = data.spare_capacity_mut();
        if str_length > data_s.len() {
            _ = bag
                .mq
                .mq_inquire_string_filter(bag, selector, index, data_write)
                .warn_as_error()?; // TODO: warn_as_error is probably wrong
        } else {
            data_write.copy_from_slice(&data_s[..str_length]);
        }
        unsafe {
            data.set_len(str_length);
        }

        Ok(Completion(
            Self::new(
                StringCcsid {
                    le: NATIVE_IS_LE,
                    ccsid,
                    data,
                },
                operator,
            ),
            None,
        ))
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for Vec<sys::MQBYTE> {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        let mut data_s = [const { mem::MaybeUninit::uninit() }; STACK_BUFFER_SIZE];
        let length = bag
            .mq
            .mq_inquire_byte_string(bag, selector, index, &mut data_s)
            .warn_as_error()?; // TODO: warn_as_error is probably wrong
        let byte_str_length: usize = length
            .try_into()
            .expect("mq_inquire_string_filter should not return a negative length");
        let mut data = Self::with_capacity(byte_str_length);
        let data_write = data.spare_capacity_mut();
        if byte_str_length > data_s.len() {
            _ = bag
                .mq
                .mq_inquire_byte_string(bag, selector, index, data_write)
                .warn_as_error()?; // TODO: warn_as_error is probably wrong
        } else {
            data_write.copy_from_slice(&data_s[..byte_str_length]);
        }
        unsafe {
            data.set_len(byte_str_length);
        }
        Ok(Completion::new(data))
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: Mqai>> BagItemPut<L> for mqai::Filter<&[sys::MQBYTE]> {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_byte_string_filter(bag, selector, *self)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_set_byte_string_filter(bag, selector, index, *self)
    }
}

impl<L: Library<MQ: Mqai>> BagItemPut<L> for mqai::Filter<Vec<sys::MQBYTE>> {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        let Self { operator, value } = self;
        bag.mq.mq_add_byte_string_filter(
            bag,
            selector,
            mqai::Filter {
                operator: *operator,
                value,
            },
        )
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<()> {
        let Self { operator, value } = self;
        bag.mq.mq_set_byte_string_filter(
            bag,
            selector,
            index,
            mqai::Filter {
                operator: *operator,
                value,
            },
        )
    }
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for mqai::Filter<Vec<sys::MQBYTE>> {
    fn inq_bag_item<'bag, B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        let mut data_s = [const { mem::MaybeUninit::uninit() }; STACK_BUFFER_SIZE];
        let (length, operator) = bag
            .mq
            .mq_inquire_byte_string_filter(bag, selector, index, &mut data_s)
            .warn_as_error()?; // TODO: warn_as_error is probably wrong
        let byte_str_length: usize = length
            .try_into()
            .expect("mq_inquire_byte_string_filter should not return a negative length");
        let mut data = Vec::with_capacity(byte_str_length);
        let data_write = data.spare_capacity_mut();
        if byte_str_length > data_s.len() {
            _ = bag
                .mq
                .mq_inquire_byte_string_filter(bag, selector, index, data_write)
                .warn_as_error()?; // TODO: warn_as_error is probably wrong
        } else {
            data_write.copy_from_slice(&data_s[..byte_str_length]);
        }
        unsafe {
            data.set_len(byte_str_length);
        }
        Ok(Completion::new(Self::new(data, operator)))
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for (MqaiSelector, values::MQITEM) {
    type Error = Error;

    #[inline]
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultCompErr<Self, Self::Error> {
        bag.mq.mq_inquire_item_info(bag, selector, index)
    }
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for values::MQITEM {
    type Error = Error;

    #[inline]
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultCompErr<Self, Self::Error> {
        BagItemGet::inq_bag_item(selector, index, bag).map_completion(|(_, item)| item)
    }
}

impl<L: Library<MQ: Mqai>> BagItemGet<L> for values::MqaiSelector {
    type Error = Error;

    #[inline]
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultCompErr<Self, Self::Error> {
        BagItemGet::inq_bag_item(selector, index, bag).map_completion(|(selector, _)| selector)
    }
}

#[cfg(test)]
mod tests {
    use mqai::Filter;

    use super::*;
    use crate::{
        admin::Bag,
        sys,
        test::mq_library,
        values::{MqaiSelector, MQCBO},
        StrCcsidOwned,
    };

    #[test]
    fn put_inq_bag_item_types() -> Result<(), Box<dyn std::error::Error>> {
        const BYTES: [u8; 2] = [0x0, 0x1];
        const STR: &str = "test";
        let long_s: String = vec!['a'; 2 ^ 14].into_iter().collect();
        let large_bytes = vec![127u8; 2 ^ 14];

        let lib = mq_library();

        // StrCcsidOwned
        test_put_inq_bag_item(STR, lib, |s: StrCcsidOwned| assert!(s == STR))?;
        test_put_inq_bag_item(&*long_s, lib, |s: StrCcsidOwned| assert!(s == &*long_s))?;

        // Vec<sys::MQBYTE>
        test_put_inq_bag_item(BYTES.as_slice(), lib, |subject: Vec<sys::MQBYTE>| assert!(subject == BYTES))?;
        test_put_inq_bag_item(large_bytes.as_slice(), lib, |subject: Vec<sys::MQBYTE>| {
            assert!(subject == large_bytes);
        })?;

        // Filter<StrCcsidOwned>
        test_put_inq_bag_item(&Filter::greater(STR), lib, |subject: Filter<StrCcsidOwned>| {
            assert!(subject == Filter::greater(STR));
        })?;
        test_put_inq_bag_item(&Filter::greater(&*long_s), lib, |subject: Filter<StrCcsidOwned>| {
            assert!(subject == Filter::greater(&*long_s));
        })?;

        // Filter<Vec<sys::MQBYTE>>
        test_put_inq_bag_item(
            &Filter::greater(BYTES.as_slice()),
            lib,
            |subject: Filter<Vec<sys::MQBYTE>>| {
                assert!(subject == Filter::greater(BYTES));
            },
        )?;
        test_put_inq_bag_item(
            &Filter::greater(large_bytes.as_slice()),
            lib,
            |subject: Filter<Vec<sys::MQBYTE>>| {
                assert!(subject == Filter::greater(large_bytes.as_slice()));
            },
        )?;

        test_put_inq_bag_item(&99i32, lib, |subject: (MqaiSelector, values::MQITEM)| {
            assert_eq!(subject, (MqaiSelector(0), values::MQITEM(sys::MQITEM_INTEGER)));
        })?;

        test_put_inq_bag_item(&88i32, lib, |subject: MqaiSelector| {
            assert_eq!(subject, MqaiSelector(0));
        })?;

        test_put_inq_bag_item(STR, lib, |subject: values::MQITEM| {
            assert_eq!(subject, values::MQITEM(sys::MQITEM_STRING));
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
        let bag = Bag::new_lib(lib, MQCBO(sys::MQCBO_NONE)).discard_warning()?;

        let not_present: Result<Option<T>, T::Error> = bag.inquire(MqaiSelector(0)).discard_warning();
        assert!(matches!(not_present, Ok(None)));

        bag.add(MqaiSelector(0), item).discard_warning()?;
        assert(
            bag.inquire(MqaiSelector(0))
                .discard_warning()?
                .expect("Inquire on value should exist"),
        );
        bag.set(MqaiSelector(0), item).discard_warning()?;
        assert(
            bag.inquire(MqaiSelector(0))
                .discard_warning()?
                .expect("Inquire on value should exist"),
        );
        Ok(())
    }
}
