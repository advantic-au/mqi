use std::marker::PhantomData;

use libmqm_sys::Mqai;
use crate::types::{MQLONG, MQBYTE, Selector, MQIND, MQCBO, MQIA, MQCA};

use libmqm_sys::lib as sys;

use crate::core::mqai::BagHandle;
use crate::core::{mqai, Library, MqFunctions, MqInqError, WriteRaw};
use crate::{prelude::*, Buffer};
use crate::{constants, Completion, Error, ResultComp, ResultCompErr};

pub trait BagDrop: Sized {
    fn drop_bag<L: Library<MQ: Mqai>>(bag: &mut Bag<Self, L>) -> ResultComp<()>;
}

use super::{BagItemGet, BagItemPut};

pub trait InqSelect: Copy {
    fn selector(&self) -> Selector;
    fn index(&self) -> Option<MQIND> {
        None
    }
}

impl InqSelect for Selector {
    fn selector(&self) -> Selector {
        *self
    }
}

impl InqSelect for MQIA {
    fn selector(&self) -> Selector {
        (*self).into()
    }
}

impl InqSelect for MQCA {
    fn selector(&self) -> Selector {
        (*self).into()
    }
}

impl<T: Into<Selector> + Copy> InqSelect for (T, MQIND) {
    fn selector(&self) -> Selector {
        self.0.into()
    }

    fn index(&self) -> Option<MQIND> {
        Some(self.1)
    }
}

#[derive(Debug)]
pub struct Owned {}
#[derive(Debug)]
pub struct Embedded {}

impl BagDrop for Owned {
    fn drop_bag<L: Library<MQ: Mqai>>(bag: &mut Bag<Self, L>) -> ResultComp<()> {
        if bag.is_deletable() {
            bag.mq.mq_delete_bag(&mut bag.handle)
        } else {
            Ok(Completion::new(()))
        }
    }
}
impl BagDrop for Embedded {
    fn drop_bag<L: Library<MQ: Mqai>>(_bag: &mut Bag<Self, L>) -> ResultComp<()> {
        Ok(Completion::new(()))
    }
}

#[derive(Debug)]
pub struct Bag<B: BagDrop, L: Library<MQ: Mqai>> {
    handle: mqai::BagHandle,
    pub(super) mq: MqFunctions<L>,
    _marker: PhantomData<B>,
}

impl<T: BagDrop, L: Library<MQ: Mqai>> std::ops::Deref for Bag<T, L> {
    type Target = mqai::BagHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<L: Library<MQ: Mqai>> Bag<Owned, L> {
    pub fn new_lib(lib: L, options: MQCBO) -> ResultComp<Self> {
        let mq = MqFunctions(lib);
        let bag = mq.mq_create_bag(options)?;

        mq.mq_set_integer(&bag, constants::MQIASY_CODED_CHAR_SET_ID.into(), MQIND::default(), 1208)
            .discard_warning()?;

        Ok(bag.map(|handle| Self {
            handle,
            mq,
            _marker: PhantomData,
        }))
    }
}

impl<L: Library<MQ: Mqai> + Clone> BagItemGet<L> for Bag<Embedded, L> {
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<Self> {
        bag.mq.mq_inquire_bag(bag, selector, index).map_completion(|handle| Self {
            handle,
            mq: bag.mq.clone(),
            _marker: PhantomData,
        })
    }

    type Error = Error;
}

impl<B: BagDrop, L: Library<MQ: Mqai>> Bag<B, L> {
    #[must_use]
    pub const fn handle(&self) -> &mqai::BagHandle {
        &self.handle
    }

    pub const fn mut_handle(&mut self) -> &mut mqai::BagHandle {
        &mut self.handle
    }

    pub fn add_inquiry(&self, selector: Selector) -> ResultComp<()> {
        self.mq.mq_add_inquiry(self, selector)
    }

    pub fn add_bag<'a, 'bag: 'a>(&'a self, selector: Selector, to_attach: &'bag Bag<Owned, L>) -> ResultComp<()> {
        self.mq.mq_add_bag(self, selector, to_attach)
    }

    pub fn add<T: BagItemPut<L> + ?Sized>(&self, selector: Selector, value: &T) -> ResultCompErr<(), T::Error> {
        value.add_to_bag(selector, self)
    }

    pub fn inquire<T: BagItemGet<L>>(&self, selector: impl InqSelect) -> ResultCompErr<Option<T>, T::Error> {
        match T::inq_bag_item(selector.selector(), selector.index().unwrap_or_default(), self) {
            Err(e) => match e.mqi_error() {
                Some(&Error(constants::MQCC_FAILED, _, constants::MQRC_SELECTOR_NOT_PRESENT)) => Ok(Completion::new(None)),
                _ => Err(e),
            },
            other => other.map_completion(Option::Some),
        }
    }

    pub fn set<T: BagItemPut<L> + ?Sized>(&self, selector: impl InqSelect, value: &T) -> ResultCompErr<(), T::Error> {
        T::set_bag_item(value, selector.selector(), selector.index().unwrap_or_default(), self)
    }

    pub fn delete(&self, selector: impl InqSelect) -> ResultComp<()> {
        self.mq
            .mq_delete_item(self, selector.selector(), selector.index().unwrap_or_default())
    }

    pub fn clear(&self) -> ResultComp<()> {
        self.mq.mq_clear_bag(self)
    }

    pub fn truncate(&self, count: MQLONG) -> ResultComp<()> {
        self.mq.mq_truncate_bag(self, count)
    }

    /// Renders the [`Bag`] to the provided [`Buffer`]
    ///
    /// Uses the `mqBagToBuffer` MQ API call
    ///
    pub fn to_buffer<'b, A: Buffer<'b, impl WriteRaw<MQBYTE>>>(&self, buffer: A) -> ResultCompErr<A, MqInqError> {
        let mut buf = buffer;
        self.mq
            .mq_bag_to_buffer(&BagHandle::from(sys::MQHB_NONE), self.handle(), Some(buf.as_mut()))
            .map_completion(|length| buf.truncate(length.try_into().expect("mq buffer length should convert to usize")))
    }

    /// Calculates the required buffer length in bytes for the [`Bag::to_buffer`] function.
    ///
    /// Uses the `mqBagToBuffer` MQ API call
    ///
    pub fn buffer_len(&self) -> ResultComp<usize> {
        match self
            .mq
            .mq_bag_to_buffer(&BagHandle::from(sys::MQHB_NONE), self.handle(), Option::<&mut [MQBYTE]>::None)
        {
            Err(MqInqError::Length(len, _)) => Ok(Completion(len, None)),
            other => other,
        }
        .map_completion(|length| length.try_into().expect("mq buffer length should convert to usize"))
        .map_err(std::convert::Into::into)
    }

    pub fn from_buffer(&mut self, buffer: &[MQBYTE]) -> ResultComp<()> {
        let mq = &mut self.mq;
        let handle = &mut self.handle;

        mq.mq_buffer_to_bag(&BagHandle::from(sys::MQHB_NONE), buffer, handle)
    }
}

impl<B: BagDrop, L: Library<MQ: Mqai>> Drop for Bag<B, L> {
    fn drop(&mut self) {
        let _ = B::drop_bag(self);
    }
}

#[cfg(all(test, any(feature = "link", feature = "dlopen2")))]
mod tests {
    use super::*;
    use crate::test::mq_library;

    #[test]
    fn add_items() {
        let mq_lib = mq_library();
        let bag = Bag::new_lib(mq_lib, constants::MQCBO_GROUP_BAG).expect("creation of bag to not fail");
        let property = bag
            .inquire::<MQLONG>(Selector(0))
            .expect("retrieval of an item should not fail");
        property.map_or_else(|| eprintln!("No CCSID!"), |ccsid| println!("CCSID is {ccsid}"));

        bag.add(Selector(0), "abc").discard_warning().expect("Failed to add string");

        bag.delete(Selector(0))
            .discard_warning()
            .expect("deletion of an item should not fail");
    }
}
