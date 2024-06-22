use std::marker::PhantomData;

use libmqm_sys::function;

use crate::core::mqai::{MqaiSelector, MQCBO, MQCMD};
use crate::core::{self, mqai, ConnectionHandle, Library};
use crate::{
    define_mqvalue, mapping, sys, Completion, Error, MqMask, MqValue, ResultComp, ResultCompErr, ResultCompErrExt,
    ResultCompExt,
};

pub trait BagDrop: Sized {
    fn drop_bag<L: Library<MQ: function::MQAI>>(bag: &mut Bag<Self, L>) -> ResultComp<()>;
}

use super::WithMQError;
use super::{BagItemGet, BagItemPut};

define_mqvalue!(MQIND, mapping::MQIND_CONST);
impl Default for MqValue<MQIND> {
    fn default() -> Self {
        Self::from(sys::MQIND_NONE)
    }
}

impl MqValue<MqaiSelector> {
    #[must_use]
    pub const fn with_index(self, index: MqValue<MQIND>) -> (Self, MqValue<MQIND>) {
        (self, index)
    }
}

pub trait InqSelect: Copy {
    fn selector(&self) -> MqValue<MqaiSelector>;
    fn index(&self) -> Option<MqValue<MQIND>> {
        None
    }
}

impl InqSelect for MqValue<MqaiSelector> {
    fn selector(&self) -> MqValue<MqaiSelector> {
        *self
    }
}

impl InqSelect for sys::MQLONG {
    fn selector(&self) -> MqValue<MqaiSelector> {
        MqValue::from(*self)
    }
}

impl InqSelect for (MqValue<MqaiSelector>, MqValue<MQIND>) {
    fn selector(&self) -> MqValue<MqaiSelector> {
        self.0
    }

    fn index(&self) -> Option<MqValue<MQIND>> {
        Some(self.1)
    }
}

#[derive(Debug)]
pub struct Owned {}
#[derive(Debug)]
pub struct Embedded {}

impl BagDrop for Owned {
    fn drop_bag<L: Library<MQ: function::MQAI>>(bag: &mut Bag<Self, L>) -> ResultComp<()> {
        if bag.is_deletable() {
            bag.mq.mq_delete_bag(&mut bag.bag)
        } else {
            Ok(Completion((), None))
        }
    }
}
impl BagDrop for Embedded {
    fn drop_bag<L: Library<MQ: function::MQAI>>(_bag: &mut Bag<Self, L>) -> ResultComp<()> {
        Ok(Completion((), None))
    }
}

#[derive(Debug)]
pub struct Bag<B: BagDrop, L: Library<MQ: function::MQAI>> {
    bag: mqai::BagHandle,
    pub(super) mq: core::MQFunctions<L>,
    _marker: PhantomData<B>,
}

impl<T: BagDrop, L: Library<MQ: function::MQAI>> std::ops::Deref for Bag<T, L> {
    type Target = mqai::BagHandle;

    fn deref(&self) -> &Self::Target {
        &self.bag
    }
}

impl<L: Library<MQ: function::MQAI>> Bag<Owned, L> {
    pub fn new_lib(lib: L, options: MqMask<MQCBO>) -> ResultComp<Self> {
        let mq = core::MQFunctions(lib);
        let bag = mq.mq_create_bag(options)?;

        mq.mq_set_integer(
            &bag,
            MqValue::from(sys::MQIASY_CODED_CHAR_SET_ID),
            MqValue::default(),
            1208,
        )
        .discard_completion()?;

        Ok(bag.map(|bag| Self {
            bag,
            mq,
            _marker: PhantomData,
        }))
    }
}

impl<L: Library<MQ: function::MQAI>> BagItemGet<L> for Bag<Embedded, L> {
    fn inq_bag_item<B: BagDrop>(
        selector: MqValue<MqaiSelector>,
        index: MqValue<MQIND>,
        bag: &Bag<B, L>,
    ) -> ResultComp<Self> {
        bag.mq
            .mq_inquire_bag(bag, selector, index)
            .map_completion(|bag_handle| Self {
                bag: bag_handle,
                mq: bag.mq.clone(),
                _marker: PhantomData,
            })
    }

    type Error = Error;
}

impl<B: BagDrop, L: Library<MQ: function::MQAI>> Bag<B, L> {
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn handle(&self) -> &mqai::BagHandle {
        &self.bag
    }

    pub fn add_inquiry(&self, selector: MqValue<MqaiSelector>) -> ResultComp<()> {
        self.mq.mq_add_inquiry(self, selector)
    }

    pub fn add_bag<'a, 'bag: 'a>(
        &'a self,
        selector: MqValue<MqaiSelector>,
        to_attach: &'bag Bag<Owned, L>,
    ) -> ResultComp<()> {
        self.mq.mq_add_bag(self, selector, to_attach)
    }

    pub fn add<T: BagItemPut<L>>(&self, selector: MqValue<MqaiSelector>, value: T) -> ResultCompErr<(), T::Error> {
        value.add_to_bag(selector, self)
    }

    pub fn inquire<T: BagItemGet<L>>(&self, selector: impl InqSelect) -> ResultCompErr<Option<T>, T::Error> {
        match T::inq_bag_item(selector.selector(), selector.index().unwrap_or_default(), self) {
            Err(e) => match e.mqi() {
                Some(&Error(cc, _, rc)) if cc == sys::MQCC_FAILED && rc == sys::MQRC_SELECTOR_NOT_PRESENT => {
                    Ok(Completion(None, None))
                }
                _ => Err(e),
            },
            other => other.map_completion(Option::Some),
        }
    }

    // pub fn inq<T: inq::InqSelector<L>>(
    //     &self,
    //     selector: &T,
    //     index: Option<sys::MQLONG>,
    // ) -> Result<Option<T::Out>, <T::Out as BagItemGet<L>>::Error> {
    //     self.inquire(selector.attribute(), index)
    // }

    pub fn set<T: BagItemPut<L>>(&self, selector: impl InqSelect, value: T) -> ResultCompErr<(), T::Error> {
        T::set_bag_item(value, selector.selector(), selector.index().unwrap_or_default(), self)
    }

    pub fn delete(&self, selector: impl InqSelect) -> ResultComp<()> {
        self.mq
            .mq_delete_item(self, selector.selector(), selector.index().unwrap_or_default())
    }

    pub fn execute(
        &self,
        handle: &ConnectionHandle,
        command: MqValue<MQCMD>,
        options: Option<&mqai::BagHandle>,
        admin_q: Option<&core::ObjectHandle>,
        response_q: Option<&core::ObjectHandle>,
    ) -> ResultComp<Bag<Owned, L>> {
        // There shouldn't be any warnings for creating a bag - so treat the warning as an error
        let response_bag = Bag::new_lib(self.mq.0.clone(), MqMask::from(sys::MQCBO_ADMIN_BAG)).warn_as_error()?;
        self.mq
            .mq_execute(
                handle,
                command,
                options,
                self.handle(),
                response_bag.handle(),
                admin_q,
                response_q,
            )
            .map_completion(|()| response_bag)
    }
}

impl<B: BagDrop, L: Library<MQ: function::MQAI>> Drop for Bag<B, L> {
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
        let bag = Bag::new(MqMask::from(sys::MQCBO_GROUP_BAG)).expect("Failed to create bag");
        let property = bag
            .inquire::<sys::MQLONG>(MqValue::from(0))
            .expect("Failed to retrieve item");
        property.map_or_else(|| eprintln!("No CCSID!"), |ccsid| println!("CCSID is {ccsid}"));

        bag.add(MqValue::from(0), "abc").discard_completion().expect("Failed to add string");

        bag.delete(MqValue::from(0)).discard_completion().expect("Failed to delete item");
    }
}
