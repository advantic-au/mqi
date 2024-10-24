use crate::{macros::all_option_tuples, sys, types, values, Conn, Properties};

use super::get::{GetConvert, GetOption, GetParam, GetWait, MatchOptions};

all_option_tuples!(GetOption, GetParam);

impl GetOption for values::MQGMO {
    fn apply_param(self, param: &mut GetParam) {
        param.gmo.Options |= self.value();
    }
}

impl GetOption for GetWait {
    fn apply_param(self, param: &mut GetParam) {
        match self {
            Self::NoWait => param.gmo.Options |= sys::MQGMO_NO_WAIT,
            Self::Wait(interval) => {
                param.gmo.Options |= sys::MQGMO_WAIT;
                param.gmo.WaitInterval = interval;
            }
        }
    }
}

impl GetOption for GetConvert {
    fn apply_param(self, param: &mut GetParam) {
        match self {
            Self::NoConvert => {}
            Self::Convert => param.gmo.Options |= sys::MQGMO_CONVERT,
            Self::ConvertTo(ccsid, encoding) => {
                param.gmo.Options |= sys::MQGMO_CONVERT;
                param.md.CodedCharSetId = ccsid.0;
                param.md.Encoding = encoding.value();
            }
        }
    }
}

impl<C: Conn> GetOption for &mut Properties<C> {
    fn apply_param(self, param: &mut GetParam) {
        param.gmo.Options |= sys::MQGMO_PROPERTIES_IN_HANDLE;
        param.gmo.MsgHandle = unsafe { self.handle().raw_handle() }
    }
}

impl GetOption for MatchOptions<'_> {
    fn apply_param(self, param: &mut GetParam) {
        // Set up the MQMD
        if let Some(msg_id) = self.msg_id {
            param.md.MsgId = *msg_id.0;
        }
        if let Some(correl_id) = self.correl_id {
            param.md.CorrelId = *correl_id.0;
        }
        if let Some(group_id) = self.group_id {
            param.md.GroupId = *group_id.0;
        }
        param.md.MsgSeqNumber = self.seq_number.unwrap_or(0);
        param.md.Offset = self.offset.unwrap_or(0);

        // Set up the GMO
        if let Some(token) = self.token {
            param.gmo.MsgToken = token.0;
        }
        param.gmo.MatchOptions = self.correl_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_CORREL_ID)
            | self.msg_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_ID)
            | self.group_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_GROUP_ID)
            | self.seq_number.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_SEQ_NUMBER)
            | self.offset.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_OFFSET)
            | self.token.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_TOKEN);
    }
}

impl GetOption for types::CorrelationId {
    fn apply_param(self, param: &mut GetParam) {
        param.md.CorrelId = *self.0;
        param.gmo.MatchOptions |= sys::MQMO_MATCH_CORREL_ID;
    }
}

impl GetOption for types::MessageId {
    fn apply_param(self, param: &mut GetParam) {
        param.md.MsgId = *self.0;
        param.gmo.MatchOptions |= sys::MQMO_MATCH_MSG_ID;
    }
}

impl GetOption for types::GroupId {
    fn apply_param(self, param: &mut GetParam) {
        param.md.GroupId = *self.0;
        param.gmo.MatchOptions |= sys::MQMO_MATCH_GROUP_ID;
    }
}

impl GetOption for types::MsgToken {
    fn apply_param(self, param: &mut GetParam) {
        param.gmo.MsgToken = self.0;
        param.gmo.MatchOptions |= sys::MQMO_MATCH_MSG_TOKEN;
    }
}
