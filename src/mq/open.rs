use crate::{core::values, sys, MqMask, MqiValue, ResultCompErr, ResultCompErrExt as _};

use super::{Conn, MqStruct, Object, OpenOption, OpenParam, OpenValue};

impl<C: Conn> Object<C> {
    pub fn open<'oo, R>(
        connection: C,
        open_option: impl OpenOption<'oo>,
        options: MqMask<values::MQOO>,
    ) -> ResultCompErr<R, <R as MqiValue<OpenParam<'oo>, Self>>::Error>
    where
        R: OpenValue<Self>,
    {
        let mut oo = (
            MqStruct::new(sys::MQOD {
                Version: sys::MQOD_VERSION_4,
                ..sys::MQOD::default()
            }),
            options,
        );
        open_option.apply_param(&mut oo);
        R::consume(&mut oo, |(od, options)| {
            connection
                .mq()
                .mqopen(connection.handle(), od, *options)
                .map_completion(|handle| Self {
                    handle,
                    connection,
                    close_options: MqMask::from(sys::MQCO_NONE),
                })
        })
    }
}
