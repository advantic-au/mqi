use std::marker::PhantomData;

use libmqm_sys::function;

use crate::core::mqai::{Command, CreateBagOptions, MqaiSelector};
use crate::core::{self, mqai, ConnectionHandle, Library};
use crate::{sys, CompletionCode, Error, Mask, MqValue, ReasonCode, ResultComp, ResultErr};

pub trait BagDrop: Sized {
    fn drop_bag<L: Library>(bag: &mut Bag<Self, L>) -> ResultErr<()>
    where
        L::MQ: function::MQAI;
}

use super::WithMQError;
use super::{inq, BagItemGet, BagItemPut};

#[derive(Debug, Clone, Copy, Hash)]
pub enum BagIndex {
    Position(i32),
    None,
    All,
}

impl From<BagIndex> for sys::MQLONG {
    fn from(value: BagIndex) -> Self {
        match value {
            BagIndex::Position(n) => n as _,
            BagIndex::None => sys::MQIND_NONE,
            BagIndex::All => sys::MQIND_ALL,
        }
    }
}

#[derive(Debug)]
pub struct Owned {}
#[derive(Debug)]
pub struct Embedded {}

impl BagDrop for Owned {
    fn drop_bag<L: Library>(bag: &mut Bag<Self, L>) -> ResultErr<()>
    where
        L::MQ: function::MQAI,
    {
        if bag.is_deletable() {
            bag.mq.mq_delete_bag(&mut bag.bag)
        } else {
            Ok(())
        }
    }
}
impl BagDrop for Embedded {
    fn drop_bag<L: Library>(_bag: &mut Bag<Self, L>) -> ResultErr<()>
    where
        L::MQ: function::MQAI,
    {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Bag<B: BagDrop, L: Library>
where
    L::MQ: function::MQAI,
{
    bag: mqai::BagHandle,
    pub(super) mq: core::MQFunctions<L>,
    _marker: PhantomData<B>,
}

impl<T: BagDrop, L: Library> std::ops::Deref for Bag<T, L>
where
    L::MQ: function::MQAI,
{
    type Target = mqai::BagHandle;

    fn deref(&self) -> &Self::Target {
        &self.bag
    }
}

impl<L: Library> Bag<Owned, L>
where
    L::MQ: function::MQAI,
{
    pub fn new_lib(lib: L, options: Mask<CreateBagOptions>) -> ResultErr<Self> {
        let mq = core::MQFunctions(lib);
        let bag = mq.mq_create_bag(options)?;
        mq.mq_set_integer(&bag, MqValue::new(sys::MQIASY_CODED_CHAR_SET_ID), sys::MQIND_NONE, 1208)?;

        Ok(Self {
            bag,
            mq,
            _marker: PhantomData,
        })
    }
}

impl<L: Library> BagItemGet<L> for Bag<Embedded, L>
where
    L::MQ: function::MQAI,
{
    fn inq_bag_item<B: BagDrop>(
        selector: MqValue<MqaiSelector>,
        index: sys::MQLONG,
        bag: &Bag<B, L>,
    ) -> ResultErr<Self> {
        let bag_handle = bag.mq.mq_inquire_bag(bag, selector, index)?;
        Ok(Self {
            bag: bag_handle,
            mq: bag.mq.clone(),
            _marker: PhantomData,
        })
    }

    type Error = Error;
}

impl<B: BagDrop, L: Library> Bag<B, L>
where
    L::MQ: function::MQAI,
{
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn handle(&self) -> &mqai::BagHandle {
        &self.bag
    }

    pub fn add_inquiry(&self, selector: MqValue<MqaiSelector>) -> ResultErr<()> {
        self.mq.mq_add_inquiry(self, selector)
    }

    pub fn add_bag<'a, 'bag: 'a>(
        &'a self,
        selector: MqValue<MqaiSelector>,
        to_attach: &'bag Bag<Owned, L>,
    ) -> ResultErr<()> {
        self.mq.mq_add_bag(self, selector, to_attach)
    }

    pub fn add<T: BagItemPut<L>>(&self, selector: MqValue<MqaiSelector>, value: T) -> Result<(), T::Error> {
        value.add_to_bag(selector, self)
    }

    pub fn inquire<T: BagItemGet<L>>(
        &self,
        selector: MqValue<MqaiSelector>,
        index: Option<sys::MQLONG>,
    ) -> Result<Option<T>, T::Error> {
        match T::inq_bag_item(selector, index.unwrap_or(sys::MQIND_NONE), self) {
            Err(e) => match e.mqi() {
                Some(Error(
                    CompletionCode(sys::MQCC_FAILED),
                    _verb, // Ignore the verb
                    ReasonCode(sys::MQRC_SELECTOR_NOT_PRESENT),
                )) => Ok(None),
                _ => Err(e),
            },
            other => other.map(Option::Some),
        }
    }

    pub fn inq<T: inq::InqSelector<L>>(
        &self,
        selector: &T,
        index: Option<sys::MQLONG>,
    ) -> Result<Option<T::Out>, <T::Out as BagItemGet<L>>::Error> {
        self.inquire(selector.attribute(), index)
    }

    pub fn set<T: BagItemPut<L>>(
        &self,
        selector: MqValue<MqaiSelector>,
        index: BagIndex,
        value: T,
    ) -> Result<(), T::Error> {
        T::set_bag_item(value, selector, index, self)
    }

    pub fn delete(&self, selector: MqValue<MqaiSelector>, index: BagIndex) -> ResultErr<()> {
        self.mq.mq_delete_item(self, selector, index.into())
    }

    pub fn execute(
        &self,
        handle: &ConnectionHandle,
        command: MqValue<Command>,
        options: Option<&mqai::BagHandle>,
        admin_q: Option<&core::ObjectHandle>,
        response_q: Option<&core::ObjectHandle>,
    ) -> ResultComp<Bag<Owned, L>> {
        let response_bag = Bag::new_lib(self.mq.0.clone(), Mask::from(sys::MQCBO_ADMIN_BAG))?;
        let completion = self.mq.mq_execute(
            handle,
            command,
            options,
            self.handle(),
            response_bag.handle(),
            admin_q,
            response_q,
        )?;
        Ok(completion.map(|()| response_bag))
    }
}

impl<B: BagDrop, L: Library> Drop for Bag<B, L>
where
    L::MQ: function::MQAI,
{
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
        let bag = Bag::new(Mask::from(sys::MQCBO_GROUP_BAG)).expect("Failed to create bag");
        let property = bag
            .inquire::<sys::MQLONG>(MqValue::new(0), Option::None)
            .expect("Failed to retrieve item");
        property.map_or_else(|| eprintln!("No CCSID!"), |ccsid| println!("CCSID is {ccsid}"));

        bag.add(MqValue::new(0), "abc").expect("Failed to add string");

        bag.delete(MqValue::new(0), BagIndex::None)
            .expect("Failed to delete item");
    }
}
