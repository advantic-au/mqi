use crate::{prelude::*, ResultComp, ResultCompErr};
use crate::{constants, types};

use super::{Conn, MqStruct, Object, OpenAttr, OpenOption, OpenParamOption, OpenValue};

use libmqm_default as default;

impl<C: Conn> Object<C> {
    /// Establish access and return an MQ object ([`Object`])
    pub fn open<'oo>(connection: C, open_option: &impl OpenOption<'oo, types::MQOO>) -> ResultComp<Self> {
        Self::open_as(connection, open_option)
    }

    /// Establish access and return an MQ object ([`Object`]) and type inferred [`OpenAttr`] in a tuple.
    pub fn open_with<'oo, A>(connection: C, open_option: &impl OpenOption<'oo, types::MQOO>) -> ResultComp<(Self, A)>
    where
        A: OpenAttr<Self, types::MQOO>,
    {
        Self::open_as(connection, open_option)
    }

    /// Establish access and return a type inferred value of [`OpenValue`]
    pub(super) fn open_as<'oo, R>(
        connection: C,
        open_option: &impl OpenOption<'oo, types::MQOO>,
    ) -> ResultCompErr<R, <R as OpenValue<Self>>::Error>
    where
        R: OpenValue<Self>,
    {
        let mut oo = OpenParamOption {
            mqod: MqStruct::new(default::MQOD_DEFAULT),
            options: constants::MQOO_BIND_AS_Q_DEF,
        };
        open_option.apply_param(&mut oo);
        R::open_consume(&mut oo, |OpenParamOption { mqod, options }| {
            connection
                .mq()
                .mqopen(connection.handle(), mqod, *options)
                .map_completion(|handle| Self {
                    handle,
                    connection,
                    close_options: constants::MQCO_NONE,
                })
        })
    }
}
