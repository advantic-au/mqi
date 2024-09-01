
use crate::{
    core::{values, ObjectHandle},
    MqStruct, MqiOption, MqiValue,
};


use crate::{
    core::{self, values::MQCO},
    Conn, MqMask,
};
use crate::sys;
use crate::ResultComp;

pub type OpenParamOption<'a, T> = (MqStruct<'a, sys::MQOD>, MqMask<T>);
pub type OpenParam<'a> = OpenParamOption<'a, values::MQOO>;

#[must_use]
#[derive(Debug)]
pub struct Object<C: Conn> {
    pub(super) handle: core::ObjectHandle,
    pub(super) connection: C,
    pub(super) close_options: MqMask<MQCO>,
}

pub trait OpenOption<'oo>: MqiOption<OpenParam<'oo>> {}
pub trait OpenValue<T>: for<'oo> MqiValue<OpenParam<'oo>, T> {}

impl<'oo, T: MqiOption<OpenParam<'oo>>> OpenOption<'oo> for T {}

impl<C: Conn> Object<C> {
    #[must_use]
    pub const fn handle(&self) -> &core::ObjectHandle {
        &self.handle
    }

    #[must_use]
    pub const fn connection(&self) -> &C {
        &self.connection
    }

    /// # Safety
    /// Consumers of the API must ensure that the `handle` is naturally associated with the `connection` and
    /// the `handle` isn't used in any other `Object`
    pub unsafe fn from_parts(connection: C, handle: ObjectHandle) -> Self {
        Self {
            handle,
            connection,
            close_options: MqMask::from(sys::MQCO_NONE),
        }
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

impl<C: Conn> Drop for Object<C> {
    fn drop(&mut self) {
        // TODO: handle close failure
        if self.handle.is_closeable() {
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