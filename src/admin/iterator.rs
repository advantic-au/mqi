use std::marker::PhantomData;

use libmqm_sys::function;

use crate::core::mqai::MqaiSelector;
use crate::core::Library;
use crate::{sys, MqValue, ResultComp, ResultCompErr, ResultCompErrExt};

use crate::Error;

use super::{Bag, BagDrop, BagItemGet, WithMQError};

pub struct BagItem<'bag, T, B: BagDrop, L: Library<MQ: function::MQAI>> {
    selector: MqValue<MqaiSelector>,
    index: sys::MQLONG,
    count: sys::MQLONG,
    bag: &'bag Bag<B, L>,
    data: PhantomData<T>,
}

impl<T: BagItemGet<L>, B: BagDrop, L: Library<MQ: function::MQAI>> Iterator for BagItem<'_, T, B, L> {
    type Item = ResultCompErr<T, T::Error>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = (self.count - self.index).try_into().ok();
        (size.unwrap_or(0), size)
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.index {
            return None;
        };
        let result = match T::inq_bag_item(self.selector, MqValue::from(self.index), self.bag) {
            Err(e) => match e.mqi() {
                Some(&Error(cc, _, rc))
                    if cc == sys::MQCC_FAILED && (rc == sys::MQRC_SELECTOR_NOT_PRESENT || rc == sys::MQRC_INDEX_NOT_PRESENT) =>
                {
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

impl<B: BagDrop, L: Library<MQ: function::MQAI>> Bag<B, L> {
    pub fn try_iter<T: BagItemGet<L>>(&self, selector: MqValue<MqaiSelector>) -> ResultComp<BagItem<'_, T, B, L>> {
        self.mq.mq_count_items(self, selector).map_completion(|count| BagItem {
            selector,
            count,
            index: 0,
            bag: self,
            data: PhantomData,
        })
    }
}
