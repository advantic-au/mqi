use crate::{
    sys,
    types::{QueueManagerName, QueueName},
    Conn, ResultComp, MqiAttr, MqiOption, MqiValue,
};

use super::{Object, ObjectDescriptor};

impl<'a> MqiOption<'a, ObjectDescriptor<'a>> for QueueName {
    fn apply_param(&self, param: &mut ObjectDescriptor<'_>) {
        param.ObjectName = self.0.into();
        param.ObjectType = sys::MQOT_Q;
    }
}

impl<'a> MqiOption<'a, ObjectDescriptor<'a>> for QueueManagerName {
    fn apply_param(&self, param: &mut ObjectDescriptor<'a>) {
        param.ObjectQMgrName = self.0.into();
        param.ObjectType = sys::MQOT_Q_MGR;
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

impl<C: Conn> MqiValue<Self> for Object<C> {
    type Param<'a> = ObjectDescriptor<'a>;

    fn from_mqi<F: FnOnce(&mut Self::Param<'_>) -> ResultComp<Self>>(mqod: &mut Self::Param<'_>, open: F) -> ResultComp<Self> {
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
