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
    /// Establish access and return an MQ object ([`Object`])
    pub fn open<'oo>(connection: C, open_option: &impl option::OpenOption<'oo, types::MQOO>) -> ResultComp<Self> {
        Self::open_as(connection, open_option)
    }

    /// Establish access and return an MQ object and the name of the resolved object
    pub fn open_resolved<'oo>(
        connection: C,
        open_option: &impl option::OpenOption<'oo, types::MQOO>,
    ) -> ResultComp<(Self, Option<types::QueueName>, Option<types::QueueManagerName>)> {
        Self::open_as(connection, open_option).map_completion(|(obj, (qmgr, q))| (obj, qmgr, q))
    }

    /// Establish access and return an MQ object ([`Object`]) and type inferred [`OpenAttr`](option::OpenAttr) in a tuple
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
