use std::{num, ptr};

use libmqm_sys::MQI;

use crate::{
    core::{values, Library},
    sys, MqValue, ResultComp,
};

use super::{open_options::ObjectString, MqStruct, MqiAttr, MqiValue, QueueManagerShare, StrCcsidOwned};

pub type StatParam<'a> = MqStruct<'a, sys::MQSTS>;

impl<'cb, L: Library<MQ: MQI>, H> QueueManagerShare<'cb, L, H> {
    pub fn stat<R>(&self, stat_type: MqValue<values::MQSTAT>) -> ResultComp<R>
    where
        R: for<'a> MqiValue<(), Param<'a> = StatParam<'a>>,
    {
        let mut sts = MqStruct::default();
        R::from_mqi(&mut sts, |param| self.mq().mqstat(self.handle(), stat_type, param))
    }
}

impl MqiValue<Self> for () {
    type Param<'a> = ();

    fn from_mqi<F: FnOnce(&mut Self::Param<'_>) -> ResultComp<Self>>(sts: &mut Self::Param<'_>, stat: F) -> ResultComp<Self> {
        stat(sts)
    }
}

const DEFAULT_OBJECTSTRING_LENGTH: sys::MQLONG = 4096;



impl<'b> MqiAttr<StatParam<'b>> for Option<ObjectString<StrCcsidOwned>> {
    fn from_mqi<Y, F: FnOnce(&mut StatParam<'b>) -> Y>(sts: &mut StatParam<'b>, stat: F) -> (Self, Y) {
        if sts.ObjectString.VSBufSize == 0 {
            sts.ObjectString.VSBufSize = DEFAULT_OBJECTSTRING_LENGTH;
        }
        let mut buffer: Vec<_> = Vec::with_capacity(
            sts.ObjectString
                .VSBufSize
                .try_into()
                .expect("buffer length to convert to usize"),
        );
        sts.ObjectString.VSPtr = ptr::from_mut(&mut *buffer).cast();

        let stat_result = stat(sts);
        unsafe {
            buffer.set_len(
                sts.ObjectString
                    .VSLength
                    .try_into()
                    .expect("buffer length to convert to usize"),
            );
        }
        (
            if buffer.is_empty() {
                None
            } else {
                Some(ObjectString(StrCcsidOwned::from_vec(
                    buffer,
                    num::NonZero::new(sts.ObjectString.VSCCSID),
                )))
            },
            stat_result,
        )
    }
}
