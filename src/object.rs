use std::{rc::Rc, sync::Arc};

use crate::{Connection, connection::AsConnection, constants, handle::ObjectHandle, result::ResultComp, types::MQCO};

/// Has the ability to return a reference to a [Object](crate::Object)
pub trait AsObject {
    type AsConnection: AsConnection;

    /// Returns a reference to the contained [Object](crate::Object)
    fn as_object(&self) -> &crate::Object<Self::AsConnection>;
}

/// An object refers to a managed entity within IBM MQ, such as a queue, topic, channel, or queue manager
///
/// Dropping an open object attempts to close it, but cannot return an MQ
/// warning or error. Call [`close`](Self::close) when the close outcome needs
/// to be observed.
#[derive(Debug)]
#[must_use]
pub struct Object<C: AsConnection> {
    /// Handle of the object
    pub(crate) handle: ObjectHandle,
    /// Connection associated with object
    pub(crate) connection: C,
    /// Close options applied on drop of the object
    pub(crate) drop_close_options: MQCO,
}

impl<C: AsConnection> Object<C> {
    /// Return the MQ handle for this open object.
    #[must_use]
    pub const fn handle(&self) -> &ObjectHandle {
        &self.handle
    }

    /// Return the connection associated with this object.
    #[must_use]
    pub const fn connection(&self) -> &C {
        &self.connection
    }

    /// Construct an object from an existing connection and MQ object handle.
    ///
    /// The object closes the handle when dropped, using `MQCO_NONE`.
    pub const fn from_parts(connection: C, handle: ObjectHandle) -> Self {
        Self {
            handle,
            connection,
            drop_close_options: constants::MQCO_NONE,
        }
    }

    /// Set the close options used when this object is dropped or explicitly closed.
    pub const fn close_options(&mut self, options: MQCO) {
        self.drop_close_options = options;
    }

    /// Close the object and return the MQ completion status.
    ///
    /// Use this instead of relying on `Drop` when close warnings or errors need
    /// to be handled by the application. This performs one close attempt;
    /// `Drop` will not retry if the attempt fails.
    ///
    /// # Errors
    /// Returns an error when MQCLOSE fails to close the object.
    pub fn close(self) -> ResultComp<()> {
        let mut s = self;
        let close_options = s.drop_close_options;
        let mut obj_handle = std::mem::take(&mut s.handle);
        let Connection { handle, mq, .. } = s.connection.as_connection();
        mq.mqclose(*handle, &mut obj_handle, close_options)
    }
}

impl<C: AsConnection> Drop for Object<C> {
    fn drop(&mut self) {
        let Connection { handle, mq, .. } = self.connection.as_connection();

        if self.handle.is_closeable() {
            // Drop cannot report MQ warnings or errors; use close() to observe them.
            let _ = mq.mqclose(*handle, &mut self.handle, self.drop_close_options);
        }
    }
}

impl<C: AsConnection> AsObject for Object<C> {
    type AsConnection = C;

    fn as_object(&self) -> &crate::Object<Self::AsConnection> {
        self
    }
}

impl<C: AsConnection> AsObject for &Object<C> {
    type AsConnection = C;

    fn as_object(&self) -> &crate::Object<Self::AsConnection> {
        self
    }
}

impl<C: AsConnection> AsObject for Rc<Object<C>> {
    type AsConnection = C;

    fn as_object(&self) -> &crate::Object<Self::AsConnection> {
        self
    }
}

impl<C: AsConnection> AsObject for Arc<Object<C>> {
    type AsConnection = C;

    fn as_object(&self) -> &crate::Object<Self::AsConnection> {
        self
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

    #[cfg(feature = "mock")]
    #[test]
    fn close_returns_mq_warning_without_retrying() {
        let connection = crate::test::mock::connect_ok(|mock_library| {
            mock_library
                .expect_MQCLOSE()
                .withf(|_, &hobj, _, _, _| hobj == 7)
                .returning(|_, _, _, cc, rc| {
                    crate::test::mock::mqi_outcome(cc, rc, constants::MQCC_WARNING, constants::MQRC_TRUNCATED_MSG_ACCEPTED);
                })
                .once();
        });

        let object = Object::from_parts(&connection, 7.into());
        let result = object.close().expect("MQCLOSE warning should be returned as a completion");

        assert_eq!(result.warning(), Some((constants::MQRC_TRUNCATED_MSG_ACCEPTED, "MQCLOSE")));
    }

    #[cfg(feature = "mock")]
    #[test]
    fn close_returns_mq_error_without_retrying() {
        let connection = crate::test::mock::connect_ok(|mock_library| {
            mock_library
                .expect_MQCLOSE()
                .withf(|_, &hobj, _, _, _| hobj == 7)
                .returning(|_, _, _, cc, rc| {
                    crate::test::mock::mqi_outcome(cc, rc, constants::MQCC_FAILED, constants::MQRC_NONE);
                })
                .once();
        });

        let object = Object::from_parts(&connection, 7.into());
        let error = object.close().expect_err("MQCLOSE failure should be returned");

        assert_eq!(error.0, constants::MQCC_FAILED);
        assert_eq!(error.1, "MQCLOSE");
        assert_eq!(error.2, constants::MQRC_NONE);
    }
}
