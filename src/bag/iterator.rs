use std::marker::PhantomData;

use libmqm_sys::Mqai;

use super::{Bag, BagDrop, BagItemGet, Embedded};
use crate::{
    Library, constants,
    prelude::*,
    result::{Error, ResultComp, ResultCompErr, WithMqError as _},
    types::{MQIND, MQLONG, Selector},
};

#[derive(Debug)]
pub struct BagItem<'bag, T, B, L>
where
    B: BagDrop,
    L: Library<MQ: Mqai>,
{
    selector: Selector,
    index: MQLONG,
    count: MQLONG,
    bag: &'bag Bag<B, L>,
    data: PhantomData<T>,
}

impl<T, B, L> Iterator for BagItem<'_, T, B, L>
where
    T: BagItemGet<L>,
    B: BagDrop,
    L: Library<MQ: Mqai>,
{
    type Item = ResultCompErr<T, T::Error>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = (self.count - self.index).try_into().ok();
        (size.unwrap_or(0), size)
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.index {
            return None;
        }
        let result = match T::inq_bag_item(self.selector, MQIND(self.index), self.bag) {
            Err(e) => match e.mqi_error() {
                Some(&Error(
                    constants::MQCC_FAILED,
                    _,
                    constants::MQRC_SELECTOR_NOT_PRESENT | constants::MQRC_INDEX_NOT_PRESENT,
                )) => None,
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
    L: Library<MQ: Mqai> + Clone,
{
    pub fn try_iter<T: BagItemGet<L>>(&self, selector: Selector) -> ResultComp<BagItem<'_, T, B, L>> {
        self.mq.mq_count_items(self, selector).map_completion(|count| BagItem {
            selector,
            count,
            index: 0,
            bag: self,
            data: PhantomData,
        })
    }

    pub fn try_bag_iter(&self, selector: Selector) -> ResultComp<BagItem<'_, Bag<Embedded, L>, B, L>> {
        self.try_iter(selector)
    }
}

#[cfg(all(test, any(feature = "link", feature = "dlopen2")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::{result::Completion, test::mq_library};

    #[test]
    fn test_empty_iterator() -> Result<(), Box<dyn std::error::Error>> {
        let lib = mq_library();

        let bag = Bag::new_lib(&lib, constants::MQCBO_NONE).warn_as_error()?;
        let mut i = bag.try_iter::<MQLONG>(Selector(0)).warn_as_error()?;

        assert_eq!(i.size_hint(), (0, Some(0)));
        assert!(i.next().is_none());
        Ok(())
    }

    #[test]
    fn test_one_iterator() -> Result<(), Box<dyn std::error::Error>> {
        let lib = mq_library();

        let bag = Bag::new_lib(&lib, constants::MQCBO_NONE).warn_as_error()?;
        bag.add(Selector(0), &99).warn_as_error()?;
        let mut i = bag.try_iter::<MQLONG>(Selector(0)).warn_as_error()?;

        assert_eq!(i.size_hint(), (1, Some(1)));
        assert!(matches!(i.next(), Some(Ok(Completion(99, _)))));
        assert!(i.next().is_none());

        Ok(())
    }
}
