use std::{collections::VecDeque, iter, slice};

use crate::{common::ResultCompErrExt as _, core::values, sys, Conn, MqValue, Object, ResultComp};

pub use super::attribute_types::*;

#[derive(Debug, Clone, Copy)]
pub struct AttributeType {
    pub(super) attribute: MqValue<values::MQXA>,
    pub(super) text_len: u32,
}

impl AttributeType {
    pub const fn int_item(self, value: sys::MQLONG) -> Result<IntItem, AttributeError> {
        IntItem::new(self.attribute, value)
    }

    pub const fn text_item(self, value: &[sys::MQCHAR]) -> Result<TextItem<&[sys::MQCHAR]>, AttributeError> {
        TextItem::new(self, value)
    }

    /// # Safety
    /// Consumers must ensure the `text_len` is correct for the given `attribute`
    #[must_use]
    pub const unsafe fn new(attribute: MqValue<values::MQXA>, text_len: u32) -> Self {
        Self { attribute, text_len }
    }
}

impl MqValue<values::MQXA> {
    #[inline]
    #[must_use]
    pub const fn is_text(&self) -> bool {
        let val = self.value();
        val >= sys::MQCA_FIRST && val <= sys::MQCA_LAST
    }

    #[inline]
    #[must_use]
    pub const fn is_int(&self) -> bool {
        let val = self.value();
        val >= sys::MQIA_FIRST && val <= sys::MQIA_LAST
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InqResItem<T> {
    Text(TextItem<T>),
    Long(IntItem),
}

#[derive(Debug, Clone, Copy)]
pub enum AttributeValue<T> {
    Text(T),
    Long(sys::MQLONG),
}

impl<T> InqResItem<T> {
    #[must_use]
    pub fn into_tuple(self) -> (MqValue<values::MQXA>, AttributeValue<T>) {
        match self {
            Self::Text(TextItem { selector, value }) => (selector, AttributeValue::Text(value)),
            Self::Long(IntItem { selector, value }) => (selector, AttributeValue::Long(value)),
        }
    }
}

struct MultiItemIter<'a> {
    text_pos: usize,
    text_attr: &'a [sys::MQCHAR],
    text_len: slice::Iter<'a, u32>,
    selectors: slice::Iter<'a, MqValue<values::MQXA>>,
    int_attr: slice::Iter<'a, sys::MQLONG>,
}

impl MultiItem {
    pub fn iter(&self) -> impl Iterator<Item = InqResItem<&[sys::MQCHAR]>> {
        MultiItemIter {
            text_pos: 0,
            text_attr: &self.text_attr,
            text_len: self.text_len.iter(),
            selectors: self.selectors.iter(),
            int_attr: self.int_attr.iter(),
        }
    }

    #[must_use]
    pub fn into_first(self) -> Option<InqResItem<Vec<sys::MQCHAR>>> {
        let selector = *self.selectors.first()?;

        if selector.is_int() {
            Some(InqResItem::Long(unsafe {
                IntItem::new_unchecked(selector, *self.int_attr.first()?)
            }))
        } else if selector.is_text() {
            let len = *self.text_len.first()?;
            let mut self_mut = self;
            self_mut.text_attr.truncate(len as usize);
            Some(InqResItem::Text(TextItem {
                selector,
                value: self_mut.text_attr,
            }))
        } else {
            None
        }
    }
}

impl<'a> Iterator for MultiItemIter<'a> {
    type Item = InqResItem<&'a [sys::MQCHAR]>;

    fn next(&mut self) -> Option<Self::Item> {
        let selector = *self.selectors.next()?;
        // let val = attribute.value();
        if selector.is_int() {
            Some(InqResItem::Long(unsafe {
                IntItem::new_unchecked(selector, *self.int_attr.next()?)
            }))
        } else if selector.is_text() {
            let len = *self.text_len.next()?;
            let mqchar = &self.text_attr[self.text_pos..(len as usize) + self.text_pos];
            self.text_pos += len as usize;
            Some(InqResItem::Text(unsafe { TextItem::new_unchecked(selector, mqchar) }))
        } else {
            None
        }
    }
}

impl<C: Conn> Object<C> {
    pub fn inq<'a>(&self, selectors: impl IntoIterator<Item = &'a AttributeType>) -> ResultComp<MultiItem> {
        let mut text_total = 0;
        let mut int_count = 0;
        let mut text_len = Vec::new();

        let select: VecDeque<_> = selectors.into_iter().collect();
        let mut selectors = Vec::<_>::with_capacity(select.len());
        for &AttributeType {
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
        let mut output = MultiItem {
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

    pub fn inq_item(&self, selector: AttributeType) -> ResultComp<Option<InqResItem<Vec<sys::MQCHAR>>>> {
        self.inq(iter::once(&selector)).map_completion(MultiItem::into_first)
    }
}

pub trait SetItems: sealed::Sealed {
    fn selectors(&self) -> &[MqValue<values::MQXA>];
    fn int_attr(&self) -> &[sys::MQLONG];
    fn text_attr(&self) -> &[sys::MQCHAR];
}

mod sealed {
    pub trait Sealed {}
}

#[derive(Debug, Clone, Copy)]
pub struct IntItem {
    selector: MqValue<values::MQXA>,
    value: sys::MQLONG,
}

#[derive(Debug, Clone, Copy)]
pub struct TextItem<T> {
    selector: MqValue<values::MQXA>,
    value: T,
}

#[derive(Debug, Clone, Default)]
pub struct MultiItem {
    selectors: Vec<MqValue<values::MQXA>>,
    int_attr: Vec<sys::MQLONG>,
    text_attr: Vec<sys::MQCHAR>,
    text_len: Vec<u32>,
}

impl sealed::Sealed for MultiItem {}
impl SetItems for MultiItem {
    fn selectors(&self) -> &[MqValue<values::MQXA>] {
        &self.selectors
    }

    fn int_attr(&self) -> &[sys::MQLONG] {
        &self.int_attr
    }

    fn text_attr(&self) -> &[sys::MQCHAR] {
        &self.text_attr
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AttributeError {
    #[error("{} is not an integer attribute", .0)]
    NotIntType(MqValue<values::MQXA>),
    #[error("{} is not a text attribute", .0)]
    NotTextType(MqValue<values::MQXA>),
    #[error("actual text attribute length = {}, expected length = {}", .0, .1)]
    InvalidTextLength(usize, usize),
}

impl MultiItem {
    pub fn push_text_item(&mut self, text_item: &TextItem<&[sys::MQCHAR]>) {
        self.selectors.push(text_item.selector);
        #[allow(clippy::cast_possible_truncation)]
        self.text_len.push(text_item.value.len() as u32);
        self.text_attr.extend_from_slice(text_item.value);
    }

    pub fn push_int_item(&mut self, int_item: &IntItem) {
        self.selectors.push(int_item.selector);
        self.int_attr.push(int_item.value);
    }
}

impl IntItem {
    pub const fn new(selector: MqValue<values::MQXA>, value: sys::MQLONG) -> Result<Self, AttributeError> {
        if selector.is_int() {
            Ok(Self { selector, value })
        } else {
            Err(AttributeError::NotIntType(selector))
        }
    }

    /// # Safety
    /// Consumers must ensure the `selector` is within the MQIA constant range
    #[must_use]
    pub const unsafe fn new_unchecked(selector: MqValue<values::MQXA>, value: sys::MQLONG) -> Self {
        Self { selector, value }
    }
}

impl<'a> TextItem<&'a [sys::MQCHAR]> {
    pub const fn new(attr_type: AttributeType, value: &'a [sys::MQCHAR]) -> Result<Self, AttributeError> {
        if !attr_type.attribute.is_text() {
            Err(AttributeError::NotTextType(attr_type.attribute))
        } else if value.len() != attr_type.text_len as usize {
            Err(AttributeError::InvalidTextLength(attr_type.text_len as usize, value.len()))
        } else {
            Ok(Self {
                selector: attr_type.attribute,
                value,
            })
        }
    }

    /// # Safety
    /// Consumers must ensure the `selector` is within the MQCA constant range and the slice is the correct length
    #[must_use]
    pub const unsafe fn new_unchecked(selector: MqValue<values::MQXA>, value: &'a [sys::MQCHAR]) -> Self {
        Self { selector, value }
    }
}

impl sealed::Sealed for IntItem {}
impl SetItems for IntItem {
    fn selectors(&self) -> &[MqValue<values::MQXA>] {
        slice::from_ref(&self.selector)
    }

    fn int_attr(&self) -> &[sys::MQLONG] {
        slice::from_ref(&self.value)
    }

    fn text_attr(&self) -> &[sys::MQCHAR] {
        &[]
    }
}

impl<T> sealed::Sealed for InqResItem<T> {}
impl<T: AsRef<[sys::MQCHAR]>> SetItems for InqResItem<T> {
    fn selectors(&self) -> &[MqValue<values::MQXA>] {
        match self {
            Self::Text(t) => t.selectors(),
            Self::Long(l) => l.selectors(),
        }
    }

    fn int_attr(&self) -> &[sys::MQLONG] {
        match self {
            Self::Text(t) => t.int_attr(),
            Self::Long(l) => l.int_attr(),
        }
    }

    fn text_attr(&self) -> &[sys::MQCHAR] {
        match self {
            Self::Text(t) => t.text_attr(),
            Self::Long(l) => l.text_attr(),
        }
    }
}

impl<T> sealed::Sealed for TextItem<T> {}
impl<T: AsRef<[sys::MQCHAR]>> SetItems for TextItem<T> {
    fn selectors(&self) -> &[MqValue<values::MQXA>] {
        slice::from_ref(&self.selector)
    }

    fn int_attr(&self) -> &[sys::MQLONG] {
        &[]
    }

    fn text_attr(&self) -> &[sys::MQCHAR] {
        self.value.as_ref()
    }
}

impl<C: Conn> Object<C> {
    pub fn set(&self, items: &impl SetItems) -> ResultComp<()> {
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
