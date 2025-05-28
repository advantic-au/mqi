use crate::{ObjectHandle, Conn};
use crate::{structs, constants, Error, ResultComp, ResultCompErr};
use crate::types::{MQCO, MQOO};

pub struct OpenParamOption<'a, T> {
    pub mqod: structs::MQOD<'a>,
    pub options: T,
}

pub type OpenParam<'a> = OpenParamOption<'a, MQOO>;

#[must_use]
#[derive(Debug)]
pub struct Object<C: Conn> {
    pub(super) handle: ObjectHandle,
    pub(super) connection: C,
    pub(super) close_options: MQCO,
}

/// A trait that manipulates the parameters to the [`mqopen`](`crate::MqFunctions::mqopen`) function
#[diagnostic::on_unimplemented(
    message = "{Self} does not implement `OpenOption` so it can't be used as an argument for MQI open"
)]
/// # Safety
/// This trait can directly manipulate the [`MQOD`](structs::MQOD) structure which is used by [`MQOPEN`](libmqm_sys::Mqi::MQOPEN).
/// Incorrect values in the [`MQOD`](structs::MQOD) can lead to undefined behaviour.
/// Implementations of the trait must ensure that pointers and offsets contained in the structure point to active data.
pub unsafe trait OpenOption<'oo, T> {
    fn apply_param(&self, param: &mut OpenParamOption<'oo, T>);
}

/// # Safety
/// This trait can directly manipulate the [`MQOD`](structs::MQOD) structure which is used by [`MQOPEN`](libmqm_sys::Mqi::MQOPEN).
/// Incorrect values in the [`MQOD`](structs::MQOD) can lead to undefined behaviour.
/// Implementations of the trait must ensure that pointers and offsets contained in the structure point to active data.
pub unsafe trait OpenValue<S> {
    type Error: From<Error> + std::fmt::Debug;

    fn open_consume<'oo, F>(param: &mut OpenParam<'oo>, mqi: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut OpenParam<'oo>) -> ResultComp<S>,
        Self: std::marker::Sized;
}

/// # Safety
/// This trait can directly manipulate the [`MQOD`](structs::MQOD) structure which is used by [`MQOPEN`](libmqm_sys::Mqi::MQOPEN).
/// Incorrect values in the [`MQOD`](structs::MQOD) can lead to undefined behaviour.
/// Implementations of the trait must ensure that pointers and offsets contained in the structure point to active data.
pub unsafe trait OpenAttr<S, O> {
    fn open_extract<'a, F>(param: &mut OpenParamOption<'a, O>, mqi: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut OpenParamOption<'a, O>) -> ResultComp<S>,
        Self: Sized;
}

impl<C: Conn> Object<C> {
    #[must_use]
    pub const fn handle(&self) -> &ObjectHandle {
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
            close_options: constants::MQCO_NONE,
        }
    }

    pub const fn close_options(&mut self, options: MQCO) {
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use libmqm_sys::lib as sys;

    #[test]
    fn close_option() {
        assert_eq!((constants::MQCO_DELETE | MQCO(0xFF00)).to_string(), "MQCO_DELETE|0xFF00");
        assert_eq!(
            (constants::MQCO_DELETE | constants::MQCO_QUIESCE).to_string(),
            "MQCO_DELETE|MQCO_QUIESCE"
        );
        assert_eq!(constants::MQCO_DELETE.to_string(), "MQCO_DELETE");
        assert_eq!(MQCO(0).to_string(), "MQCO_NONE");
        assert_eq!(MQCO(0xFF00).to_string(), "0xFF00");

        let (list_iter, _) = constants::MQCO_DELETE.bitflags_list();
        let list = list_iter.collect::<Vec<_>>();
        assert_eq!(list, &[(1, "MQCO_DELETE")]);

        let (list_iter, _) = constants::MQCO_NONE.bitflags_list();
        let list = list_iter.collect::<Vec<_>>();
        assert_eq!(list, &[]);

        let (list_iter, _) = (constants::MQCO_DELETE | constants::MQCO_QUIESCE).bitflags_list();
        let list = list_iter.collect::<Vec<_>>();
        assert_eq!(
            list,
            &[(sys::MQCO_DELETE, "MQCO_DELETE"), (sys::MQCO_QUIESCE, "MQCO_QUIESCE")]
        );

        // assert_eq!(format!("{oo:?}"), "");
    }
}
