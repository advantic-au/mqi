use core::slice;
use crate::{core::values, sys, Conn, MqValue, ResultComp};

use super::Object;

pub trait SetItems {
    fn selectors(&self) -> &[MqValue<values::MQXA>];
    fn int_attr(&self) -> &[sys::MQLONG];
    fn text_attr(&self) -> &[sys::MQCHAR];
}

pub struct IntItem {
    item: MqValue<values::MQXA>,
    value: sys::MQLONG,
}

pub struct TextItem<'a> {
    item: MqValue<values::MQXA>,
    value: &'a [sys::MQCHAR]
}

impl IntItem {
    #[must_use]
    pub const fn new(item: MqValue<values::MQXA>, value: sys::MQLONG) -> Self {
        Self { item, value }
    }
}

impl SetItems for IntItem {
    fn selectors(&self) -> &[MqValue<values::MQXA>] {
        slice::from_ref(&self.item)
    }

    fn int_attr(&self) -> &[sys::MQLONG] {
        slice::from_ref(&self.value)
    }

    fn text_attr(&self) -> &[sys::MQCHAR] {
        &[]
    }
}

impl<C: Conn> Object<C> {
    pub fn set<'a, T: 'a>(&self, items: &impl SetItems) -> ResultComp<()> {
        let connection = self.connection();
        connection.mq().mqset(
            connection.handle(),
            self.handle(),
            items.selectors(),
            items.int_attr(),
            items.text_attr(),
        )
    }
}
