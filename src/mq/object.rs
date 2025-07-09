use crate::{ObjectHandle, ResultComp, constants, option::Conn, types::MQCO};

#[must_use]
#[derive(Debug)]
pub struct Object<C: Conn> {
    pub(super) handle: ObjectHandle,
    pub(super) connection: C,
    pub(super) close_options: MQCO,
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
    use libmqm_sys as mq;

    use super::*;

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
        assert_eq!(list, &[(mq::MQCO_DELETE, "MQCO_DELETE"), (mq::MQCO_QUIESCE, "MQCO_QUIESCE")]);

        // assert_eq!(format!("{oo:?}"), "");
    }
}
