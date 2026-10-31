use std::iter;

use super::option;
use crate::{Connection, Object, connection::AsConnection, prelude::*, result::ResultComp, types};

impl<C: AsConnection> Object<C> {
    /// Inquire about multiple integer and text attributes on this object.
    ///
    /// Pass selectors from [`attribute`](crate::attribute), then inspect the
    /// returned values in the same order with [`option::MultiItem::iter`]. Use
    /// [`inquire_integer`](Self::inquire_integer) or [`inquire_text`](Self::inquire_text)
    /// when reading a single attribute of a known type.
    ///
    /// This function uses [`MQINQ`](libmqm_sys::MQINQ).
    ///
    /// # Errors
    /// Returns an error if MQ cannot retrieve the requested attributes.
    pub fn inquire<'a>(&self, selectors: impl IntoIterator<Item = &'a option::AttributeType>) -> ResultComp<option::MultiItem> {
        let mut text_total = 0;
        let mut int_count = 0;
        let mut text_len = Vec::new();

        let selectors_iter = selectors.into_iter();
        let mut selectors = Vec::with_capacity(selectors_iter.size_hint().0);
        for &option::AttributeType {
            attribute,
            text_len: len,
        } in selectors_iter
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
            int_attr: vec![0; int_count],
            text_attr: vec![0; text_total as usize],
            text_len,
        };

        let Connection { mq, handle, .. } = self.connection.as_connection();
        mq.mqinq(
            *handle,
            self.handle(),
            &output.selectors,
            &mut output.int_attr,
            &mut output.text_attr,
        )
        .map_completion(|()| output)
    }

    /// Inquire about one attribute and return its value with its type.
    ///
    /// The returned item is `None` if the selector is not a supported integer
    /// or text attribute. Use [`option::InqResItem::into_tuple`] to obtain the selector
    /// and an [`AttributeValue`](option::AttributeValue) together.
    ///
    /// This function uses [`MQINQ`](libmqm_sys::MQINQ).
    ///
    /// # Errors
    /// Returns an error if MQ cannot retrieve the selected attribute.
    pub fn inquire_item(&self, selector: option::AttributeType) -> ResultComp<Option<option::InqResItem<Vec<types::MQCHAR>>>> {
        self.inquire(iter::once(&selector))
            .map_completion(option::MultiItem::into_first)
    }

    /// Inquire about one integer attribute.
    ///
    /// Use an integer selector such as `attribute::MQIA_CURRENT_Q_DEPTH`.
    /// Returns `None` if MQ does not return the requested value.
    ///
    /// This function uses [`MQINQ`](libmqm_sys::MQINQ).
    ///
    /// # Panics
    /// Panics if `selector` is not an integer attribute, or if MQ returns a value with a non-integer type.
    ///
    /// # Errors
    /// Returns an error if MQ cannot retrieve the selected attribute.
    pub fn inquire_integer(&self, selector: option::AttributeType) -> ResultComp<Option<types::MQLONG>> {
        assert!(selector.attribute.is_int());
        self.inquire_item(selector)
            .map_completion(|item| item.map(|item| item.long_value().expect("MQLONG value returned from MQINQ")))
    }

    /// Inquire about one fixed-width text attribute.
    ///
    /// The returned `MQCHAR` vector contains the full fixed-width attribute
    /// value; convert or trim it as appropriate for the selected MQ attribute.
    /// Returns `None` if MQ does not return the requested value.
    ///
    /// This function uses [`MQINQ`](libmqm_sys::MQINQ).
    ///
    /// # Panics
    /// Panics if `selector` is not a text attribute, or if MQ returns a value with a non-text type.
    ///
    /// # Errors
    /// Returns an error if MQ cannot retrieve the selected attribute.
    pub fn inquire_text(&self, selector: option::AttributeType) -> ResultComp<Option<Vec<types::MQCHAR>>> {
        assert!(selector.attribute.is_text());
        self.inquire_item(selector)
            .map_completion(|item| item.map(|item| item.text_value().expect("MQCHAR value returned from MQINQ")))
    }

    /// Set one or more integer or text attributes on this object.
    ///
    /// Build the values with the selector's `int_item` or `text_item` method,
    /// then add them to an [`option::MultiItem`]. Only attributes supported by
    /// MQSET for this object type can be changed.
    ///
    /// This function uses [`MQSET`](libmqm_sys::MQSET).
    ///
    /// # Errors
    /// Returns an error if MQ cannot change the requested attributes.
    pub fn set(&self, items: &impl option::SetItems) -> ResultComp<()> {
        let Connection { mq, handle, .. } = self.connection.as_connection();
        mq.mqset(*handle, self.handle(), items.selectors(), items.int_attr(), items.text_attr())
    }
}
