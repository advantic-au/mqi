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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ReconnectionStat {
    pub warning: Option<MQCC>,
    pub reason: MQRC,
    pub object_type: MQOT,
    pub object_name: ObjectName,      // TODO: fix wrapper?
    pub object_qmgr_name: ObjectName, // TODO: fix wrapper?
}

#[derive(Debug, Clone)]
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
