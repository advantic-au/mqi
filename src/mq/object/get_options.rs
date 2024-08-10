use crate::{core::values, sys, types, Conn, Message, MqMask, MqiOption};

use super::get::{GetConvert, GetParam, GetWait, MatchOptions};

impl MqiOption<'_, GetParam> for MqMask<values::MQGMO> {
    fn apply_param(&self, (_md, gmo): &mut GetParam) {
        gmo.Options |= self.value();
    }
}

impl MqiOption<'_, GetParam> for GetWait {
    fn apply_param(&self, (_md, gmo): &mut GetParam) {
        match self {
            Self::NoWait => gmo.Options |= sys::MQGMO_NO_WAIT,
            Self::Wait(interval) => {
                gmo.Options |= sys::MQGMO_WAIT;
                gmo.WaitInterval = *interval;
            }
        }
    }
}

impl MqiOption<'_, GetParam> for GetConvert {
    fn apply_param(&self, (md, gmo): &mut GetParam) {
        match self {
            Self::NoConvert => {}
            Self::Convert => gmo.Options |= sys::MQGMO_CONVERT,
            Self::ConvertTo(ccsid, encoding) => {
                gmo.Options |= sys::MQGMO_CONVERT;
                md.CodedCharSetId = *ccsid;
                md.Encoding = encoding.value();
            }
        }
    }
}

impl<C: Conn> MqiOption<'_, GetParam> for &mut Message<C> {
    fn apply_param(&self, (_md, gmo): &mut GetParam) {
        gmo.Options |= sys::MQGMO_PROPERTIES_IN_HANDLE;
        gmo.MsgHandle = unsafe { self.handle().raw_handle() }
    }
}

impl MqiOption<'_, GetParam> for MatchOptions<'_> {
    fn apply_param(&self, (md, gmo): &mut GetParam) {
        // Set up the MQMD
        if let Some(msg_id) = self.msg_id {
            md.MsgId = msg_id.0;
        }
        if let Some(correl_id) = self.correl_id {
            md.CorrelId = correl_id.0;
        }
        if let Some(group_id) = self.group_id {
            md.GroupId = group_id.0;
        }
        md.MsgSeqNumber = self.seq_number.unwrap_or(0);
        md.Offset = self.offset.unwrap_or(0);

        // Set up the GMO
        if let Some(token) = self.token {
            gmo.MsgToken = token.0;
        }
        gmo.MatchOptions = self.correl_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_CORREL_ID)
            | self.msg_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_ID)
            | self.group_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_GROUP_ID)
            | self.seq_number.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_SEQ_NUMBER)
            | self.offset.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_OFFSET)
            | self.token.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_TOKEN);
    }
}

impl MqiOption<'_, GetParam> for types::CorrelationId {
    fn apply_param(&self, (md, gmo): &mut GetParam) {
        md.CorrelId = self.0;
        gmo.MatchOptions |= sys::MQMO_MATCH_CORREL_ID;
    }
}

impl MqiOption<'_, GetParam> for types::MessageId {
    fn apply_param(&self, (md, gmo): &mut GetParam) {
        md.MsgId = self.0;
        gmo.MatchOptions |= sys::MQMO_MATCH_MSG_ID;
    }
}

impl MqiOption<'_, GetParam> for types::GroupId {
    fn apply_param(&self, (md, gmo): &mut GetParam) {
        md.GroupId = self.0;
        gmo.MatchOptions |= sys::MQMO_MATCH_GROUP_ID;
    }
}

impl MqiOption<'_, GetParam> for types::MsgToken {
    fn apply_param(&self, (_md, gmo): &mut GetParam) {
        gmo.MsgToken = self.0;
        gmo.MatchOptions |= sys::MQMO_MATCH_MSG_TOKEN;
    }
}
