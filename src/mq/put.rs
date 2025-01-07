use std::borrow::Cow;

use libmqm_default as default;
use libmqm_sys::Mqi;

use crate::core::{ConnectionHandle, Library, MqFunctions};
use crate::headers::{fmt, TextEnc};
use crate::types::MessageFormat;
use crate::{sys, Conn, MqStruct, Object, ResultComp};
use crate::values;

use super::values::{CCSID, MQENC, MQPMO};
use super::{OpenOption, OpenParamOption};

/// A trait that provides a rendered message for the [`mqput`](`crate::core::MqFunctions::mqput`) function
#[diagnostic::on_unimplemented(message = "{Self} does not implement `PutMessage` so it can't be used as an argument for MQI put")]
pub trait PutMessage {
    fn render(&self) -> Cow<[u8]>;
    fn format(&self) -> MessageFormat;
}

/// A trait that provides a bag handle and message format for the [`mq_put_bag`](`crate::core::MqFunctions::mq_put_bag`) function
#[cfg(feature = "mqai")]
#[diagnostic::on_unimplemented(message = "{Self} does not implement `PutBag` so it can't be used as a bag for MQI mq_put_bag")]
pub trait PutBag {
    fn bag(&self) -> &crate::core::mqai::BagHandle;
    fn format(&self) -> MessageFormat;
}

pub type PutParam<'a> = (MqStruct<'static, sys::MQMD2>, MqStruct<'a, sys::MQPMO>);

impl PutMessage for str {
    fn render(&self) -> Cow<[u8]> {
        self.as_bytes().into()
    }

    fn format(&self) -> MessageFormat {
        MessageFormat {
            ccsid: CCSID(1208),
            encoding: MQENC(sys::MQENC_NATIVE),
            fmt: TextEnc::Ascii(fmt::MQFMT_STRING),
        }
    }
}

impl<B: AsRef<[u8]>> PutMessage for (B, MessageFormat) {
    fn render(&self) -> Cow<[u8]> {
        Cow::Borrowed(self.0.as_ref())
    }

    fn format(&self) -> MessageFormat {
        self.1
    }
}

#[cfg(feature = "mqai")]
impl<C: Conn> Object<C>
where
    C::Lib: Library<MQ: libmqm_sys::Mqai>,
{
    pub fn put_bag<'po>(&self, put_options: &impl PutOption<'po>, bag: &impl PutBag) -> ResultComp<()> {
        self.put_bag_with(put_options, bag)
    }

    pub fn put_bag_with<'po, R>(&self, put_options: &impl PutOption<'po>, bag: &impl PutBag) -> ResultComp<R>
    where
        R: PutAttr,
    {
        let MessageFormat {
            ccsid: CCSID(ccsid),
            encoding,
            fmt,
        } = bag.format();
        let md = MqStruct::new(sys::MQMD2 {
            CodedCharSetId: ccsid,
            Encoding: encoding.value(),
            Format: *fmt.into_ascii().as_ref(),
            ..default::MQMD2_DEFAULT
        });
        let mqpmo = MqStruct::new(default::MQPMO_DEFAULT);

        let mut put_param = (md, mqpmo);
        put_options.apply_param(&mut put_param);
        R::extract(&mut put_param, |(md, pmo)| {
            let connection = self.connection();
            connection
                .mq()
                .mq_put_bag(connection.handle(), self.handle(), &mut **md, &mut *pmo, bag.bag())
        })
    }
}

impl<C: Conn> Object<C> {
    pub fn put_message<'po>(&self, put_options: &impl PutOption<'po>, message: &(impl PutMessage + ?Sized)) -> ResultComp<()> {
        self.put_message_with(put_options, message)
    }

    pub fn put_message_with<'po, R>(
        &self,
        put_options: &impl PutOption<'po>,
        message: &(impl PutMessage + ?Sized),
    ) -> ResultComp<R>
    where
        R: PutAttr,
    {
        put(put_options, message, |(md, pmo), data| {
            let connection = self.connection();
            connection
                .mq()
                .mqput(connection.handle(), self.handle(), Some(&mut **md), pmo, data)
        })
    }
}

/// A trait that manipulates the parameters to the [`mqput`](`crate::core::MqFunctions::mqput`) function
#[diagnostic::on_unimplemented(message = "{Self} does not implement `PutOption` so it can't be used as an argument for MQI put")]
pub trait PutOption<'po> {
    fn apply_param(&self, param: &mut PutParam<'po>);
}

pub trait PutAttr {
    fn extract<'p, F>(param: &mut PutParam<'p>, mqi: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut PutParam<'p>) -> ResultComp<()>,
        Self: Sized;
}

pub(super) fn put_message_with<'po, 'oo, R, L>(
    functions: &MqFunctions<L>,
    handle: ConnectionHandle,
    open_options: &impl OpenOption<'oo, MQPMO>,
    put_options: &impl PutOption<'po>,
    message: &(impl PutMessage + ?Sized),
) -> ResultComp<R>
where
    R: PutAttr,
    L: Library<MQ: Mqi>,
{
    let mut open_params = OpenParamOption {
        mqod: MqStruct::new(default::MQOD_DEFAULT),
        options: values::MQPMO::default(),
    };
    open_options.apply_param(&mut open_params);
    put(put_options, message, |(md, pmo), data| {
        pmo.Options |= open_params.options.value();
        functions.mqput1(handle, &mut open_params.mqod, Some(&mut **md), pmo, data)
    })
}

fn put<'po, T, F>(options: &impl PutOption<'po>, message: &(impl PutMessage + ?Sized), put: F) -> ResultComp<T>
where
    T: PutAttr,
    F: FnOnce(&mut PutParam, &[u8]) -> ResultComp<()>,
{
    let MessageFormat {
        ccsid: CCSID(ccsid),
        encoding,
        fmt,
    } = message.format();
    let md = MqStruct::new(sys::MQMD2 {
        CodedCharSetId: ccsid,
        Encoding: encoding.value(),
        Format: *fmt.into_ascii().as_ref(),
        ..default::MQMD2_DEFAULT
    });
    let mqpmo = MqStruct::new(default::MQPMO_DEFAULT);

    let mut put_param = (md, mqpmo);

    options.apply_param(&mut put_param);
    T::extract(&mut put_param, |param| put(param, &message.render()))
}
