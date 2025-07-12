use std::slice;

use crate::types;

#[derive(Debug, Clone, Copy)]
pub struct AttributeType {
    pub(super) attribute: types::MQXA,
    pub(super) text_len: u32,
}

impl AttributeType {
    pub const fn int_item(self, value: types::MQLONG) -> Result<IntItem, AttributeError> {
        IntItem::new(self.attribute, value)
    }

    pub const fn text_item(self, value: &[types::MQCHAR]) -> Result<TextItem<&[types::MQCHAR]>, AttributeError> {
        TextItem::new(self, value)
    }

    /// # Safety
    /// Consumers must ensure the `text_len` is correct for the given `attribute`
    #[must_use]
    pub const unsafe fn new(attribute: types::MQXA, text_len: u32) -> Self {
        Self { attribute, text_len }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InqResItem<T> {
    Text(TextItem<T>),
    Long(IntItem),
}

#[derive(Debug, Clone, Copy)]
pub struct IntItem {
    selector: types::MQXA,
    value: types::MQLONG,
}

#[derive(Debug, Clone, Copy)]
pub struct TextItem<T> {
    selector: types::MQXA,
    value: T,
}

#[derive(Debug, Clone, Copy)]
pub enum AttributeValue<T> {
    Text(T),
    Long(types::MQLONG),
}

impl<T> InqResItem<T> {
    #[must_use]
    pub fn into_tuple(self) -> (types::MQXA, AttributeValue<T>) {
        match self {
            Self::Text(TextItem { selector, value }) => (selector, AttributeValue::Text(value)),
            Self::Long(IntItem { selector, value }) => (selector, AttributeValue::Long(value)),
        }
    }
}

impl IntItem {
    pub const fn new(selector: types::MQXA, value: types::MQLONG) -> Result<Self, AttributeError> {
        if selector.is_int() {
            Ok(Self { selector, value })
        } else {
            Err(AttributeError::NotIntType(selector))
        }
    }

    /// # Safety
    /// Consumers must ensure the `selector` is within the MQIA constant range
    #[must_use]
    pub const unsafe fn new_unchecked(selector: types::MQXA, value: types::MQLONG) -> Self {
        Self { selector, value }
    }
}

impl<'a> TextItem<&'a [types::MQCHAR]> {
    pub const fn new(attr_type: AttributeType, value: &'a [types::MQCHAR]) -> Result<Self, AttributeError> {
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
    pub const unsafe fn new_unchecked(selector: types::MQXA, value: &'a [types::MQCHAR]) -> Self {
        Self { selector, value }
    }
}

#[derive(derive_more::Display, derive_more::Error, Debug)]
pub enum AttributeError {
    #[error(ignore)]
    #[display("{_0} is not an integer attribute")]
    NotIntType(types::MQXA),
    #[error(ignore)]
    #[display("{_0} is not a text attribute")]
    NotTextType(types::MQXA),
    #[display("actual text attribute length = {_0}, expected length = {_1}")]
    InvalidTextLength(usize, usize),
}

struct MultiItemIter<'a> {
    text_pos: usize,
    text_attr: &'a [types::MQCHAR],
    text_len: slice::Iter<'a, u32>,
    selectors: slice::Iter<'a, types::MQXA>,
    int_attr: slice::Iter<'a, types::MQLONG>,
}

#[derive(Debug, Clone, Default)]
pub struct MultiItem {
    pub(crate) selectors: Vec<types::MQXA>,
    pub(crate) int_attr: Vec<types::MQLONG>,
    pub(crate) text_attr: Vec<types::MQCHAR>,
    pub(crate) text_len: Vec<u32>,
}

impl MultiItem {
    pub fn iter(&self) -> impl Iterator<Item = InqResItem<&[types::MQCHAR]>> {
        MultiItemIter {
            text_pos: 0,
            text_attr: &self.text_attr,
            text_len: self.text_len.iter(),
            selectors: self.selectors.iter(),
            int_attr: self.int_attr.iter(),
        }
    }

    #[must_use]
    pub fn into_first(self) -> Option<InqResItem<Vec<types::MQCHAR>>> {
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
    type Item = InqResItem<&'a [types::MQCHAR]>;

    fn next(&mut self) -> Option<Self::Item> {
        let selector = *self.selectors.next()?;
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

pub trait SetItems: sealed::Sealed {
    fn selectors(&self) -> &[types::MQXA];
    fn int_attr(&self) -> &[types::MQLONG];
    fn text_attr(&self) -> &[types::MQCHAR];
}

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for MultiItem {}
impl SetItems for MultiItem {
    fn selectors(&self) -> &[types::MQXA] {
        &self.selectors
    }

    fn int_attr(&self) -> &[types::MQLONG] {
        &self.int_attr
    }

    fn text_attr(&self) -> &[types::MQCHAR] {
        &self.text_attr
    }
}

impl MultiItem {
    pub fn push_text_item(&mut self, text_item: &TextItem<&[types::MQCHAR]>) {
        self.selectors.push(text_item.selector);
        #[expect(clippy::cast_possible_truncation)]
        self.text_len.push(text_item.value.len() as u32);
        self.text_attr.extend_from_slice(text_item.value);
    }

    pub fn push_int_item(&mut self, int_item: IntItem) {
        self.selectors.push(int_item.selector);
        self.int_attr.push(int_item.value);
    }
}

impl sealed::Sealed for IntItem {}
impl SetItems for IntItem {
    fn selectors(&self) -> &[types::MQXA] {
        slice::from_ref(&self.selector)
    }

    fn int_attr(&self) -> &[types::MQLONG] {
        slice::from_ref(&self.value)
    }

    fn text_attr(&self) -> &[types::MQCHAR] {
        &[]
    }
}

impl<T> sealed::Sealed for InqResItem<T> {}
impl<T: AsRef<[types::MQCHAR]>> SetItems for InqResItem<T> {
    fn selectors(&self) -> &[types::MQXA] {
        match self {
            Self::Text(t) => t.selectors(),
            Self::Long(l) => l.selectors(),
        }
    }

    fn int_attr(&self) -> &[types::MQLONG] {
        match self {
            Self::Text(t) => t.int_attr(),
            Self::Long(l) => l.int_attr(),
        }
    }

    fn text_attr(&self) -> &[types::MQCHAR] {
        match self {
            Self::Text(t) => t.text_attr(),
            Self::Long(l) => l.text_attr(),
        }
    }
}

impl<T> sealed::Sealed for TextItem<T> {}
impl<T: AsRef<[types::MQCHAR]>> SetItems for TextItem<T> {
    fn selectors(&self) -> &[types::MQXA] {
        slice::from_ref(&self.selector)
    }

    fn int_attr(&self) -> &[types::MQLONG] {
        &[]
    }

    fn text_attr(&self) -> &[types::MQCHAR] {
        self.value.as_ref()
    }
}
