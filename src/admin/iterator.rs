use std::marker::PhantomData;

use libmqm_sys::function;

use crate::values::{MqaiSelector, MQIND, MQCC, MQRC};
use crate::core::Library;
use crate::prelude::*;
use crate::{sys, ResultComp, ResultCompErr, WithMqError as _};

use crate::Error;

use super::{Bag, BagDrop, BagItemGet, Embedded};

pub struct BagItem<'bag, T, B, L>
where
    B: BagDrop,
    L: Library<MQ: function::Mqai>,
{
    selector: MqaiSelector,
    index: sys::MQLONG,
    count: sys::MQLONG,
    bag: &'bag Bag<B, L>,
    data: PhantomData<T>,
}

impl<T, B, L> Iterator for BagItem<'_, T, B, L>
where
    T: BagItemGet<L>,
    B: BagDrop,
    L: Library<MQ: function::Mqai>,
{
    type Item = ResultCompErr<T, T::Error>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = (self.count - self.index).try_into().ok();
        (size.unwrap_or(0), size)
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.index {
            return None;
        };
        let result = match T::inq_bag_item(self.selector, MQIND(self.index), self.bag) {
            Err(e) => match e.mqi_error() {
                Some(&Error(MQCC(sys::MQCC_FAILED), _, MQRC(sys::MQRC_SELECTOR_NOT_PRESENT | sys::MQRC_INDEX_NOT_PRESENT))) => {
                    None
                }
                _ => Some(Err(e)),
            },
            other => Some(other),
        };
        self.index += 1;

        result
    }
}

impl<B, L> Bag<B, L>
where
    B: BagDrop,
    L: Library<MQ: function::Mqai> + Clone,
{
    pub fn try_iter<T: BagItemGet<L>>(&self, selector: MqaiSelector) -> ResultComp<BagItem<T, B, L>> {
        self.mq.mq_count_items(self, selector).map_completion(|count| BagItem {
            selector,
            count,
            index: 0,
            bag: self,
            data: PhantomData,
        })
    }

    pub fn try_bag_iter(&self, selector: MqaiSelector) -> ResultComp<BagItem<Bag<Embedded, L>, B, L>> {
        self.try_iter(selector)
    }
}
