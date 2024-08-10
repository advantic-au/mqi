use std::borrow::Cow;
use std::mem;

use libmqm_sys::function;

use crate::headers::{fmt, TextEnc};
use crate::types::{Fmt, MessageFormat};
use crate::{sys, Conn, Message, MqMask, MqStruct, Object, QueueManagerShare, ResultComp, ResultCompErrExt, MqiAttr, MqiOption};
use crate::core;

use super::ObjectDescriptor;

pub trait PutMessage {
    type Data: ?Sized;

    fn render(&self) -> Cow<[u8]>;
    fn format(&self) -> MessageFormat;
}

pub type PutParam<'a> = (MqStruct<'static, sys::MQMD2>, MqStruct<'a, sys::MQPMO>);

#[derive(Debug)]
pub enum Properties<'handle, C: Conn> {
    Reply(&'handle Message<C>, &'handle mut Message<C>),
    Forward(&'handle Message<C>, &'handle mut Message<C>),
    Report(&'handle Message<C>, &'handle mut Message<C>),
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
            format: TextEnc::Ascii(fmt::MQFMT_STRING),
        }
    }
}

impl PutMessage for [u8] {
    type Data = Self;

    fn render(&self) -> Cow<[u8]> {
        self.into()
    }

    fn format(&self) -> MessageFormat {
        MessageFormat {
            ccsid: 1208,
            encoding: MqMask::from(sys::MQENC_NATIVE),
            format: TextEnc::Ascii(fmt::MQFMT_NONE),
        }
    }
}

impl<C: Conn> Object<C> {
    pub fn put_message<T: for<'a> MqiAttr<PutParam<'a>>>(
        &self,
        options: &impl for<'a> MqiOption<'a, PutParam<'a>>,
        message: &(impl PutMessage + ?Sized),
    ) -> ResultComp<T> {
        put(options, message, |(md, pmo), data| {
            let connection = self.connection();
            connection
                .mq()
                .mqput(connection.handle(), self.handle(), Some(&mut **md), pmo, data)
        })
    }
}

impl<L: core::Library<MQ: function::MQI>, H> QueueManagerShare<'_, L, H> {
    pub fn put_message<T: for<'b> MqiAttr<PutParam<'b>>>(
        &self,
        od: &impl for<'a> MqiOption<'a, ObjectDescriptor<'a>>,
        options: &impl for<'a> MqiOption<'a, PutParam<'a>>,
        message: &(impl PutMessage + ?Sized),
    ) -> ResultComp<T> {
        let mut mqod = MqStruct::new(sys::MQOD {
            Version: sys::MQOD_VERSION_4,
            ..sys::MQOD::default()
        });
        od.apply_param(&mut mqod);
        put(options, message, |(md, pmo), data| {
            self.mq().mqput1(self.handle(), &mut mqod, Some(&mut **md), pmo, data)
        })
    }
}

fn put<T: for<'a> MqiAttr<PutParam<'a>>, F: FnOnce(&mut PutParam, &[u8]) -> ResultComp<()>>(
    options: &impl for<'a> MqiOption<'a, PutParam<'a>>,
    message: &(impl PutMessage + ?Sized),
    put: F,
) -> ResultComp<T> {
    let MessageFormat { ccsid, encoding, format } = message.format();
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

    let (attr, result) = T::apply_param(&mut put_param, |p| put(p, &message.render()));
    result.map_completion(|()| attr)
}
