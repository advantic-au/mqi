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

#[expect(unused_parens)]
mod get_impl {
    use crate::get::{GetAttr, GetValue, GetParam, GetState};
    use crate::macros::all_multi_tuples;
    use crate::prelude::*;
    use crate::{ResultCompErr, ResultComp};

    macro_rules! impl_getvalue {
        ([$first:ident, $($ty:ident),*]) => {
            #[expect(non_snake_case)]
            impl<B, $first, $($ty),*> GetValue<B> for ($first, $($ty),*)
            where
                $first: GetValue<B>,
                $($ty: GetAttr<B>),*
            {
                type Error = $first::Error;

                #[inline]
                fn consume<F>(param: &mut GetParam, mqi: F) -> ResultCompErr<Self, Self::Error>
                where
                    F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
                {
                    let mut rest_outer = None;
                    $first::consume(param, |param| {
                        <($($ty),*) as GetAttr<B>>::extract(param, mqi).map_completion(|(rest, state)| {
                            rest_outer = Some(rest);
                            state
                        })
                    })
                    .map_completion(|a| {
                        let ($($ty),*) = rest_outer.expect("rest_outer should be set by the extract closure");
                        (a, $($ty),*)
                    })
                }

                fn max_data_size() -> Option<std::num::NonZero<usize>> {
                    $first::max_data_size()
                }
            }
        };
    }

    macro_rules! impl_getattr {
        ([$first:ident, $($ty:ident),*]) => {
            #[expect(non_snake_case)]
            impl<B, $first, $($ty),*> GetAttr<B> for ($first, $($ty),*)
            where
                $first: GetAttr<B>,
                $($ty: GetAttr<B>),*
            {
                #[inline]
                fn extract<F>(param: &mut GetParam, mqi: F) -> ResultComp<(Self, GetState<B>)>
                where
                    F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>
                {
                    let mut rest_outer = None;
                    $first::extract(param, |param| {
                        <($($ty),*) as GetAttr<B>>::extract(param, mqi).map_completion(|(rest, state)| {
                            rest_outer = Some(rest);
                            state
                        })
                    })
                    .map_completion(|(a, s)| {
                        let ($($ty),*) = rest_outer.expect("rest_outer should be set by extract closure");
                        ((a, $($ty),*), s)
                    })
                }
            }
        }
    }

    all_multi_tuples!(impl_getvalue);
    all_multi_tuples!(impl_getattr);
}
