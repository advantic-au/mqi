use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::common::ResultCompErrExt as _;

use libmqm_sys::function;

use crate::{
    core::{
        self,
        values::{MQCO, MQOO},
        ConnectionHandle, Library, MQFunctions,
    },
    Conn, MqMask,
};
use crate::{sys, MqStr, QMName, QName, StructBuilder};
use crate::ResultComp;
use crate::QueueManagerShare;

#[must_use]
pub struct Object<C: Conn> {
    handle: core::ObjectHandle,
    connection: C,
    close_options: MqMask<MQCO>,
    name: QName,               // When a model queue is used
    qmgr_name: Option<QMName>, // When a model queue is used
    resolved_name: Option<QName>,
    resolved_qmgr_name: Option<QMName>,
}

impl<L: Library<MQ: function::MQI>, H> Conn for Arc<QueueManagerShare<'_, L, H>> {
    type Lib = L;

    fn mq(&self) -> &MQFunctions<Self::Lib> {
        self.deref().mq()
    }

    fn handle(&self) -> &ConnectionHandle {
        self.deref().handle()
    }    
}

impl<L: Library<MQ: function::MQI>, H> Conn for QueueManagerShare<'_, L, H> {
    type Lib = L;

    fn mq(&self) -> &MQFunctions<Self::Lib> {
        Self::mq(self)
    }

    fn handle(&self) -> &ConnectionHandle {
        Self::handle(self)
    }
}

impl<L: Library<MQ: function::MQI>, H> Conn for &QueueManagerShare<'_, L, H> {
    type Lib = L;

    fn mq(&self) -> &MQFunctions<Self::Lib> {
        QueueManagerShare::<L, H>::mq(self)
    }

    fn handle(&self) -> &ConnectionHandle {
        QueueManagerShare::<L, H>::handle(self)
    }
}

impl<C: Conn> Object<C> {
    #[must_use]
    pub const fn handle(&self) -> &core::ObjectHandle {
        &self.handle
    }

    #[must_use]
    pub const fn connection(&self) -> &C {
        &self.connection
    }

    pub fn open(connection: C, mqod: &impl StructBuilder<sys::MQOD>, options: MqMask<MQOO>) -> ResultComp<Self> {
        let mut mqod_build = mqod.build();
        let result = connection.mq().mqopen(connection.handle(), &mut mqod_build, options);
        result.map_completion(|handle| Self {
            handle,
            connection,
            close_options: MqMask::from(sys::MQCO_NONE),
            name: mqod_build.ObjectName.into(),
            qmgr_name: Some(mqod_build.ObjectQMgrName.into()).filter(MqStr::has_value),
            resolved_name: Some(mqod_build.ResolvedQName.into()).filter(MqStr::has_value),
            resolved_qmgr_name: Some(mqod_build.ResolvedQMgrName.into()).filter(MqStr::has_value),
        })
    }

    pub fn close_options(&mut self, options: MqMask<MQCO>) {
        self.close_options = options;
    }

    pub fn close(self) -> ResultComp<()> {
        let mut s = self;
        s.connection
            .mq()
            .mqclose(s.connection.handle(), &mut s.handle, s.close_options)
    }
}

impl<C: Conn> Deref for Object<C> {
    type Target = core::ObjectHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<C: Conn> DerefMut for Object<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

impl<C: Conn> Drop for Object<C> {
    fn drop(&mut self) {
        // TODO: handle close failure
        if self.is_closeable() {
            let _ = self
                .connection
                .mq()
                .mqclose(self.connection.handle(), &mut self.handle, self.close_options);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::values::MQCO;
    use crate::sys;
    use crate::MqMask;

    #[test]
    fn close_option() {
        assert_eq!(
            MqMask::<MQCO>::from(sys::MQCO_DELETE | 0xFF00).to_string(),
            "MQCO_DELETE|0xFF00"
        );
        assert_eq!(
            MqMask::<MQCO>::from(sys::MQCO_DELETE | sys::MQCO_QUIESCE).to_string(),
            "MQCO_DELETE|MQCO_QUIESCE"
        );
        assert_eq!(MqMask::<MQCO>::from(sys::MQCO_DELETE).to_string(), "MQCO_DELETE");
        assert_eq!(MqMask::<MQCO>::from(0).to_string(), "MQCO_NONE");
        assert_eq!(MqMask::<MQCO>::from(0xFF00).to_string(), "0xFF00");

        let (list_iter, _) = MqMask::<MQCO>::from(sys::MQCO_DELETE).masked_list();
        let list = list_iter.collect::<Vec<_>>();
        assert_eq!(list, &[(1, "MQCO_DELETE")]);

        let (list_iter, _) = MqMask::<MQCO>::from(sys::MQCO_NONE).masked_list();
        let list = list_iter.collect::<Vec<_>>();
        assert_eq!(list, &[]);

        let (list_iter, _) = MqMask::<MQCO>::from(sys::MQCO_DELETE | sys::MQCO_QUIESCE).masked_list();
        let list = list_iter.collect::<Vec<_>>();
        assert_eq!(
            list,
            &[(sys::MQCO_DELETE, "MQCO_DELETE"), (sys::MQCO_QUIESCE, "MQCO_QUIESCE")]
        );

        // assert_eq!(format!("{oo:?}"), "");
    }
}
