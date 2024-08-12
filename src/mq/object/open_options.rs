use std::{cmp, num, ptr};

use crate::{
    core::values,
    sys,
    types::{QueueManagerName, QueueName},
    Conn, EncodedString, MqStr, MqValue, MqiAttr, MqiOption, MqiValue, ResultComp, StrCcsidOwned,
};

use super::{Object, ObjectDescriptor};

#[derive(Debug, Clone, Copy)]
pub struct SelectionString<T>(pub T);

#[derive(Debug, Clone, Copy)]
pub struct ObjectString<T>(pub T);

#[derive(Debug, Clone, Copy)]
pub struct AlternateUserId(pub MqStr<12>);

#[derive(Debug, Clone)]
pub struct ResObjectString(pub StrCcsidOwned);

impl<'b, T: EncodedString + ?Sized> MqiOption<'b, ObjectDescriptor<'b>> for SelectionString<&T> {
    fn apply_param(&'b self, param: &mut ObjectDescriptor<'b>) {
        param.attach_selection_string(self.0);
    }
}

impl<'b, T: EncodedString + ?Sized> MqiOption<'b, ObjectDescriptor<'b>> for ObjectString<&T> {
    fn apply_param(&'b self, param: &mut ObjectDescriptor<'b>) {
        param.attach_object_string(self.0);
    }
}

impl MqiOption<'_, ObjectDescriptor<'_>> for QueueName {
    fn apply_param(&self, param: &mut ObjectDescriptor<'_>) {
        param.ObjectName = self.0.into();
        param.ObjectType = sys::MQOT_Q;
    }
}

impl MqiOption<'_, ObjectDescriptor<'_>> for QueueManagerName {
    fn apply_param(&self, param: &mut ObjectDescriptor<'_>) {
        param.ObjectQMgrName = self.0.into();
        param.ObjectType = sys::MQOT_Q_MGR;
    }
}

impl MqiOption<'_, ObjectDescriptor<'_>> for AlternateUserId {
    fn apply_param(&self, param: &mut ObjectDescriptor<'_>) {
        // TODO: handle the MQOO and MQPMO options
        param.Version = cmp::max(sys::MQOD_VERSION_3, param.Version);
        param.AlternateUserId = self.0.into();
    }
}

impl<'b> MqiAttr<ObjectDescriptor<'b>> for Option<QueueName> {
    fn apply_param<Y, F: for<'a> FnOnce(&'a mut ObjectDescriptor<'b>) -> Y>(
        param: &mut ObjectDescriptor<'b>,
        open: F,
    ) -> (Self, Y) {
        let open_result = open(param);
        (
            Some(QueueName(param.ResolvedQName.into())).filter(|queue_name| queue_name.has_value()),
            open_result,
        )
    }
}

impl<'b> MqiAttr<ObjectDescriptor<'b>> for MqValue<values::MQOT> {
    fn apply_param<Y, F: for<'a> FnOnce(&'a mut ObjectDescriptor<'b>) -> Y>(
        param: &mut ObjectDescriptor<'b>,
        open: F,
    ) -> (Self, Y) {
        param.Version = cmp::max(sys::MQOD_VERSION_4, param.Version);
        let open_result = open(param);
        (param.ObjectType.into(), open_result)
    }
}

impl<'b, C: Conn> MqiValue<'b, Self> for Object<C> {
    type Param<'a> = ObjectDescriptor<'a>;

    fn from_mqi<F: FnOnce(&mut Self::Param<'b>) -> ResultComp<Self>>(mqod: &mut Self::Param<'b>, open: F) -> ResultComp<Self> {
        open(mqod)
    }
}

impl<'a> MqiAttr<ObjectDescriptor<'a>> for Option<QueueManagerName> {
    fn apply_param<Y, F: FnOnce(&mut ObjectDescriptor<'a>) -> Y>(od: &mut ObjectDescriptor<'a>, open: F) -> (Self, Y) {
        let open_result = open(od);
        (
            Self::Some(QueueManagerName(od.ResolvedQMgrName.into())).filter(|queue_name| queue_name.has_value()),
            open_result,
        )
    }
}

const DEFAULT_RESOBJECTSTRING_LENGTH: sys::MQLONG = 4096;

impl<'b> MqiAttr<ObjectDescriptor<'b>> for Option<ResObjectString> {
    fn apply_param<Y, F: for<'a> FnOnce(&'a mut ObjectDescriptor<'b>) -> Y>(od: &mut ObjectDescriptor<'b>, open: F) -> (Self, Y) {
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

        let open_result = open(od);

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
