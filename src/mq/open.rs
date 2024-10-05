use crate::{prelude::*, values, sys, MqiValue, ResultComp, ResultCompErr};

use super::{Conn, MqStruct, Object, OpenAttr, OpenOption, OpenParam, OpenParamOption, OpenValue};

impl<C: Conn> Object<C> {
    pub fn open<'oo>(connection: C, open_option: impl OpenOption<'oo, values::MQOO>) -> ResultComp<Self> {
        Self::open_as(connection, open_option)
    }

    pub fn open_with<'oo, A>(connection: C, open_option: impl OpenOption<'oo, values::MQOO>) -> ResultComp<(Self, A)>
    where
        A: OpenAttr<Self>,
    {
        Self::open_as(connection, open_option)
    }

    pub(super) fn open_as<'oo, R>(
        connection: C,
        open_option: impl OpenOption<'oo, values::MQOO>,
    ) -> ResultCompErr<R, <R as MqiValue<OpenParam<'oo>, Self>>::Error>
    where
        R: OpenValue<Self>,
    {
        let mut oo = OpenParamOption {
            mqod: MqStruct::new(sys::MQOD {
                Version: sys::MQOD_VERSION_4,
                ..sys::MQOD::default()
            }),
            options: values::MQOO(sys::MQOO_BIND_AS_Q_DEF),
        };
        open_option.apply_param(&mut oo);
        R::consume(&mut oo, |OpenParamOption { mqod, options }| {
            connection
                .mq()
                .mqopen(connection.handle(), mqod, *options)
                .map_completion(|handle| Self {
                    handle,
                    connection,
                    close_options: values::MQCO(sys::MQCO_NONE),
                })
        })
    }
}
