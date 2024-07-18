use std::{
    collections::{vec_deque::Iter, VecDeque},
    ptr, str,
};

use crate::{common::ResultCompErrExt as _, core::values, sys, Conn, MqValue, Object, ResultComp};

pub use super::inq_types::*;

pub type InqReqType = (MqValue<values::MQXA>, InqReqItem);
pub type InqResType<'a, T> = (MqValue<values::MQXA>, InqResItem<'a, T>);

#[derive(Debug, Clone, Copy)]
pub enum InqReqItem {
    Str(usize),
    Long,
}

#[derive(Debug, Clone)]
pub enum InqResItem<'a, T: ?Sized> {
    Str(&'a T),
    Long(sys::MQLONG),
}

#[derive(Debug, Clone)]
pub struct InqRes {
    strings: Vec<sys::MQCHAR>,
    longs: VecDeque<sys::MQLONG>,
    select: VecDeque<InqReqType>,
}

struct InqResIter<'a> {
    text_pos: usize,
    strings: &'a [sys::MQCHAR],
    select: Iter<'a, InqReqType>,
    longs: Iter<'a, sys::MQLONG>,
}

impl InqRes {
    pub fn iter_mqchar(&self) -> impl Iterator<Item = InqResType<[sys::MQCHAR]>> {
        InqResIter {
            text_pos: 0,
            strings: &self.strings,
            select: self.select.iter(),
            longs: self.longs.iter(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = InqResType<str>> {
        self.iter_mqchar().map(|(attr, item)| {
            (
                attr,
                match item {
                    // SAFETY: MQ client CCSID is UTF-8. IBM MQ documentation states the MQINQ will
                    // use the client CCSID. Interpret as utf-8 unchecked, without allocation.
                    // Note: some fields, such as the initial key are binary and therefore should
                    // use the `iter_mqchar` function.
                    // Refer https://www.ibm.com/docs/en/ibm-mq/9.4?topic=application-using-mqinq-in-client-aplication
                    InqResItem::Str(value) => InqResItem::Str(
                        unsafe { str::from_utf8_unchecked(&*(ptr::from_ref(value) as *const [u8])) }
                            .trim_end_matches([' ', '\0']),
                    ),
                    InqResItem::Long(value) => InqResItem::Long(value),
                },
            )
        })
    }
}

impl<'a> Iterator for InqResIter<'a> {
    type Item = InqResType<'a, [sys::MQCHAR]>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.select.next() {
            Some(&(sel, InqReqItem::Str(len))) => {
                let mqchar = &self.strings[self.text_pos..len + self.text_pos];
                self.text_pos += len;
                Some((sel, InqResItem::Str(mqchar)))
            }
            Some(&(sel, InqReqItem::Long)) => self.longs.next().map(|&l| (sel, InqResItem::Long(l))),
            None => None,
        }
    }
}

impl<C: Conn> Object<C> {
    pub fn inq<'a>(&self, selectors: impl IntoIterator<Item = &'a InqReqType>) -> ResultComp<InqRes> {
        let mut text_len = 0;
        let mut int_len = 0;
        let select: VecDeque<_> = selectors.into_iter().copied().collect();
        let mut select_inq = Vec::<_>::with_capacity(select.len());
        for (n, val) in &select {
            match val {
                InqReqItem::Str(len) => text_len += len,
                InqReqItem::Long => int_len += 1,
            }
            select_inq.push(*n);
        }
        let mut text_attr = Vec::with_capacity(text_len);
        let mut int_attr = Vec::with_capacity(int_len);

        let connection = self.connection();

        connection
            .mq()
            .mqinq(
                connection.handle(),
                self.handle(),
                &select_inq,
                &mut int_attr.spare_capacity_mut()[..int_len],
                &mut text_attr.spare_capacity_mut()[..text_len],
            )
            .map_completion(|()| {
                unsafe {
                    text_attr.set_len(text_len);
                    int_attr.set_len(int_len);
                };
                InqRes {
                    strings: text_attr,
                    longs: VecDeque::from(int_attr),
                    select,
                }
            })
    }
}
