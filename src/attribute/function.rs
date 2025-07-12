use std::{collections::VecDeque, iter};

use super::option;
use crate::{Object, ResultComp, connection::Conn, prelude::*, types};

impl<C: Conn> Object<C> {
    /// This function uses the [`MQINQ`](libmqm_sys::MQINQ) MQ API function.
    pub fn inq<'a>(&self, selectors: impl IntoIterator<Item = &'a option::AttributeType>) -> ResultComp<option::MultiItem> {
        let mut text_total = 0;
        let mut int_count = 0;
        let mut text_len = Vec::new();

        let select: VecDeque<_> = selectors.into_iter().collect();
        let mut selectors = Vec::with_capacity(select.len());
        for &option::AttributeType {
            attribute,
            text_len: len,
        } in select
        {
            if attribute.is_text() {
                text_total += len;
                text_len.push(len);
            } else if attribute.is_int() {
                int_count += 1;
            }
            selectors.push(attribute);
        }
        let mut output = option::MultiItem {
            selectors,
            int_attr: Vec::with_capacity(int_count),
            text_attr: Vec::with_capacity(text_total as usize),
            text_len,
        };

        let connection = self.connection();
        connection
            .mq()
            .mqinq(
                connection.handle(),
                self.handle(),
                &output.selectors,
                &mut output.int_attr.spare_capacity_mut()[..int_count],
                &mut output.text_attr.spare_capacity_mut()[..text_total as usize],
            )
            .map_completion(|()| {
                unsafe {
                    output.text_attr.set_len(text_total as usize);
                    output.int_attr.set_len(int_count);
                };
                output
            })
    }

    /// This function uses the [`MQINQ`](libmqm_sys::MQINQ) MQ API function.
    pub fn inq_item(&self, selector: option::AttributeType) -> ResultComp<Option<option::InqResItem<Vec<types::MQCHAR>>>> {
        self.inq(iter::once(&selector)).map_completion(option::MultiItem::into_first)
    }

    /// This function uses the [`MQSET`](libmqm_sys::MQSET) MQ API function.
    pub fn set(&self, items: &impl option::SetItems) -> ResultComp<()> {
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
