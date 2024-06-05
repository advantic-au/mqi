use std::marker::PhantomData;

use libmqm_sys::function;

use super::BagItemGet;

use crate::core::mqai::MqaiSelector;
use crate::core::Library;
use crate::MqValue;

pub trait InqSelector<L: Library> {
    type Out: BagItemGet<L>
    where
        L::MQ: function::MQAI;
    fn attribute(&self) -> MqValue<MqaiSelector>;
}

pub struct Selector<T>(MqValue<MqaiSelector>, PhantomData<T>);

impl<T> Selector<T> {
    #[must_use]
    pub const fn new(attribute: MqValue<MqaiSelector>) -> Self {
        Self(attribute, PhantomData)
    }
}

impl<L: Library, T: BagItemGet<L>> InqSelector<L> for Selector<T>
where
    L::MQ: function::MQAI,
{
    type Out = T;

    fn attribute(&self) -> MqValue<MqaiSelector> {
        self.0
    }
}
