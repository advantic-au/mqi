use core::str;
use std::{borrow::Cow, cmp, mem::transmute, num::NonZero, ops::Deref, str::Utf8Error};

use crate::{
    common::ResultCompErrExt as _,
    core::values,
    headers::{fmt, ChainedHeader, EncodedHeader, Header, HeaderError, TextEnc},
    sys,
    types::{self, Fmt, MessageFormat},
    Buffer, Completion, Conn, Error, Message, MqMask, MqStruct, MqValue, ResultCompErr, StrCcsidCow,
};

use super::Object;

#[derive(Clone, Debug)]
pub struct Mqmd<T> {
    pub mqmd: MqStruct<'static, sys::MQMD2>,
    pub next: T,
}

#[derive(Clone, Debug)]
pub struct Headers<'a, T> {
    message_length: usize,
    init_format: MessageFormat,
    format: MessageFormat,
    data: Cow<'a, [u8]>,
    error: Option<HeaderError>,
    next: T,
}

impl<'a, T> Headers<'a, T> {
    pub fn all_headers(&'a self) -> impl Iterator<Item = Header<'a>> {
        Header::iter(&self.data, self.init_format).filter_map(|result| match result {
            Ok((header, ..)) => Some(header),
            Err(_) => None,
        })
    }

    pub fn header<C: ChainedHeader + 'a>(&'a self) -> impl Iterator<Item = EncodedHeader<'a, C>> {
        self.all_headers().filter_map(C::from_header)
    }

    pub const fn header_error(&self) -> Option<&HeaderError> {
        self.error.as_ref()
    }

    pub const fn format(&self) -> MessageFormat {
        self.format
    }

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
#[derive(thiserror::Error, Debug)]
pub enum GetStringError {
    #[error("Message parsing error: {}", .0)]
    Utf8Parse(Utf8Error, Option<types::Warning>),
    #[error("Unexpected format or CCSID. Message format = '{}', CCSID = {}", .0, .1)]
    UnexpectedFormat(TextEnc<Fmt>, sys::MQLONG, Option<types::Warning>),
    #[error(transparent)]
    MQ(#[from] Error),
}

#[derive(thiserror::Error, Debug)]
pub enum GetStringCcsidError {
    #[error("Unexpected format. Message format = '{}'", .0)]
    UnexpectedFormat(TextEnc<Fmt>, Option<types::Warning>),
    #[error(transparent)]
    MQ(#[from] Error),
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
    ConvertTo(sys::MQLONG, MqMask<values::MQENC>),
}

pub trait GetOptions {
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>);
}

impl GetOptions for MqMask<values::MQGMO> {
    fn apply_mqget(self, _md: &mut MqStruct<sys::MQMD2>, gmo: &mut MqStruct<sys::MQGMO>) {
        gmo.Options |= self.value();
    }
}

impl GetOptions for GetWait {
    fn apply_mqget(self, _md: &mut MqStruct<sys::MQMD2>, gmo: &mut MqStruct<sys::MQGMO>) {
        match self {
            Self::NoWait => gmo.Options |= sys::MQGMO_NO_WAIT,
            Self::Wait(interval) => {
                gmo.Options |= sys::MQGMO_WAIT;
                gmo.WaitInterval = interval;
            }
        }
    }
}

impl GetOptions for GetConvert {
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        match self {
            Self::NoConvert => {}
            Self::Convert => gmo.Options |= sys::MQGMO_CONVERT,
            Self::ConvertTo(ccsid, encoding) => {
                gmo.Options |= sys::MQGMO_CONVERT;
                md.CodedCharSetId = ccsid;
                md.Encoding = encoding.value();
            }
        }
    }
}

impl<F> GetOptions for F
where
    F: FnOnce(&mut MqStruct<'static, sys::MQMD2>, &mut MqStruct<'static, sys::MQGMO>),
{
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        self(md, gmo);
    }
}

impl<C: Conn> GetOptions for &mut Message<C> {
    fn apply_mqget(self, _md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        gmo.Options |= sys::MQGMO_PROPERTIES_IN_HANDLE;
        gmo.MsgHandle = unsafe { self.handle().raw_handle() }
    }
}

impl GetOptions for MatchOptions<'_> {
    fn apply_mqget(self, md: &mut MqStruct<sys::MQMD2>, gmo: &mut MqStruct<sys::MQGMO>) {
        // Set up the MQMD
        if let Some(msg_id) = self.msg_id {
            md.MsgId = msg_id.0;
        }
        if let Some(correl_id) = self.correl_id {
            md.CorrelId = correl_id.0;
        }
        if let Some(group_id) = self.group_id {
            md.GroupId = group_id.0;
        }
        md.MsgSeqNumber = self.seq_number.unwrap_or(0);
        md.Offset = self.offset.unwrap_or(0);

        // Set up the GMO
        if let Some(token) = self.token {
            gmo.MsgToken = token.0;
        }
        gmo.MatchOptions = self.correl_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_CORREL_ID)
            | self.msg_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_ID)
            | self.group_id.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_GROUP_ID)
            | self.seq_number.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_SEQ_NUMBER)
            | self.offset.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_OFFSET)
            | self.token.map_or(sys::MQMO_NONE, |_| sys::MQMO_MATCH_MSG_TOKEN);
    }
}

impl GetOptions for () {
    fn apply_mqget(self, _md: &mut MqStruct<'static, sys::MQMD2>, _gmo: &mut MqStruct<'static, sys::MQGMO>) {}
}

impl<A: GetOptions> GetOptions for (A,) {
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        self.0.apply_mqget(md, gmo);
    }
}

impl<A: GetOptions, B: GetOptions> GetOptions for (A, B) {
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        self.0.apply_mqget(md, gmo);
        self.1.apply_mqget(md, gmo);
    }
}

impl<A: GetOptions, B: GetOptions, C: GetOptions> GetOptions for (A, B, C) {
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        self.0.apply_mqget(md, gmo);
        self.1.apply_mqget(md, gmo);
        self.2.apply_mqget(md, gmo);
    }
}

impl<A: GetOptions, B: GetOptions, C: GetOptions, D: GetOptions> GetOptions for (A, B, C, D) {
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        self.0.apply_mqget(md, gmo);
        self.1.apply_mqget(md, gmo);
        self.2.apply_mqget(md, gmo);
        self.3.apply_mqget(md, gmo);
    }
}

impl<A: GetOptions, B: GetOptions, C: GetOptions, D: GetOptions, E: GetOptions> GetOptions for (A, B, C, D, E) {
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        self.0.apply_mqget(md, gmo);
        self.1.apply_mqget(md, gmo);
        self.2.apply_mqget(md, gmo);
        self.3.apply_mqget(md, gmo);
        self.4.apply_mqget(md, gmo);
    }
}

impl<A: GetOptions, B: GetOptions, C: GetOptions, D: GetOptions, E: GetOptions, F: GetOptions> GetOptions for (A, B, C, D, E, F) {
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        self.0.apply_mqget(md, gmo);
        self.1.apply_mqget(md, gmo);
        self.2.apply_mqget(md, gmo);
        self.3.apply_mqget(md, gmo);
        self.4.apply_mqget(md, gmo);
        self.5.apply_mqget(md, gmo);
    }
}

impl<A: GetOptions, B: GetOptions, C: GetOptions, D: GetOptions, E: GetOptions, F: GetOptions, G: GetOptions> GetOptions
    for (A, B, C, D, E, F, G)
{
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        self.0.apply_mqget(md, gmo);
        self.1.apply_mqget(md, gmo);
        self.2.apply_mqget(md, gmo);
        self.3.apply_mqget(md, gmo);
        self.4.apply_mqget(md, gmo);
        self.5.apply_mqget(md, gmo);
        self.6.apply_mqget(md, gmo);
    }
}

impl<A: GetOptions, B: GetOptions, C: GetOptions, D: GetOptions, E: GetOptions, F: GetOptions, G: GetOptions, H: GetOptions>
    GetOptions for (A, B, C, D, E, F, G, H)
{
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        self.0.apply_mqget(md, gmo);
        self.1.apply_mqget(md, gmo);
        self.2.apply_mqget(md, gmo);
        self.3.apply_mqget(md, gmo);
        self.4.apply_mqget(md, gmo);
        self.5.apply_mqget(md, gmo);
        self.6.apply_mqget(md, gmo);
        self.7.apply_mqget(md, gmo);
    }
}

impl GetOptions for types::CorrelationId {
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        md.CorrelId = self.0;
        gmo.MatchOptions |= sys::MQMO_MATCH_CORREL_ID;
    }
}

impl GetOptions for types::MessageId {
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        md.MsgId = self.0;
        gmo.MatchOptions |= sys::MQMO_MATCH_MSG_ID;
    }
}

impl GetOptions for types::GroupId {
    fn apply_mqget(self, md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        md.GroupId = self.0;
        gmo.MatchOptions |= sys::MQMO_MATCH_GROUP_ID;
    }
}

impl GetOptions for types::MsgToken {
    fn apply_mqget(self, _md: &mut MqStruct<'static, sys::MQMD2>, gmo: &mut MqStruct<'static, sys::MQGMO>) {
        gmo.MsgToken = self.0;
        gmo.MatchOptions |= sys::MQMO_MATCH_MSG_TOKEN;
    }
}

pub trait GetMessage<'a>: Sized {
    type Error: std::fmt::Debug + From<Error>;
    type Payload;

    #[allow(clippy::too_many_arguments)]
    fn create_from<C: Conn>(
        object: &Object<C>,
        data: impl Buffer<'a>,
        data_length: usize,
        message_length: usize,
        md: &MqStruct<'static, sys::MQMD2>,
        gmo: &MqStruct<sys::MQGMO>,
        format: MessageFormat,
        warning: Option<types::Warning>,
    ) -> Result<Self, Self::Error>;

    #[must_use]
    fn max_data_size() -> Option<NonZero<usize>> {
        None
    }

    #[allow(unused_variables)]
    fn apply_mqget(md: &mut MqStruct<sys::MQMD2>, gmo: &mut MqStruct<sys::MQGMO>) {}

    fn payload(&self) -> &Self::Payload;
    fn into_payload(self) -> Self::Payload;
}

impl<'a> GetMessage<'a> for StrCcsidCow<'a> {
    type Error = GetStringCcsidError;
    type Payload = Self;

    fn create_from<C: Conn>(
        _object: &Object<C>,
        data: impl Buffer<'a>,
        data_length: usize,
        _message_length: usize,
        _md: &MqStruct<'static, sys::MQMD2>,
        _gmo: &MqStruct<sys::MQGMO>,
        format: MessageFormat,
        warning: Option<types::Warning>,
    ) -> Result<Self, Self::Error> {
        if format.format != TextEnc::Ascii(fmt::MQFMT_STRING) {
            return Err(GetStringCcsidError::UnexpectedFormat(format.format, warning));
        }

        Ok(Self {
            ccsid: NonZero::new(format.ccsid),
            data: data.truncate(data_length).into_cow(),
            le: (format.encoding & sys::MQENC_INTEGER_REVERSED) != 0,
        })
    }

    fn payload(&self) -> &Self::Payload {
        self
    }

    fn into_payload(self) -> Self::Payload {
        self
    }
}

impl<'a> GetMessage<'a> for Cow<'a, str> {
    type Error = GetStringError;
    type Payload = Self;

    fn create_from<C: Conn>(
        _object: &Object<C>,
        data: impl Buffer<'a>,
        data_length: usize,
        _message_length: usize,
        _md: &MqStruct<'static, sys::MQMD2>,
        _gmo: &MqStruct<sys::MQGMO>,
        format: MessageFormat,
        warning: Option<types::Warning>,
    ) -> Result<Self, Self::Error> {
        if format.format != TextEnc::Ascii(fmt::MQFMT_STRING) || format.ccsid != 1208 {
            return Err(GetStringError::UnexpectedFormat(format.format, format.ccsid, warning));
        }

        match (warning, data.truncate(data_length).into_cow()) {
            (Some((rc, verb)), _) if rc == sys::MQRC_NOT_CONVERTED => {
                Err(Error(MqValue::from(sys::MQCC_WARNING), verb, rc).into())
            }
            (warning, Cow::Borrowed(bytes)) => Ok(Cow::Borrowed(
                std::str::from_utf8(bytes).map_err(|e| GetStringError::Utf8Parse(e, warning))?,
            )),
            (warning, Cow::Owned(bytes)) => Ok(Cow::Owned(
                String::from_utf8(bytes).map_err(|e| GetStringError::Utf8Parse(e.utf8_error(), warning))?,
            )),
        }
    }

    fn payload(&self) -> &Self::Payload {
        self
    }

    fn into_payload(self) -> Self::Payload {
        self
    }
}

impl<'a> GetMessage<'a> for Cow<'a, [u8]> {
    type Error = Error;
    type Payload = Self;

    fn create_from<C: Conn>(
        _object: &Object<C>,
        data: impl Buffer<'a>,
        data_length: usize,
        _message_length: usize,
        _md: &MqStruct<'static, sys::MQMD2>,
        _gmo: &MqStruct<sys::MQGMO>,
        _format: MessageFormat,
        _warning: Option<types::Warning>,
    ) -> Result<Self, Self::Error> {
        Ok(data.truncate(data_length).into_cow())
    }

    fn payload(&self) -> &Self::Payload {
        self
    }

    fn into_payload(self) -> Self::Payload {
        self
    }
}

impl<'a, T: GetMessage<'a>> GetMessage<'a> for Headers<'a, T> {
    type Error = T::Error;

    type Payload = T::Payload;

    fn create_from<C: Conn>(
        object: &Object<C>,
        buffer: impl Buffer<'a>,
        data_length: usize,
        message_length: usize,
        md: &MqStruct<'static, sys::MQMD2>,
        gmo: &MqStruct<sys::MQGMO>,
        format: MessageFormat,
        warning: Option<types::Warning>,
    ) -> Result<Self, Self::Error> {
        let data = &buffer.as_ref()[..data_length];
        let mut header_length = 0;
        let mut final_format = format;
        let mut error = None;
        for result in Header::iter(data, format) {
            match result {
                Ok((.., header_size, message_format)) => {
                    header_length += header_size;
                    final_format = message_format;
                }
                Err(e) => error = Some(e),
            }
        }

        let (headers, tail) = buffer.split_at(header_length);

        Ok(Self {
            init_format: format,
            format: final_format,
            data: headers.into_cow(),
            error,
            next: T::create_from(
                object,
                tail,
                data_length - header_length,
                message_length - header_length,
                md,
                gmo,
                final_format,
                warning,
            )?,
            message_length,
        })
    }

    fn payload(&self) -> &Self::Payload {
        self.next.payload()
    }

    fn into_payload(self) -> Self::Payload {
        self.next.into_payload()
    }
}

impl<'a, T: GetMessage<'a>> GetMessage<'a> for Mqmd<T> {
    type Error = T::Error;
    type Payload = T::Payload;

    fn create_from<C: Conn>(
        object: &Object<C>,
        data: impl Buffer<'a>,
        data_length: usize,
        message_length: usize,
        md: &MqStruct<'static, sys::MQMD2>,
        gmo: &MqStruct<sys::MQGMO>,
        format: MessageFormat,
        warning: Option<types::Warning>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            mqmd: md.clone(),
            next: T::create_from(object, data, data_length, message_length, md, gmo, format, warning)?,
        })
    }

    fn max_data_size() -> Option<NonZero<usize>> {
        T::max_data_size()
    }

    fn apply_mqget(md: &mut MqStruct<sys::MQMD2>, gmo: &mut MqStruct<sys::MQGMO>) {
        T::apply_mqget(md, gmo);
    }

    fn payload(&self) -> &Self::Payload {
        self.next.payload()
    }

    fn into_payload(self) -> Self::Payload {
        self.next.into_payload()
    }
}

impl<T> Deref for Mqmd<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.next
    }
}

impl<C: Conn> Object<C> {
    pub fn get_message<'a, T: GetMessage<'a>>(
        &self,
        options: impl GetOptions,
        buffer: impl Buffer<'a>,
    ) -> ResultCompErr<Option<T>, T::Error> {
        let mut buffer = buffer;
        let write_area = match T::max_data_size() {
            Some(max_len) => &mut buffer.as_mut()[..max_len.into()],
            None => buffer.as_mut(),
        };

        let mut md = MqStruct::default();
        let mut mqgmo = MqStruct::new(sys::MQGMO {
            Version: sys::MQGMO_VERSION_4,
            ..sys::MQGMO::default()
        });

        options.apply_mqget(&mut md, &mut mqgmo);
        T::apply_mqget(&mut md, &mut mqgmo);

        let get_result = match self
            .connection()
            .mq()
            .mqget(
                self.connection().handle(),
                self.handle(),
                Some(&mut *md),
                &mut mqgmo,
                write_area,
            )
            .map_completion(|length| {
                (
                    length,
                    match mqgmo.ReturnedLength {
                        sys::MQRL_UNDEFINED => cmp::min(
                            write_area
                                .len()
                                .try_into()
                                .expect("length of buffer must fit in positive i32"),
                            length,
                        ),
                        returned_length => returned_length,
                    },
                )
            }) {
            Err(Error(cc, _, rc)) if cc == sys::MQCC_FAILED && rc == sys::MQRC_NO_MSG_AVAILABLE => Ok(Completion::new(None)),
            other => other.map_completion(Some),
        }?;

        Ok(match get_result {
            Completion(Some((message_length, data_length)), warning) => Completion(
                Some(T::create_from(
                    self,
                    buffer,
                    data_length.try_into().expect("length within positive usize range"),
                    message_length.try_into().expect("length within positive usize range"),
                    &md,
                    &mqgmo,
                    MessageFormat {
                        ccsid: md.CodedCharSetId,
                        encoding: MqMask::from(md.Encoding),
                        format: TextEnc::Ascii(unsafe { transmute::<[i8; 8], Fmt>(md.Format) }),
                    },
                    warning,
                )?),
                warning,
            ),
            comp => comp.map(|_| None),
        })
    }
}
