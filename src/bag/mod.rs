use std::marker::PhantomData;

use libmqm_sys::{Mqai, mqai};

use crate::{
    Buffer, result::Completion, result::Error, Library, MqFunctions, MqInqError, result::ResultComp, result::ResultCompErr, WriteRaw, constants,
    handle::BagHandle,
    prelude::*,
    types::{MQBYTE, MQCA, MQCBO, MQIA, MQIND, MQLONG, Selector},
};

mod bag_item;
mod filter;

pub use bag_item::*;
pub use filter::*;

pub mod iterator;

pub trait BagDrop: Sized {
    fn drop_bag<L: Library<MQ: Mqai>>(bag: &mut Bag<Self, L>) -> ResultComp<()>;
}

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
    handle: BagHandle,
    pub(super) mq: MqFunctions<L>,
    _marker: PhantomData<B>,
}

impl<T: BagDrop, L: Library<MQ: Mqai>> std::ops::Deref for Bag<T, L> {
    type Target = BagHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<L: Library<MQ: Mqai>> Bag<Owned, L> {
    /// This function uses the [`mqCreateBag`](libmqm_sys::mqai::mqCreateBag) MQ API function.
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
    pub const fn handle(&self) -> &BagHandle {
        &self.handle
    }

    pub const fn mut_handle(&mut self) -> &mut BagHandle {
        &mut self.handle
    }

    /// This function uses the [`mqAddInquiry`](libmqm_sys::mqai::mqAddInquiry) MQ API function.
    pub fn add_inquiry(&self, selector: Selector) -> ResultComp<()> {
        self.mq.mq_add_inquiry(self, selector)
    }

    /// This function uses the [`mqAddBag`](libmqm_sys::mqai::mqAddBag) MQ API function.
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

    /// This function uses the [`mqDeleteItem`](libmqm_sys::mqai::mqDeleteItem) MQ API function.
    pub fn delete(&self, selector: impl InqSelect) -> ResultComp<()> {
        self.mq
            .mq_delete_item(self, selector.selector(), selector.index().unwrap_or_default())
    }

    /// This function uses the [`mqClearBag`](libmqm_sys::mqai::mqClearBag) MQ API function.
    pub fn clear(&self) -> ResultComp<()> {
        self.mq.mq_clear_bag(self)
    }

    /// This function uses the [`mqTruncateBag`](libmqm_sys::mqai::mqTruncateBag) MQ API function.
    pub fn truncate(&self, count: MQLONG) -> ResultComp<()> {
        self.mq.mq_truncate_bag(self, count)
    }

    /// Renders the [`Bag`] to the provided [`Buffer`]
    ///
    /// This function uses the [`mqBagToBuffer`](libmqm_sys::mqai::mqBagToBuffer) MQ API function.
    pub fn to_buffer<'b, A: Buffer<'b, impl WriteRaw<MQBYTE>>>(&self, buffer: A) -> ResultCompErr<A, MqInqError> {
        let mut buf = buffer;
        self.mq
            .mq_bag_to_buffer(&BagHandle::from(mqai::MQHB_NONE), self.handle(), Some(buf.as_mut()))
            .map_completion(|length| buf.truncate(length.try_into().expect("mq buffer length should convert to usize")))
    }

    /// Calculates the required buffer length in bytes for the [`Bag::to_buffer`] function.
    ///
    /// This function uses the [`mqBagToBuffer`](libmqm_sys::mqai::mqBagToBuffer) MQ API function
    pub fn buffer_len(&self) -> ResultComp<usize> {
        match self.mq.mq_bag_to_buffer(
            &BagHandle::from(mqai::MQHB_NONE),
            self.handle(),
            Option::<&mut [MQBYTE]>::None,
        ) {
            Err(MqInqError::Length(len, _)) => Ok(Completion(len, None)),
            other => other,
        }
        .map_completion(|length| length.try_into().expect("mq buffer length should convert to usize"))
        .map_err(std::convert::Into::into)
    }

    /// Renders the [`Buffer`] into the [`Bag`](Self)
    ///
    /// This function uses the [`mqBufferToBag`](libmqm_sys::mqai::mqBufferToBag) MQ API function
    pub fn from_buffer(&mut self, buffer: &[MQBYTE]) -> ResultComp<()> {
        let mq = &mut self.mq;
        let handle = &mut self.handle;

        mq.mq_buffer_to_bag(&BagHandle::from(mqai::MQHB_NONE), buffer, handle)
    }

    /// The number of items in a [`Bag`] that matches the selector
    ///
    /// This function uses the [`mqCountItems`](libmqm_sys::mqai::mqCountItems) MQ API function
    pub fn count(&self, selector: Selector) -> ResultComp<MQLONG> {
        self.mq.mq_count_items(self, selector)
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
