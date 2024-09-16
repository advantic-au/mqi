use crate::{prelude::*, values, sys, MqiValue, ResultComp, ResultCompErr};

use super::{Conn, MqStruct, Object, OpenAttr, OpenOption, OpenParam, OpenValue, QueueManager};

impl<C: Conn + Clone> QueueManager<C> {
    pub fn open<'oo>(&self, open_option: impl OpenOption<'oo>) -> ResultComp<Object<C>> {
        self.open_as(open_option)
    }

    pub fn open_with<'oo, A>(&self, open_option: impl OpenOption<'oo>) -> ResultComp<(Object<C>, A)>
    where
        A: OpenAttr<Object<C>>,
    {
        self.open_as(open_option)
    }

    pub(super) fn open_as<'oo, R>(
        &self,
        open_option: impl OpenOption<'oo>,
    ) -> ResultCompErr<R, <R as MqiValue<OpenParam<'oo>, Object<C>>>::Error>
    where
        R: OpenValue<Object<C>>,
    {
        let mut oo = (
            MqStruct::new(sys::MQOD {
                Version: sys::MQOD_VERSION_4,
                ..sys::MQOD::default()
            }),
            values::MQOO(sys::MQOO_BIND_AS_Q_DEF),
        );
        open_option.apply_param(&mut oo);
        R::consume(&mut oo, |(od, options)| {
            self.0
                .mq()
                .mqopen(self.0.handle(), od, *options)
                .map_completion(|handle| Object {
                    handle,
                    connection: self.0.clone(),
                    close_options: values::MQCO(sys::MQCO_NONE),
                })
        })
    }
}
