use std::{borrow::Cow, cmp, num::NonZero, str::Utf8Error};

use libmqm_default as default;

use crate::{
    core::{WriteRaw, CCSID},
    headers::{ChainedHeader, EncodedHeader, Header, HeaderError, TextEnc},
    prelude::*,
    constants, sys, types, Buffer, Completion, Conn, Error, MqStruct, Object, ResultComp, ResultCompErr, StrCcsidCow,
    types::MQENC,
};

#[derive(Clone, Debug, derive_more::Constructor)]
pub struct Headers<'a> {
    message_length: usize,
    init_format: types::MessageFormat,
    data: Cow<'a, [u8]>,
    error: Option<HeaderError>,
}

impl<'a> Headers<'a> {
    pub fn all_headers(&'a self) -> impl Iterator<Item = Header<'a>> {
        Header::iter(&self.data, self.init_format).filter_map(|result| match result {
            Ok((header, ..)) => Some(header),
            Err(_) => None,
        })
    }

    pub fn header<C: ChainedHeader + 'a>(&'a self) -> impl Iterator<Item = EncodedHeader<'a, C>> {
        self.all_headers().filter_map(C::from_header)
    }

    #[must_use]
    pub const fn error(&self) -> Option<&HeaderError> {
        self.error.as_ref()
    }

    #[must_use]
    pub const fn message_length(&self) -> usize {
        self.message_length
    }
}

#[derive(Debug, Clone, Default)]
pub struct MatchOptions<'a> {
    pub msg_id: Option<&'a types::MessageId>,
    pub correl_id: Option<&'a types::CorrelationId>,
    pub group_id: Option<&'a types::GroupId>,
    pub seq_number: Option<sys::MQLONG>,
    pub offset: Option<sys::MQLONG>,
    pub token: Option<&'a types::MsgToken>,
}

pub const ANY_MESSAGE: MatchOptions = MatchOptions {
    msg_id: None,
    correl_id: None,
    group_id: None,
    seq_number: None,
    offset: None,
    token: None,
};

// TODO: add MQ warnings to error messages
#[derive(derive_more::Error, derive_more::From, derive_more::Display, Debug)]
pub enum GetStringError {
    #[display("Message parsing error: {_0}")]
    Utf8Parse(Utf8Error, Option<types::Warning>),
    #[display("Unexpected format or CCSID. Message format = '{_0}', CCSID = {_1}")]
    UnexpectedFormat(TextEnc<types::Fmt>, CCSID, Option<types::Warning>),
    #[from]
    MQ(Error),
}

#[derive(derive_more::Error, derive_more::Display, derive_more::From, Debug)]
pub enum GetStringCcsidError {
    #[display("Unexpected format. Message format = '{_0}'")]
    UnexpectedFormat(TextEnc<types::Fmt>, Option<types::Warning>),
    #[from]
    MQ(Error),
}

#[derive(Default)]
pub enum GetWait {
    #[default]
    NoWait,
    Wait(sys::MQLONG),
}

pub enum GetConvert {
    NoConvert,
    Convert,
    ConvertTo(CCSID, MQENC),
}

pub struct GetParam {
    pub md: MqStruct<'static, sys::MQMD2>,
    pub gmo: MqStruct<'static, sys::MQGMO>,
}

pub struct GetState<B> {
    /// The buffer holding the message data from the `MQGET` call.
    pub buffer: B,
    /// The length of the message data returned by the `MQGET` call, confined by buffer size
    pub data_length: usize,
    /// The full length of the message data unconfined by buffer size
    pub message_length: usize,
    /// The format of the returned message
    pub format: types::MessageFormat,
}

impl<B> GetState<B> {
    pub fn into_truncated_buffer<'b, R>(self) -> B
    where
        B: Buffer<'b, R>,
    {
        self.buffer.truncate(self.data_length)
    }
}

pub trait GetAttr<'b, R> {
    fn get_extract<F, B>(param: &mut GetParam, get: F) -> ResultComp<(Self, GetState<B>)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        B: Buffer<'b, R>,
        Self: Sized;
}

#[cfg(feature = "mqai")]
pub trait GetBagAttr {
    fn get_bag_extract<F>(param: &mut GetParam, mqi: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<()>,
        Self: Sized;
}

/// # Examples
/// Implements [`GetValue`] for a fixed array of bytes
///
/// ```
/// use std::num::NonZero;
/// use mqi::{get, prelude::*, Buffer, Error, ResultComp};
///
/// pub struct Fixed<const N: usize>(pub [u8; N]);
///
/// impl<'b, const N: usize, B> get::GetValue<'b, u8, B> for Fixed<N> {
///    type Error = Error;
///
///    fn get_consume<F>(param: &mut get::GetParam, get: F) -> ResultComp<Self>
///    where
///        F: FnOnce(&mut get::GetParam) -> ResultComp<get::GetState<B>>,
///        B: Buffer<'b, u8>,
///    {
///        get(param).map_completion(|state| {
///            // Copy the message data into the fixed array
///            let msg_data: &[u8] = &state.buffer.as_ref()[..state.data_length];
///            let mut target = Self([0; N]);
///            target.0[..state.data_length].copy_from_slice(msg_data);
///            target
///        })
///    }
///
///    fn get_max_data_size() -> Option<NonZero<usize>> {
///        NonZero::new(N)
///    }
/// }
/// ```
pub trait GetValue<'b, R, B>: std::marker::Sized {
    type Error: std::fmt::Debug;

    /// Execute and consumes the result of the provided `get` function, creating `Self` from the [`GetState`]
    fn get_consume<F>(param: &mut GetParam, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        B: Buffer<'b, R>;

    /// The maximum size in bytes `Self` can consume from a `get` function call
    #[must_use]
    #[inline]
    fn get_max_data_size() -> Option<NonZero<usize>> {
        None
    }
}

/// A trait that manipulates the parameters to the [`mqget`](`crate::core::MqFunctions::mqget`) function
#[diagnostic::on_unimplemented(message = "{Self} does not implement `GetOption` so it can't be used as an argument for MQI get")]
pub trait GetOption {
    fn apply_param(&self, param: &mut GetParam);
}

#[cfg(feature = "mqai")]
mod mqai {
    use crate::{
        admin::{Bag, Owned},
        core::Library,
        prelude::*,
        constants, Completion, Conn, Error, MqStruct, Object, ResultComp,
    };
    use libmqm_default as default;
    use libmqm_sys::Mqai;

    use super::{GetBagAttr, GetOption, GetParam};

    impl<C: Conn> Object<C>
    where
        C::Lib: crate::core::Library<MQ: libmqm_sys::Mqai>,
    {
        pub fn get_bag_with<R: GetBagAttr>(
            &self,
            options: &impl GetOption,
            bag: &mut Bag<Owned, impl Library<MQ: Mqai>>,
        ) -> ResultComp<Option<R>> {
            let mut param = GetParam {
                md: MqStruct::new(default::MQMD2_DEFAULT),
                gmo: MqStruct::new(default::MQGMO_DEFAULT),
            };
            let mut no_msg_available = false;

            options.apply_param(&mut param);

            let result = R::get_bag_extract(&mut param, |param| {
                let connection = self.connection();
                let mqi_get_bag = connection.mq().mq_get_bag(
                    connection.handle(),
                    self.handle(),
                    &mut *param.md,
                    &mut param.gmo,
                    Some(&*bag),
                );
                no_msg_available = mqi_get_bag
                    .as_ref()
                    .is_err_and(|err| matches!(err, &Error(constants::MQCC_FAILED, _, constants::MQRC_NO_MSG_AVAILABLE)));
                mqi_get_bag
            });

            if no_msg_available {
                Ok(Completion::new(None))
            } else {
                result.map_completion(Some)
            }
        }

        pub fn get_bag(&self, options: &impl GetOption, bag: &mut Bag<Owned, impl Library<MQ: Mqai>>) -> ResultComp<bool> {
            self.get_bag_with::<()>(options, bag).map_completion(|o| o.is_some())
        }
    }
}

impl<C: Conn> Object<C> {
    pub fn get_data<'b, R>(&self, options: &impl GetOption, buffer: &'b mut [R]) -> ResultComp<Option<&'b [R]>>
    where
        R: WriteRaw<u8>,
    {
        self.get_as(options, buffer)
            .map_completion(|o| o.map(|buffer: &mut [R]| &*buffer))
    }

    pub fn get_data_with<'b, A, R>(&self, options: &impl GetOption, buffer: &'b mut [R]) -> ResultComp<Option<(&'b [R], A)>>
    where
        A: GetAttr<'b, R>,
        R: WriteRaw<u8>,
    {
        self.get_as(options, buffer)
            .map_completion(|o| o.map(|(buffer, attr): (&mut [R], A)| (&*buffer, attr)))
    }

    pub fn get_string<'b>(
        &self,
        options: &impl GetOption,
        buffer: impl Buffer<'b, u8>,
    ) -> ResultCompErr<Option<StrCcsidCow<'b>>, GetStringCcsidError> {
        self.get_as(options, buffer)
    }

    pub fn get_string_with<'b, A>(
        &self,
        options: &impl GetOption,
        buffer: impl Buffer<'b, u8>,
    ) -> ResultCompErr<Option<(StrCcsidCow<'b>, A)>, GetStringCcsidError>
    where
        A: GetAttr<'b, u8>,
    {
        self.get_as(options, buffer)
    }

    pub fn get_as<'b, V, R, B>(&self, options: &impl GetOption, buffer: B) -> ResultCompErr<Option<V>, V::Error>
    where
        B: Buffer<'b, R>,
        V: GetValue<'b, R, B>,
        R: WriteRaw<u8>,
    {
        let mut param = GetParam {
            md: MqStruct::new(default::MQMD2_DEFAULT),
            gmo: MqStruct::new(sys::MQGMO {
                Version: sys::MQGMO_VERSION_3, // Version 3 for ReturnedLength
                ..default::MQGMO_DEFAULT
            }),
        };
        let mut no_msg_available = false;

        options.apply_param(&mut param);

        let result = V::get_consume(&mut param, |param| {
            let mut buffer = buffer;
            let write_area = match V::get_max_data_size() {
                Some(max_len) => &mut buffer.as_mut()[..max_len.into()],
                None => buffer.as_mut(),
            };

            let mqi_get = self
                .connection()
                .mq()
                .mqget(
                    self.connection().handle(),
                    self.handle(),
                    Some(&mut *param.md),
                    &mut param.gmo,
                    write_area,
                )
                .map_completion(|length| {
                    (
                        length,
                        match types::MQRL(param.gmo.ReturnedLength) {
                            constants::MQRL_UNDEFINED => cmp::min(
                                write_area
                                    .len()
                                    .try_into()
                                    .expect("length of buffer should be within positive i32 range"),
                                length,
                            ),
                            returned_length => returned_length.0,
                        },
                    )
                })
                .map_completion(|(message_length, data_length)| GetState {
                    buffer,
                    data_length: data_length
                        .try_into()
                        .expect("data length should be within positive usize range"),
                    message_length: message_length
                        .try_into()
                        .expect("message length should be within positive usize range"),
                    format: types::MessageFormat {
                        ccsid: CCSID(param.md.CodedCharSetId),
                        encoding: MQENC(param.md.Encoding),
                        fmt: TextEnc::Ascii(param.md.Format),
                    },
                });
            no_msg_available = mqi_get
                .as_ref()
                .is_err_and(|e| matches!(e, &Error(constants::MQCC_FAILED, _, constants::MQRC_NO_MSG_AVAILABLE)));

            mqi_get
        });

        if no_msg_available {
            Ok(Completion::new(None))
        } else {
            result.map_completion(Some)
        }
    }
}
