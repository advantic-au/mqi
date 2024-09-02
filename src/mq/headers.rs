use std::{
    ffi::CStr,
    fmt::{Debug, Display},
    mem,
    num::NonZeroI32,
    ptr,
};

use crate::{core::values::MQENC, sys};

use super::{
    encoding::{ascii7_ebcdic, ebcdic_ascii7, is_ebcdic},
    types::{Fmt, MessageFormat, StrucId},
    StrCcsid, StringCcsid,
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

#[allow(clippy::cast_possible_wrap)]
const INTEGER_NATIVE_MASK: sys::MQLONG = sys::MQENC_NATIVE & (sys::MQENC_INTEGER_MASK as sys::MQLONG);

pub mod fmt {
    use crate::{sys, types::Fmt};
    use super::cstr_array;

    pub const MQFMT_NONE: Fmt = cstr_array(sys::MQFMT_NONE);
    pub const MQFMT_STRING: Fmt = cstr_array(sys::MQFMT_STRING);
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
    MalformedLength(sys::MQLONG),
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
    Dlh(EncodedHeader<'a, sys::MQDLH>),
    Dh(EncodedHeader<'a, sys::MQDH>),
    Iih(EncodedHeader<'a, sys::MQIIH>),
    Rfh2(EncodedHeader<'a, sys::MQRFH2>),
    Rfh(EncodedHeader<'a, sys::MQRFH>),
    Cih(EncodedHeader<'a, sys::MQCIH>),
}

pub type NextHeader<'a> = (Header<'a>, &'a [u8], usize, MessageFormat);

#[derive(Debug, Clone, Copy)]
pub struct EncodedHeader<'a, T: ChainedHeader> {
    pub ccsid: sys::MQLONG,
    pub encoding: MQENC,
    pub raw_header: &'a T,
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
    pub fn next_ccsid(&self) -> sys::MQLONG {
        let next_ccsid = self.native_mqlong(T::next_raw_ccsid(self.raw_header));
        if next_ccsid == 0 {
            self.ccsid
        } else {
            next_ccsid
        }
    }

    #[must_use]
    pub fn next_encoding(&self) -> MQENC {
        let next_encoding = self.native_mqlong(T::next_raw_encoding(self.raw_header)).into();
        if next_encoding == 0 {
            self.encoding
        } else {
            next_encoding
        }
    }

    #[must_use]
    pub fn next_format(&self) -> TextEnc<Fmt> {
        if is_ebcdic(self.ccsid).unwrap_or(false) {
            TextEnc::Ebcdic(T::next_raw_format(self.raw_header))
        } else {
            TextEnc::Ascii(T::next_raw_format(self.raw_header))
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> Result<usize, HeaderError> {
        T::raw_struc_length(self.raw_header).map_or(Ok(mem::size_of::<T>()), |length| {
            let ln = self.native_mqlong(length);
            T::validate_length(ln)?;
            ln.try_into().map_err(|_| HeaderError::MalformedLength(ln))
        })
    }

    #[must_use]
    fn struc_matches(&self) -> bool {
        swap_to_native(T::raw_version(self.raw_header), (self.encoding & INTEGER_NATIVE_MASK) != 0) == T::VERSION && {
            let struc_id = T::raw_struc_id(self.raw_header);
            let ebcdic = is_ebcdic(self.ccsid).unwrap_or(false);
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
    fn native_mqlong(&self, value: sys::MQLONG) -> sys::MQLONG {
        swap_to_native(value, (self.encoding & INTEGER_NATIVE_MASK) != 0)
    }
}

pub trait ChainedHeader: Sized {
    const FMT_ASCII: Fmt;
    const FMT_EBCDIC: Fmt;
    const STRUC_ID_ASCII: StrucId;
    const STRUC_ID_EBCDIC: StrucId;
    const VERSION: sys::MQLONG;

    #[must_use]
    fn next_raw_ccsid(&self) -> sys::MQLONG;
    #[must_use]
    fn next_raw_encoding(&self) -> sys::MQLONG;
    #[must_use]
    fn next_raw_format(&self) -> Fmt;
    #[must_use]
    fn raw_struc_id(&self) -> StrucId;
    #[must_use]
    fn raw_version(&self) -> sys::MQLONG;

    fn validate_length(length: sys::MQLONG) -> Result<(), HeaderError> {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let struc_len = mem::size_of::<Self>() as _;
        if length < struc_len {
            Err(HeaderError::MalformedLength(length))
        } else {
            Ok(())
        }
    }

    #[allow(unused_variables)]
    #[inline]
    fn raw_struc_length(&self) -> Option<sys::MQLONG> {
        None
    }

    fn header(encoded: EncodedHeader<'_, Self>) -> Header<'_>;
    fn from_header(header: Header<'_>) -> Option<EncodedHeader<'_, Self>>;
}

#[derive(Clone, Copy)]
pub enum TextEnc<T> {
    Ascii(T),
    Ebcdic(T),
}

impl<const N: usize> Debug for TextEnc<[u8; N]> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ascii(fmt) => f
                .debug_tuple("Ascii")
                .field(&String::from_utf8_lossy(fmt.as_slice()))
                .finish(),
            Self::Ebcdic(fmt) => {
                let ascii = ebcdic_ascii7(fmt);
                f.debug_tuple("Ebcdic").field(&String::from_utf8_lossy(&ascii)).finish()
            }
        }
    }
}

impl<const N: usize> Display for TextEnc<[u8; N]> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ascii(fmt) => std::fmt::Display::fmt(&String::from_utf8_lossy(fmt.as_slice()), f),
            Self::Ebcdic(fmt) => {
                let ascii = ebcdic_ascii7(fmt);
                std::fmt::Display::fmt(&String::from_utf8_lossy(&ascii), f)
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
    next_ccsid: i32,
    next_encoding: MQENC,
) -> Result<NextHeader<'a>, HeaderError> {
    let struc_len = mem::size_of::<T>();
    if struc_len > data.len() {
        Err(HeaderError::DataTruncated(struc_len, data.len()))?;
    }

    let header = EncodedHeader::<T> {
        ccsid: next_ccsid,
        encoding: next_encoding,
        raw_header: unsafe { &*((*data).as_ptr().cast()) },
        tail: &[],
    };
    //let header = EncodedHeader::<T>::new(next_ccsid, next_encoding, &data[..struc_len]);

    if !header.struc_matches() {
        Err(HeaderError::UnexpectedStruc(T::raw_struc_id(header.raw_header)))?;
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
const fn swap_to_native(value: sys::MQLONG, native: bool) -> sys::MQLONG {
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
    const VERSION: sys::MQLONG = sys::MQDH_VERSION_1;

    fn next_raw_ccsid(&self) -> sys::MQLONG {
        self.CodedCharSetId
    }

    fn next_raw_encoding(&self) -> sys::MQLONG {
        self.Encoding
    }

    fn next_raw_format(&self) -> Fmt {
        unsafe { *ptr::from_ref(&self.Format).cast() }
    }

    fn raw_struc_id(&self) -> StrucId {
        unsafe { *ptr::from_ref(&self.StrucId).cast() }
    }

    fn raw_version(&self) -> sys::MQLONG {
        self.Version
    }

    fn raw_struc_length(&self) -> Option<sys::MQLONG> {
        Some(self.StrucLength)
    }

    fn header(encoded: EncodedHeader<'_, Self>) -> Header<'_> {
        Header::Dh(encoded)
    }

    fn from_header(header: Header<'_>) -> Option<EncodedHeader<'_, Self>> {
        match header {
            Header::Dh(dh) => Some(dh),
            _ => None,
        }
    }
}

impl ChainedHeader for sys::MQCIH {
    const FMT_ASCII: Fmt = cstr_array(sys::MQFMT_CICS);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(sys::MQCIH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);
    const VERSION: sys::MQLONG = sys::MQCIH_VERSION_2;

    fn next_raw_ccsid(&self) -> sys::MQLONG {
        self.CodedCharSetId
    }

    fn next_raw_encoding(&self) -> sys::MQLONG {
        self.Encoding
    }

    fn next_raw_format(&self) -> Fmt {
        unsafe { *ptr::from_ref(&self.Format).cast() }
    }

    fn raw_struc_id(&self) -> StrucId {
        unsafe { *ptr::from_ref(&self.StrucId).cast() }
    }

    fn raw_version(&self) -> sys::MQLONG {
        self.Version
    }

    fn raw_struc_length(&self) -> Option<sys::MQLONG> {
        Some(self.StrucLength)
    }

    fn header(encoded: EncodedHeader<'_, Self>) -> Header<'_> {
        Header::Cih(encoded)
    }

    fn from_header(header: Header<'_>) -> Option<EncodedHeader<'_, Self>> {
        match header {
            Header::Cih(cih) => Some(cih),
            _ => None,
        }
    }
}

impl ChainedHeader for sys::MQDLH {
    const FMT_ASCII: Fmt = cstr_array(sys::MQFMT_DEAD_LETTER_HEADER);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(sys::MQDLH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);
    const VERSION: sys::MQLONG = sys::MQDLH_VERSION_1;

    fn next_raw_ccsid(&self) -> sys::MQLONG {
        self.CodedCharSetId
    }

    fn next_raw_encoding(&self) -> sys::MQLONG {
        self.Encoding
    }

    fn next_raw_format(&self) -> Fmt {
        unsafe { *ptr::from_ref(&self.Format).cast() }
    }

    fn raw_struc_id(&self) -> StrucId {
        unsafe { *ptr::from_ref(&self.StrucId).cast() }
    }

    fn raw_version(&self) -> sys::MQLONG {
        self.Version
    }

    fn header(encoded: EncodedHeader<'_, Self>) -> Header<'_> {
        Header::Dlh(encoded)
    }

    fn from_header(header: Header<'_>) -> Option<EncodedHeader<'_, Self>> {
        match header {
            Header::Dlh(dlh) => Some(dlh),
            _ => None,
        }
    }
}

impl ChainedHeader for sys::MQIIH {
    const FMT_ASCII: Fmt = cstr_array(sys::MQFMT_IMS);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(sys::MQIIH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);
    const VERSION: sys::MQLONG = sys::MQIIH_VERSION_1;

    fn next_raw_ccsid(&self) -> sys::MQLONG {
        self.CodedCharSetId
    }

    fn next_raw_encoding(&self) -> sys::MQLONG {
        self.Encoding
    }

    fn next_raw_format(&self) -> Fmt {
        unsafe { *ptr::from_ref(&self.Format).cast() }
    }

    fn raw_struc_id(&self) -> StrucId {
        unsafe { *ptr::from_ref(&self.StrucId).cast() }
    }

    fn raw_version(&self) -> sys::MQLONG {
        self.Version
    }

    fn header(encoded: EncodedHeader<'_, Self>) -> Header<'_> {
        Header::Iih(encoded)
    }

    fn from_header(header: Header<'_>) -> Option<EncodedHeader<'_, Self>> {
        match header {
            Header::Iih(iih) => Some(iih),
            _ => None,
        }
    }
}

impl ChainedHeader for sys::MQRFH2 {
    const FMT_ASCII: Fmt = cstr_array(sys::MQFMT_RF_HEADER_2);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(sys::MQRFH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);
    const VERSION: sys::MQLONG = sys::MQRFH_VERSION_2;

    fn next_raw_ccsid(&self) -> sys::MQLONG {
        self.CodedCharSetId
    }

    fn next_raw_encoding(&self) -> sys::MQLONG {
        self.Encoding
    }

    fn next_raw_format(&self) -> Fmt {
        unsafe { *ptr::from_ref(&self.Format).cast() }
    }

    fn raw_struc_id(&self) -> StrucId {
        unsafe { *ptr::from_ref(&self.StrucId).cast() }
    }

    fn raw_version(&self) -> sys::MQLONG {
        self.Version
    }

    fn raw_struc_length(&self) -> Option<sys::MQLONG> {
        Some(self.StrucLength)
    }

    fn header(encoded: EncodedHeader<'_, Self>) -> Header<'_> {
        Header::Rfh2(encoded)
    }

    fn from_header(header: Header<'_>) -> Option<EncodedHeader<'_, Self>> {
        match header {
            Header::Rfh2(rfh2) => Some(rfh2),
            _ => None,
        }
    }

    fn validate_length(length: sys::MQLONG) -> Result<(), HeaderError> {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let struc_len = mem::size_of::<Self>() as _;
        // +4 bytes for MQLONG length field
        if length == struc_len || length >= struc_len + 4 {
            Ok(())
        } else {
            Err(HeaderError::MalformedLength(length))
        }
    }
}

impl<'a> EncodedHeader<'a, sys::MQRFH2> {
    #[must_use]
    pub fn name_value_data(&self) -> StrCcsid<'a> {
        StringCcsid::new(
            &self.tail[4..], // Exclude 4 bytes for the length prelude
            NonZeroI32::new(self.native_mqlong(self.raw_header.NameValueCCSID)),
            (self.encoding & sys::MQENC_INTEGER_REVERSED) != 0,
        )
    }
}

impl ChainedHeader for sys::MQRFH {
    const FMT_ASCII: Fmt = cstr_array(sys::MQFMT_RF_HEADER_1);
    const FMT_EBCDIC: Fmt = ascii7_ebcdic(&Self::FMT_ASCII);
    const STRUC_ID_ASCII: StrucId = cstr_array(sys::MQRFH_STRUC_ID);
    const STRUC_ID_EBCDIC: StrucId = ascii7_ebcdic(&Self::STRUC_ID_ASCII);
    const VERSION: sys::MQLONG = sys::MQRFH_VERSION_1;

    fn next_raw_ccsid(&self) -> sys::MQLONG {
        self.CodedCharSetId
    }

    fn next_raw_encoding(&self) -> sys::MQLONG {
        self.Encoding
    }

    fn next_raw_format(&self) -> Fmt {
        unsafe { *ptr::from_ref(&self.Format).cast() }
    }

    fn raw_struc_id(&self) -> StrucId {
        unsafe { *ptr::from_ref(&self.StrucId).cast() }
    }

    fn raw_version(&self) -> sys::MQLONG {
        self.Version
    }

    fn raw_struc_length(&self) -> Option<sys::MQLONG> {
        Some(self.StrucLength)
    }

    fn header(encoded: EncodedHeader<'_, Self>) -> Header<'_> {
        Header::Rfh(encoded)
    }

    fn from_header(header: Header<'_>) -> Option<EncodedHeader<'_, Self>> {
        match header {
            Header::Rfh(rfh) => Some(rfh),
            _ => None,
        }
    }
}

impl<'a> EncodedHeader<'a, sys::MQRFH> {
    #[must_use]
    pub fn name_value_data(&self) -> StrCcsid<'a> {
        StringCcsid::new(
            self.tail,
            NonZeroI32::new(self.ccsid),
            (self.encoding & sys::MQENC_INTEGER_REVERSED) != 0,
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
    Ok(if EncodedHeader::<sys::MQRFH2>::fmt_matches(next_format.fmt) {
        Some(parse_header::<sys::MQRFH2>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<sys::MQRFH>::fmt_matches(next_format.fmt) {
        Some(parse_header::<sys::MQRFH>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<sys::MQIIH>::fmt_matches(next_format.fmt) {
        Some(parse_header::<sys::MQIIH>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<sys::MQCIH>::fmt_matches(next_format.fmt) {
        Some(parse_header::<sys::MQCIH>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<sys::MQDLH>::fmt_matches(next_format.fmt) {
        Some(parse_header::<sys::MQDLH>(data, next_format.ccsid, next_format.encoding)?)
    } else if EncodedHeader::<sys::MQDH>::fmt_matches(next_format.fmt) {
        Some(parse_header::<sys::MQDH>(data, next_format.ccsid, next_format.encoding)?)
    } else {
        None
    })
}

#[cfg(test)]
mod tests {
    use std::{mem::transmute, ptr, slice::from_raw_parts};

    use crate::{
        core::values,
        headers::{EncodedHeader, Header, HeaderError},
        sys,
        types::{Fmt, MessageFormat},
    };

    use super::{fmt, next_header, ChainedHeader, TextEnc};

    const NEXT_DEAD: MessageFormat = MessageFormat {
        ccsid: 1208,
        encoding: values::MQENC(sys::MQENC_NATIVE),
        fmt: TextEnc::Ebcdic(sys::MQDLH::FMT_EBCDIC),
    };

    const NEXT_RFH2: MessageFormat = MessageFormat {
        ccsid: 1208,
        encoding: values::MQENC(sys::MQENC_NATIVE),
        fmt: TextEnc::Ebcdic(sys::MQRFH2::FMT_EBCDIC),
    };

    const NEXT_STRING: MessageFormat = MessageFormat {
        ccsid: 1208,
        encoding: values::MQENC(sys::MQENC_NATIVE),
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
        let dlh = unsafe { transmute::<sys::MQDLH, [u8; sys::MQDLH_LENGTH_1]>(sys::MQDLH::default()) };
        let header = next_header(dlh.as_slice(), &NEXT_DEAD);

        assert!(matches!(header, Ok(Some((Header::Dlh(_), ..)))));
    }

    #[test]
    pub fn rfh2() {
        let rfh2 = sys::MQRFH2::default();
        let rfh2_data = unsafe { transmute::<sys::MQRFH2, [u8; sys::MQRFH2_CURRENT_LENGTH]>(rfh2) };
        let header = next_header(rfh2_data.as_slice(), &NEXT_RFH2);

        assert!(matches!(header, Ok(Some((Header::Rfh2(_), ..)))));
    }

    #[test]
    pub fn header_iter() {
        const TOTAL_LENGTH: usize = sys::MQDLH_LENGTH_1 + sys::MQRFH2_LENGTH_2;
        let mut data: [u8; TOTAL_LENGTH] = [0; TOTAL_LENGTH];
        let mut dlh = sys::MQDLH::default();
        let rfh2 = sys::MQRFH2::default();
        dlh.Format = unsafe { transmute::<Fmt, [i8; 8]>(sys::MQRFH2::FMT_ASCII) };
        dlh.CodedCharSetId = 1208;
        dlh.Encoding = sys::MQENC_NATIVE;
        data[..sys::MQDLH_LENGTH_1].copy_from_slice(unsafe { from_raw_parts(ptr::from_ref(&dlh).cast(), sys::MQDLH_LENGTH_1) });
        data[sys::MQDLH_LENGTH_1..].copy_from_slice(unsafe { from_raw_parts(ptr::from_ref(&rfh2).cast(), sys::MQRFH2_LENGTH_2) });

        let headers = Header::iter(
            data.as_slice(),
            MessageFormat {
                ccsid: 1208,
                encoding: values::MQENC(sys::MQENC_NATIVE),
                fmt: TextEnc::Ascii(sys::MQDLH::FMT_ASCII),
            },
        );

        let headers_vec: Vec<_> = headers.collect();

        assert!(matches!(headers_vec[0], Ok((Header::Dlh(_), ..))));
        assert!(matches!(
            headers_vec[1],
            Ok((Header::Rfh2(EncodedHeader { raw_header: &_, .. }), ..))
        ));
        assert_eq!(headers_vec.len(), 2);
    }
}
