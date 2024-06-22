use libmqm_sys::function;
use std::{fmt::Debug, num::NonZeroI32};
use thiserror::Error;

use crate::core::mqai::{self, MqaiSelector};
use crate::core::Library;
use crate::{mq, sys, Completion, Error, MqMask, MqStr, MqValue, ResultComp, ResultCompErr, ResultCompErrExt, ResultCompExt};

use super::{Bag, BagDrop, MQIND};

// TODO: Candidate to move to common module
pub trait WithMQError {
    fn mqi(&self) -> Option<&Error>;
}

impl WithMQError for Error {
    fn mqi(&self) -> Option<&Error> {
        Some(self)
    }
}
// TODO: Candidate to move to common module (END)

#[derive(Error, Debug)]
pub enum PutStringCcsidError {
    #[error("Provided CCSID = {}, bag CCSID = {}", .0.map_or(0, NonZeroI32::get), .1.map_or(0, NonZeroI32::get))]
    CcsidMismatch(Option<NonZeroI32>, Option<NonZeroI32>),
    #[error(transparent)]
    Mqi(#[from] Error),
}

pub trait BagItemPut<L: Library<MQ: function::MQAI>> {
    type Error;

    fn add_to_bag<B: BagDrop>(self, selector: MqValue<MqaiSelector>, bag: &Bag<B, L>) -> ResultCompErr<(), Self::Error>;
    fn set_bag_item<B: BagDrop>(
        self,
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultCompErr<(), Self::Error>;
}

pub trait BagItemGet<L: Library<MQ: function::MQAI>>: Sized {
    type Error: WithMQError + Debug;
    fn inq_bag_item<B: BagDrop>(
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultCompErr<Self, Self::Error>;
}

impl<L: Library<MQ: function::MQAI>> BagItemPut<L> for sys::MQLONG {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(self, selector: MqValue<MqaiSelector>, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_integer(bag, selector, self)
    }

    fn set_bag_item<B: BagDrop>(
        self,
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<()> {
        bag.mq.mq_set_integer(bag, selector, index, self)
    }
}

impl<L: Library<MQ: function::MQAI>> BagItemGet<L> for sys::MQLONG {
    fn inq_bag_item<B: BagDrop>(
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer(bag, selector, index)
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: function::MQAI>, T> BagItemGet<L> for MqValue<T> {
    fn inq_bag_item<B: BagDrop>(
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer(bag, selector, index).map_completion(Self::from)
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: function::MQAI>, T> BagItemGet<L> for MqMask<T> {
    fn inq_bag_item<B: BagDrop>(
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer(bag, selector, index).map_completion(Self::from)
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: function::MQAI>> BagItemPut<L> for mqai::Filter<sys::MQLONG> {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(self, selector: MqValue<MqaiSelector>, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_integer_filter(bag, selector, self)
    }

    fn set_bag_item<B: BagDrop>(
        self,
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<()> {
        bag.mq.mq_set_integer_filter(bag, selector, index, self)
    }
}

impl<L: Library<MQ: function::MQAI>> BagItemGet<L> for mqai::Filter<sys::MQLONG> {
    fn inq_bag_item<B: BagDrop>(
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer_filter(bag, selector, index)
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: function::MQAI>> BagItemPut<L> for i64 {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(self, selector: MqValue<MqaiSelector>, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_integer64(bag, selector, self)
    }

    fn set_bag_item<B: BagDrop>(
        self,
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<()> {
        bag.mq.mq_set_integer64(bag, selector, index, self)
    }
}

impl<L: Library<MQ: function::MQAI>> BagItemGet<L> for i64 {
    fn inq_bag_item<B: BagDrop>(
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<Self> {
        bag.mq.mq_inquire_integer64(bag, selector, index)
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: function::MQAI>> BagItemPut<L> for &[sys::MQCHAR] {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(self, selector: MqValue<MqaiSelector>, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_byte_string(bag, selector, self)
    }

    fn set_bag_item<B: BagDrop>(
        self,
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<()> {
        bag.mq.mq_set_byte_string(bag, selector, index, self)
    }
}

impl<L: Library<MQ: function::MQAI>> BagItemPut<L> for Vec<sys::MQCHAR> {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(self, selector: MqValue<MqaiSelector>, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_byte_string(bag, selector, &self)
    }

    fn set_bag_item<B: BagDrop>(
        self,
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<()> {
        bag.mq
            .mq_set_byte_string(bag, selector, index, AsRef::<[sys::MQCHAR]>::as_ref(&self))
    }
}

impl<T: mqai::EncodedString, L: Library<MQ: function::MQAI>> BagItemPut<L> for T {
    type Error = PutStringCcsidError;

    fn add_to_bag<B: BagDrop>(self, selector: MqValue<MqaiSelector>, bag: &Bag<B, L>) -> ResultCompErr<(), Self::Error> {
        let bag_ccsid = NonZeroI32::new(bag.mq.mq_inquire_integer(
            bag,
            MqValue::from(sys::MQIASY_CODED_CHAR_SET_ID),
            MqValue::default(),
        ).warn_as_error()?);
        if bag_ccsid != self.ccsid() {
            return Err(PutStringCcsidError::CcsidMismatch(self.ccsid(), bag_ccsid));
        }
        bag.mq.mq_add_string(bag, selector, self.data()).map_err(Into::into)
    }

    fn set_bag_item<B: BagDrop>(
        self,
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultCompErr<(), Self::Error> {
        let bag_ccsid = NonZeroI32::new(bag.mq.mq_inquire_integer(
            bag,
            MqValue::from(sys::MQIASY_CODED_CHAR_SET_ID),
            MqValue::default(),
        ).warn_as_error()?);
        if bag_ccsid != self.ccsid() {
            return Err(PutStringCcsidError::CcsidMismatch(self.ccsid(), bag_ccsid));
        }
        bag.mq
            .mq_set_string(bag, selector, index, self.data())
            .map_err(Into::into)
    }
}

impl<T: mqai::EncodedString + std::fmt::Debug, L: Library<MQ: function::MQAI>> BagItemPut<L> for mqai::Filter<T> {
    type Error = PutStringCcsidError;

    fn add_to_bag<B: BagDrop>(self, selector: MqValue<MqaiSelector>, bag: &Bag<B, L>) -> ResultCompErr<(), Self::Error> {
        let Self { operator, value } = self;
        let bag_ccsid = NonZeroI32::new(bag.mq.mq_inquire_integer(
            bag,
            MqValue::from(sys::MQIASY_CODED_CHAR_SET_ID),
            MqValue::default(),
        ).warn_as_error()?);
        if bag_ccsid != value.ccsid() {
            return Err(PutStringCcsidError::CcsidMismatch(value.ccsid(), bag_ccsid));
        }
        bag.mq
            .mq_add_string_filter(
                bag,
                selector,
                mqai::Filter {
                    operator,
                    value: value.data(),
                },
            )
            .map_err(Into::into)
    }

    fn set_bag_item<B: BagDrop>(
        self,
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultCompErr<(), Self::Error> {
        let Self { operator, value } = self;
        let bag_ccsid = NonZeroI32::new(bag.mq.mq_inquire_integer(
            bag,
            MqValue::from(sys::MQIASY_CODED_CHAR_SET_ID),
            MqValue::default(),
        ).warn_as_error()?);
        if bag_ccsid != value.ccsid() {
            return Err(PutStringCcsidError::CcsidMismatch(value.ccsid(), bag_ccsid));
        }
        bag.mq
            .mq_set_string_filter(
                bag,
                selector,
                index,
                mqai::Filter {
                    operator,
                    value: value.data(),
                },
            )
            .map_err(Into::into)
    }
}

impl<L: Library<MQ: function::MQAI>, const N: usize> BagItemGet<L> for MqStr<N> {
    fn inq_bag_item<B: BagDrop>(
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<Self> {
        let mut result = Self::default();
        bag.mq.mq_inquire_string(bag, selector, index, result.as_mut()).map_completion(|_| result) // FIXME: This ignores CCSID
    }

    type Error = crate::Error;
}


// TODO: Handle warnings better here
impl<L: Library<MQ: function::MQAI>> BagItemGet<L> for mq::StringCcsid {
    fn inq_bag_item<B: BagDrop>(
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<Self> {
        let mut data: Vec<u8> = Vec::with_capacity(page_size::get());
        let (mut str_length, mut ccsid) = bag
            .mq
            .mq_inquire_string(bag, selector, index, data.spare_capacity_mut()).warn_as_error()?; // TODO: warn_as_error is probably wrong
        let ulength: usize = str_length
            .try_into()
            .expect("mq_inquire_string returned a negative length");
        if ulength > data.len() {
            data = Vec::with_capacity(ulength);
            (str_length, ccsid) = bag
                .mq
                .mq_inquire_string(bag, selector, index, data.spare_capacity_mut()).warn_as_error()?; // TODO: warn_as_error is probably wrong
        }
        unsafe {
            data.set_len(
                str_length
                    .try_into()
                    .expect("mq_inquire_string returned a negative length"),
            );
        }

        Ok(Completion(Self {
            ccsid: NonZeroI32::new(ccsid),
            data,
        }, None))

    }

    type Error = crate::Error;
}

impl<L: Library<MQ: function::MQAI>> BagItemGet<L> for mqai::Filter<mq::StringCcsid> {
    fn inq_bag_item<B: BagDrop>(
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<Self> {
        let mut data = Vec::with_capacity(page_size::get());
        let (mut str_length, mut ccsid, mut operator) =
            bag.mq
                .mq_inquire_string_filter(bag, selector, index, data.spare_capacity_mut()).warn_as_error()?; // TODO: warn_as_error is probably wrong

        let ulength: usize = str_length
            .try_into()
            .expect("mq_inquire_string_filter returned a negative length");
        if ulength > data.capacity() {
            data = Vec::with_capacity(ulength);
            (str_length, ccsid, operator) =
                bag.mq
                    .mq_inquire_string_filter(bag, selector, index, data.spare_capacity_mut()).warn_as_error()?; // TODO: warn_as_error is probably wrong
        }
        unsafe {
            data.set_len(
                str_length
                    .try_into()
                    .expect("mq_inquire_string_filter returned a negative length"),
            );
        }

        let b = unsafe { &*std::ptr::from_ref::<[u8]>(data.as_ref()) };

        Ok(Completion(Self::new(
            mq::StringCcsid {
                ccsid: NonZeroI32::new(ccsid),
                data: b.into(),
            },
            operator,
        ), None))
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: function::MQAI>> BagItemGet<L> for Vec<sys::MQCHAR> {
    fn inq_bag_item<B: BagDrop>(
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<Self> {
        let mut data = Self::with_capacity(page_size::get());

        let mut str_length = bag
            .mq
            .mq_inquire_byte_string(bag, selector, index, data.spare_capacity_mut()).warn_as_error()?; // TODO: warn_as_error is probably wrong
        let ulength: usize = str_length
            .try_into()
            .expect("mq_inquire_string_filter returned a negative length");
        if ulength > data.capacity() {
            data = Self::with_capacity(ulength);
            str_length = bag
                .mq
                .mq_inquire_byte_string(bag, selector, index, data.spare_capacity_mut()).warn_as_error()?; // TODO: warn_as_error is probably wrong
        }
        unsafe {
            data.set_len(
                str_length
                    .try_into()
                    .expect("mq_inquire_string_filter returned a negative length"),
            );
        }
        Ok(Completion(data, None))
    }

    type Error = crate::Error;
}

impl<L: Library<MQ: function::MQAI>> BagItemPut<L> for mqai::Filter<&[sys::MQCHAR]> {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(self, selector: MqValue<MqaiSelector>, bag: &Bag<B, L>) -> ResultComp<()> {
        bag.mq.mq_add_byte_string_filter(bag, selector, self)
    }

    fn set_bag_item<B: BagDrop>(
        self,
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<()> {
        bag.mq.mq_set_byte_string_filter(bag, selector, index, self)
    }
}

impl<L: Library<MQ: function::MQAI>> BagItemPut<L> for mqai::Filter<Vec<sys::MQCHAR>> {
    type Error = Error;

    fn add_to_bag<B: BagDrop>(self, selector: MqValue<MqaiSelector>, bag: &Bag<B, L>) -> ResultComp<()> {
        let Self { operator, value } = self;
        bag.mq.mq_add_byte_string_filter(
            bag,
            selector,
            mqai::Filter {
                operator,
                value: AsRef::<[sys::MQCHAR]>::as_ref(&value),
            },
        )
    }

    fn set_bag_item<B: BagDrop>(
        self,
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<()> {
        let Self { operator, value } = self;
        bag.mq.mq_set_byte_string_filter(
            bag,
            selector,
            index,
            mqai::Filter {
                operator,
                value: AsRef::<[sys::MQCHAR]>::as_ref(&value),
            },
        )
    }
}

impl<L: Library<MQ: function::MQAI>> BagItemGet<L> for mqai::Filter<Vec<sys::MQCHAR>> {
    fn inq_bag_item<'bag, B: BagDrop>(
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<Self> {
        let mut data = Vec::with_capacity(page_size::get());
        let (mut length, mut operator) =
            bag.mq
                .mq_inquire_byte_string_filter(bag, selector, index, data.spare_capacity_mut()).warn_as_error()?; // TODO: warn_as_error is probably wrong
        let str_length: usize = length
            .try_into()
            .expect("mq_inquire_byte_string_filter returned a negative length");
        if str_length > data.capacity() {
            data = Vec::with_capacity(str_length);
            (length, operator) =
                bag.mq
                    .mq_inquire_byte_string_filter(bag, selector, index, data.spare_capacity_mut()).warn_as_error()?; // TODO: warn_as_error is probably wrong
        }
        unsafe {
            data.set_len(
                length
                    .try_into()
                    .expect("mq_inquire_byte_string_filter returned a negative length"),
            );
        }
        Ok(Completion(Self::new(data, operator), None))
    }

    type Error = crate::Error;
}

impl mqai::EncodedString for &str {
    fn ccsid(&self) -> Option<NonZeroI32> {
        NonZeroI32::new(1208) // = UTF-8 CCSID. str types are _always_ UTF-8
    }

    fn data(&self) -> &[u8] {
        unsafe { &*(std::ptr::from_ref::<str>(*self) as *const [u8]) }
    }
}

impl mqai::EncodedString for mq::StringCcsid {
    fn ccsid(&self) -> Option<NonZeroI32> {
        self.ccsid
    }

    fn data(&self) -> &[u8] {
        &self.data
    }
}
