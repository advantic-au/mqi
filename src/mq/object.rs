use std::{
    cmp::min,
    collections::VecDeque,
    mem::size_of_val,
    ops::{Deref, DerefMut},
};

use crate::sys;
use crate::{
    core::{self, Library},
    ResultCompErrExt as _,
};
use crate::{ObjectName, ResultComp};
use num_enum::IntoPrimitive;

use super::{ConnectionShare, StringCcsid};

pub type InqReqType = (sys::MQLONG, InqReqItem);
pub type InqResType = (sys::MQLONG, InqResItem);

#[must_use]
pub struct Object<L: Library, H, C: Deref<Target = ConnectionShare<L, H>>> {
    handle: core::ObjectHandle,
    connection: C,
    close_options: (CloseDeleteOptions, CloseReadAheadOptions),
}

#[derive(Debug, Default, Clone, Copy, IntoPrimitive)]
#[repr(i32)]
pub enum CloseDeleteOptions {
    #[default]
    MQCO_NONE = sys::MQCO_NONE,
    MQCO_DELETE = sys::MQCO_DELETE,
    MQCO_DELETE_PURGE = sys::MQCO_DELETE_PURGE,
}

#[derive(Debug, Default, Clone, Copy, IntoPrimitive)]
#[repr(i32)]
pub enum CloseReadAheadOptions {
    #[default]
    MQCO_IMMEDIATE = sys::MQCO_IMMEDIATE,
    MQCO_QUIESCE = sys::MQCO_QUIESCE,
}

#[derive(Debug, Clone, Copy)]
pub enum InqReqItem {
    Str(usize),
    Long,
}

#[derive(Debug, Clone)]
pub enum InqResItem {
    Str(StringCcsid),
    Long(sys::MQLONG),
}
#[derive(Debug, Clone)]
pub struct InqResIterator {
    text_pos: usize,

    strings: Vec<sys::MQCHAR>,
    longs: VecDeque<sys::MQLONG>,
    select: VecDeque<InqReqType>,
}

impl Iterator for InqResIterator {
    type Item = InqResType;

    fn next(&mut self) -> Option<InqResType> {
        match self.select.pop_front() {
            Some((sel, InqReqItem::Str(len))) => {
                let data: &[u8] =
                    unsafe { &*(std::ptr::addr_of!(self.strings[self.text_pos..len + self.text_pos]) as *const [u8]) };
                let value = InqResItem::Str(StringCcsid {
                    ccsid: Option::None,
                    data: data.into(),
                });
                self.text_pos += len;
                Some((sel, value))
            }
            Some((sel, InqReqItem::Long)) => {
                let value = InqResItem::Long(self.longs.pop_front().expect("InqResIterator inconsistent state"));
                Some((sel, value))
            }
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

impl<L: Library, H, C: Deref<Target = ConnectionShare<L, H>>> Object<L, H, C> {
    fn new(handle: core::ObjectHandle, connection: C) -> Self {
        Self {
            handle,
            connection,
            close_options: Default::default(),
        }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub(super) fn handle(&self) -> &core::ObjectHandle {
        &self.handle
    }

    pub fn open(connection: C, mqod: &sys::MQOD, options: sys::MQLONG) -> ResultComp<Self> {
        let result = connection.mq.mqopen(connection.handle(), mqod, options);
        result.map_completion(|oh| Self::new(oh, connection))
    }

    pub fn inq<'a, I: Iterator<Item = &'a InqReqType>>(&self, selectors: I) -> ResultComp<InqResIterator> {
        let mut text_len = 0;
        let mut int_len = 0;
        let select: VecDeque<_> = selectors.copied().collect();
        let mut select_inq = Vec::<i32>::with_capacity(select.len());
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
            .mq
            .mqinq(
                self.connection.handle(),
                self,
                &select_inq,
                &mut int_attr.spare_capacity_mut()[..int_len],
                &mut text_attr.spare_capacity_mut()[..text_len],
            )
            .map_completion(|()| {
                unsafe {
                    text_attr.set_len(text_len);
                    int_attr.set_len(int_len);
                };
                InqResIterator {
                    text_pos: 0,
                    strings: text_attr,
                    longs: VecDeque::from(int_attr),
                    select,
                }
            })
    }

    pub fn put<B>(&self, mqmd: &mut sys::MQMD, pmo: &mut sys::MQPMO, body: &B) -> ResultComp<()> {
        self.connection
            .mq
            .mqput(self.connection.handle(), self, mqmd, pmo, body)
    }

    pub fn get_message<B>(
        &self,
        handle: &core::MessageHandle,
        mo: &MatchOptions,
        wait: Option<sys::MQLONG>,
        body: &mut B,
    ) -> ResultComp<GetMessage> {
        let md = sys::MQMD2 {
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
            .mq
            .mqget(
                self.connection.handle(),
                self,
                &mut core::MQMDType::Mqmd2(md),
                &mut gmo,
                body,
            )
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

    pub fn close_delete(&mut self, options: CloseDeleteOptions) {
        self.close_options.0 = options;
    }

    pub fn close_read_ahead(&mut self, options: CloseReadAheadOptions) {
        self.close_options.1 = options;
    }

    pub fn close(self) -> ResultComp<()> {
        let (delete, read_ahead) = self.close_options;
        let mut s = self;
        s.connection.mq.mqclose(
            s.connection.handle(),
            &mut s.handle,
            Into::<sys::MQLONG>::into(delete) | Into::<sys::MQLONG>::into(read_ahead),
        )
    }
}

impl<L: Library, H, C: Deref<Target = ConnectionShare<L, H>>> Deref for Object<L, H, C> {
    type Target = core::ObjectHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<L: Library, H, C: Deref<Target = ConnectionShare<L, H>>> DerefMut for Object<L, H, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

impl<L: Library, H, C: Deref<Target = ConnectionShare<L, H>>> Drop for Object<L, H, C> {
    fn drop(&mut self) {
        // TODO: handle close failure
        // TODO: handle custom options
        let (delete, read_ahead) = self.close_options;
        let close_options = Into::<i32>::into(delete) | Into::<i32>::into(read_ahead);
        if self.is_closeable() {
            let _ = self
                .connection
                .mq
                .mqclose(self.connection.handle(), &mut self.handle, close_options);
        }
    }
}
