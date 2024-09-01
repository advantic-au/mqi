use std::borrow::Cow;
use std::mem;

use libmqm_sys::function;

use crate::headers::{fmt, TextEnc};
use crate::types::{Fmt, MessageFormat};
use crate::{sys, Conn, Properties, MqMask, MqStruct, Object, QueueManagerShare, ResultComp, MqiAttr, MqiOption, ResultCompErrExt};
use crate::core::{self, values};

use super::OpenParamOption;

pub trait PutMessage {
    type Data: ?Sized;

    fn render(&self) -> Cow<[u8]>;
    fn format(&self) -> MessageFormat;
}

pub type PutParam<'a> = (MqStruct<'static, sys::MQMD2>, MqStruct<'a, sys::MQPMO>);

#[derive(Debug)]
pub enum PropertyAction<'handle, C: Conn> {
    Reply(&'handle Properties<C>, &'handle mut Properties<C>),
    Forward(&'handle Properties<C>, &'handle mut Properties<C>),
    Report(&'handle Properties<C>, &'handle mut Properties<C>),
}

impl PutMessage for str {
    type Data = Self;

    fn render(&self) -> Cow<[u8]> {
        self.as_bytes().into()
    }

    fn format(&self) -> MessageFormat {
        MessageFormat {
            ccsid: 1208,
            encoding: MqMask::from(sys::MQENC_NATIVE),
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
    pub fn put_message<R>(&self, put_options: impl PutOption, message: &(impl PutMessage + ?Sized)) -> ResultComp<R>
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

pub trait OpenPutOption<'a>: MqiOption<OpenParamOption<'a, values::MQPMO>> {}
pub trait PutOption: for<'a> MqiOption<PutParam<'a>> {}
pub trait PutAttr: for<'a> MqiAttr<PutParam<'a>, ()> {}

impl<'a, T: MqiOption<OpenParamOption<'a, values::MQPMO>>> OpenPutOption<'a> for T {}
impl<T: for<'a> MqiOption<PutParam<'a>>> PutOption for T {}
impl<T> PutAttr for T where T: for<'a> MqiAttr<PutParam<'a>, ()> {}

impl<L: core::Library<MQ: function::MQI>, H> QueueManagerShare<'_, L, H> {
    pub fn put_message<'oo, R>(
        &self,
        open_options: impl OpenPutOption<'oo>,
        put_options: impl PutOption,
        message: &(impl PutMessage + ?Sized),
    ) -> ResultComp<R>
    where
        R: PutAttr,
    {
        let mut mqod = (
            MqStruct::new(sys::MQOD {
                Version: sys::MQOD_VERSION_4,
                ..sys::MQOD::default()
            }),
            MqMask::default(),
        );
        open_options.apply_param(&mut mqod);
        put(put_options, message, |(md, pmo), data| {
            pmo.Options |= mqod.1.value();
            self.mq().mqput1(self.handle(), &mut mqod.0, Some(&mut **md), pmo, data)
        })
    }
}

fn put<T, F>(options: impl for<'a> MqiOption<PutParam<'a>>, message: &(impl PutMessage + ?Sized), put: F) -> ResultComp<T>
where
    T: for<'a> MqiAttr<PutParam<'a>, ()>,
    F: FnOnce(&mut PutParam, &[u8]) -> ResultComp<()>,
{
    let MessageFormat {
        ccsid,
        encoding,
        fmt: format,
    } = message.format();
    let md = MqStruct::new(sys::MQMD2 {
        CodedCharSetId: ccsid,
        Encoding: encoding.value(),
        Format: unsafe { mem::transmute::<Fmt, [i8; 8]>(format.into_ascii().into()) },
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