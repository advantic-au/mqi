use std::{cmp, num, ptr};

use libmqm_sys::MQI;

use crate::{
    core::{values, Library}, sys, CompletionCode, MqValue, ReasonCode, ResultComp
};

use super::{open_options::ObjectString, types::{ObjectName, QueueManagerName}, MqStruct, MqiAttr, MqiValue, QueueManagerShare, StrCcsidOwned};

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
        sts.Version = cmp::max(2, sts.Version);

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

pub struct AsyncPut {
    pub warning: Option<CompletionCode>,
    pub reason: ReasonCode,
    pub put_success_count: sys::MQLONG,
    pub put_warning_count: sys::MQLONG,
    pub put_failure_count: sys::MQLONG,
    pub object_type: MqValue<values::MQOT>,
    pub object_name: ObjectName, // TODO: fix wrapper?
    pub object_qmgr_name: QueueManagerName,
    pub resolved_object_name: ObjectName, // TODO: fix wrapper?
    pub resolved_object_qmgr_name: QueueManagerName,
    pub object_string: StrCcsidOwned,
}

#[derive(Debug, Clone, Copy)]
struct SubName<T>(pub T);

impl<'b> MqiAttr<StatParam<'b>> for Option<SubName<StrCcsidOwned>> {
    fn from_mqi<Y, F: FnOnce(&mut StatParam<'b>) -> Y>(sts: &mut StatParam<'b>, stat: F) -> (Self, Y) {
        sts.Version = cmp::max(2, sts.Version);

        if sts.SubName.VSBufSize == 0 {
            sts.SubName.VSBufSize = DEFAULT_OBJECTSTRING_LENGTH;
        }
        let mut buffer: Vec<_> = Vec::with_capacity(
            sts.SubName
                .VSBufSize
                .try_into()
                .expect("buffer length to convert to usize"),
        );
        sts.SubName.VSPtr = ptr::from_mut(&mut *buffer).cast();

        let stat_result = stat(sts);

        unsafe {
            buffer.set_len(
                sts.SubName
                    .VSLength
                    .try_into()
                    .expect("buffer length to convert to usize"),
            );
        }
        (
            if buffer.is_empty() {
                None
            } else {
                Some(SubName(StrCcsidOwned::from_vec(
                    buffer,
                    num::NonZero::new(sts.SubName.VSCCSID),
                )))
            },
            stat_result,
        )
    }
}