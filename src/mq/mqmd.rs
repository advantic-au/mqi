use crate::{get, sys};

trait Sealed {}
#[allow(private_bounds)]
pub trait MQMD: Sealed + std::fmt::Debug {
    fn match_by(match_options: &get::MatchOptions) -> Self;
}
impl Sealed for sys::MQMD {}
impl Sealed for sys::MQMD2 {}

macro_rules! impl_mqmd {
    ($path:path) => {
        impl MQMD for $path {
            fn match_by(match_options: &get::MatchOptions) -> Self {
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
