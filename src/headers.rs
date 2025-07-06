use std::{
    ffi::CStr,
    fmt::{Debug, Display},
    mem, ptr,
};

use libmqm_sys as mq;
use maybe_owned::MaybeOwned;

use super::{
    StrCcsid, StringCcsid,
    encoding::{ascii7_ebcdic, ebcdic_ascii7},
    types::{Fmt, MessageFormat, StrucId},
};
use crate::{CCSID, MqChar, constants, conversion, types};

/// Copy a Cstr to an array of length N (const)
const fn cstr_array<const N: usize>(mqi: &CStr) -> MqChar<N> {
    let mut i = 0;
    let bytes = mqi.to_bytes();
    let mut result = [0; N];
    #[expect(clippy::cast_possible_wrap, reason = "Treating MQCHAR as always positive is desired here")]
    while i < N {
        result[i] = bytes[i] as types::MQCHAR;
        i += 1;
    }
    result
}

const INTEGER_NATIVE_MASK: types::MQENC = constants::MQENC_NATIVE.intersection(constants::MQENC_INTEGER_MASK);

pub mod fmt {
    use libmqm_sys as mq;

    use super::cstr_array;
    use crate::types::Fmt;

    pub const MQFMT_NONE: Fmt = cstr_array(mq::MQFMT_NONE);
    pub const MQFMT_STRING: Fmt = cstr_array(mq::MQFMT_STRING);
    pub const MQFMT_ADMIN: Fmt = cstr_array(mq::MQFMT_ADMIN);

    pub const MQFMT_AMQP: Fmt = cstr_array(mq::MQFMT_AMQP);
    pub const MQFMT_CHANNEL_COMPLETED: Fmt = cstr_array(mq::MQFMT_CHANNEL_COMPLETED);
    pub const MQFMT_CICS: Fmt = cstr_array(mq::MQFMT_CICS);
    pub const MQFMT_COMMAND_1: Fmt = cstr_array(mq::MQFMT_COMMAND_1);
    pub const MQFMT_COMMAND_2: Fmt = cstr_array(mq::MQFMT_COMMAND_2);
    pub const MQFMT_DEAD_LETTER_HEADER: Fmt = cstr_array(mq::MQFMT_DEAD_LETTER_HEADER);
    pub const MQFMT_DIST_HEADER: Fmt = cstr_array(mq::MQFMT_DIST_HEADER);
    pub const MQFMT_EMBEDDED_PCF: Fmt = cstr_array(mq::MQFMT_EMBEDDED_PCF);
    pub const MQFMT_EVENT: Fmt = cstr_array(mq::MQFMT_EVENT);
    pub const MQFMT_IMS: Fmt = cstr_array(mq::MQFMT_IMS);
    pub const MQFMT_IMS_VAR_STRING: Fmt = cstr_array(mq::MQFMT_IMS_VAR_STRING);
    pub const MQFMT_MD_EXTENSION: Fmt = cstr_array(mq::MQFMT_MD_EXTENSION);
    pub const MQFMT_PCF: Fmt = cstr_array(mq::MQFMT_PCF);
    pub const MQFMT_REF_MSG_HEADER: Fmt = cstr_array(mq::MQFMT_REF_MSG_HEADER);
    pub const MQFMT_RF_HEADER: Fmt = cstr_array(mq::MQFMT_RF_HEADER);
    pub const MQFMT_RF_HEADER_1: Fmt = cstr_array(mq::MQFMT_RF_HEADER_1);
    pub const MQFMT_RF_HEADER_2: Fmt = cstr_array(mq::MQFMT_RF_HEADER_2);
    pub const MQFMT_TRIGGER: Fmt = cstr_array(mq::MQFMT_TRIGGER);
    pub const MQFMT_WORK_INFO_HEADER: Fmt = cstr_array(mq::MQFMT_WORK_INFO_HEADER);
    pub const MQFMT_XMIT_Q_HEADER: Fmt = cstr_array(mq::MQFMT_XMIT_Q_HEADER);
}

#[derive(derive_more::Error, derive_more::Display, Debug, Clone)]
pub enum HeaderError {
    #[display("Unexpected STRUC_ID: {:?} or version", _0)]
    #[error(ignore)]
    UnexpectedStruc(StrucId),
    #[display(
        "Length of remaining data was insufficient for expected header: {} exceeds data remaining ({})",
        _0,
        _1
    )]
    #[error(ignore)]
    DataTruncated(usize, usize),
    #[display("Length provided by header is malformed: {}", _0)]
    #[error(ignore)]
    MalformedLength(types::MQLONG),
    #[display(
        "Length provided by header is not within the data offset: {} not within offset {}..{}",
        _0,
        _1,
        _2
    )]
    #[error(ignore)]
    StrucLengthOffsetMismatch(usize, usize, usize),
}

#[derive(Debug, Clone)]
pub enum Header<'a> {
    Dlh(EncodedHeader<'a, mq::MQDLH>),
    Dh(EncodedHeader<'a, mq::MQDH>),
    Iih(EncodedHeader<'a, mq::MQIIH>),
    Rfh2(EncodedHeader<'a, mq::MQRFH2>),
    Rfh(EncodedHeader<'a, mq::MQRFH>),
    Cih(EncodedHeader<'a, mq::MQCIH>),
}

pub type NextHeader<'a> = (Header<'a>, &'a [u8], usize, MessageFormat);

#[derive(Debug, Clone)]
pub struct EncodedHeader<'a, T: ChainedHeader> {
    pub ccsid: CCSID,
    pub encoding: types::MQENC,
    pub raw_header: MaybeOwned<'a, T>,
    pub tail: &'a [u8],
}

impl Header<'_> {
    pub const fn iter(data: &[u8], format: MessageFormat) -> HeaderIter<'_> {
        HeaderIter {
            format,
            data,
            in_error: false,
        }
    }
}

impl<T: ChainedHeader> EncodedHeader<'_, T> {
    #[must_use]
    pub fn next_ccsid(&self) -> CCSID {
        let next_ccsid = self.native_mqlong(T::next_raw_ccsid(&self.raw_header));
        if next_ccsid == 0 { self.ccsid } else { CCSID(next_ccsid) }
    }

    #[must_use]
    pub fn next_encoding(&self) -> types::MQENC {
        let next_encoding = self.native_mqlong(T::next_raw_encoding(&self.raw_header)).into();
        if next_encoding == 0 { self.encoding } else { next_encoding }
    }

    #[must_use]
    pub fn next_format(&self) -> TextEnc<Fmt> {
        if self.ccsid.is_ebcdic().unwrap_or(false) {
            TextEnc::Ebcdic(T::next_raw_format(&self.raw_header))
        } else {
            TextEnc::Ascii(T::next_raw_format(&self.raw_header))
        }
    }

    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> Result<usize, HeaderError> {
        T::raw_struc_length(&self.raw_header).map_or(Ok(mem::size_of::<T>()), |length| {
            let ln = self.native_mqlong(length);
            T::validate_length(ln)?;
            ln.try_into().map_err(|_| HeaderError::MalformedLength(ln))
        })
    }

    #[must_use]
    fn struc_matches(&self) -> bool {
        swap_to_native(T::raw_version(&self.raw_header), self.encoding.contains(INTEGER_NATIVE_MASK)) == T::VERSION && {
            let struc_id = T::raw_struc_id(&self.raw_header);
            let ebcdic = self.ccsid.is_ebcdic().unwrap_or(false);
            (ebcdic && struc_id == T::STRUC_ID_EBCDIC) || (!ebcdic && struc_id == T::STRUC_ID_ASCII)
        }
    }

    #[must_use]
    fn fmt_matches(fmt: TextEnc<Fmt>) -> bool {
        match fmt {
            TextEnc::Ascii(fmt) => fmt == T::FMT_ASCII,
            TextEnc::Ebcdic(fmt) => fmt == T::FMT_EBCDIC,
        }
    }

    #[must_use]
    const fn native_mqlong(&self, value: types::MQLONG) -> types::MQLONG {
        swap_to_native(value, self.encoding.contains(INTEGER_NATIVE_MASK))
    }
}

pub trait ChainedHeader: Sized {
    const FMT_ASCII: Fmt;
    const FMT_EBCDIC: Fmt;
    const STRUC_ID_ASCII: StrucId;
    const STRUC_ID_EBCDIC: StrucId;
    const VERSION: types::MQLONG;

    #[must_use]
    fn next_raw_ccsid(&self) -> types::MQLONG;
    #[must_use]
    fn next_raw_encoding(&self) -> types::MQLONG;
    #[must_use]
    fn next_raw_format(&self) -> Fmt;
    #[must_use]
    fn raw_struc_id(&self) -> StrucId;
    #[must_use]
    fn raw_version(&self) -> types::MQLONG;

    fn validate_length(length: types::MQLONG) -> Result<(), HeaderError> {
        #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let struc_len = mem::size_of::<Self>() as _;
        if length < struc_len {
            Err(HeaderError::MalformedLength(length))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn raw_struc_length(&self) -> Option<types::MQLONG> {
        None
    }

    fn header(encoded: EncodedHeader<Self>) -> Header;
    fn from_header(header: Header) -> Option<EncodedHeader<Self>>;
}

#[derive(Clone, Copy)]
pub enum TextEnc<T> {
    Ascii(T),
    Ebcdic(T),
}

impl<const N: usize> Debug for TextEnc<MqChar<N>> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Ascii(ascii) => f
                .debug_tuple("Ascii")
                .field(&String::from_utf8_lossy(conversion::slice_mqchar_to_byte(ascii)))
                .finish(),
            Self::Ebcdic(ebcdic) => {
                let ascii = ebcdic_ascii7(ebcdic);
                f.debug_tuple("Ebcdic")
                    .field(&String::from_utf8_lossy(conversion::slice_mqchar_to_byte(&ascii)))
                    .finish()
            }
        }
    }
}

impl<const N: usize> Display for TextEnc<MqChar<N>> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Ascii(ascii) => std::fmt::Display::fmt(&String::from_utf8_lossy(conversion::slice_mqchar_to_byte(ascii)), f),
            Self::Ebcdic(ebcdic) => {
                let ascii = ebcdic_ascii7(ebcdic);
                std::fmt::Display::fmt(&String::from_utf8_lossy(conversion::slice_mqchar_to_byte(&ascii)), f)
            }
        }
    }
}

impl<T> AsRef<T> for TextEnc<T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::Ascii(text) | Self::Ebcdic(text) => text,
        }
    }
}

impl<const N: usize> From<TextEnc<Self>> for MqChar<N> {
    fn from(value: TextEnc<Self>) -> Self {
        match value {
            TextEnc::Ascii(fmt) | TextEnc::Ebcdic(fmt) => fmt,
        }
    }
}

impl<const N: usize> PartialEq for TextEnc<MqChar<N>> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ascii(a), Self::Ascii(a2)) | (Self::Ebcdic(a), Self::Ebcdic(a2)) => a == a2,
            (Self::Ascii(a), Self::Ebcdic(e)) | (Self::Ebcdic(e), Self::Ascii(a)) => &ebcdic_ascii7(e) == a,
        }
    }
}

impl<const N: usize> PartialEq<MqChar<N>> for TextEnc<MqChar<N>> {
    fn eq(&self, other: &MqChar<N>) -> bool {
        self.as_ref() == other
    }
}

impl<const N: usize> TextEnc<MqChar<N>> {
    #[must_use]
    pub const fn into_ascii(self) -> Self {
        match self {
            Self::Ascii(_) => self,
            Self::Ebcdic(fmt) => Self::Ascii(ebcdic_ascii7(&fmt)),
        }
    }

    #[must_use]
    pub const fn into_ebcdic(self) -> Self {
        match self {
            Self::Ascii(fmt) => Self::Ebcdic(ascii7_ebcdic(&fmt)),
            Self::Ebcdic(_) => self,
        }
    }
}

fn parse_header<'a, T: ChainedHeader + 'a>(
    data: &'a [u8],
    next_ccsid: CCSID,
    next_encoding: types::MQENC,
) -> Result<NextHeader<'a>, HeaderError> {
    let struc_len = mem::size_of::<T>();
    if struc_len > data.len() {
        Err(HeaderError::DataTruncated(struc_len, data.len()))?;
    }

    let (unaligned, _, _) = unsafe { data.align_to::<T>() };

    let header = EncodedHeader::<T> {
        ccsid: next_ccsid,
        encoding: next_encoding,
        raw_header: if unaligned.is_empty() {
            MaybeOwned::Borrowed(unsafe { &*data.as_ptr().cast() })
        } else {
            MaybeOwned::Owned(unsafe { ptr::read_unaligned(data.as_ptr().cast()) })
        },
        tail: &[],
    };

    if !header.struc_matches() {
        Err(HeaderError::UnexpectedStruc(T::raw_struc_id(&header.raw_header)))?;
    }

    let total_len = header.len()?;
    if total_len > data.len() || total_len < struc_len {
        Err(HeaderError::StrucLengthOffsetMismatch(total_len, struc_len, data.len()))?;
    }
    let next_data = &data[total_len..];
    let next_fmt = MessageFormat {
        ccsid: header.next_ccsid(),
        encoding: header.next_encoding(),
        fmt: header.next_format(),
    };
    let encoded_header = EncodedHeader::<T> {
        tail: &data[struc_len..total_len],
        ..header
    };

    Ok((T::header(encoded_header), next_data, total_len, next_fmt))
}

#[inline]
#[must_use]
const fn swap_to_native(value: types::MQLONG, native: bool) -> types::MQLONG {
    if native { value } else { value.swap_bytes() }
}

impl ChainedHeader for mq::MQDH {
    const FMT_ASCII: Fmt = cstr_array(mq::MQFMT_DIST_HEADER);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(mq::MQDH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);
    const VERSION: types::MQLONG = mq::MQDH_VERSION_1;

    fn next_raw_ccsid(&self) -> types::MQLONG {
        self.CodedCharSetId
    }

    fn next_raw_encoding(&self) -> types::MQLONG {
        self.Encoding
    }

    fn next_raw_format(&self) -> Fmt {
        unsafe { *(&raw const self.Format).cast() }
    }

    fn raw_struc_id(&self) -> StrucId {
        unsafe { *(&raw const self.StrucId).cast() }
    }

    fn raw_version(&self) -> types::MQLONG {
        self.Version
    }

    fn raw_struc_length(&self) -> Option<types::MQLONG> {
        Some(self.StrucLength)
    }

    fn header(encoded: EncodedHeader<Self>) -> Header {
        Header::Dh(encoded)
    }

    fn from_header(header: Header) -> Option<EncodedHeader<Self>> {
        match header {
            Header::Dh(dh) => Some(dh),
            _ => None,
        }
    }
}

impl ChainedHeader for mq::MQCIH {
    const FMT_ASCII: Fmt = cstr_array(mq::MQFMT_CICS);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(mq::MQCIH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);
    const VERSION: types::MQLONG = mq::MQCIH_VERSION_2;

    fn next_raw_ccsid(&self) -> types::MQLONG {
        self.CodedCharSetId
    }

    fn next_raw_encoding(&self) -> types::MQLONG {
        self.Encoding
    }

    fn next_raw_format(&self) -> Fmt {
        unsafe { *(&raw const self.Format).cast() }
    }

    fn raw_struc_id(&self) -> StrucId {
        unsafe { *(&raw const self.StrucId).cast() }
    }

    fn raw_version(&self) -> types::MQLONG {
        self.Version
    }

    fn raw_struc_length(&self) -> Option<types::MQLONG> {
        Some(self.StrucLength)
    }

    fn header(encoded: EncodedHeader<Self>) -> Header {
        Header::Cih(encoded)
    }

    fn from_header(header: Header) -> Option<EncodedHeader<Self>> {
        match header {
            Header::Cih(cih) => Some(cih),
            _ => None,
        }
    }
}

impl ChainedHeader for mq::MQDLH {
    const FMT_ASCII: Fmt = cstr_array(mq::MQFMT_DEAD_LETTER_HEADER);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(mq::MQDLH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);
    const VERSION: types::MQLONG = mq::MQDLH_VERSION_1;

    fn next_raw_ccsid(&self) -> types::MQLONG {
        self.CodedCharSetId
    }

    fn next_raw_encoding(&self) -> types::MQLONG {
        self.Encoding
    }

    fn next_raw_format(&self) -> Fmt {
        unsafe { *(&raw const self.Format).cast() }
    }

    fn raw_struc_id(&self) -> StrucId {
        unsafe { *(&raw const self.StrucId).cast() }
    }

    fn raw_version(&self) -> types::MQLONG {
        self.Version
    }

    fn header(encoded: EncodedHeader<Self>) -> Header {
        Header::Dlh(encoded)
    }

    fn from_header(header: Header) -> Option<EncodedHeader<Self>> {
        match header {
            Header::Dlh(dlh) => Some(dlh),
            _ => None,
        }
    }
}

impl ChainedHeader for mq::MQIIH {
    const FMT_ASCII: Fmt = cstr_array(mq::MQFMT_IMS);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(mq::MQIIH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);
    const VERSION: types::MQLONG = mq::MQIIH_VERSION_1;

    fn next_raw_ccsid(&self) -> types::MQLONG {
        self.CodedCharSetId
    }

    fn next_raw_encoding(&self) -> types::MQLONG {
        self.Encoding
    }

    fn next_raw_format(&self) -> Fmt {
        unsafe { *(&raw const self.Format).cast() }
    }

    fn raw_struc_id(&self) -> StrucId {
        unsafe { *(&raw const self.StrucId).cast() }
    }

    fn raw_version(&self) -> types::MQLONG {
        self.Version
    }

    fn header(encoded: EncodedHeader<Self>) -> Header {
        Header::Iih(encoded)
    }

    fn from_header(header: Header) -> Option<EncodedHeader<Self>> {
        match header {
            Header::Iih(iih) => Some(iih),
            _ => None,
        }
    }
}

impl ChainedHeader for mq::MQRFH2 {
    const FMT_ASCII: Fmt = cstr_array(mq::MQFMT_RF_HEADER_2);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(mq::MQRFH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);
    const VERSION: types::MQLONG = mq::MQRFH_VERSION_2;

    fn next_raw_ccsid(&self) -> types::MQLONG {
        self.CodedCharSetId
    }

    fn next_raw_encoding(&self) -> types::MQLONG {
        self.Encoding
    }

    fn next_raw_format(&self) -> Fmt {
        unsafe { *(&raw const self.Format).cast() }
    }

    fn raw_struc_id(&self) -> StrucId {
        unsafe { *(&raw const self.StrucId).cast() }
    }

    fn raw_version(&self) -> types::MQLONG {
        self.Version
    }

    fn raw_struc_length(&self) -> Option<types::MQLONG> {
        Some(self.StrucLength)
    }

    fn header(encoded: EncodedHeader<Self>) -> Header {
        Header::Rfh2(encoded)
    }

    fn from_header(header: Header) -> Option<EncodedHeader<Self>> {
        match header {
            Header::Rfh2(rfh2) => Some(rfh2),
            _ => None,
        }
    }

    fn validate_length(length: types::MQLONG) -> Result<(), HeaderError> {
        #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let struc_len = mem::size_of::<Self>() as _;
        // +4 bytes for MQLONG length field
        if length == struc_len || length >= struc_len + 4 {
            Ok(())
        } else {
            Err(HeaderError::MalformedLength(length))
        }
    }
}

impl<'a> EncodedHeader<'a, mq::MQRFH2> {
    #[must_use]
    pub fn name_value_data(&self) -> StrCcsid<'a> {
        StringCcsid::new(
            conversion::slice_byte_to_mqchar(&self.tail[4..]), // Exclude 4 bytes for the length prelude
            CCSID(self.native_mqlong(self.raw_header.NameValueCCSID)),
            self.encoding.contains(constants::MQENC_INTEGER_REVERSED),
        )
    }
}

impl ChainedHeader for mq::MQRFH {
    const FMT_ASCII: Fmt = cstr_array(mq::MQFMT_RF_HEADER_1);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(mq::MQRFH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);
    const VERSION: types::MQLONG = mq::MQRFH_VERSION_1;

    fn next_raw_ccsid(&self) -> types::MQLONG {
        self.CodedCharSetId
    }

    fn next_raw_encoding(&self) -> types::MQLONG {
        self.Encoding
    }

    fn next_raw_format(&self) -> Fmt {
        unsafe { *(&raw const self.Format).cast() }
    }

    fn raw_struc_id(&self) -> StrucId {
        unsafe { *(&raw const self.StrucId).cast() }
    }

    fn raw_version(&self) -> types::MQLONG {
        self.Version
    }

    fn raw_struc_length(&self) -> Option<types::MQLONG> {
        Some(self.StrucLength)
    }

    fn header(encoded: EncodedHeader<Self>) -> Header {
        Header::Rfh(encoded)
    }

    fn from_header(header: Header) -> Option<EncodedHeader<Self>> {
        match header {
            Header::Rfh(rfh) => Some(rfh),
            _ => None,
        }
    }
}

impl<'a> EncodedHeader<'a, mq::MQRFH> {
    #[must_use]
    pub const fn name_value_data(&self) -> StrCcsid<'a> {
        StringCcsid::new(
            conversion::slice_byte_to_mqchar(self.tail),
            self.ccsid,
            self.encoding.contains(constants::MQENC_INTEGER_REVERSED),
        )
    }
}

#[must_use]
pub struct HeaderIter<'a> {
    format: MessageFormat,
    data: &'a [u8],
    in_error: bool,
}

type HeaderItem<'a> = Result<(Header<'a>, usize, MessageFormat), HeaderError>;

impl<'a> Iterator for HeaderIter<'a> {
    type Item = HeaderItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.in_error {
            return None;
        }
        match next_header(self.data, &self.format).transpose()? {
            Ok((header, data, length, format)) => {
                self.data = data;
                self.format = format;
                Some(Ok((header, length, format)))
            }
            Err(error) => {
                self.in_error = true;
                Some(Err(error))
            }
        }
    }
}

fn next_header<'a>(data: &'a [u8], next_format: &MessageFormat) -> Result<Option<NextHeader<'a>>, HeaderError> {
    Ok(if EncodedHeader::<mq::MQRFH2>::fmt_matches(next_format.fmt) {
        Some(parse_header::<mq::MQRFH2>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<mq::MQRFH>::fmt_matches(next_format.fmt) {
        Some(parse_header::<mq::MQRFH>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<mq::MQIIH>::fmt_matches(next_format.fmt) {
        Some(parse_header::<mq::MQIIH>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<mq::MQCIH>::fmt_matches(next_format.fmt) {
        Some(parse_header::<mq::MQCIH>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<mq::MQDLH>::fmt_matches(next_format.fmt) {
        Some(parse_header::<mq::MQDLH>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<mq::MQDH>::fmt_matches(next_format.fmt) {
        Some(parse_header::<mq::MQDH>(data, next_format.ccsid, next_format.encoding)?)
    } else {
        None
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{mem::transmute, slice::from_raw_parts};

    use libmqm_default as default;

    use super::*;
    use crate::{
        constants,
        headers::{EncodedHeader, Header, HeaderError},
        types::MessageFormat,
    };

    const NEXT_DEAD: MessageFormat = MessageFormat {
        ccsid: CCSID(1208),
        encoding: constants::MQENC_NATIVE,
        fmt: TextEnc::Ascii(mq::MQDLH::FMT_ASCII),
    };

    const NEXT_RFH2: MessageFormat = MessageFormat {
        ccsid: CCSID(1208),
        encoding: constants::MQENC_NATIVE,
        fmt: TextEnc::Ebcdic(mq::MQRFH2::FMT_EBCDIC),
    };

    const NEXT_STRING: MessageFormat = MessageFormat {
        ccsid: CCSID(1208),
        encoding: constants::MQENC_NATIVE,
        fmt: TextEnc::Ascii(fmt::MQFMT_STRING),
    };

    #[test]
    pub fn not_header() {
        let header = next_header(b"X", &NEXT_STRING);
        assert!(matches!(header, Ok(None)));
    }

    #[test]
    pub fn too_short() {
        let header = next_header(b"X", &NEXT_DEAD);
        assert!(matches!(header, Err(HeaderError::DataTruncated(_, 1))));
    }

    #[test]
    pub fn dlh() {
        let dlh = unsafe { transmute::<mq::MQDLH, [u8; mq::MQDLH_LENGTH_1]>(default::MQDLH_DEFAULT) };
        let header = next_header(dlh.as_slice(), &NEXT_DEAD);

        assert!(matches!(header, Ok(Some((Header::Dlh(_), ..)))));
    }

    #[test]
    pub fn rfh2() {
        let rfh2 = default::MQRFH2_DEFAULT;
        let rfh2_data = unsafe { transmute::<mq::MQRFH2, [u8; mq::MQRFH2_CURRENT_LENGTH]>(rfh2) };
        let header = next_header(rfh2_data.as_slice(), &NEXT_RFH2);

        assert!(matches!(header, Ok(Some((Header::Rfh2(_), ..)))));
    }

    #[test]
    pub fn header_iter() {
        const TOTAL_LENGTH: usize = mq::MQDLH_LENGTH_1 + mq::MQRFH2_LENGTH_2;
        let mut data: [u8; TOTAL_LENGTH] = [0; TOTAL_LENGTH];
        let mut dlh = default::MQDLH_DEFAULT;
        let rfh2 = default::MQRFH2_DEFAULT;
        dlh.Format = mq::MQRFH2::FMT_ASCII;
        dlh.CodedCharSetId = 1208;
        dlh.Encoding = mq::MQENC_NATIVE;
        let dlh_ptr = &raw const dlh;
        let rfh2_ptr = &raw const rfh2;
        data[..mq::MQDLH_LENGTH_1].copy_from_slice(unsafe { from_raw_parts(dlh_ptr.cast(), mq::MQDLH_LENGTH_1) });
        data[mq::MQDLH_LENGTH_1..].copy_from_slice(unsafe { from_raw_parts(rfh2_ptr.cast(), mq::MQRFH2_LENGTH_2) });

        let headers = Header::iter(
            data.as_slice(),
            MessageFormat {
                ccsid: CCSID(1208),
                encoding: constants::MQENC_NATIVE,
                fmt: TextEnc::Ascii(mq::MQDLH::FMT_ASCII),
            },
        );

        let headers_vec: Vec<_> = headers.collect();

        assert!(matches!(headers_vec[0], Ok((Header::Dlh(_), ..))));
        assert!(matches!(headers_vec[1], Ok((Header::Rfh2(EncodedHeader { .. }), ..))));
        assert_eq!(headers_vec.len(), 2);
    }
}
