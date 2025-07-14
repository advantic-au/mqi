use libmqm_default as default;
use libmqm_sys as mq;
use libmqm_sys::Mqi;

use super::option;
use crate::{Library, MqFunctions, constants, handle::ConnectionHandle, prelude::*, result::ResultComp, structs, types};

const DEFAULT_OBJECTSTRING_LENGTH: types::MQLONG = 4096;

/// This function uses the [`MQSTAT`](libmqm_sys::MQSTAT) MQ API function.
pub fn stat_put<L: Library<MQ: Mqi>>(functions: &MqFunctions<L>, handle: ConnectionHandle) -> ResultComp<option::AsyncPutStat> {
    let mut sts = structs::MQSTS::new(mq::MQSTS {
        Version: mq::MQSTS_VERSION_2,
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
            .map_completion(|()| option::AsyncPutStat::new(&sts, buffer))
    }
}

/// This function uses the [`MQSTAT`](libmqm_sys::MQSTAT) MQ API function.
pub fn stat_reconnection<L: Library<MQ: Mqi>>(
    functions: &MqFunctions<L>,
    handle: ConnectionHandle,
) -> ResultComp<option::ReconnectionStat> {
    let mut sts = structs::MQSTS::new(default::MQSTS_DEFAULT);

    // SAFETY: MQSTS No pointers populated
    unsafe {
        functions
            .mqstat(handle, constants::MQSTAT_TYPE_RECONNECTION, &mut sts)
            .map_completion(|()| option::ReconnectionStat::new(&sts))
    }
}

/// This function uses the [`MQSTAT`](libmqm_sys::MQSTAT) MQ API function.
pub fn stat_reconnection_error<L: Library<MQ: Mqi>>(
    functions: &MqFunctions<L>,
    handle: ConnectionHandle,
) -> ResultComp<option::ReconnectionErrorStat> {
    let mut sts = structs::MQSTS::new(mq::MQSTS {
        Version: mq::MQSTS_VERSION_2,
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
            .map_completion(|()| option::ReconnectionErrorStat::new(&sts, object_string_buffer, sub_name_buffer))
    }
}
