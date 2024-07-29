use std::{ffi::CStr, fmt::Display, mem, ops::Deref, ptr};

use crate::{core::values::MQENC, sys, MqMask};

use super::{
    encoding::{ascii7_ebcdic, ebcdic_ascii7, is_ebcdic},
    types::{Fmt, MessageFormat, StrucId},
};

/// Copy a Cstr to an array of length N (const)
const fn cstr_array<const N: usize>(mqi: &CStr) -> [u8; N] {
    let mut i = 0;
    let bytes = mqi.to_bytes();
    let mut result = [0; N];
    while i < N {
        result[i] = bytes[i];
        i += 1;
    }
    result
}

pub mod fmt {
    use crate::{sys, types::Fmt};
    use super::cstr_array;

    pub const MQFMT_NONE: Fmt = cstr_array(sys::MQFMT_NONE);
    pub const MQFMT_STRING: Fmt = cstr_array(sys::MQFMT_STRING);
}

#[derive(thiserror::Error, Debug)]
pub enum HeaderError {
    #[error("Unexpected STRUC_ID: {:?}", .0)]
    UnexpectedStrucId(StrucId),
    #[error("Length of remaining data was insufficient for expected header: {} exceeds data remaining ({})", .0, .1)]
    DataTruncated(usize, usize),
    #[error("Length provided by header is malformed: {}", .0)]
    MalformedLength(sys::MQLONG),
    #[error("Length provided by header is not within the data offset: {} not within offset {}..{}", .0, .1, .2)]
    StrucLengthOffsetMismatch(usize, usize, usize),
}

#[derive(Debug, Clone)]
pub enum Header<'a> {
    Dlh(EncodedHeader<'a, sys::MQDLH>),
    Dh(EncodedHeader<'a, sys::MQDH>, &'a [u8]),
    Iih(EncodedHeader<'a, sys::MQIIH>),
    Rfh2(EncodedHeader<'a, sys::MQRFH2>, &'a [u8]),
    Rfh(EncodedHeader<'a, sys::MQRFH>, &'a [u8]),
    Cih(EncodedHeader<'a, sys::MQCIH>, &'a [u8]),
}

pub type NextHeader<'a> = (Header<'a>, &'a [u8], MessageFormat<TextEnc<Fmt>>);

#[derive(Debug, Clone, Copy)]
pub struct EncodedHeader<'a, T> {
    pub ccsid: sys::MQLONG,
    pub encoding: MqMask<MQENC>,
    pub raw_header: &'a T,
}

impl<'a, T: ChainedHeader> EncodedHeader<'a, T> {
    #[must_use]
    pub fn next_ccsid(&self) -> sys::MQLONG {
        swap_non_native(self.raw_header.next_ccsid(), self.encoding & sys::MQENC_NATIVE != 0)
    }

    #[must_use]
    pub fn next_encoding(&self) -> MqMask<MQENC> {
        swap_non_native(self.raw_header.next_encoding(), self.encoding & sys::MQENC_NATIVE != 0).into()
    }

    #[must_use]
    pub fn next_format(&self) -> TextEnc<Fmt> {
        if is_ebcdic(self.ccsid).unwrap_or(false) {
            TextEnc::Ebcdic(self.raw_header.next_format())
        } else {
            TextEnc::Ascii(self.raw_header.next_format())
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> Result<usize, HeaderError> {
        self.raw_header.struc_length().map_or(Ok(mem::size_of::<T>()), |length| {
            length.try_into().map_err(|_| HeaderError::MalformedLength(length))
        })
    }

    #[must_use]
    fn fmt_matches(fmt: TextEnc<Fmt>) -> bool {
        match fmt {
            TextEnc::Ascii(fmt) => fmt == T::FMT_ASCII,
            TextEnc::Ebcdic(fmt) => fmt == T::FMT_EBCDIC,
        }
    }

    #[must_use]
    fn struc_id_matches(&self) -> bool {
        let struc_id = self.raw_header.struc_id();
        let ebcdic = is_ebcdic(self.ccsid).unwrap_or(false);
        (ebcdic && struc_id == T::STRUC_ID_EBCDIC) || (!ebcdic && struc_id == T::STRUC_ID_ASCII)
    }
}

pub trait ChainedHeader: Sized {
    const FMT_ASCII: Fmt;
    const FMT_EBCDIC: Fmt;
    const STRUC_ID_ASCII: StrucId;
    const STRUC_ID_EBCDIC: StrucId;

    #[must_use]
    fn next_ccsid(&self) -> sys::MQLONG;
    #[must_use]
    fn next_encoding(&self) -> sys::MQLONG;
    #[must_use]
    fn next_format(&self) -> Fmt;
    #[must_use]
    fn struc_id(&self) -> StrucId;

    #[allow(unused_variables)]
    #[inline]
    fn struc_length(&self) -> Option<sys::MQLONG> {
        None
    }

    fn header<'a>(encoded: EncodedHeader<'a, Self>, data: &'a [u8]) -> Header<'a>;
}

#[derive(Debug, Clone, Copy)]
pub enum TextEnc<T> {
    Ascii(T),
    Ebcdic(T),
}

impl<const N: usize> Display for TextEnc<[u8; N]> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextEnc::Ascii(fmt) => String::from_utf8_lossy(fmt.as_slice()).fmt(f),
            TextEnc::Ebcdic(fmt) => {
                let ascii = ebcdic_ascii7(fmt);
                String::from_utf8_lossy(&ascii).fmt(f)
            }
        }
    }
}

impl<T> AsRef<T> for TextEnc<T> {
    fn as_ref(&self) -> &T {
        match self {
            TextEnc::Ascii(text) | TextEnc::Ebcdic(text) => text,
        }
    }
}

impl<const N: usize> From<TextEnc<[u8; N]>> for [u8; N] {
    fn from(value: TextEnc<[u8; N]>) -> Self {
        match value {
            TextEnc::Ascii(fmt) | TextEnc::Ebcdic(fmt) => fmt,
        }
    }
}

impl<const N: usize> PartialEq for TextEnc<[u8; N]> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ascii(a), Self::Ascii(a2)) | (Self::Ebcdic(a), Self::Ebcdic(a2)) => a == a2,
            (Self::Ascii(a), Self::Ebcdic(e)) | (Self::Ebcdic(e), Self::Ascii(a)) => &ebcdic_ascii7(e) == a,
        }
    }
}

impl<const N: usize> TextEnc<[u8; N]> {
    pub fn into_ascii(self) -> Self {
        match self {
            TextEnc::Ascii(_) => self,
            TextEnc::Ebcdic(fmt) => TextEnc::Ascii(ebcdic_ascii7(&fmt)),
        }
    }

    pub fn into_ebcdic(self) -> Self {
        match self {
            TextEnc::Ascii(fmt) => TextEnc::Ebcdic(ascii7_ebcdic(&fmt)),
            TextEnc::Ebcdic(_) => self,
        }
    }
}

impl<'a, T> EncodedHeader<'a, T> {
    pub const fn new(ccsid: sys::MQLONG, encoding: MqMask<MQENC>, header: &'a T) -> Self {
        Self {
            ccsid,
            encoding,
            raw_header: header,
        }
    }
}

impl<T> Deref for EncodedHeader<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.raw_header
    }
}

fn parse_header<'a, T: ChainedHeader + 'a>(
    data: &'a [u8],
    next_ccsid: i32,
    next_encoding: MqMask<MQENC>,
) -> Result<NextHeader<'a>, HeaderError> {
    let struc_len = mem::size_of::<T>();
    if struc_len > data.len() {
        Err(HeaderError::DataTruncated(struc_len, data.len()))?;
    }
    let header = EncodedHeader::<T>::new(next_ccsid, next_encoding, unsafe { &*((*data).as_ptr().cast()) });

    if !header.struc_id_matches() {
        Err(HeaderError::UnexpectedStrucId(header.struc_id()))?;
    }

    let total_len = header.len()?;
    if total_len > data.len() || total_len < struc_len {
        Err(HeaderError::StrucLengthOffsetMismatch(total_len, struc_len, data.len()))?;
    }
    let next_data = &data[total_len..];

    let next_fmt = MessageFormat {
        ccsid: header.next_ccsid(),
        encoding: header.next_encoding(),
        format: header.next_format(),
    };

    Ok((T::header(header, &data[struc_len..total_len]), next_data, next_fmt))
}

#[inline]
#[must_use]
const fn swap_non_native(value: sys::MQLONG, native: bool) -> sys::MQLONG {
    if native {
        value
    } else {
        value.swap_bytes()
    }
}

impl ChainedHeader for sys::MQDH {
    const FMT_ASCII: Fmt = cstr_array(sys::MQFMT_DIST_HEADER);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(sys::MQDH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);

    fn next_ccsid(&self) -> sys::MQLONG {
        self.CodedCharSetId
    }

    fn next_encoding(&self) -> sys::MQLONG {
        self.Encoding
    }

    fn next_format(&self) -> Fmt {
        unsafe { *ptr::from_ref(&self.Format).cast() }
    }

    fn struc_id(&self) -> StrucId {
        unsafe { *ptr::from_ref(&self.StrucId).cast() }
    }

    fn struc_length(&self) -> Option<sys::MQLONG> {
        Some(self.StrucLength)
    }

    fn header<'a>(encoded: EncodedHeader<'a, Self>, data: &'a [u8]) -> Header<'a> {
        Header::Dh(encoded, data)
    }
}

impl ChainedHeader for sys::MQCIH {
    const FMT_ASCII: Fmt = cstr_array(sys::MQFMT_CICS);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(sys::MQCIH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);

    fn next_ccsid(&self) -> sys::MQLONG {
        self.CodedCharSetId
    }

    fn next_encoding(&self) -> sys::MQLONG {
        self.Encoding
    }

    fn next_format(&self) -> Fmt {
        unsafe { *ptr::from_ref(&self.Format).cast() }
    }

    fn struc_id(&self) -> StrucId {
        unsafe { *ptr::from_ref(&self.StrucId).cast() }
    }

    fn struc_length(&self) -> Option<sys::MQLONG> {
        Some(self.StrucLength)
    }

    fn header<'a>(encoded: EncodedHeader<'a, Self>, data: &'a [u8]) -> Header<'a> {
        Header::Cih(encoded, data)
    }
}

impl ChainedHeader for sys::MQDLH {
    const FMT_ASCII: Fmt = cstr_array(sys::MQFMT_DEAD_LETTER_HEADER);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(sys::MQDLH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);

    fn next_ccsid(&self) -> sys::MQLONG {
        self.CodedCharSetId
    }

    fn next_encoding(&self) -> sys::MQLONG {
        self.Encoding
    }

    fn next_format(&self) -> Fmt {
        unsafe { *ptr::from_ref(&self.Format).cast() }
    }

    fn struc_id(&self) -> StrucId {
        unsafe { *ptr::from_ref(&self.StrucId).cast() }
    }

    fn header<'a>(encoded: EncodedHeader<'a, Self>, _data: &'a [u8]) -> Header<'a> {
        Header::Dlh(encoded)
    }
}

// TODO: Handle zero ccsid
impl ChainedHeader for sys::MQIIH {
    const FMT_ASCII: Fmt = cstr_array(sys::MQFMT_IMS);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(sys::MQIIH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);

    fn next_ccsid(&self) -> sys::MQLONG {
        self.CodedCharSetId
    }

    fn next_encoding(&self) -> sys::MQLONG {
        self.Encoding
    }

    fn next_format(&self) -> Fmt {
        unsafe { *ptr::from_ref(&self.Format).cast() }
    }

    fn struc_id(&self) -> StrucId {
        unsafe { *ptr::from_ref(&self.StrucId).cast() }
    }

    fn header<'a>(encoded: EncodedHeader<'a, Self>, _data: &'a [u8]) -> Header<'a> {
        Header::Iih(encoded)
    }
}

impl ChainedHeader for sys::MQRFH2 {
    const FMT_ASCII: Fmt = cstr_array(sys::MQFMT_RF_HEADER_2);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(sys::MQRFH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);

    fn next_ccsid(&self) -> sys::MQLONG {
        self.CodedCharSetId
    }

    fn next_encoding(&self) -> sys::MQLONG {
        self.Encoding
    }

    fn next_format(&self) -> Fmt {
        unsafe { *ptr::from_ref(&self.Format).cast() }
    }

    fn struc_id(&self) -> StrucId {
        unsafe { *ptr::from_ref(&self.StrucId).cast() }
    }

    fn struc_length(&self) -> Option<sys::MQLONG> {
        Some(self.StrucLength)
    }

    fn header<'a>(encoded: EncodedHeader<'a, Self>, data: &'a [u8]) -> Header<'a> {
        Header::Rfh2(encoded, data)
    }
}

impl ChainedHeader for sys::MQRFH {
    const FMT_ASCII: Fmt = cstr_array(sys::MQFMT_RF_HEADER_1);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(sys::MQRFH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);

    fn next_ccsid(&self) -> sys::MQLONG {
        self.CodedCharSetId
    }

    fn next_encoding(&self) -> sys::MQLONG {
        self.Encoding
    }

    fn next_format(&self) -> Fmt {
        unsafe { *ptr::from_ref(&self.Format).cast() }
    }

    fn struc_id(&self) -> StrucId {
        unsafe { *ptr::from_ref(&self.StrucId).cast() }
    }

    fn struc_length(&self) -> Option<sys::MQLONG> {
        Some(self.StrucLength)
    }

    fn header<'a>(encoded: EncodedHeader<'a, Self>, data: &'a [u8]) -> Header<'a> {
        Header::Rfh(encoded, data)
    }
}

pub fn next_header<'a>(data: &'a [u8], next_format: &MessageFormat<TextEnc<Fmt>>) -> Result<Option<NextHeader<'a>>, HeaderError> {
    Ok(if EncodedHeader::<sys::MQRFH2>::fmt_matches(next_format.format) {
        Some(parse_header::<sys::MQRFH2>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<sys::MQRFH>::fmt_matches(next_format.format) {
        Some(parse_header::<sys::MQRFH>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<sys::MQIIH>::fmt_matches(next_format.format) {
        Some(parse_header::<sys::MQIIH>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<sys::MQCIH>::fmt_matches(next_format.format) {
        Some(parse_header::<sys::MQCIH>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<sys::MQDLH>::fmt_matches(next_format.format) {
        Some(parse_header::<sys::MQDLH>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<sys::MQDH>::fmt_matches(next_format.format) {
        Some(parse_header::<sys::MQDH>(data, next_format.ccsid, next_format.encoding)?)
    } else {
        None
    })
}

#[cfg(test)]
mod tests {
    use std::mem::transmute;

    use crate::{
        headers::{Header, HeaderError},
        sys,
        types::{Fmt, MessageFormat},
        MqMask,
    };

    use super::{fmt, next_header, ChainedHeader, TextEnc};

    const NEXT_DEAD: MessageFormat<TextEnc<Fmt>> = MessageFormat {
        ccsid: 1208,
        encoding: MqMask::from(sys::MQENC_NATIVE),
        format: TextEnc::Ebcdic(sys::MQDLH::FMT_EBCDIC),
    };

    const NEXT_RFH2: MessageFormat<TextEnc<Fmt>> = MessageFormat {
        ccsid: 1208,
        encoding: MqMask::from(sys::MQENC_NATIVE),
        format: TextEnc::Ebcdic(sys::MQRFH2::FMT_EBCDIC),
    };

    const NEXT_STRING: MessageFormat<TextEnc<Fmt>> = MessageFormat {
        ccsid: 1208,
        encoding: MqMask::from(sys::MQENC_NATIVE),
        format: TextEnc::Ascii(fmt::MQFMT_STRING),
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
        let dlh = unsafe { transmute::<sys::MQDLH, [u8; sys::MQDLH_LENGTH_1]>(sys::MQDLH::default()) };
        let header = next_header(dlh.as_slice(), &NEXT_DEAD);

        assert!(matches!(header, Ok(Some((Header::Dlh(_), ..)))));
    }

    #[test]
    pub fn rfh2() {
        let rfh2 = sys::MQRFH2::default();
        let rfh2_data = unsafe { transmute::<sys::MQRFH2, [u8; sys::MQRFH2_CURRENT_LENGTH]>(rfh2) };
        let header = next_header(rfh2_data.as_slice(), &NEXT_RFH2);

        assert!(matches!(header, Ok(Some((Header::Rfh2(_, _), ..)))));
    }
}
