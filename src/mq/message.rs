use num_enum::IntoPrimitive;

use crate::core;
use crate::core::Library;
use crate::core::MQFunctions;
use crate::sys;
use crate::Completion;
use crate::ResultCompErrExt as _;
use crate::{CompletionCode, Error, ReasonCode, ResultComp, ResultErr};

use super::ConnectionShare;

pub struct Message<'ch, L: Library> {
    handle: core::MessageHandle,
    mq: MQFunctions<L>,
    connection: &'ch core::ConnectionHandle,
}

pub struct MsgPropIter<'mh, L: Library> {
    name: String,
    message: &'mh Message<'mh, L>,
    inq_prop_opts: sys::MQIMPO,
}

impl<L: Library> Iterator for MsgPropIter<'_, L> {
    type Item = ResultComp<()>;

    fn next(&mut self) -> Option<Self::Item> {
        fn next_result<A: Library>(it: &mut MsgPropIter<A>) -> ResultComp<()> {
            let name_ref: &str = &it.name;
            let len = name_ref
                .len()
                .try_into()
                .expect("name length exceeds maximum positive MQLONG");
            let name_mqcharv = sys::MQCHARV {
                VSPtr: name_ref.as_ptr().cast_mut().cast(),
                VSOffset: 0,
                VSBufSize: len,
                VSLength: len,
                VSCCSID: 1208, // name is always UTF-8
            };
            let mut returning_name = Vec::<u8>::with_capacity(page_size::get());
            let rn_mqcharv = &mut it.inq_prop_opts.ReturnedName;
            rn_mqcharv.VSPtr = returning_name.spare_capacity_mut().as_mut_ptr().cast();
            rn_mqcharv.VSOffset = 0;
            rn_mqcharv.VSBufSize = returning_name
                .capacity()
                .try_into()
                .expect("page size exceeds maximum positive MQLONG");
            rn_mqcharv.VSLength = 0;
            rn_mqcharv.VSCCSID = 0;

            let mut value = Vec::<u8>::with_capacity(page_size::get());
            let mut prop_type = sys::MQTYPE_AS_SET;
            let mut prop_desc = sys::MQPD::default();

            let inq_length = it.message.mq.mqinqmp(
                Some(it.message.connection),
                &it.message.handle,
                &mut it.inq_prop_opts,
                &name_mqcharv,
                &mut prop_desc,
                &mut prop_type,
                value.spare_capacity_mut(),
            );
            if let Ok(Completion(length, ..)) = inq_length {
                unsafe {
                    value.set_len(
                        length
                            .try_into()
                            .expect("property length exceeds maximum positive MQLONG"),
                    );
                    returning_name.set_len(
                        it.inq_prop_opts
                            .ReturnedName
                            .VSLength
                            .try_into()
                            .expect("property name length exceeds maximum positive MQLONG"),
                    );
                }
            }
            it.inq_prop_opts.Options |= sys::MQIMPO_INQ_NEXT;

            inq_length.map_completion(|_| ())
        }

        match next_result(self) {
            Err(Error(CompletionCode(sys::MQCC_FAILED), _, ReasonCode(sys::MQRC_PROPERTY_NOT_AVAILABLE))) => None,
            result => Some(result),
        }
    }
}

#[repr(i32)]
pub enum Value {
    Boolean(bool) = sys::MQTYPE_BOOLEAN,
    Bytes(Vec<u8>) = sys::MQTYPE_BYTE_STRING,
    Int16(i16) = sys::MQTYPE_INT16,
    Int32(i32) = sys::MQTYPE_INT32,
    Int64(i64) = sys::MQTYPE_INT64,
    Float32(f32) = sys::MQTYPE_FLOAT32,
    Float64(f64) = sys::MQTYPE_FLOAT64,
    String(String) = sys::MQTYPE_STRING,
    Null = sys::MQTYPE_NULL,
    Data(sys::MQLONG, Vec<u8>, Coding),
}

pub enum Coding {
    NumericEncoding(sys::MQLONG),
    TextCCSID(sys::MQLONG),
}

#[derive(Default, IntoPrimitive)]
#[repr(i32)]
pub enum MessageHandleOptions {
    Validate = sys::MQCMHO_VALIDATE,
    NoValidation = sys::MQCMHO_NO_VALIDATION,
    #[default]
    Default = sys::MQCMHO_DEFAULT_VALIDATION,
}

impl Value {
    #[must_use]
    #[allow(clippy::cast_possible_wrap)] // Masks are unsigned.
    pub fn from(data: &[u8], prop_type: sys::MQLONG, ccsid: sys::MQLONG, encoding: sys::MQLONG) -> Self {
        static ENC_NATIVE_INTEGER: sys::MQLONG = sys::MQENC_INTEGER_MASK as sys::MQLONG & sys::MQENC_NATIVE;
        static ENC_NATIVE_FLOAT: sys::MQLONG = sys::MQENC_FLOAT_MASK as sys::MQLONG & sys::MQENC_NATIVE;
        match prop_type {
            sys::MQTYPE_BOOLEAN => Ok(Self::Boolean(data[0] != 0)),
            sys::MQTYPE_BYTE_STRING => Ok(Self::Bytes(data.to_owned())),
            sys::MQTYPE_INT16 if (encoding & ENC_NATIVE_INTEGER) != 0 => {
                data.try_into().map(|d| Self::Int16(i16::from_ne_bytes(d)))
            }
            sys::MQTYPE_INT32 if (encoding & ENC_NATIVE_INTEGER) != 0 => {
                data.try_into().map(|d| Self::Int32(i32::from_ne_bytes(d)))
            }
            sys::MQTYPE_INT64 if (encoding & ENC_NATIVE_INTEGER) != 0 => {
                data.try_into().map(|d| Self::Int64(i64::from_ne_bytes(d)))
            }
            sys::MQTYPE_FLOAT32 if (encoding & ENC_NATIVE_FLOAT) != 0 => {
                data.try_into().map(|d| Self::Float32(f32::from_ne_bytes(d)))
            }
            sys::MQTYPE_FLOAT64 if (encoding & ENC_NATIVE_FLOAT) != 0 => {
                data.try_into().map(|d| Self::Float64(f64::from_ne_bytes(d)))
            }
            sys::MQTYPE_STRING if ccsid == 1208 => Ok(Self::String(String::from_utf8_lossy(data).into_owned())),
            sys::MQTYPE_STRING => Ok(Self::Data(prop_type, data.into(), Coding::TextCCSID(ccsid))),
            sys::MQTYPE_NULL => Ok(Self::Null),
            _ => Ok(Self::Data(prop_type, data.into(), Coding::NumericEncoding(encoding))),
        }
        .expect("Unexpected data length")
    }
}

impl<L: Library> Drop for Message<'_, L> {
    fn drop(&mut self) {
        let mqdmho = sys::MQDMHO::default();
        let _ = self.mq.mqdltmh(Some(self.connection), &mut self.handle, &mqdmho);
    }
}

impl<'connection, L: Library> Message<'connection, L> {
    pub fn new(
        lib: L,
        connection: &'connection core::ConnectionHandle,
        options: MessageHandleOptions,
    ) -> ResultErr<Self> {
        let mqcmho = sys::MQCMHO {
            Options: options.into(),
            ..sys::MQCMHO::default()
        };
        let mq = MQFunctions(lib);
        mq.mqcrtmh(Some(connection), &mqcmho)
            .map(|handle| Self { handle, mq, connection })
    }

    pub fn inq_properties(&self, name: impl Into<String>) -> MsgPropIter<'_, L> {
        MsgPropIter {
            name: name.into(),
            message: self,
            inq_prop_opts: sys::MQIMPO::default(),
        }
    }
}

impl<L: Library, H> ConnectionShare<L, H> {
    pub fn put<B>(&self, mqod: &sys::MQOD, mqmd: &mut sys::MQMD, pmo: &mut sys::MQPMO, body: &B) -> ResultComp<()> {
        self.mq.mqput1(self.handle(), mqod, mqmd, pmo, body)
    }
}
