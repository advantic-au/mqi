use std::{
    cmp::min,
    collections::{vec_deque::Iter, VecDeque},
    fmt::Debug,
    mem::size_of_val,
    ops::{Deref, DerefMut},
    ptr,
    str::from_utf8_unchecked,
    sync::Arc,
};

use libmqm_sys::function;

use crate::{
    core::{self, ConnectionHandle, Library, MQFunctions, MQCO, MQOO},
    Conn, MqMask, MqValue, ResultCompErrExt as _,
};
use crate::{impl_constant_lookup, mapping, sys, MqStr, QMName, QName, StructBuilder};
use crate::{ObjectName, ResultComp};

use super::QueueManagerShare;

trait Sealed {}
#[allow(private_bounds)]
pub trait MQMD: Sealed + Debug {}
impl Sealed for sys::MQMD {}
impl Sealed for sys::MQMD2 {}
impl Sealed for sys::MQMDE {}

impl MQMD for sys::MQMD {}
impl MQMD for sys::MQMD2 {}
impl MQMD for sys::MQMDE {}

#[derive(Clone, Copy)]
pub struct MQXA;
impl_constant_lookup!(MQXA, mapping::MQXA_FULL_CONST);

pub type InqReqType = (MqValue<MQXA>, InqReqItem);
pub type InqResType<'a, T> = (MqValue<MQXA>, InqResItem<'a, T>);

#[must_use]
pub struct Object<C: Conn> {
    handle: core::ObjectHandle,
    connection: C,
    close_options: MqMask<MQCO>,
    name: QName,               // When a model queue is used
    qmgr_name: Option<QMName>, // When a model queue is used
    resolved_name: Option<QName>,
    resolved_qmgr_name: Option<QMName>,
}

impl<L: Library<MQ: function::MQI>, H> Conn for Arc<QueueManagerShare<L, H>> {
    fn mq(&self) -> &MQFunctions<impl Library<MQ: function::MQI>> {
        self.deref().mq()
    }

    fn handle(&self) -> &ConnectionHandle {
        self.deref().handle()
    }
}

impl<L: Library<MQ: function::MQI>, H> Conn for QueueManagerShare<L, H> {
    fn mq(&self) -> &MQFunctions<impl Library<MQ: function::MQI>> {
        Self::mq(self)
    }

    fn handle(&self) -> &ConnectionHandle {
        Self::handle(self)
    }
}

impl<L: Library<MQ: function::MQI>, H> Conn for &QueueManagerShare<L, H> {
    fn mq(&self) -> &MQFunctions<impl Library<MQ: function::MQI>> {
        QueueManagerShare::<L, H>::mq(self)
    }

    fn handle(&self) -> &ConnectionHandle {
        QueueManagerShare::<L, H>::handle(self)
    }
}

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
                    // Refer https://www.ibm.com/docs/en/ibm-mq/9.4?topic=application-using-mqinq-in-client-aplication
                    InqResItem::Str(value) => InqResItem::Str(
                        unsafe { from_utf8_unchecked(&*(ptr::from_ref::<[sys::MQCHAR]>(value) as *const [u8])) }
                            .trim_end_matches([' ', '\0']),
                    ),
                    InqResItem::Long(value) => InqResItem::Long(value),
                },
            )
        })
    }
}

struct InqResIter<'a> {
    text_pos: usize,
    strings: &'a [sys::MQCHAR],
    select: Iter<'a, InqReqType>,
    longs: Iter<'a, sys::MQLONG>,
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

pub type CorrelationId = [u8; sys::MQ_CORREL_ID_LENGTH];
pub type MessageId = [u8; sys::MQ_MSG_ID_LENGTH];
pub type GroupId = [u8; sys::MQ_GROUP_ID_LENGTH];
pub type MsgToken = [u8; sys::MQ_MSG_TOKEN_LENGTH];

#[derive(Default)]
pub struct MatchOptions {
    pub msg_id: Option<MessageId>,
    pub correl_id: Option<CorrelationId>,
    pub group_id: Option<GroupId>,
    pub seq_number: Option<sys::MQLONG>,
    pub offset: Option<sys::MQLONG>,
    pub token: Option<MsgToken>,
}

pub struct GetMessage {
    gmo: sys::MQGMO,
    pub returned_length: sys::MQLONG,
}

impl GetMessage {
    #[must_use]
    pub const fn returned_length(&self) -> sys::MQLONG {
        self.returned_length
    }

    #[must_use]
    pub const fn message_token(&self) -> &MsgToken {
        &self.gmo.MsgToken
    }

    #[must_use]
    pub fn resolved_queue(&self) -> &ObjectName {
        self.gmo.ResolvedQName.as_ref()
    }
}

impl<C: Conn> Object<C> {
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn handle(&self) -> &core::ObjectHandle {
        &self.handle
    }

    pub fn open(connection: C, mqod: &impl StructBuilder<sys::MQOD>, options: MqMask<MQOO>) -> ResultComp<Self> {
        let mut mqod_build = mqod.build();
        let result = connection.mq().mqopen(connection.handle(), &mut mqod_build, options);
        result.map_completion(|handle| Self {
            handle,
            connection,
            close_options: MqMask::from(sys::MQCO_NONE),
            name: mqod_build.ObjectName.into(),
            qmgr_name: Some(mqod_build.ObjectQMgrName.into()).filter(MqStr::has_value),
            resolved_name: Some(mqod_build.ResolvedQName.into()).filter(MqStr::has_value),
            resolved_qmgr_name: Some(mqod_build.ResolvedQMgrName.into()).filter(MqStr::has_value),
        })
    }

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

        self.connection
            .mq()
            .mqinq(
                self.connection.handle(),
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

    // TODO: deal with optional mqmd
    pub fn put<B>(&self, mqmd: &mut impl MQMD, pmo: &mut sys::MQPMO, body: &B) -> ResultComp<()> {
        self.connection
            .mq()
            .mqput(self.connection.handle(), self.handle(), Some(mqmd), pmo, body)
    }

    pub fn get_message<B>(
        &self,
        handle: &core::MessageHandle,
        mo: &MatchOptions,
        wait: Option<sys::MQLONG>,
        body: &mut B,
    ) -> ResultComp<GetMessage> {
        let mut md = sys::MQMD2 {
            MsgId: mo.msg_id.unwrap_or_default(),
            CorrelId: mo.correl_id.unwrap_or_default(),
            GroupId: mo.group_id.unwrap_or_default(),
            MsgSeqNumber: mo.seq_number.unwrap_or_default(),
            ..sys::MQMD2::default()
        };
        let mut gmo = sys::MQGMO {
            Version: sys::MQGMO_VERSION_4,
            MsgHandle: unsafe { handle.raw_handle() },
            MsgToken: mo.token.unwrap_or_default(),
            ..sys::MQGMO::default()
        };
        gmo.MatchOptions |= mo.group_id.map_or(0, |_| sys::MQMO_MATCH_GROUP_ID)
            | mo.seq_number.map_or(0, |_| sys::MQMO_MATCH_MSG_SEQ_NUMBER)
            | mo.offset.map_or(0, |_| sys::MQMO_MATCH_OFFSET)
            | mo.token.map_or(0, |_| sys::MQMO_MATCH_MSG_TOKEN);
        // Set up the wait
        if let Some(interval) = wait {
            gmo.Options |= sys::MQGMO_WAIT;
            gmo.WaitInterval = interval;
        }

        self.connection
            .mq()
            .mqget(self.connection.handle(), self.handle(), Some(&mut md), &mut gmo, body)
            .map_completion(|len| GetMessage {
                returned_length: match gmo.ReturnedLength {
                    sys::MQRL_UNDEFINED => min(
                        len,
                        size_of_val(body)
                            .try_into()
                            .expect("body length exceeds maximum positive MQLONG"),
                    ),
                    other => other,
                },
                gmo,
            })
    }

    pub fn close_options(&mut self, options: MqMask<MQCO>) {
        self.close_options = options;
    }

    pub fn close(self) -> ResultComp<()> {
        let mut s = self;
        s.connection
            .mq()
            .mqclose(s.connection.handle(), &mut s.handle, s.close_options)
    }
}

impl<C: Conn> Deref for Object<C> {
    type Target = core::ObjectHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<C: Conn> DerefMut for Object<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

impl<C: Conn> Drop for Object<C> {
    fn drop(&mut self) {
        // TODO: handle close failure
        if self.is_closeable() {
            let _ = self
                .connection
                .mq()
                .mqclose(self.connection.handle(), &mut self.handle, self.close_options);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::MQCO;
    use crate::sys;
    use crate::MqMask;

    #[test]
    fn close_option() {
        assert_eq!(
            MqMask::<MQCO>::from(sys::MQCO_DELETE | 0xFF00).to_string(),
            "MQCO_DELETE|0xFF00"
        );
        assert_eq!(
            MqMask::<MQCO>::from(sys::MQCO_DELETE | sys::MQCO_QUIESCE).to_string(),
            "MQCO_DELETE|MQCO_QUIESCE"
        );
        assert_eq!(MqMask::<MQCO>::from(sys::MQCO_DELETE).to_string(), "MQCO_DELETE");
        assert_eq!(MqMask::<MQCO>::from(0).to_string(), "MQCO_NONE");
        assert_eq!(MqMask::<MQCO>::from(0xFF00).to_string(), "0xFF00");

        let (list_iter, _) = MqMask::<MQCO>::from(sys::MQCO_DELETE).masked_list();
        let list = list_iter.collect::<Vec<_>>();
        assert_eq!(list, &[(1, "MQCO_DELETE")]);

        let (list_iter, _) = MqMask::<MQCO>::from(sys::MQCO_NONE).masked_list();
        let list = list_iter.collect::<Vec<_>>();
        assert_eq!(list, &[(0, "MQCO_NONE")]);

        let (list_iter, _) = MqMask::<MQCO>::from(sys::MQCO_DELETE | sys::MQCO_QUIESCE).masked_list();
        let list = list_iter.collect::<Vec<_>>();
        assert_eq!(list, &[(1, "MQCO_DELETE"), (32, "MQCO_QUIESCE")]);

        // assert_eq!(format!("{oo:?}"), "");
    }
}
