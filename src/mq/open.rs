use crate::{prelude::*, values, sys, MqiValue, ResultComp, ResultCompErr};

use super::{IntoConnection, Conn, MqStruct, Object, OpenAttr, OpenOption, OpenParam, OpenValue};

impl<C: Conn> Object<C> {
    pub fn open<'oo>(connection: impl IntoConnection<C>, open_option: impl OpenOption<'oo>) -> ResultComp<Self> {
        Self::open_as(connection, open_option)
    }

    pub fn open_with<'oo, A>(connection: impl IntoConnection<C>, open_option: impl OpenOption<'oo>) -> ResultComp<(Self, A)>
    where
        A: OpenAttr<Self>,
    {
        Self::open_as(connection, open_option)
    }

    pub(super) fn open_as<'oo, R>(
        connection: impl IntoConnection<C>,
        open_option: impl OpenOption<'oo>,
    ) -> ResultCompErr<R, <R as MqiValue<OpenParam<'oo>, Self>>::Error>
    where
        R: OpenValue<Self>,
    {
        let mut oo = (
            MqStruct::new(sys::MQOD {
                Version: sys::MQOD_VERSION_4,
                ..sys::MQOD::default()
            }),
            values::MQOO(sys::MQOO_BIND_AS_Q_DEF),
        );
        open_option.apply_param(&mut oo);
        let conn = connection.into_connection();
        R::consume(&mut oo, |(od, options)| {
            conn.mq().mqopen(conn.handle(), od, *options).map_completion(|handle| Self {
                handle,
                connection: conn,
                close_options: values::MQCO(sys::MQCO_NONE),
            })
        })
    }
}
