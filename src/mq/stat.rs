use std::ptr;

use libmqm_sys::function;

use crate::{
    core::{ConnectionHandle, Library, MqFunctions},
    prelude::*,
    sys,
    values::{self, MQCC, MQRC, CCSID},
    MqStr, ResultComp,
};

use super::{types::ObjectName, MqStruct, StrCcsidOwned};

impl AsyncPutStat {
    fn new(sts: &MqStruct<sys::MQSTS>, buffer: Vec<u8>) -> Self {
        let mut buffer = buffer;
        unsafe {
            buffer.set_len(
                sts.ObjectString
                    .VSLength
                    .try_into()
                    .expect("buffer length should convert to usize"),
            );
        }

        Self {
            warning: match sts.CompCode {
                0 => None,
                value => Some(MQCC::from(value)),
            },
            reason: MQRC::from(sts.Reason),
            put_success_count: sts.PutSuccessCount,
            put_warning_count: sts.PutWarningCount,
            put_failure_count: sts.PutFailureCount,
            object_type: values::MQOT(sts.ObjectType),
            object_name: MqStr::from(sts.ObjectName),
            object_qmgr_name: MqStr::from(sts.ObjectQMgrName),
            resolved_object_name: MqStr::from(sts.ResolvedObjectName),
            resolved_object_qmgr_name: MqStr::from(sts.ResolvedQMgrName),
            object_string: if buffer.is_empty() {
                None
            } else {
                Some(StrCcsidOwned::from_vec(buffer, CCSID(sts.ObjectString.VSCCSID)))
            },
        }
    }
}

impl ReconnectionStat {
    fn new(sts: &MqStruct<sys::MQSTS>) -> Self {
        Self {
            warning: match sts.CompCode {
                0 => None,
                value => Some(MQCC::from(value)),
            },
            reason: MQRC::from(sts.Reason),
            object_type: values::MQOT(sts.ObjectType),
            object_name: MqStr::from(sts.ObjectName),
            object_qmgr_name: MqStr::from(sts.ObjectQMgrName),
        }
    }
}

impl ReconnectionErrorStat {
    fn new(sts: &MqStruct<sys::MQSTS>, object_string_buffer: Vec<u8>, sub_name_buffer: Vec<u8>) -> Self {
        let mut object_string_buffer = object_string_buffer;
        unsafe {
            object_string_buffer.set_len(
                sts.ObjectString
                    .VSLength
                    .try_into()
                    .expect("buffer length should convert to usize"),
            );
        }
        let mut sub_name_buffer = sub_name_buffer;
        unsafe {
            sub_name_buffer.set_len(
                sts.SubName
                    .VSLength
                    .try_into()
                    .expect("buffer length should convert to usize"),
            );
        }

        Self {
            warning: match sts.CompCode {
                0 => None,
                value => Some(MQCC::from(value)),
            },
            reason: MQRC::from(sts.Reason),
            object_type: values::MQOT(sts.ObjectType),
            object_name: MqStr::from(sts.ObjectName),
            object_qmgr_name: MqStr::from(sts.ObjectQMgrName),
            object_string: if object_string_buffer.is_empty() {
                None
            } else {
                Some(StrCcsidOwned::from_vec(object_string_buffer, CCSID(sts.ObjectString.VSCCSID)))
            },
            sub_name: if sub_name_buffer.is_empty() {
                None
            } else {
                Some(StrCcsidOwned::from_vec(sub_name_buffer, CCSID(sts.SubName.VSCCSID)))
            },
            open_options: values::MQOO(sts.OpenOptions),
            sub_options: values::MQSO(sts.SubOptions),
        }
    }
}

pub fn stat_put<L: Library<MQ: function::Mqi>>(functions: &MqFunctions<L>, handle: ConnectionHandle) -> ResultComp<AsyncPutStat> {
    let mut sts = MqStruct::new(sys::MQSTS {
        Version: sys::MQSTS_VERSION_2,
        ..sys::MQSTS::default()
    });

    if sts.ObjectString.VSBufSize == 0 {
        sts.ObjectString.VSBufSize = DEFAULT_OBJECTSTRING_LENGTH;
    }
    let mut buffer: Vec<_> = Vec::with_capacity(
        sts.ObjectString
            .VSBufSize
            .try_into()
            .expect("buffer length should convert to usize"),
    );
    sts.ObjectString.VSPtr = ptr::from_mut(&mut *buffer).cast();

    functions
        .mqstat(handle, values::MQSTAT(sys::MQSTAT_TYPE_ASYNC_ERROR), &mut sts)
        .map_completion(|()| AsyncPutStat::new(&sts, buffer))
}

pub fn stat_reconnection<L: Library<MQ: function::Mqi>>(
    functions: &MqFunctions<L>,
    handle: ConnectionHandle,
) -> ResultComp<ReconnectionStat> {
    let mut sts = MqStruct::default();
    functions
        .mqstat(handle, values::MQSTAT(sys::MQSTAT_TYPE_RECONNECTION), &mut sts)
        .map_completion(|()| ReconnectionStat::new(&sts))
}

pub fn stat_reconnection_error<L: Library<MQ: function::Mqi>>(
    functions: &MqFunctions<L>,
    handle: ConnectionHandle,
) -> ResultComp<ReconnectionErrorStat> {
    let mut sts = MqStruct::new(sys::MQSTS {
        Version: sys::MQSTS_VERSION_2,
        ..sys::MQSTS::default()
    });

    sts.ObjectString.VSBufSize = DEFAULT_OBJECTSTRING_LENGTH;
    let mut object_string_buffer: Vec<_> = Vec::with_capacity(
        sts.ObjectString
            .VSBufSize
            .try_into()
            .expect("buffer length should convert to usize"),
    );
    sts.ObjectString.VSPtr = ptr::from_mut(&mut *object_string_buffer).cast();

    sts.SubName.VSBufSize = DEFAULT_OBJECTSTRING_LENGTH;
    let mut sub_name_buffer: Vec<_> = Vec::with_capacity(
        sts.SubName
            .VSBufSize
            .try_into()
            .expect("buffer length should convert to usize"),
    );
    sts.SubName.VSPtr = ptr::from_mut(&mut *sub_name_buffer).cast();

    functions
        .mqstat(handle, values::MQSTAT(sys::MQSTAT_TYPE_RECONNECTION_ERROR), &mut sts)
        .map_completion(|()| ReconnectionErrorStat::new(&sts, object_string_buffer, sub_name_buffer))
}

const DEFAULT_OBJECTSTRING_LENGTH: sys::MQLONG = 4096;

pub struct AsyncPutStat {
    pub warning: Option<MQCC>,
    pub reason: MQRC,
    pub put_success_count: sys::MQLONG,
    pub put_warning_count: sys::MQLONG,
    pub put_failure_count: sys::MQLONG,
    pub object_type: values::MQOT,
    pub object_name: ObjectName,               // TODO: fix wrapper?
    pub object_qmgr_name: ObjectName,          // TODO: fix wrapper?
    pub resolved_object_name: ObjectName,      // TODO: fix wrapper?
    pub resolved_object_qmgr_name: ObjectName, // TODO: fix wrapper?
    pub object_string: Option<StrCcsidOwned>,
}

pub struct ReconnectionStat {
    pub warning: Option<MQCC>,
    pub reason: MQRC,
    pub object_type: values::MQOT,
    pub object_name: ObjectName,      // TODO: fix wrapper?
    pub object_qmgr_name: ObjectName, // TODO: fix wrapper?
}

pub struct ReconnectionErrorStat {
    pub warning: Option<MQCC>,
    pub reason: MQRC,
    pub object_type: values::MQOT,
    pub object_name: ObjectName,      // TODO: fix wrapper?
    pub object_qmgr_name: ObjectName, // TODO: fix wrapper?
    pub object_string: Option<StrCcsidOwned>,
    pub sub_name: Option<StrCcsidOwned>,
    pub open_options: values::MQOO,
    pub sub_options: values::MQSO,
}
