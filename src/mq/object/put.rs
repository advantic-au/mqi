use std::borrow::Cow;
use std::mem;

use libmqm_sys::function;

use crate::headers::{fmt, TextEnc};
use crate::types::{Fmt, MessageFormat, Warning};
use crate::{sys, Completion, Conn, Error, Message, MqMask, MqStruct, Object, QueueManagerShare, ResultComp, ResultCompErr};
use crate::core::{self, values};

pub trait PutMessage {
    type Data: ?Sized;

    #[allow(unused_variables)]
    fn apply_mqput(&self, mqpmo: &mut MqStruct<sys::MQPMO>) {}

    fn render(&self) -> Cow<[u8]>;
    fn format(&self) -> MessageFormat<TextEnc<Fmt>>;
}

pub trait PutResult: Sized {
    type Error: std::fmt::Debug + From<Error>;
    fn create_from(warning: Option<Warning>) -> Result<Self, Self::Error>;
}

pub struct Context<'handle, T> {
    context: MqMask<values::MQPMO>, // TODO: only a subset
    input_handle: Option<&'handle core::ObjectHandle>,
    message: T,
}

#[derive(Debug)]
pub enum Properties<'handle, C: Conn> {
    New(Option<&'handle Message<C>>),
    Reply(&'handle Message<C>, &'handle mut Message<C>),
    Forward(&'handle Message<C>, &'handle mut Message<C>),
    Report(&'handle Message<C>, &'handle mut Message<C>),
}

impl<C: Conn> Default for Properties<'_, C> {
    fn default() -> Self {
        Self::New(None)
    }
}

impl<'handle, C: Conn> Properties<'handle, C> {
    fn apply_mqpmo(&self, mqpmo: &mut MqStruct<sys::MQPMO>) {
        match self {
            Properties::New(original) => {
                mqpmo.Action = sys::MQACTP_NEW;
                mqpmo.OriginalMsgHandle = original.map_or(0, |m| unsafe { m.handle().raw_handle() });
            }
            Properties::Reply(original, new) => {
                mqpmo.Action = sys::MQACTP_REPLY;
                mqpmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
                mqpmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
            }
            Properties::Forward(original, new) => {
                mqpmo.Action = sys::MQACTP_FORWARD;
                mqpmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
                mqpmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
            }
            Properties::Report(original, new) => {
                mqpmo.Action = sys::MQACTP_REPORT;
                mqpmo.OriginalMsgHandle = unsafe { original.handle().raw_handle() };
                mqpmo.NewMsgHandle = unsafe { new.handle().raw_handle() };
            }
        }
    }
}

impl<'handle, T> Context<'handle, T> {
    pub const fn new(message: T, context: MqMask<values::MQPMO>, input_handle: Option<&'handle core::ObjectHandle>) -> Self {
        Self {
            context,
            input_handle,
            message,
        }
    }
}

impl<T: PutMessage> PutMessage for Context<'_, T> {
    type Data = T::Data;

    fn render(&self) -> Cow<[u8]> {
        self.message.render()
    }

    fn format(&self) -> MessageFormat<TextEnc<Fmt>> {
        self.message.format()
    }

    fn apply_mqput(&self, mqpmo: &mut MqStruct<sys::MQPMO>) {
        mqpmo.Context = self.input_handle.map_or(0, |handle| unsafe { handle.raw_handle() });
        mqpmo.Options |= self.context.value();
    }
}

impl PutResult for () {
    type Error = Error;

    fn create_from(_warning: Option<Warning>) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl PutMessage for str {
    type Data = Self;

    fn render(&self) -> Cow<[u8]> {
        self.as_bytes().into()
    }

    fn format(&self) -> MessageFormat<TextEnc<Fmt>> {
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

    fn format(&self) -> MessageFormat<TextEnc<Fmt>> {
        MessageFormat {
            ccsid: 1208,
            encoding: MqMask::from(sys::MQENC_NATIVE),
            format: TextEnc::Ascii(fmt::MQFMT_NONE),
        }
    }
}

impl<C: Conn> Object<C> {
    pub fn put_message<T: PutResult>(
        &self,
        pmo: MqMask<values::MQPMO>,
        properties: &Properties<C>,
        message: &(impl PutMessage + ?Sized),
    ) -> ResultCompErr<T, T::Error> {
        put(pmo, properties, message, |mqmd, mqpmo, data| {
            let connection = self.connection();
            connection
                .mq()
                .mqput(connection.handle(), self.handle(), Some(mqmd), mqpmo, data)
        })
    }
}

impl<L: core::Library<MQ: function::MQI>, H> QueueManagerShare<'_, L, H> {
    pub fn put_message<T: PutResult>(
        &self,
        mqod: &mut MqStruct<sys::MQOD>,
        pmo: MqMask<values::MQPMO>,
        properties: &Properties<&Self>,
        message: &(impl PutMessage + ?Sized),
    ) -> ResultCompErr<T, T::Error> {
        put(pmo, properties, message, |mqmd, mqpmo, data| {
            self.mq().mqput1(self.handle(), mqod, Some(mqmd), mqpmo, data)
        })
    }
}

fn put<C: Conn, T: PutResult, F: FnOnce(&mut sys::MQMD2, &mut sys::MQPMO, &[u8]) -> ResultComp<()>>(
    pmo: MqMask<values::MQPMO>,
    properties: &Properties<C>,
    message: &(impl PutMessage + ?Sized),
    put: F,
) -> ResultCompErr<T, T::Error> {
    let MessageFormat { ccsid, encoding, format } = message.format();
    let mut md = MqStruct::new(sys::MQMD2 {
        CodedCharSetId: ccsid,
        Encoding: encoding.value(),
        Format: unsafe { mem::transmute::<Fmt, [i8; 8]>(format.into_ascii().into()) },
        ..sys::MQMD2::default()
    });
    let mut mqpmo = MqStruct::new(sys::MQPMO {
        Version: sys::MQPMO_VERSION_3,
        Options: pmo.value(),
        ..sys::MQPMO::default()
    });
    properties.apply_mqpmo(&mut mqpmo);
    message.apply_mqput(&mut mqpmo);

    let Completion((), warning) = put(&mut md, &mut mqpmo, &message.render())?;

    Ok(Completion(T::create_from(warning)?, warning))
}
