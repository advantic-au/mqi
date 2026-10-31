use std::marker::PhantomData;

use libmqm_sys::mqai;

use crate::{
    MqFunctions, MqaiLibrary, constants,
    handle::BagHandle,
    prelude::*,
    result::{Completion, Error, ResultComp, ResultCompErr},
    traits::{Buffer, WriteRaw},
    types::{MQBYTE, MQCA, MQCBO, MQIA, MQIND, MQLONG, Selector},
    verb::MqInqError,
};

mod bag_item;
mod filter;

pub use bag_item::*;
pub use filter::*;

/// Iterator types for reading repeated items from MQAI bags.
pub mod iterator;

/// Defines how a bag's handle is released when the bag is dropped.
pub trait BagDrop: Sized {
    /// # Errors
    /// Returns an error if MQ cannot delete the bag.
    fn drop_bag<L: MqaiLibrary>(bag: &mut Bag<Self, L>) -> ResultComp<()>;
}

/// Converts an inquiry selector, optionally paired with an item index.
pub trait InqSelect: Copy {
    /// Return the selector identifying the bag item.
    fn selector(&self) -> Selector;
    /// Return an optional index for selecting one repeated item.
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
/// Marker for a bag whose handle is owned and deleted when dropped.
pub struct Owned {}
#[derive(Debug)]
/// Marker for a bag handle embedded in another bag and not deleted separately.
pub struct Embedded {}

impl BagDrop for Owned {
    fn drop_bag<L: MqaiLibrary>(bag: &mut Bag<Self, L>) -> ResultComp<()> {
        if bag.is_deletable() {
            bag.mq.mq_delete_bag(&mut bag.handle)
        } else {
            Ok(Completion::new(()))
        }
    }
}
impl BagDrop for Embedded {
    fn drop_bag<L: MqaiLibrary>(_bag: &mut Bag<Self, L>) -> ResultComp<()> {
        Ok(Completion::new(()))
    }
}

#[derive(Debug)]
/// An MQAI bag containing typed items addressed by selectors.
///
/// Bags marked [`Owned`] release their MQ handle when dropped. Bags marked
/// [`Embedded`] represent nested handles owned by another bag.
pub struct Bag<B: BagDrop, L: MqaiLibrary> {
    handle: BagHandle,
    pub(super) mq: MqFunctions<L>,
    _marker: PhantomData<B>,
}

impl<T: BagDrop, L: MqaiLibrary> std::ops::Deref for Bag<T, L> {
    type Target = BagHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<L: MqaiLibrary> Bag<Owned, L> {
    /// This function uses the [`mqCreateBag`](libmqm_sys::mqai::mqCreateBag) MQ API function.
    ///
    /// # Errors
    /// Returns an error if MQ cannot create the bag or initialize its character set.
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

impl<L: MqaiLibrary + Clone> BagItemGet<L> for Bag<Embedded, L> {
    fn inq_bag_item(selector: Selector, index: MQIND, bag: &Bag<impl BagDrop, L>) -> ResultComp<Self> {
        bag.mq.mq_inquire_bag(bag, selector, index).map_completion(|handle| Self {
            handle,
            mq: bag.mq.clone(),
            _marker: PhantomData,
        })
    }

    type Error = Error;
}

impl<B: BagDrop, L: MqaiLibrary> Bag<B, L> {
    /// Return the MQAI handle for this bag.
    #[must_use]
    pub const fn handle(&self) -> &BagHandle {
        &self.handle
    }

    /// Borrow the mutable MQAI handle slot used by MQAI calls.
    pub const fn mut_handle(&mut self) -> &mut BagHandle {
        &mut self.handle
    }

    /// This function uses the [`mqAddInquiry`](libmqm_sys::mqai::mqAddInquiry) MQ API function.
    /// # Errors
    /// Returns an error if MQ cannot add the inquiry selector.
    pub fn add_inquiry(&self, selector: Selector) -> ResultComp<()> {
        self.mq.mq_add_inquiry(self, selector)
    }

    /// This function uses the [`mqAddBag`](libmqm_sys::mqai::mqAddBag) MQ API function.
    /// # Errors
    /// Returns an error if MQ cannot add the bag.
    pub fn add_bag<'a, 'bag: 'a>(&'a self, selector: Selector, to_attach: &'bag Bag<Owned, L>) -> ResultComp<()> {
        self.mq.mq_add_bag(self, selector, to_attach)
    }

    /// # Errors
    /// Returns an error if the value cannot be added to the bag.
    pub fn add<T: BagItemPut<L> + ?Sized>(&self, selector: Selector, value: &T) -> ResultCompErr<(), T::Error> {
        value.add_to_bag(selector, self)
    }

    /// # Errors
    /// Returns an error if MQ cannot retrieve or convert the selected item.
    pub fn inquire<T: BagItemGet<L>>(&self, selector: impl InqSelect) -> ResultCompErr<Option<T>, T::Error> {
        match T::inq_bag_item(selector.selector(), selector.index().unwrap_or_default(), self) {
            Err(e) => match e.mqi_error() {
                Some(&Error(constants::MQCC_FAILED, _, constants::MQRC_SELECTOR_NOT_PRESENT)) => Ok(Completion::new(None)),
                _ => Err(e),
            },
            other => other.map_completion(Option::Some),
        }
    }

    /// # Errors
    /// Returns an error if MQ cannot update the selected item.
    pub fn set<T: BagItemPut<L> + ?Sized>(&self, selector: impl InqSelect, value: &T) -> ResultCompErr<(), T::Error> {
        T::set_bag_item(value, selector.selector(), selector.index().unwrap_or_default(), self)
    }

    /// This function uses the [`mqDeleteItem`](libmqm_sys::mqai::mqDeleteItem) MQ API function.
    /// # Errors
    /// Returns an error if MQ cannot delete the selected item.
    pub fn delete(&self, selector: impl InqSelect) -> ResultComp<()> {
        self.mq
            .mq_delete_item(self, selector.selector(), selector.index().unwrap_or_default())
    }

    /// This function uses the [`mqClearBag`](libmqm_sys::mqai::mqClearBag) MQ API function.
    /// # Errors
    /// Returns an error if MQ cannot clear the bag.
    pub fn clear(&self) -> ResultComp<()> {
        self.mq.mq_clear_bag(self)
    }

    /// This function uses the [`mqTruncateBag`](libmqm_sys::mqai::mqTruncateBag) MQ API function.
    /// # Errors
    /// Returns an error if MQ cannot truncate the bag.
    pub fn truncate(&self, count: MQLONG) -> ResultComp<()> {
        self.mq.mq_truncate_bag(self, count)
    }

    /// Renders the [`Bag`] to the provided [`Buffer`]
    ///
    /// This function uses the [`mqBagToBuffer`](libmqm_sys::mqai::mqBagToBuffer) MQ API function.
    ///
    /// # Errors
    /// Returns an error if MQ cannot encode the bag or the buffer is insufficient.
    ///
    /// # Panics
    /// Panics if MQ reports an output length that cannot be represented as `usize`.
    pub fn to_buffer<'b, A: Buffer<'b, impl WriteRaw<MQBYTE>>>(&self, buffer: A) -> ResultCompErr<A, MqInqError> {
        let mut buf = buffer;
        self.mq
            .mq_bag_to_buffer(&BagHandle::from(mqai::MQHB_NONE), self.handle(), Some(buf.as_mut()))
            .map_completion(|length| buf.truncate(length.try_into().expect("mq buffer length should convert to usize")))
    }

    /// Calculates the required buffer length in bytes for the [`Bag::to_buffer`] function.
    ///
    /// This function uses the [`mqBagToBuffer`](libmqm_sys::mqai::mqBagToBuffer) MQ API function
    ///
    /// # Errors
    /// Returns an error if MQ cannot calculate the required buffer length.
    ///
    /// # Panics
    /// Panics if MQ reports a length that cannot be represented as `usize`.
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
    /// # Errors
    /// Returns an error if MQ cannot decode the buffer into the bag.
    pub fn from_buffer(&mut self, buffer: &[MQBYTE]) -> ResultComp<()> {
        let mq = &mut self.mq;
        let handle = &mut self.handle;

        mq.mq_buffer_to_bag(&BagHandle::from(mqai::MQHB_NONE), buffer, handle)
    }

    /// The number of items in a [`Bag`] that matches the selector
    ///
    /// This function uses the [`mqCountItems`](libmqm_sys::mqai::mqCountItems) MQ API function
    /// # Errors
    /// Returns an error if MQ cannot count the selected items.
    pub fn count(&self, selector: Selector) -> ResultComp<MQLONG> {
        self.mq.mq_count_items(self, selector)
    }
}

impl<B: BagDrop, L: MqaiLibrary> Drop for Bag<B, L> {
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
