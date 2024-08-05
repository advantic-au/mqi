use std::borrow::Cow;
use std::mem;

use libmqm_sys::function;

use crate::headers::{fmt, TextEnc};
use crate::types::{Fmt, MessageFormat, Warning};
use crate::{sys, Completion, Conn, Message, MqMask, MqStruct, Object, QueueManagerShare, ResultComp};
use crate::core::{self, values};

pub trait PutMessage {
    type Data: ?Sized;

    fn render(&self) -> Cow<[u8]>;
    fn format(&self) -> MessageFormat;
}

pub trait PutOptions {
    fn apply_mqput(self, md: &mut MqStruct<'static, sys::MQMD2>, mqpmo: &mut MqStruct<'static, sys::MQPMO>);
}

impl PutOptions for () {
    fn apply_mqput(self, _md: &mut MqStruct<'static, sys::MQMD2>, _mqpmo: &mut MqStruct<'static, sys::MQPMO>) {}
}

impl<A: PutOptions> PutOptions for (A,) {
    fn apply_mqput(self, md: &mut MqStruct<'static, sys::MQMD2>, mqpmo: &mut MqStruct<'static, sys::MQPMO>) {
        self.0.apply_mqput(md, mqpmo);
    }
}

impl<A: PutOptions, B: PutOptions> PutOptions for (A, B) {
    fn apply_mqput(self, md: &mut MqStruct<'static, sys::MQMD2>, mqpmo: &mut MqStruct<'static, sys::MQPMO>) {
        self.0.apply_mqput(md, mqpmo);
        self.1.apply_mqput(md, mqpmo);
    }
}

impl<A: PutOptions, B: PutOptions, C: PutOptions> PutOptions for (A, B, C) {
    fn apply_mqput(self, md: &mut MqStruct<'static, sys::MQMD2>, mqpmo: &mut MqStruct<'static, sys::MQPMO>) {
        self.0.apply_mqput(md, mqpmo);
        self.1.apply_mqput(md, mqpmo);
        self.2.apply_mqput(md, mqpmo);
    }
}

impl<A: PutOptions, B: PutOptions, C: PutOptions, D: PutOptions> PutOptions for (A, B, C, D) {
    fn apply_mqput(self, md: &mut MqStruct<'static, sys::MQMD2>, mqpmo: &mut MqStruct<'static, sys::MQPMO>) {
        self.0.apply_mqput(md, mqpmo);
        self.1.apply_mqput(md, mqpmo);
        self.2.apply_mqput(md, mqpmo);
        self.3.apply_mqput(md, mqpmo);
    }
}

impl<F> PutOptions for F where F: FnOnce(&mut MqStruct<'static, sys::MQMD2>, &mut MqStruct<'static, sys::MQPMO>) {
    #[inline]
    fn apply_mqput(self, md: &mut MqStruct<'static, sys::MQMD2>, mqpmo: &mut MqStruct<'static, sys::MQPMO>) {
        self(md, mqpmo);
    }
}

impl PutOptions for MqMask<values::MQPMO> {
    fn apply_mqput(self, _md: &mut MqStruct<'static, sys::MQMD2>, mqpmo: &mut MqStruct<'static, sys::MQPMO>) {
        mqpmo.Options |= self.value();
    }
}

pub trait PutResult: Sized {
    fn create_from(md: &MqStruct<'static, sys::MQMD2>, pmo: &MqStruct<'static, sys::MQPMO>, warning: Option<Warning>) -> Self;
}

impl<A: PutResult, B: PutResult> PutResult for (A, B) {
    fn create_from(md: &MqStruct<'static, sys::MQMD2>, pmo: &MqStruct<'static, sys::MQPMO>, warning: Option<Warning>) -> Self {
        (A::create_from(md, pmo, warning), B::create_from(md, pmo, warning))
    }
}

impl<A: PutResult, B: PutResult, C: PutResult> PutResult for (A, B, C) {
    fn create_from(md: &MqStruct<'static, sys::MQMD2>, pmo: &MqStruct<'static, sys::MQPMO>, warning: Option<Warning>) -> Self {
        (
            A::create_from(md, pmo, warning),
            B::create_from(md, pmo, warning),
            C::create_from(md, pmo, warning),
        )
    }
}

impl<A: PutResult, B: PutResult, C: PutResult, D: PutResult> PutResult for (A, B, C, D) {
    fn create_from(md: &MqStruct<'static, sys::MQMD2>, pmo: &MqStruct<'static, sys::MQPMO>, warning: Option<Warning>) -> Self {
        (
            A::create_from(md, pmo, warning),
            B::create_from(md, pmo, warning),
            C::create_from(md, pmo, warning),
            D::create_from(md, pmo, warning),
        )
    }
}

pub struct Context<'handle> {
    context: MqMask<values::MQPMO>, // TODO: only a subset
    input_handle: Option<&'handle core::ObjectHandle>,
}

#[derive(Debug)]
pub enum Properties<'handle, C: Conn> {
    Reply(&'handle Message<C>, &'handle mut Message<C>),
    Forward(&'handle Message<C>, &'handle mut Message<C>),
    Report(&'handle Message<C>, &'handle mut Message<C>),
}

impl<C: Conn> PutOptions for &mut Message<C> {
    fn apply_mqput(self, _md: &mut MqStruct<'static, sys::MQMD2>, mqpmo: &mut MqStruct<'static, sys::MQPMO>) {
        mqpmo.Action = sys::MQACTP_NEW;
        mqpmo.OriginalMsgHandle = unsafe { self.handle().raw_handle() };
    }
}

impl<'handle, C: Conn> PutOptions for Properties<'handle, C> {
    fn apply_mqput(self, _md: &mut MqStruct<'static, sys::MQMD2>, mqpmo: &mut MqStruct<'static, sys::MQPMO>) {
        match self {
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

impl<'handle> Context<'handle> {
    #[must_use]
    pub const fn new(context: MqMask<values::MQPMO>, input_handle: Option<&'handle core::ObjectHandle>) -> Self {
        Self { context, input_handle }
    }
}

impl PutOptions for Context<'_> {
    fn apply_mqput(self, _md: &mut MqStruct<'static, sys::MQMD2>, mqpmo: &mut MqStruct<'static, sys::MQPMO>) {
        mqpmo.Context = self.input_handle.map_or(0, |handle| unsafe { handle.raw_handle() });
        mqpmo.Options |= self.context.value();
    }
}

impl PutResult for () {
    fn create_from(_md: &MqStruct<'static, sys::MQMD2>, _pmo: &MqStruct<'static, sys::MQPMO>, _warning: Option<Warning>) -> Self {
    }
}

impl PutResult for MqStruct<'_, sys::MQMD2> {
    fn create_from(md: &MqStruct<'static, sys::MQMD2>, _pmo: &MqStruct<'static, sys::MQPMO>, _warning: Option<Warning>) -> Self {
        md.clone()
    }
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
    pub fn put_message<T: PutResult>(&self, options: impl PutOptions, message: &(impl PutMessage + ?Sized)) -> ResultComp<T> {
        put(options, message, |mqmd, mqpmo, data| {
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
        options: impl PutOptions,
        message: &(impl PutMessage + ?Sized),
    ) -> ResultComp<T> {
        put(options, message, |mqmd, mqpmo, data| {
            self.mq().mqput1(self.handle(), mqod, Some(mqmd), mqpmo, data)
        })
    }
}

fn put<T: PutResult, F: FnOnce(&mut sys::MQMD2, &mut sys::MQPMO, &[u8]) -> ResultComp<()>>(
    options: impl PutOptions,
    message: &(impl PutMessage + ?Sized),
    put: F,
) -> ResultComp<T> {
    let MessageFormat { ccsid, encoding, format } = message.format();
    let mut md = MqStruct::new(sys::MQMD2 {
        CodedCharSetId: ccsid,
        Encoding: encoding.value(),
        Format: unsafe { mem::transmute::<Fmt, [i8; 8]>(format.into_ascii().into()) },
        ..sys::MQMD2::default()
    });
    let mut mqpmo = MqStruct::new(sys::MQPMO {
        Version: sys::MQPMO_VERSION_3,
        ..sys::MQPMO::default()
    });

    options.apply_mqput(&mut md, &mut mqpmo);

    let Completion((), warning) = put(&mut md, &mut mqpmo, &message.render())?;

    Ok(Completion(T::create_from(&md, &mqpmo, warning), warning))
}
