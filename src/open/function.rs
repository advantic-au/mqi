use libmqm_default as default;

use super::option;
use crate::{
    Object,
    connection::AsConnection,
    constants,
    prelude::*,
    result::{ResultComp, ResultCompErr},
    structs, types,
};

impl<C: AsConnection> Object<C> {
    /// Open a queue, topic, or other MQ object.
    ///
    /// The option argument supplies the object name and MQOO flags, for
    /// example `&(QueueName(mqstr!("APP.QUEUE")), MQOO_INPUT_AS_Q_DEF)`.
    /// Pass the connection by value; use [`crate::Connection::connection_ref`] when
    /// the connection must remain available for other operations. The returned
    /// object retains its connection and closes on drop. Use
    /// [`Object::close`] to observe the close completion status.
    ///
    /// For an object with a resolved name, use [`open_resolved`](Self::open_resolved).
    /// To extract another open attribute, use [`open_with`](Self::open_with).
    ///
    /// # Errors
    /// Returns an error when MQOPEN cannot open the requested object.
    ///
    /// # Panics
    /// Panics if `open_option` configures an MQOD version unsupported by this build.
    pub fn open<'oo>(connection: C, open_option: &impl option::OpenOption<'oo, types::MQOO>) -> ResultComp<Self> {
        Self::open_as(connection, open_option)
    }

    /// Open an object and return the resolved queue and queue-manager names.
    ///
    /// MQ may resolve an alias, remote queue, or queue-manager name while
    /// opening. The optional names in the result are the names returned by MQ;
    /// they are `None` when MQ does not provide the corresponding value.
    /// Otherwise, this behaves like [`open`](Self::open).
    ///
    /// # Errors
    /// Returns an error when MQOPEN cannot open the requested object.
    ///
    /// # Panics
    /// Panics if `open_option` configures an MQOD version unsupported by this build.
    pub fn open_resolved<'oo>(
        connection: C,
        open_option: &impl option::OpenOption<'oo, types::MQOO>,
    ) -> ResultComp<(Self, Option<types::QueueName>, Option<types::QueueManagerName>)> {
        Self::open_as(connection, open_option).map_completion(|(obj, (qmgr, q))| (obj, qmgr, q))
    }

    /// Open an object and extract an additional MQ open attribute.
    ///
    /// The attribute type is selected through [`OpenAttr`](option::OpenAttr)
    /// and inferred from the expected result type. Use [`open`](Self::open)
    /// when the object handle alone is sufficient.
    ///
    /// # Errors
    /// Returns an error when MQOPEN cannot open the requested object or the selected
    /// [`OpenAttr`](option::OpenAttr) cannot extract its result.
    ///
    /// # Panics
    /// Panics if `open_option` configures an MQOD version unsupported by this build.
    pub fn open_with<'oo, A>(connection: C, open_option: &impl option::OpenOption<'oo, types::MQOO>) -> ResultComp<(Self, A)>
    where
        A: option::OpenAttr<Self, types::MQOO>,
    {
        Self::open_as(connection, open_option)
    }

    /// Establish access and return a type inferred value of [`OpenValue`]
    pub(super) fn open_as<'oo, R>(
        connection: C,
        open_option: &impl option::OpenOption<'oo, types::MQOO>,
    ) -> ResultCompErr<R, <R as option::OpenValue<Self>>::Error>
    where
        R: option::OpenValue<Self>,
    {
        let mut oo = option::OpenParamOption {
            mqod: structs::MQOD::new(default::MQOD_DEFAULT),
            options: constants::MQOO_BIND_AS_Q_DEF,
        };
        open_option.apply_param(&mut oo);
        assert!(oo.mqod.Version <= libmqm_sys::MQOD_CURRENT_VERSION);

        // SAFETY: Implementors of option::OpenOption must ensure MQOD structure is populated correctly for mqopen
        R::open_consume(&mut oo, |option::OpenParamOption { mqod, options }| unsafe {
            let as_connection = connection.as_connection();
            as_connection
                .mq
                .mqopen(as_connection.handle, mqod, *options)
                .map_completion(|handle| Self {
                    handle,
                    connection,
                    drop_close_options: constants::MQCO_NONE,
                })
        })
    }
}
