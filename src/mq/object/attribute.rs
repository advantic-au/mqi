use std::{collections::VecDeque, ptr, slice, str};

use crate::{common::ResultCompErrExt as _, core::values, sys, Conn, MqStr, MqValue, Object, ResultComp};

pub use super::attribute_types::*;

#[derive(Debug, Clone, Copy)]
pub struct AttributeType {
    pub(super) attribute: MqValue<values::MQXA>,
    pub(super) text_len: u32,
}

impl AttributeType {
    pub const fn new_int(self, value: sys::MQLONG) -> Result<IntItem, AttributeError> {
        IntItem::new(self.attribute, value)
    }

    pub const fn new_text<const N: usize>(self, value: &MqStr<N>) -> Result<TextItem, AttributeError> {
        TextItem::new(self, value)
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

pub type InqResType<'a, T> = (MqValue<values::MQXA>, InqResItem<'a, T>);

#[derive(Debug, Clone)]
pub enum InqResItem<'a, T: ?Sized> {
    Text(&'a T),
    Long(sys::MQLONG),
}

struct MultiItemIter<'a> {
    text_pos: usize,
    text_attr: &'a [sys::MQCHAR],
    text_len: slice::Iter<'a, u32>,
    selectors: slice::Iter<'a, MqValue<values::MQXA>>,
    int_attr: slice::Iter<'a, sys::MQLONG>,
}

impl MultiItems {
    pub fn iter_mqchar(&self) -> impl Iterator<Item = InqResType<[sys::MQCHAR]>> {
        MultiItemIter {
            text_pos: 0,
            text_attr: &self.text_attr,
            text_len: self.text_len.iter(),
            selectors: self.selectors.iter(),
            int_attr: self.int_attr.iter(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = InqResType<str>> {
        self.iter_mqchar().map(|(attr, item)| {
            (
                attr,
                match item {
                    // SAFETY: MQ client CCSID is (1208) UTF-8. IBM MQ documentation states the MQINQ will
                    // use the client CCSID. Interpret as utf-8 unchecked, without allocation.
                    // Note: some fields, such as the initial key are binary and therefore should
                    // use the `iter_mqchar` function.
                    // Refer https://www.ibm.com/docs/en/ibm-mq/9.4?topic=application-using-mqinq-in-client-aplication
                    InqResItem::Text(value) => InqResItem::Text(
                        unsafe { str::from_utf8_unchecked(&*(ptr::from_ref(value) as *const [u8])) }
                            .trim_end_matches([' ', '\0']),
                    ),
                    InqResItem::Long(value) => InqResItem::Long(value),
                },
            )
        })
    }
}

impl<'a> Iterator for MultiItemIter<'a> {
    type Item = InqResType<'a, [sys::MQCHAR]>;

    fn next(&mut self) -> Option<Self::Item> {
        let attribute = *self.selectors.next()?;
        // let val = attribute.value();
        if attribute.is_int() {
            Some((attribute, InqResItem::Long(*self.int_attr.next()?)))
        } else if attribute.is_text() {
            let len = *self.text_len.next()?;
            let mqchar = &self.text_attr[self.text_pos..(len as usize) + self.text_pos];
            self.text_pos += len as usize;
            Some((attribute, InqResItem::Text(mqchar)))
        } else {
            None
        }
    }
}

impl<C: Conn> Object<C> {
    pub fn inq<'a>(&self, selectors: impl IntoIterator<Item = &'a AttributeType>) -> ResultComp<MultiItems> {
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
        let mut output = MultiItems {
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
}

pub trait SetItems: sealed::Sealed {
    fn selectors(&self) -> &[MqValue<values::MQXA>];
    fn int_attr(&self) -> &[sys::MQLONG];
    fn text_attr(&self) -> &[sys::MQCHAR];
}

mod sealed {
    pub trait Sealed {}
}

pub struct IntItem {
    selector: MqValue<values::MQXA>,
    value: sys::MQLONG,
}

pub struct TextItem<'a> {
    selector: MqValue<values::MQXA>,
    value: &'a [sys::MQCHAR],
}

#[derive(Debug, Clone, Default)]
pub struct MultiItems {
    selectors: Vec<MqValue<values::MQXA>>,
    int_attr: Vec<sys::MQLONG>,
    text_attr: Vec<sys::MQCHAR>,
    text_len: Vec<u32>,
}

impl sealed::Sealed for MultiItems {}
impl SetItems for MultiItems {
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

impl MultiItems {
    pub fn push_text_item(&mut self, text_item: &TextItem) {
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
    // #[must_use]
    pub const fn new(item: MqValue<values::MQXA>, value: sys::MQLONG) -> Result<Self, AttributeError> {
        if item.is_int() {
            Ok(Self { selector: item, value })
        } else {
            Err(AttributeError::NotIntType(item))
        }
    }
}

impl<'a> TextItem<'a> {
    pub const fn new<const N: usize>(attr_type: AttributeType, value: &'a MqStr<N>) -> Result<Self, AttributeError> {
        if !attr_type.attribute.is_text() {
            Err(AttributeError::NotTextType(attr_type.attribute))
        } else if N != attr_type.text_len as usize {
            Err(AttributeError::InvalidTextLength(attr_type.text_len as usize, N))
        } else {
            Ok(Self {
                selector: attr_type.attribute,
                value: value.as_mqchar(),
            })
        }
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

impl sealed::Sealed for TextItem<'_> {}
impl<'a> SetItems for TextItem<'a> {
    fn selectors(&self) -> &[MqValue<values::MQXA>] {
        slice::from_ref(&self.selector)
    }

    fn int_attr(&self) -> &[sys::MQLONG] {
        &[]
    }

    fn text_attr(&self) -> &[sys::MQCHAR] {
        self.value
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
