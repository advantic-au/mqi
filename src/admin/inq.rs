use std::marker::PhantomData;

use libmqm_sys::function;

use super::BagItemGet;

use crate::constants::{self, mapping};
use crate::core::Library;
use crate::mq::StringCcsid;
use crate::sys;
use crate::{impl_constant_lookup, MqStr};

#[allow(clippy::module_name_repetitions)]
pub trait InqSelector<L: Library>: constants::MQConstant {
    type Out: BagItemGet<L>
    where
        L::MQ: function::MQAI;
}

pub struct Selector<T>(sys::MQLONG, PhantomData<T>);

impl<T> Selector<T> {
    #[must_use] pub const fn new(attribute: sys::MQLONG) -> Self {
        Self(attribute, PhantomData)
    }
}

impl<L: Library, T: BagItemGet<L>> InqSelector<L> for Selector<T>
where
    Self: constants::HasConstLookup,
    L::MQ: function::MQAI,
{
    type Out = T;
}

impl<T> constants::MQConstant for Selector<T>
where
    Self: constants::HasConstLookup,
{
    fn mq_value(&self) -> sys::MQLONG {
        self.0
    }
}

impl<const N: usize> constants::HasConstLookup for Selector<MqStr<N>> {
    fn const_lookup<'a>() -> &'a (impl constants::ConstLookup + 'static) {
        &mapping::MQCA_CONST
    }
}

impl_constant_lookup!(Selector<StringCcsid>, mapping::MQCA_CONST);
impl_constant_lookup!(Selector<sys::MQLONG>, mapping::MQIA_CONST);
