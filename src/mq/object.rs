use std::{
    borrow::Cow,
    cmp::min,
    collections::{vec_deque::Iter, VecDeque},
    fmt::Debug,
    num::NonZero,
    ops::{Deref, DerefMut},
    ptr,
    str::from_utf8_unchecked,
    string::FromUtf8Error,
    sync::Arc,
};

use libmqm_sys::function;

use crate::{
    core::{
        self,
        values::{MQCO, MQENC, MQGMO, MQOO, MQXA},
        ConnectionHandle, Library, MQFunctions,
    },
    mqstr, Buffer, Completion, Conn, Error, Message, MqMask, MqStruct, MqValue, ReasonCode, ResultCompErr, ResultCompErrExt as _,
};
use crate::{sys, MqStr, QMName, QName, StructBuilder};
use crate::ResultComp;

use super::QueueManagerShare;

type Warning = (ReasonCode, &'static str);

trait Sealed {}
#[allow(private_bounds)]
pub trait MQMD: Sealed + std::fmt::Debug {
    fn match_by(match_options: &MatchOptions) -> Self;
}
impl Sealed for sys::MQMD {}
impl Sealed for sys::MQMD2 {}

macro_rules! impl_mqmd {
    ($path:path) => {
        impl MQMD for $path {
            fn match_by(match_options: &MatchOptions) -> Self {
                let mut result = Self::default();

                if let Some(msg_id) = match_options.msg_id {
                    result.MsgId = *msg_id;
                }

                if let Some(correl_id) = match_options.correl_id {
                    result.CorrelId = *correl_id;
                }

                if let Some(group_id) = match_options.group_id {
                    result.GroupId = *group_id;
                }

                result.MsgSeqNumber = match_options.seq_number.unwrap_or(0);
                result.Offset = match_options.offset.unwrap_or(0);

                result
            }
        }
    };
}

impl_mqmd!(sys::MQMD);
impl_mqmd!(sys::MQMD2);

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

impl<L: Library<MQ: function::MQI>, H> Conn for Arc<QueueManagerShare<'_, L, H>> {
    fn mq(&self) -> &MQFunctions<impl Library<MQ: function::MQI>> {
        self.deref().mq()
    }

    fn handle(&self) -> &ConnectionHandle {
        self.deref().handle()
    }
}

impl<L: Library<MQ: function::MQI>, H> Conn for QueueManagerShare<'_, L, H> {
    fn mq(&self) -> &MQFunctions<impl Library<MQ: function::MQI>> {
        Self::mq(self)
    }

    fn handle(&self) -> &ConnectionHandle {
        Self::handle(self)
    }
}

impl<L: Library<MQ: function::MQI>, H> Conn for &QueueManagerShare<'_, L, H> {
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
                    // Note: some fields, such as the initial key are binary and therefore should
                    // use the `iter_mqchar` function.
                    // Refer https://www.ibm.com/docs/en/ibm-mq/9.4?topic=application-using-mqinq-in-client-aplication
                    InqResItem::Str(value) => InqResItem::Str(
                        unsafe { from_utf8_unchecked(&*(ptr::from_ref(value) as *const [u8])) }.trim_end_matches([' ', '\0']),
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
pub struct MatchOptions<'a> {
    pub msg_id: Option<&'a MessageId>,
    pub correl_id: Option<&'a CorrelationId>,
    pub group_id: Option<&'a GroupId>,
    pub seq_number: Option<sys::MQLONG>,
    pub offset: Option<sys::MQLONG>,
    pub token: Option<&'a MsgToken>,
}

pub const ANY_MESSAGE: &MatchOptions = &MatchOptions {
    msg_id: None,
    correl_id: None,
    group_id: None,
    seq_number: None,
    offset: None,
    token: None,
};

pub struct MessageFormat {
    pub ccsid: sys::MQLONG,
    pub encoding: MqMask<MQENC>,
    pub format: MqStr<8>,
}

pub trait GetMessage: Sized {
    type Error: std::fmt::Debug + From<Error>;

    fn create_from<C: Conn>(
        object: &Object<C>,
        data: Cow<[u8]>,
        md: &MqStruct<'static, sys::MQMD2>,
        gmo: &MqStruct<sys::MQGMO>,
        format: &MessageFormat,
        warning: Option<Warning>,
    ) -> Result<Self, Self::Error>;

    #[must_use]
    fn max_data_size() -> Option<NonZero<usize>> {
        None
    }

    #[allow(unused_variables)]
    fn apply_mqget(md: &mut MqStruct<sys::MQMD2>, gmo: &mut MqStruct<sys::MQGMO>) {}
}

// TODO: define this somewhare nicer
const MQSTR: MqStr<8> = mqstr!("MQSTR");

// TODO: add MQ warnings to error messages
#[derive(thiserror::Error, Debug)]
pub enum GetStringError {
    #[error("Message parsing error: {}", .0)]
    Utf8Parse(FromUtf8Error, Option<Warning>),
    #[error("Unexpected format or CCSID. Message format = '{}', CCSID = {}", .0, .1)]
    UnexpectedFormat(MqStr<8>, sys::MQLONG, Option<Warning>),
    #[error(transparent)]
    MQ(#[from] Error),
}

impl GetMessage for String {
    type Error = GetStringError;

    fn apply_mqget(md: &mut MqStruct<sys::MQMD2>, gmo: &mut MqStruct<sys::MQGMO>) {
        gmo.Options |= sys::MQGMO_CONVERT;
        md.CodedCharSetId = 1208;
    }

    fn create_from<C: Conn>(
        _object: &Object<C>,
        data: Cow<[u8]>,
        _md: &MqStruct<sys::MQMD2>,
        _gmo: &MqStruct<sys::MQGMO>,
        format: &MessageFormat,
        warning: Option<Warning>,
    ) -> Result<Self, Self::Error> {
        if format.format != MQSTR || format.ccsid != 1208 {
            return Err(GetStringError::UnexpectedFormat(format.format, format.ccsid, warning));
        }

        match warning {
            Some((rc, verb)) if rc == sys::MQRC_NOT_CONVERTED => Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc).into()),
            other_warning => Ok(Self::from_utf8(data.into_owned()).map_err(|e| GetStringError::Utf8Parse(e, other_warning))?),
        }
    }
}

impl<T> Deref for GetMqmd<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

#[derive(Clone, Debug)]
pub struct GetMqmd<T> {
    pub mqmd: MqStruct<'static, sys::MQMD2>,
    pub body: T,
}

impl<T: GetMessage> GetMessage for GetMqmd<T> {
    type Error = T::Error;

    fn create_from<C: Conn>(
        object: &Object<C>,
        data: Cow<[u8]>,
        md: &MqStruct<'static, sys::MQMD2>,
        gmo: &MqStruct<sys::MQGMO>,
        format: &MessageFormat,
        warning: Option<Warning>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            mqmd: md.clone(),
            body: T::create_from(object, data, md, gmo, format, warning)?,
        })
    }

    fn max_data_size() -> Option<NonZero<usize>> {
        T::max_data_size()
    }

    fn apply_mqget(md: &mut MqStruct<sys::MQMD2>, gmo: &mut MqStruct<sys::MQGMO>) {
        T::apply_mqget(md, gmo);
    }
}

impl GetMessage for Vec<u8> {
    type Error = Error;

    fn create_from<C: Conn>(
        _object: &Object<C>,
        data: Cow<[u8]>,
        _md: &MqStruct<sys::MQMD2>,
        _gmo: &MqStruct<sys::MQGMO>,
        _format: &MessageFormat,
        _warning: Option<Warning>,
    ) -> Result<Self, Self::Error> {
        Ok(data.into_owned())
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

    pub fn get_message<'a, T: GetMessage>(
        &self,
        gmo: MqMask<MQGMO>,
        mo: &MatchOptions,
        wait: Option<sys::MQLONG>,
        properties: Option<&mut Message<C>>,
        buffer: impl Buffer<'a>,
    ) -> ResultCompErr<Option<T>, T::Error> {
        let mut buffer = buffer;

        let mut md = MqStruct::new(sys::MQMD2::match_by(mo));
        let mut gmo = MqStruct::new(sys::MQGMO {
            Version: sys::MQGMO_VERSION_4,
            Options: gmo.value(),
            ..sys::MQGMO::default()
        });
        if let Some(token) = mo.token {
            gmo.MsgToken = *token;
        }
        gmo.MatchOptions = mo.correl_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_CORREL_ID)
            | mo.msg_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_ID)
            | mo.group_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_GROUP_ID)
            | mo.seq_number.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_SEQ_NUMBER)
            | mo.offset.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_OFFSET)
            | mo.token.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_TOKEN);
        // Set up the wait
        if let Some(interval) = wait {
            gmo.Options |= sys::MQGMO_WAIT;
            gmo.WaitInterval = interval;
        }

        if let Some(props) = properties {
            gmo.MsgHandle = unsafe { props.handle().raw_handle() }
        }

        T::apply_mqget(&mut md, &mut gmo);

        let get_result = match self
            .connection
            .mq()
            .mqget(
                self.connection.handle(),
                self.handle(),
                Some(&mut *md),
                &mut gmo,
                buffer.as_mut(),
            )
            .map_completion(|length| match gmo.ReturnedLength {
                sys::MQRL_UNDEFINED => min(
                    buffer.len().try_into().expect("length of buffer must fit in positive i32"),
                    length,
                ),
                returned_length => returned_length,
            }) {
            Err(Error(cc, _, rc)) if cc == sys::MQCC_FAILED && rc == sys::MQRC_NO_MSG_AVAILABLE => Ok(Completion::new(None)),
            other => other.map_completion(Some),
        }?;

        Ok(match get_result {
            Completion(Some(length), warning) => {
                buffer = buffer.truncate(length.try_into().expect("length within positive usize range"));
                Completion(
                    Some(T::create_from(
                        self,
                        buffer.into_cow(),
                        &md,
                        &gmo,
                        &MessageFormat {
                            ccsid: md.CodedCharSetId,
                            encoding: MqMask::from(md.Encoding),
                            format: md.Format.into(),
                        },
                        warning,
                    )?),
                    warning,
                )
            }
            comp => comp.map(|_| None),
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
    use crate::core::values::MQCO;
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
        assert_eq!(list, &[]);

        let (list_iter, _) = MqMask::<MQCO>::from(sys::MQCO_DELETE | sys::MQCO_QUIESCE).masked_list();
        let list = list_iter.collect::<Vec<_>>();
        assert_eq!(
            list,
            &[(sys::MQCO_DELETE, "MQCO_DELETE"), (sys::MQCO_QUIESCE, "MQCO_QUIESCE")]
        );

        // assert_eq!(format!("{oo:?}"), "");
    }
}
