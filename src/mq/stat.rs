use libmqm_sys::Mqi;
use libmqm_sys::lib as sys;
use libmqm_default as default;
use crate::types::ObjectName;
use crate::types::{MQCHAR, MQLONG, MQCC, MQRC};

use crate::StrCcsidOwned;
use crate::{
    ConnectionHandle, Library, MqFunctions, CCSID,
    types::{MQOT, MQOO, MQSO},
    prelude::*,
    structs, constants, MqStr, ResultComp,
};

impl AsyncPutStat {
    fn new(sts: &structs::MQSTS, buffer: Vec<MQCHAR>) -> Self {
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
            object_type: MQOT(sts.ObjectType),
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
    fn new(sts: &structs::MQSTS) -> Self {
        Self {
            warning: match sts.CompCode {
                0 => None,
                value => Some(MQCC::from(value)),
            },
            reason: MQRC::from(sts.Reason),
            object_type: MQOT(sts.ObjectType),
            object_name: MqStr::from(sts.ObjectName),
            object_qmgr_name: MqStr::from(sts.ObjectQMgrName),
        }
    }
}

impl ReconnectionErrorStat {
    fn new(sts: &structs::MQSTS, object_string_buffer: Vec<MQCHAR>, sub_name_buffer: Vec<MQCHAR>) -> Self {
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
            object_type: MQOT(sts.ObjectType),
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
            open_options: MQOO(sts.OpenOptions),
            sub_options: MQSO(sts.SubOptions),
        }
    }
}

pub fn stat_put<L: Library<MQ: Mqi>>(functions: &MqFunctions<L>, handle: ConnectionHandle) -> ResultComp<AsyncPutStat> {
    let mut sts = structs::MQSTS::new(sys::MQSTS {
        Version: sys::MQSTS_VERSION_2,
        ..default::MQSTS_DEFAULT
    });

    if sts.ObjectString.VSBufSize == 0 {
        sts.ObjectString.VSBufSize = DEFAULT_OBJECTSTRING_LENGTH;
    }
    let mut buffer = Vec::with_capacity(
        sts.ObjectString
            .VSBufSize
            .try_into()
            .expect("buffer length should convert to usize"),
    );
    sts.ObjectString.VSPtr = (&raw mut *buffer).cast();

    // SAFETY: MQSTS ObjectString MQCHARV constructed from buffer
    unsafe {
        functions
            .mqstat(handle, constants::MQSTAT_TYPE_ASYNC_ERROR, &mut sts)
            .map_completion(|()| AsyncPutStat::new(&sts, buffer))
    }
}

pub fn stat_reconnection<L: Library<MQ: Mqi>>(
    functions: &MqFunctions<L>,
    handle: ConnectionHandle,
) -> ResultComp<ReconnectionStat> {
    let mut sts = structs::MQSTS::new(default::MQSTS_DEFAULT);

    // SAFETY: MQSTS No pointers populated
    unsafe {
        functions
            .mqstat(handle, constants::MQSTAT_TYPE_RECONNECTION, &mut sts)
            .map_completion(|()| ReconnectionStat::new(&sts))
    }
}

pub fn stat_reconnection_error<L: Library<MQ: Mqi>>(
    functions: &MqFunctions<L>,
    handle: ConnectionHandle,
) -> ResultComp<ReconnectionErrorStat> {
    let mut sts = structs::MQSTS::new(sys::MQSTS {
        Version: sys::MQSTS_VERSION_2,
        ..default::MQSTS_DEFAULT
    });

    sts.ObjectString.VSBufSize = DEFAULT_OBJECTSTRING_LENGTH;
    let mut object_string_buffer = Vec::with_capacity(
        sts.ObjectString
            .VSBufSize
            .try_into()
            .expect("buffer length should convert to usize"),
    );
    sts.ObjectString.VSPtr = (&raw mut *object_string_buffer).cast();

    sts.SubName.VSBufSize = DEFAULT_OBJECTSTRING_LENGTH;
    let mut sub_name_buffer = Vec::with_capacity(
        sts.SubName
            .VSBufSize
            .try_into()
            .expect("buffer length should convert to usize"),
    );
    sts.SubName.VSPtr = (&raw mut *sub_name_buffer).cast();

    // SAFETY: MQSTS ObjectString and SubName MQCHARV constructed from buffers
    unsafe {
        functions
            .mqstat(handle, constants::MQSTAT_TYPE_RECONNECTION_ERROR, &mut sts)
            .map_completion(|()| ReconnectionErrorStat::new(&sts, object_string_buffer, sub_name_buffer))
    }
}

const DEFAULT_OBJECTSTRING_LENGTH: MQLONG = 4096;

pub struct AsyncPutStat {
    pub warning: Option<MQCC>,
    pub reason: MQRC,
    pub put_success_count: MQLONG,
    pub put_warning_count: MQLONG,
    pub put_failure_count: MQLONG,
    pub object_type: MQOT,
    pub object_name: ObjectName,               // TODO: fix wrapper?
    pub object_qmgr_name: ObjectName,          // TODO: fix wrapper?
    pub resolved_object_name: ObjectName,      // TODO: fix wrapper?
    pub resolved_object_qmgr_name: ObjectName, // TODO: fix wrapper?
    pub object_string: Option<StrCcsidOwned>,
}

pub struct ReconnectionStat {
    pub warning: Option<MQCC>,
    pub reason: MQRC,
    pub object_type: MQOT,
    pub object_name: ObjectName,      // TODO: fix wrapper?
    pub object_qmgr_name: ObjectName, // TODO: fix wrapper?
}

pub struct ReconnectionErrorStat {
    pub warning: Option<MQCC>,
    pub reason: MQRC,
    pub object_type: MQOT,
    pub object_name: ObjectName,      // TODO: fix wrapper?
    pub object_qmgr_name: ObjectName, // TODO: fix wrapper?
    pub object_string: Option<StrCcsidOwned>,
    pub sub_name: Option<StrCcsidOwned>,
    pub open_options: MQOO,
    pub sub_options: MQSO,
}
