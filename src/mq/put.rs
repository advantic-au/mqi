use std::borrow::Cow;
use std::mem;

use libmqm_sys::function;

use crate::core::{ConnectionHandle, Library, MqFunctions};
use crate::headers::{fmt, TextEnc};
use crate::types::{Fmt, MessageFormat};
use crate::{sys, Conn, MqStruct, Object, ResultComp, MqiAttr};
use crate::values;
use crate::prelude::*;

use super::values::{CCSID, MQENC, MQPMO};
use super::{OpenOption, OpenParamOption};

/// A trait that provides a rendered message for the [`mqput`](`crate::core::MqFunctions::mqput`) function
#[diagnostic::on_unimplemented(message = "{Self} does not implement `PutMessae` so it can't be used as an argument for MQI put")]
pub trait PutMessage {
    type Data: ?Sized;

    fn render(&self) -> Cow<[u8]>;
    fn format(&self) -> MessageFormat;
}

pub type PutParam<'a> = (MqStruct<'static, sys::MQMD2>, MqStruct<'a, sys::MQPMO>);

impl PutMessage for str {
    type Data = Self;

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
    type Data = Self;

    fn render(&self) -> Cow<[u8]> {
        Cow::Borrowed(self.0.as_ref())
    }

    fn format(&self) -> MessageFormat {
        self.1
    }
}

impl<C: Conn> Object<C> {
    pub fn put_message(&self, put_options: impl PutOption, message: &(impl PutMessage + ?Sized)) -> ResultComp<()> {
        self.put_message_with(put_options, message)
    }

    pub fn put_message_with<R>(&self, put_options: impl PutOption, message: &(impl PutMessage + ?Sized)) -> ResultComp<R>
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
pub trait PutOption {
    fn apply_param(self, param: &mut PutParam);
}
pub trait PutAttr: for<'a> MqiAttr<PutParam<'a>, ()> {}

impl<T> PutAttr for T where T: for<'a> MqiAttr<PutParam<'a>, ()> {}

pub(super) fn put_message_with<'oo, R, L>(
    functions: &MqFunctions<L>,
    handle: ConnectionHandle,
    open_options: impl OpenOption<'oo, MQPMO>,
    put_options: impl PutOption,
    message: &(impl PutMessage + ?Sized),
) -> ResultComp<R>
where
    R: PutAttr,
    L: Library<MQ: function::Mqi>,
{
    let mut open_params = OpenParamOption {
        mqod: MqStruct::new(sys::MQOD {
            Version: sys::MQOD_VERSION_4,
            ..sys::MQOD::default()
        }),
        options: values::MQPMO::default(),
    };
    open_options.apply_param(&mut open_params);
    put(put_options, message, |(md, pmo), data| {
        pmo.Options |= open_params.options.value();
        functions.mqput1(handle, &mut open_params.mqod, Some(&mut **md), pmo, data)
    })
}

fn put<T, F>(options: impl PutOption, message: &(impl PutMessage + ?Sized), put: F) -> ResultComp<T>
where
    T: for<'a> MqiAttr<PutParam<'a>, ()>,
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
        Format: unsafe { mem::transmute::<Fmt, [i8; 8]>(fmt.into_ascii().into()) },
        ..sys::MQMD2::default()
    });
    let mqpmo = MqStruct::new(sys::MQPMO {
        Version: sys::MQPMO_VERSION_3,
        ..sys::MQPMO::default()
    });

    let mut put_param = (md, mqpmo);

    options.apply_param(&mut put_param);
    T::extract(&mut put_param, |param| put(param, &message.render())).map_completion(|(attr, ..)| attr)
}
