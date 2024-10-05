use crate::{values, core::ObjectHandle, MqStruct, MqiAttr, MqiValue};

use crate::{
    core::{self, values::MQCO},
    Conn,
};
use crate::sys;
use crate::ResultComp;

pub struct OpenParamOption<'a, T> {
    pub mqod: MqStruct<'a, sys::MQOD>,
    pub options: T,
}

pub type OpenParam<'a> = OpenParamOption<'a, values::MQOO>;

#[must_use]
#[derive(Debug)]
pub struct Object<C: Conn> {
    pub(super) handle: core::ObjectHandle,
    pub(super) connection: C,
    pub(super) close_options: MQCO,
}

/// A trait that manipulates the parameters to the [`mqopen`](`crate::core::MqFunctions::mqopen`) function
#[diagnostic::on_unimplemented(
    message = "{Self} does not implement `OpenOption` so it can't be used as an argument for MQI open"
)]
pub trait OpenOption<'oo, T> {
    fn apply_param(self, param: &mut OpenParamOption<'oo, T>);
}
pub trait OpenValue<S>: for<'oo> MqiValue<OpenParam<'oo>, S> {}
pub trait OpenAttr<S>: for<'oo> MqiAttr<OpenParam<'oo>, S> {}

impl<S, T: for<'oo> MqiAttr<OpenParam<'oo>, S>> OpenAttr<S> for T {}

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
    pub const unsafe fn from_parts(connection: C, handle: ObjectHandle) -> Self {
        Self {
            handle,
            connection,
            close_options: values::MQCO(sys::MQCO_NONE),
        }
    }

    pub fn close_options(&mut self, options: MQCO) {
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
    use crate::values::MQCO;
    use crate::sys;

    #[test]
    fn close_option() {
        assert_eq!(MQCO(sys::MQCO_DELETE | 0xFF00).to_string(), "MQCO_DELETE|0xFF00");
        assert_eq!(
            MQCO(sys::MQCO_DELETE | sys::MQCO_QUIESCE).to_string(),
            "MQCO_DELETE|MQCO_QUIESCE"
        );
        assert_eq!(MQCO(sys::MQCO_DELETE).to_string(), "MQCO_DELETE");
        assert_eq!(MQCO(0).to_string(), "MQCO_NONE");
        assert_eq!(MQCO(0xFF00).to_string(), "0xFF00");

        let (list_iter, _) = MQCO(sys::MQCO_DELETE).masked_list();
        let list = list_iter.collect::<Vec<_>>();
        assert_eq!(list, &[(1, "MQCO_DELETE")]);

        let (list_iter, _) = MQCO(sys::MQCO_NONE).masked_list();
        let list = list_iter.collect::<Vec<_>>();
        assert_eq!(list, &[]);

        let (list_iter, _) = MQCO(sys::MQCO_DELETE | sys::MQCO_QUIESCE).masked_list();
        let list = list_iter.collect::<Vec<_>>();
        assert_eq!(
            list,
            &[(sys::MQCO_DELETE, "MQCO_DELETE"), (sys::MQCO_QUIESCE, "MQCO_QUIESCE")]
        );

        // assert_eq!(format!("{oo:?}"), "");
    }
}
