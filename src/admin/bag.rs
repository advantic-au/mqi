use std::marker::PhantomData;

use libmqm_sys::function;

use crate::values::{MqaiSelector, MQIND, MQCBO, MQCC, MQRC};
use crate::core::{self, mqai, Library};
use crate::prelude::*;
use crate::{sys, Completion, Error, ResultComp, ResultCompErr, WithMqError as _};

pub trait BagDrop: Sized {
    fn drop_bag<L: Library<MQ: function::Mqai>>(bag: &mut Bag<Self, L>) -> ResultComp<()>;
}

use super::{BagItemGet, BagItemPut};

impl MqaiSelector {
    #[must_use]
    pub const fn with_index(self, index: MQIND) -> (Self, MQIND) {
        (self, index)
    }
}

pub trait InqSelect: Copy {
    fn selector(&self) -> MqaiSelector;
    fn index(&self) -> Option<MQIND> {
        None
    }
}

impl InqSelect for MqaiSelector {
    fn selector(&self) -> MqaiSelector {
        *self
    }
}

impl InqSelect for sys::MQLONG {
    fn selector(&self) -> MqaiSelector {
        MqaiSelector(*self)
    }
}

impl InqSelect for (MqaiSelector, MQIND) {
    fn selector(&self) -> MqaiSelector {
        self.0
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
    fn drop_bag<L: Library<MQ: function::Mqai>>(bag: &mut Bag<Self, L>) -> ResultComp<()> {
        if bag.is_deletable() {
            bag.mq.mq_delete_bag(&mut bag.bag)
        } else {
            Ok(Completion::new(()))
        }
    }
}
impl BagDrop for Embedded {
    fn drop_bag<L: Library<MQ: function::Mqai>>(_bag: &mut Bag<Self, L>) -> ResultComp<()> {
        Ok(Completion::new(()))
    }
}

#[derive(Debug)]
pub struct Bag<B: BagDrop, L: Library<MQ: function::Mqai>> {
    bag: mqai::BagHandle,
    pub(super) mq: core::MqFunctions<L>,
    _marker: PhantomData<B>,
}

impl<T: BagDrop, L: Library<MQ: function::Mqai>> std::ops::Deref for Bag<T, L> {
    type Target = mqai::BagHandle;

    fn deref(&self) -> &Self::Target {
        &self.bag
    }
}

impl<L: Library<MQ: function::Mqai>> Bag<Owned, L> {
    pub fn new_lib(lib: L, options: MQCBO) -> ResultComp<Self> {
        let mq = core::MqFunctions(lib);
        let bag = mq.mq_create_bag(options)?;

        mq.mq_set_integer(&bag, MqaiSelector(sys::MQIASY_CODED_CHAR_SET_ID), MQIND::default(), 1208)
            .discard_warning()?;

        Ok(bag.map(|bag| Self {
            bag,
            mq,
            _marker: PhantomData,
        }))
    }
}

impl<L: Library<MQ: function::Mqai> + Clone> BagItemGet<L> for Bag<Embedded, L> {
    fn inq_bag_item<B: BagDrop>(selector: MqaiSelector, index: MQIND, bag: &Bag<B, L>) -> ResultComp<Self> {
        bag.mq.mq_inquire_bag(bag, selector, index).map_completion(|bag_handle| Self {
            bag: bag_handle,
            mq: bag.mq.clone(),
            _marker: PhantomData,
        })
    }

    type Error = Error;
}

impl<B: BagDrop, L: Library<MQ: function::Mqai>> Bag<B, L> {
    #[must_use]
    pub const fn handle(&self) -> &mqai::BagHandle {
        &self.bag
    }

    pub fn add_inquiry(&self, selector: MqaiSelector) -> ResultComp<()> {
        self.mq.mq_add_inquiry(self, selector)
    }

    pub fn add_bag<'a, 'bag: 'a>(&'a self, selector: MqaiSelector, to_attach: &'bag Bag<Owned, L>) -> ResultComp<()> {
        self.mq.mq_add_bag(self, selector, to_attach)
    }

    pub fn add<T: BagItemPut<L> + ?Sized>(&self, selector: MqaiSelector, value: &T) -> ResultCompErr<(), T::Error> {
        value.add_to_bag(selector, self)
    }

    pub fn inquire<T: BagItemGet<L>>(&self, selector: impl InqSelect) -> ResultCompErr<Option<T>, T::Error> {
        match T::inq_bag_item(selector.selector(), selector.index().unwrap_or_default(), self) {
            Err(e) => match e.mqi_error() {
                Some(&Error(MQCC(sys::MQCC_FAILED), _, MQRC(sys::MQRC_SELECTOR_NOT_PRESENT))) => Ok(Completion::new(None)),
                _ => Err(e),
            },
            other => other.map_completion(Option::Some),
        }
    }

    pub fn set<T: BagItemPut<L>>(&self, selector: impl InqSelect, value: &T) -> ResultCompErr<(), T::Error> {
        T::set_bag_item(value, selector.selector(), selector.index().unwrap_or_default(), self)
    }

    pub fn delete(&self, selector: impl InqSelect) -> ResultComp<()> {
        self.mq
            .mq_delete_item(self, selector.selector(), selector.index().unwrap_or_default())
    }
}

impl<B: BagDrop, L: Library<MQ: function::Mqai>> Drop for Bag<B, L> {
    fn drop(&mut self) {
        let _ = B::drop_bag(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sys;

    #[test]
    fn add_items() {
        let bag = Bag::new(MQCBO(sys::MQCBO_GROUP_BAG)).expect("creation of bag to not fail");
        let property = bag
            .inquire::<sys::MQLONG>(MqaiSelector(0))
            .expect("retrieval of an item should not fail");
        property.map_or_else(|| eprintln!("No CCSID!"), |ccsid| println!("CCSID is {ccsid}"));

        bag.add(MqaiSelector(0), "abc")
            .discard_warning()
            .expect("Failed to add string");

        bag.delete(MqaiSelector(0))
            .discard_warning()
            .expect("deletion of an item should not fail");
    }
}
