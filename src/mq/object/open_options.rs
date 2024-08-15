use std::{cmp, num, ptr};

use crate::{
    core::values,
    sys,
    types::{QueueManagerName, QueueName},
    Conn, EncodedString, MqStr, MqValue, MqiAttr, MqiOption, MqiValue, ResultComp, StrCcsidOwned,
};

use super::{Object, OpenParam};

#[derive(Debug, Clone, Copy)]
pub struct SelectionString<T>(pub T);

#[derive(Debug, Clone, Copy)]
pub struct ObjectString<T>(pub T);

#[derive(Debug, Clone, Copy)]
pub struct AlternateUserId(pub MqStr<12>);

#[derive(Debug, Clone)]
pub struct ResObjectString(pub StrCcsidOwned);

impl<'a, T: EncodedString + ?Sized, O> MqiOption<OpenParam<'a, O>> for SelectionString<&'a T> {
    fn apply_param(self, (od, ..): &mut OpenParam<'a, O>) {
        od.attach_selection_string(self.0);
    }
}

impl<'a, T: EncodedString + ?Sized, O> MqiOption<OpenParam<'a, O>> for ObjectString<&'a T> {
    fn apply_param(self, (od, ..): &mut OpenParam<'a, O>) {
        od.attach_object_string(self.0);
    }
}

impl<'b, O> MqiOption<OpenParam<'b, O>> for QueueName {
    fn apply_param(self, (od, ..): &mut OpenParam<'_, O>) {
        od.ObjectName = self.0.into();
        od.ObjectType = sys::MQOT_Q;
    }
}

impl<'b, O> MqiOption<OpenParam<'b, O>> for QueueManagerName {
    fn apply_param(self, (od, ..): &mut OpenParam<'_, O>) {
        od.ObjectQMgrName = self.0.into();
        od.ObjectType = sys::MQOT_Q_MGR;
    }
}

impl<'b> MqiOption<OpenParam<'b, values::MQOO>> for AlternateUserId {
    fn apply_param(self, (od, oo): &mut OpenParam<'_, values::MQOO>) {
        *oo |= sys::MQOO_ALTERNATE_USER_AUTHORITY;
        od.Version = cmp::max(sys::MQOD_VERSION_3, od.Version);
        od.AlternateUserId = self.0.into();
    }
}

impl<'b> MqiOption<OpenParam<'b, values::MQPMO>> for AlternateUserId {
    fn apply_param(self, (od, pmo): &mut OpenParam<'_, values::MQPMO>) {
        *pmo |= sys::MQPMO_ALTERNATE_USER_AUTHORITY;
        od.Version = cmp::max(sys::MQOD_VERSION_3, od.Version);
        od.AlternateUserId = self.0.into();
    }
}


impl<'b, O> MqiAttr<OpenParam<'b, O>> for Option<QueueName> {
    fn from_mqi<Y, F: FnOnce(&mut OpenParam<'b, O>) -> Y>(param: &mut OpenParam<'b, O>, open: F) -> (Self, Y) {
        let open_result = open(param);
        (
            Some(QueueName(param.0.ResolvedQName.into())).filter(|queue_name| queue_name.has_value()),
            open_result,
        )
    }
}

impl<'b, O> MqiAttr<OpenParam<'b, O>> for MqValue<values::MQOT> {
    fn from_mqi<Y, F: for<'a> FnOnce(&'a mut OpenParam<'b, O>) -> Y>(param: &mut OpenParam<'b, O>, open: F) -> (Self, Y) {
        param.0.Version = cmp::max(sys::MQOD_VERSION_4, param.0.Version);
        let open_result = open(param);
        (param.0.ObjectType.into(), open_result)
    }
}

impl<C: Conn> MqiValue<Self> for Object<C> {
    type Param<'a> = OpenParam<'a, values::MQOO>;

    fn from_mqi<F: FnOnce(&mut Self::Param<'_>) -> ResultComp<Self>>(mqod: &mut Self::Param<'_>, open: F) -> ResultComp<Self> {
        open(mqod)
    }
}

impl<'a, O> MqiAttr<OpenParam<'a, O>> for Option<QueueManagerName> {
    fn from_mqi<Y, F: FnOnce(&mut OpenParam<'a, O>) -> Y>(param: &mut OpenParam<'a, O>, open: F) -> (Self, Y) {
        let open_result = open(param);
        (
            Self::Some(QueueManagerName(param.0.ResolvedQMgrName.into())).filter(|queue_name| queue_name.has_value()),
            open_result,
        )
    }
}

const DEFAULT_RESOBJECTSTRING_LENGTH: sys::MQLONG = 4096;

impl<'b, O> MqiAttr<OpenParam<'b, O>> for Option<ResObjectString> {
    fn from_mqi<Y, F: for<'a> FnOnce(&'a mut OpenParam<'b, O>) -> Y>(param: &mut OpenParam<'b, O>, open: F) -> (Self, Y) {
        let od = &mut param.0;
        if od.ResObjectString.VSBufSize == 0 {
            od.ResObjectString.VSBufSize = DEFAULT_RESOBJECTSTRING_LENGTH;
        }
        let mut buffer: Vec<_> = Vec::with_capacity(
            od.ResObjectString
                .VSBufSize
                .try_into()
                .expect("buffer length to convert to usize"),
        );
        od.ResObjectString.VSPtr = ptr::from_mut(&mut *buffer).cast();

        od.ResObjectString.VSCCSID = 500;

        let open_result = open(param);

        let od = &mut param.0;
        unsafe {
            buffer.set_len(
                od.ResObjectString
                    .VSLength
                    .try_into()
                    .expect("buffer length to convert to usize"),
            );
        }
        (
            if buffer.is_empty() {
                None
            } else {
                Some(ResObjectString(StrCcsidOwned::from_vec(
                    buffer,
                    num::NonZero::new(od.ResObjectString.VSCCSID),
                )))
            },
            open_result,
        )
    }
}
