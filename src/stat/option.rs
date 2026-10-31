use crate::{
    MqStr,
    string::{CCSID, StrCcsidOwned},
    structs,
    types::{MQCC, MQCHAR, MQLONG, MQOO, MQOT, MQRC, MQSO, ObjectName},
};

fn set_buffer_len(buffer: &mut Vec<MQCHAR>, length: MQLONG, buffer_size: MQLONG) {
    let length = usize::try_from(length).expect("MQ returned a negative string length");
    let buffer_size = usize::try_from(buffer_size).expect("provided buffer size is negative");
    assert!(
        length <= buffer_size && length <= buffer.capacity(),
        "MQ returned a string length larger than the provided buffer"
    );
    // SAFETY: The MQ status call wrote no more than VSBufSize bytes, and the reported length was checked against capacity.
    unsafe { buffer.set_len(length) };
}

impl AsyncPutStat {
    pub(crate) fn new(sts: &structs::MQSTS, buffer: Vec<MQCHAR>) -> Self {
        let mut buffer = buffer;
        set_buffer_len(&mut buffer, sts.ObjectString.VSLength, sts.ObjectString.VSBufSize);

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
    pub(crate) fn new(sts: &structs::MQSTS) -> Self {
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
    pub(crate) fn new(sts: &structs::MQSTS, object_string_buffer: Vec<MQCHAR>, sub_name_buffer: Vec<MQCHAR>) -> Self {
        let mut object_string_buffer = object_string_buffer;
        set_buffer_len(
            &mut object_string_buffer,
            sts.ObjectString.VSLength,
            sts.ObjectString.VSBufSize,
        );
        let mut sub_name_buffer = sub_name_buffer;
        set_buffer_len(&mut sub_name_buffer, sts.SubName.VSLength, sts.SubName.VSBufSize);

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

/// Counts and object details returned after an asynchronous put operation.
#[derive(Debug, Clone)]
pub struct AsyncPutStat {
    /// Completion code when the operation completed with an MQ warning.
    pub warning: Option<MQCC>,
    /// MQ reason code returned for the operation.
    pub reason: MQRC,
    /// Number of messages put successfully.
    pub put_success_count: MQLONG,
    /// Number of messages put with warnings.
    pub put_warning_count: MQLONG,
    /// Number of messages that could not be put.
    pub put_failure_count: MQLONG,
    /// MQ object type reported by the status structure.
    pub object_type: MQOT,
    /// Name of the object associated with the operation.
    pub object_name: ObjectName,
    /// Queue-manager name associated with the object.
    pub object_qmgr_name: ObjectName,
    /// Resolved name of the object used by MQ.
    pub resolved_object_name: ObjectName,
    /// Queue-manager name associated with the resolved object.
    pub resolved_object_qmgr_name: ObjectName,
    /// Optional object string returned by MQ, with its CCSID preserved.
    pub object_string: Option<StrCcsidOwned>,
}

/// Object status and result code reported during connection recovery.
#[derive(Debug, Clone)]
pub struct ReconnectionStat {
    /// Completion code when the operation completed with an MQ warning.
    pub warning: Option<MQCC>,
    /// MQ reason code describing the reconnection status.
    pub reason: MQRC,
    /// MQ object type reported for the object being reconnected.
    pub object_type: MQOT,
    /// Name of the object being reconnected.
    pub object_name: ObjectName,
    /// Queue-manager name associated with the object.
    pub object_qmgr_name: ObjectName,
}

/// Detailed status for an object that could not be restored during reconnection.
#[derive(Debug, Clone)]
pub struct ReconnectionErrorStat {
    /// Completion code when the operation completed with an MQ warning.
    pub warning: Option<MQCC>,
    /// MQ reason code describing the reconnection failure.
    pub reason: MQRC,
    /// MQ object type reported for the object.
    pub object_type: MQOT,
    /// Name of the object that could not be restored.
    pub object_name: ObjectName,
    /// Queue-manager name associated with the object.
    pub object_qmgr_name: ObjectName,
    /// Optional object string returned by MQ, with its CCSID preserved.
    pub object_string: Option<StrCcsidOwned>,
    /// Optional subscription name associated with the failed object.
    pub sub_name: Option<StrCcsidOwned>,
    /// Open options reported for the object.
    pub open_options: MQOO,
    /// Subscription options reported for the object.
    pub sub_options: MQSO,
}

#[cfg(test)]
mod tests {
    use super::set_buffer_len;

    #[test]
    fn set_buffer_len_truncates_to_reported_length() {
        let mut buffer = vec![0; 4];

        set_buffer_len(&mut buffer, 2, 4);

        assert_eq!(buffer.len(), 2);
    }

    #[test]
    #[should_panic(expected = "MQ returned a negative string length")]
    fn set_buffer_len_rejects_negative_length() {
        set_buffer_len(&mut vec![0; 4], -1, 4);
    }

    #[test]
    #[should_panic(expected = "MQ returned a string length larger than the provided buffer")]
    fn set_buffer_len_rejects_length_larger_than_buffer_size() {
        set_buffer_len(&mut vec![0; 2], 3, 2);
    }

    #[test]
    #[should_panic(expected = "provided buffer size is negative")]
    fn set_buffer_len_rejects_negative_buffer_size() {
        set_buffer_len(&mut vec![0; 4], 0, -1);
    }
}
