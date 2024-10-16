use libmqm_sys::function;
use std::fmt::Debug;

use crate::core::mqai;
use crate::values::{MqaiSelector, CCSID, MQIND};
use crate::core::Library;
use crate::{prelude::*, StrCcsidOwned, StringCcsid, NATIVE_IS_LE};
use crate::{sys, Completion, EncodedString, Error, MqStr, ResultComp, ResultCompErr, WithMqError};

use super::{Bag, BagDrop};

#[derive(derive_more::Error, derive_more::Display, derive_more::From, Debug)]
pub enum PutStringCcsidError {
    #[display("Provided CCSID = {}, bag CCSID = {}", _0, _1)]
    CcsidMismatch(CCSID, CCSID),
    #[from]
    Mqi(Error),
}

pub trait BagItemPut<L: Library<MQ: function::Mqai>> {
    type Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultCompErr<(), Self::Error>;
    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultCompErr<(), Self::Error>;
}

pub trait BagItemGet<L: Library<MQ: function::Mqai>>: Sized {
    type Error: WithMqError + Debug;
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultCompErr<Self, Self::Error>;
}

impl<L: Library<MQ: function::Mqai>> BagItemPut<L> for sys::MQLONG {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_integer(bag, selector, *self)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_set_integer(bag, selector, index, *self)
    }
}

impl<L: Library<MQ: function::Mqai>> BagItemGet<L> for sys::MQLONG {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer(bag, selector, index)
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: function::Mqai>> BagItemPut<L> for mqai::Filter<sys::MQLONG> {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_integer_filter(bag, selector, *self)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_set_integer_filter(bag, selector, index, *self)
    }
}

impl<L: Library<MQ: function::Mqai>> BagItemGet<L> for mqai::Filter<sys::MQLONG> {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer_filter(bag, selector, index)
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: function::Mqai>> BagItemPut<L> for i64 {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_integer64(bag, selector, *self)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_set_integer64(bag, selector, index, *self)
    }
}

impl<L: Library<MQ: function::Mqai>> BagItemGet<L> for i64 {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer64(bag, selector, index)
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: function::Mqai>> BagItemPut<L> for [sys::MQCHAR] {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_byte_string(bag, selector, self)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_set_byte_string(bag, selector, index, self)
    }
}

impl<L: Library<MQ: function::Mqai>> BagItemPut<L> for Vec<sys::MQCHAR> {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_byte_string(bag, selector, self)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq
            .mq_set_byte_string(bag, selector, index, AsRef::<[sys::MQCHAR]>::as_ref(self))
    }
}

impl<T: EncodedString + ?Sized, L: Library<MQ: function::Mqai>> BagItemPut<L> for T {
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

impl<T: EncodedString, L: Library<MQ: function::Mqai>> BagItemPut<L> for mqai::Filter<T> {
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

impl<L: Library<MQ: function::Mqai>, const N: usize> BagItemGet<L> for MqStr<N> {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        let mut result = Self::default();
        bag.mq
            .mq_inquire_string(bag, selector, index, result.as_mut())
            .map_completion(|_| result) // TODO: This ignores CCSID
    }

    type Error = crate::Error;
}

// TODO: Handle warnings better here
impl<L: Library<MQ: function::Mqai>> BagItemGet<L> for StrCcsidOwned {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        let mut data: Vec<u8> = Vec::with_capacity(page_size::get());
        let (mut str_length, mut ccsid) = bag
            .mq
            .mq_inquire_string(bag, selector, index, data.spare_capacity_mut())
            .warn_as_error()?; // TODO: warn_as_error is probably wrong
        let ulength: usize = str_length.try_into().expect("mq_inquire_string should not return negative");
        if ulength > data.len() {
            data = Vec::with_capacity(ulength);
            (str_length, ccsid) = bag
                .mq
                .mq_inquire_string(bag, selector, index, data.spare_capacity_mut())
                .warn_as_error()?; // TODO: warn_as_error is probably wrong
        }
        unsafe {
            data.set_len(str_length.try_into().expect("mq_inquire_string should not return negative"));
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

impl<L: Library<MQ: function::Mqai>> BagItemGet<L> for mqai::Filter<StrCcsidOwned> {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        let mut data = Vec::with_capacity(page_size::get());
        let (mut str_length, mut ccsid, mut operator) = bag
            .mq
            .mq_inquire_string_filter(bag, selector, index, data.spare_capacity_mut())
            .warn_as_error()?; // TODO: warn_as_error is probably wrong

        let ulength: usize = str_length
            .try_into()
            .expect("mq_inquire_string_filter should not return a negative length");
        if ulength > data.capacity() {
            data = Vec::with_capacity(ulength);
            (str_length, ccsid, operator) = bag
                .mq
                .mq_inquire_string_filter(bag, selector, index, data.spare_capacity_mut())
                .warn_as_error()?; // TODO: warn_as_error is probably wrong
        }
        unsafe {
            data.set_len(
                str_length
                    .try_into()
                    .expect("mq_inquire_string_filter should not return a negative length"),
            );
        }

        let b = unsafe { &*std::ptr::from_ref::<[u8]>(data.as_ref()) };

        Ok(Completion(
            Self::new(
                StringCcsid {
                    le: NATIVE_IS_LE,
                    ccsid,
                    data: b.into(),
                },
                operator,
            ),
            None,
        ))
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: function::Mqai>> BagItemGet<L> for Vec<sys::MQCHAR> {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        let mut data = Self::with_capacity(page_size::get());

        let mut str_length = bag
            .mq
            .mq_inquire_byte_string(bag, selector, index, data.spare_capacity_mut())
            .warn_as_error()?; // TODO: warn_as_error is probably wrong
        let ulength: usize = str_length
            .try_into()
            .expect("mq_inquire_string_filter should not return a negative length");
        if ulength > data.capacity() {
            data = Self::with_capacity(ulength);
            str_length = bag
                .mq
                .mq_inquire_byte_string(bag, selector, index, data.spare_capacity_mut())
                .warn_as_error()?; // TODO: warn_as_error is probably wrong
        }
        unsafe {
            data.set_len(
                str_length
                    .try_into()
                    .expect("mq_inquire_string_filter should not return a negative length"),
            );
        }
        Ok(Completion::new(data))
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: function::Mqai>> BagItemPut<L> for mqai::Filter<&[sys::MQCHAR]> {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_byte_string_filter(bag, selector, *self)
    }

    fn set_bag_item<B: BagDrop>(&self, selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_set_byte_string_filter(bag, selector, index, *self)
    }
}

impl<L: Library<MQ: function::Mqai>> BagItemPut<L> for mqai::Filter<Vec<sys::MQCHAR>> {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(&self, selector: MqaiSelector, bag: &Bag<B, L>) -> ResultComp<()> {
        let Self { operator, value } = self;
        bag.mq.mq_add_byte_string_filter(
            bag,
            selector,
            mqai::Filter {
                operator: *operator,
                value: AsRef::<[sys::MQCHAR]>::as_ref(&value),
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
                value: AsRef::<[sys::MQCHAR]>::as_ref(&value),
            },
        )
    }
}

impl<L: Library<MQ: function::Mqai>> BagItemGet<L> for mqai::Filter<Vec<sys::MQCHAR>> {
    fn inq_bag_item<'bag, B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        let mut data = Vec::with_capacity(page_size::get());
        let (mut length, mut operator) = bag
            .mq
            .mq_inquire_byte_string_filter(bag, selector, index, data.spare_capacity_mut())
            .warn_as_error()?; // TODO: warn_as_error is probably wrong
        let str_length: usize = length
            .try_into()
            .expect("mq_inquire_byte_string_filter should not return a negative length");
        if str_length > data.capacity() {
            data = Vec::with_capacity(str_length);
            (length, operator) = bag
                .mq
                .mq_inquire_byte_string_filter(bag, selector, index, data.spare_capacity_mut())
                .warn_as_error()?; // TODO: warn_as_error is probably wrong
        }
        unsafe {
            data.set_len(
                length
                    .try_into()
                    .expect("mq_inquire_byte_string_filter should not return a negative length"),
            );
        }
        Ok(Completion::new(Self::new(data, operator)))
    }

    type Error = crate::Error;
}
