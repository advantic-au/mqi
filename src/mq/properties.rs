use std::{borrow::Cow, marker::PhantomData, num::NonZero, ptr};

use libmqm_default as default;
use libmqm_sys::{Mqi, lib as sys};

use crate::{
    Buffer, Completion, Conn, ConnectionHandle, EncodedString, Error, Library, MessageHandle, MqFunctions, MqInqError,
    ResultComp, ResultCompErr, ResultErr, WriteRaw, constants,
    prelude::*,
    properties_options::{NameUsage, PropertyParam, PropertyState, PropertyValue, SetProperty},
    structs,
    types::{MQBMHO, MQBYTE, MQCHAR, MQCMHO, MQDMPO, MQIMPO, MQMHBO, MQSMPO, MQTYPE, MessageFormat},
};

#[derive(Debug)]
pub struct Properties<C: Conn> {
    handle: MessageHandle,
    connection: C,
}

enum InqBuffer<'a, T> {
    Slice(&'a mut [T]),
    Owned(Vec<T>),
}

impl<T> InqBuffer<'_, T> {
    #[must_use]
    pub fn truncate(self, len: usize) -> Self {
        match self {
            Self::Slice(s) => {
                let buf_len = s.len();
                Self::Slice(&mut s[..std::cmp::min(len, buf_len)])
            }
            Self::Owned(mut v) => {
                v.truncate(len);
                Self::Owned(v)
            }
        }
    }
}

impl<T> AsRef<[T]> for InqBuffer<'_, T> {
    fn as_ref(&self) -> &[T] {
        match self {
            InqBuffer::Slice(s) => s,
            InqBuffer::Owned(o) => o,
        }
    }
}

impl<T> AsMut<[T]> for InqBuffer<'_, T> {
    fn as_mut(&mut self) -> &mut [T] {
        match self {
            InqBuffer::Slice(s) => s,
            InqBuffer::Owned(o) => o,
        }
    }
}

impl<'a, T: Clone> From<InqBuffer<'a, T>> for Cow<'a, [T]>
where
    [T]: ToOwned,
{
    fn from(value: InqBuffer<'a, T>) -> Self {
        match value {
            InqBuffer::Slice(s) => Cow::Borrowed(&*s),
            InqBuffer::Owned(o) => o.into(),
        }
    }
}

impl<'a, T: Clone> Buffer<'a, T> for InqBuffer<'a, T> {
    fn truncate(self, size: usize) -> Self {
        Self::truncate(self, size)
    }

    fn into_cow(self) -> Cow<'a, [T]> {
        self.into()
    }

    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn split_at(self, at: usize) -> (Self, Self) {
        match self {
            Self::Slice(s) => {
                let (head, tail) = s.split_at(at);
                (Self::Slice(head), Self::Slice(tail))
            }
            Self::Owned(v) => {
                let (head, tail) = v.split_at(at);
                (Self::Owned(head), Self::Owned(tail))
            }
        }
    }
}

impl<C: Conn> Drop for Properties<C> {
    fn drop(&mut self) {
        let mqdmho = default::MQDMHO_DEFAULT;

        if self.handle.is_deleteable() {
            let _ = self
                .connection
                .mq()
                .mqdltmh(Some(self.connection.handle()), &mut self.handle, &mqdmho);
        }
    }
}

#[expect(clippy::too_many_arguments)]
unsafe fn inqmp<'a, 'b, A: Library<MQ: Mqi>>(
    mq: &MqFunctions<A>,
    connection_handle: Option<ConnectionHandle>,
    message_handle: &MessageHandle,
    mqimpo: &mut structs::MQIMPO,
    name: &structs::MQCHARV,
    mqpd: &mut structs::MQPD,
    value_type: &mut MQTYPE,
    mut value: InqBuffer<'a, u8>,
    max_value_size: Option<NonZero<usize>>,
    mut returned_name: Option<InqBuffer<'b, MQCHAR>>,
    max_name_size: Option<NonZero<usize>>,
) -> ResultCompErr<(InqBuffer<'a, u8>, Option<InqBuffer<'b, MQCHAR>>), MqInqError> {
    if let Some(rn) = returned_name.as_mut() {
        let rn_ref = rn.as_mut();
        mqimpo.ReturnedName.VSPtr = rn_ref.as_mut_ptr().cast();
        mqimpo.ReturnedName.VSBufSize = rn_ref.len().try_into().expect("length should convert to usize");
    } else {
        mqimpo.ReturnedName.VSPtr = ptr::null_mut();
    }

    match (
        unsafe {
            mq.mqinqmp(
                connection_handle,
                message_handle,
                mqimpo,
                name,
                mqpd,
                value_type,
                Some(value.as_mut()),
            )
        },
        returned_name,
    ) {
        (Err(MqInqError::Length(length, Error(.., constants::MQRC_PROPERTY_VALUE_TOO_BIG))), rn)
            if max_value_size.is_none_or(|max_len| Into::<usize>::into(max_len) > value.len()) =>
        {
            let len = length.try_into().expect("length should convert to usize");
            let value_vec = InqBuffer::Owned(vec![0; len]);
            unsafe {
                inqmp(
                    mq,
                    connection_handle,
                    message_handle,
                    mqimpo,
                    name,
                    mqpd,
                    value_type,
                    value_vec,
                    max_value_size,
                    rn,
                    max_name_size,
                )
            }
        }
        (Err(MqInqError::Length(length, Error(.., constants::MQRC_PROPERTY_NAME_TOO_BIG))), Some(rn))
            if max_name_size.is_none_or(|max_len| Into::<usize>::into(max_len) > rn.len()) =>
        {
            let len = length.try_into().expect("length should convert to usize");
            let name_vec = InqBuffer::Owned(vec![0; len]);
            unsafe {
                inqmp(
                    mq,
                    connection_handle,
                    message_handle,
                    mqimpo,
                    name,
                    mqpd,
                    value_type,
                    value,
                    max_value_size,
                    Some(name_vec),
                    max_name_size,
                )
            }
        }
        (other, rn) => other.map_completion(|length| {
            (
                value.truncate(length.try_into().expect("length should convert to usize")),
                rn.map(|name| {
                    name.truncate(
                        mqimpo
                            .ReturnedName
                            .VSLength
                            .try_into()
                            .expect("length should convert to usize"),
                    )
                }),
            )
        }),
    }
}

pub struct MsgPropIter<'name, 'message, P, N: EncodedString + ?Sized, C: Conn> {
    name: &'name N,
    message: &'message Properties<C>,
    options: MQIMPO,
    _marker: PhantomData<P>,
}

impl<P: PropertyValue, N: EncodedString + ?Sized, C: Conn> Iterator for MsgPropIter<'_, '_, P, N, C> {
    type Item = ResultCompErr<P, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.message.property::<P>(self.name, self.options) {
            Ok(Completion(Some(value), warning)) => Some(Ok(Completion(value, warning))),
            Ok(Completion(None, _)) => None,
            Err(e) => Some(Err(e)),
        };

        self.options.insert(constants::MQIMPO_INQ_NEXT);

        result
    }
}

impl<C: Conn> Properties<C> {
    pub const fn handle(&self) -> &MessageHandle {
        &self.handle
    }

    pub fn new(connection: C, options: MQCMHO) -> ResultErr<Self> {
        let mqcmho = libmqm_sys::lib::MQCMHO {
            Options: options.0,
            ..default::MQCMHO_DEFAULT
        };
        connection
            .mq()
            .mqcrtmh(Some(connection.handle()), &mqcmho)
            .map(|handle| Self { handle, connection })
    }

    pub fn property_iter<'message, 'name, P, N>(
        &'message self,
        name: &'name N,
        options: MQIMPO,
    ) -> MsgPropIter<'name, 'message, P, N, C>
    where
        P: PropertyValue,
        N: EncodedString + ?Sized,
    {
        MsgPropIter {
            name,
            message: self,
            options: options | constants::MQIMPO_INQ_NEXT,
            _marker: PhantomData,
        }
    }

    pub fn property<P>(&self, name: &(impl EncodedString + ?Sized), options: MQIMPO) -> ResultCompErr<Option<P>, Error>
    where
        P: PropertyValue,
    {
        const DEFAULT_BUF_SIZE: usize = 1024;
        let mut val_return_buffer = [0; DEFAULT_BUF_SIZE]; // Returned value buffer
        let mut name_return_buffer = [0; DEFAULT_BUF_SIZE]; // Returned name buffer

        let mut property_not_available = false;

        let mut param = PropertyParam {
            impo: structs::MQIMPO::new(sys::MQIMPO {
                Options: options.0,
                ..default::MQIMPO_DEFAULT
            }),
            value_type: MQTYPE::default(),
            mqpd: structs::MQPD::new(default::MQPD_DEFAULT),
            name_required: NameUsage::default(),
        };

        let mut inq_value_buffer = InqBuffer::Slice(val_return_buffer.as_mut_slice());
        inq_value_buffer = match P::max_value_size() {
            Some(max_size) => inq_value_buffer.truncate(max_size.into()),
            None => inq_value_buffer,
        };
        let name = structs::MQCHARV::from_encoded_str(name);

        let result = P::property_consume(&mut param, |param| {
            let mut inq_name_buffer = match param.name_required {
                NameUsage::Ignored => None,
                used => {
                    let buf = InqBuffer::Slice(name_return_buffer.as_mut_slice());
                    Some(match used {
                        NameUsage::MaxLength(length) => buf.truncate(length.into()),
                        _ => buf,
                    })
                }
            };
            param.impo.ReturnedName = inq_name_buffer
                .as_mut()
                .map_or(default::MQCHARV_DEFAULT, |name| sys::MQCHARV {
                    VSPtr: (&raw mut *name).cast(),
                    VSBufSize: name
                        .as_ref()
                        .len()
                        .try_into()
                        .expect("length of buffer should fit within MQLONG range"),
                    ..default::MQCHARV_DEFAULT
                });

            let mqi_inqmp = unsafe {
                inqmp(
                    self.connection.mq(),
                    Some(self.connection.handle()),
                    &self.handle,
                    &mut param.impo,
                    &name,
                    &mut param.mqpd,
                    &mut param.value_type,
                    inq_value_buffer,
                    P::max_value_size(),
                    inq_name_buffer,
                    param.name_required.into(),
                )
                .map_err(Into::into) // Convert the error into an ordinary MQ error
                .map_completion(|(value, name)| PropertyState {
                    name: name.map(Into::into),
                    value: value.into(),
                })
            };

            property_not_available = mqi_inqmp
                .as_ref()
                .is_err_and(|e| matches!(e, &Error(constants::MQCC_FAILED, .., constants::MQRC_PROPERTY_NOT_AVAILABLE)));

            mqi_inqmp
        });

        if property_not_available {
            Ok(Completion::new(None))
        } else {
            result.map_completion(Some).map_err(Into::into)
        }
    }

    pub fn delete_property(&self, name: &(impl EncodedString + ?Sized), options: MQDMPO) -> ResultComp<()> {
        let mut mqdmpo = structs::MQDMPO::new(default::MQDMPO_DEFAULT);
        *mqdmpo.Options.as_mut() = options;

        let name_mqcharv = structs::MQCHARV::from_encoded_str(name);

        self.connection
            .mq()
            .mqdltmp(Some(self.connection.handle()), &self.handle, &mqdmpo, &name_mqcharv)
    }

    pub fn set_property(
        &self,
        name: &(impl EncodedString + ?Sized),
        value: &(impl SetProperty + ?Sized),
        location: MQSMPO,
    ) -> ResultComp<()> {
        let mut mqpd = structs::MQPD::new(default::MQPD_DEFAULT);
        let mut mqsmpo = structs::MQSMPO::new(default::MQSMPO_DEFAULT);
        *mqsmpo.Options.as_mut() = location;
        let (data, value_type) = value.apply_mqsetmp(&mut mqpd, &mut mqsmpo);

        let name_mqcharv = structs::MQCHARV::from_encoded_str(name);
        // SAFETY: The name MQCHARV formed from reference
        unsafe {
            self.connection.mq().mqsetmp(
                Some(self.connection.handle()),
                &self.handle,
                &mqsmpo,
                &name_mqcharv,
                &mut mqpd,
                value_type,
                data,
            )
        }
    }

    pub fn close(self) -> ResultErr<()> {
        let mut s = self;
        let mqdmho = default::MQDMHO_DEFAULT;
        s.connection.mq().mqdltmh(Some(s.connection.handle()), &mut s.handle, &mqdmho)
    }

    pub fn to_buffer<'a, A: Buffer<'a, impl WriteRaw<MQBYTE>>>(
        &self,
        name: &(impl EncodedString + ?Sized),
        options: MQMHBO,
        buffer: A,
    ) -> ResultCompErr<(MessageFormat, A), MqInqError> {
        let read_only_options = options - constants::MQMHBO_DELETE_PROPERTIES;
        let mut buf = buffer;
        let mut mhbo = structs::MQMHBO::new(default::MQMHBO_DEFAULT);
        *mhbo.Options.as_mut() = read_only_options;
        let mut mqmd = structs::MQMD2::new(default::MQMD2_DEFAULT);
        let name_mqcharv = structs::MQCHARV::from_encoded_str(name);

        // SAFETY: The name MQCHARV formed from references
        unsafe {
            self.connection
                .mq()
                .mqmhbuf(
                    Some(self.connection.handle()),
                    self.handle(),
                    &mhbo,
                    &name_mqcharv,
                    &mut *mqmd,
                    buf.as_mut(),
                )
                .map_completion(|len| {
                    (
                        MessageFormat::from_mqmd2(&mqmd),
                        buf.truncate(len.try_into().expect("length should convert to usize")),
                    )
                })
        }
    }

    pub fn to_buffer_mut<'a, A: Buffer<'a, impl WriteRaw<MQBYTE>>>(
        &mut self,
        name: &(impl EncodedString + ?Sized),
        options: MQMHBO,
        buffer: A,
    ) -> ResultCompErr<(MessageFormat, A), MqInqError> {
        let mut buf = buffer;
        let mhbo = structs::MQMHBO::new(sys::MQMHBO {
            Options: options.0,
            ..default::MQMHBO_DEFAULT
        });
        let mut mqmd = structs::MQMD2::new(default::MQMD2_DEFAULT);
        let name_mqcharv = structs::MQCHARV::from_encoded_str(name);

        // SAFETY: The name MQCHARV is formed from references
        unsafe {
            self.connection
                .mq()
                .mqmhbuf(
                    Some(self.connection.handle()),
                    self.handle(),
                    &mhbo,
                    &name_mqcharv,
                    &mut *mqmd,
                    buf.as_mut(),
                )
                .map_completion(|len| {
                    (
                        MessageFormat::from_mqmd2(&mqmd),
                        buf.truncate(len.try_into().expect("length should convert to usize")),
                    )
                })
        }
    }

    pub fn from_buffer(&mut self, options: MQBMHO, format: &MessageFormat, buffer: &[MQBYTE]) -> ResultComp<()> {
        // Drop the delete properties option as this fn does not modify the buffer
        let options_read_only = options - constants::MQBMHO_DELETE_PROPERTIES;
        let mut mqmd = format.into_mqmd2();
        let bmho = structs::MQBMHO::new(sys::MQBMHO {
            Options: options_read_only.0,
            ..default::MQBMHO_DEFAULT
        });

        self.connection
            .mq()
            .mqbufmh(Some(self.connection.handle()), &self.handle, &bmho, &mut *mqmd, buffer)
            .map_completion(|_| {})
    }

    pub fn from_buffer_mut<'a>(
        &mut self,
        options: MQBMHO,
        format: &MessageFormat,
        buffer: &'a mut [MQBYTE],
    ) -> ResultComp<(MessageFormat, &'a [MQBYTE])> {
        let mut mqmd = format.into_mqmd2();
        let bmho = structs::MQBMHO::new(sys::MQBMHO {
            Options: options.0,
            ..default::MQBMHO_DEFAULT
        });

        self.connection
            .mq()
            .mqbufmh(Some(self.connection.handle()), &self.handle, &bmho, &mut *mqmd, buffer)
            .map_completion(|len| {
                (
                    MessageFormat::from_mqmd2(&mqmd),
                    &buffer[..len.try_into().expect("length should convert to usize")],
                )
            })
    }
}

#[cfg(test)]
#[cfg(feature = "mock")]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use std::{error::Error, rc::Rc};

    use super::*;
    use crate::{
        CCSID, Connection, ResultErr, ThreadNone, constants,
        constants::{
            MQCC_FAILED, MQCC_OK, MQRC_CALL_IN_PROGRESS, MQRC_NONE, MQRC_PROPERTY_NAME_TOO_BIG, MQRC_PROPERTY_NOT_AVAILABLE,
            MQRC_PROPERTY_VALUE_TOO_BIG, MQTYPE_BYTE_STRING,
        },
        headers::{TextEnc, fmt::MQFMT_NONE},
        properties_options::Name,
        test::mock::{self, MockFunctions},
        types::{MQCC, MQRC, MessageFormat},
    };

    /// Set up mocks for the MQINQMP function
    fn mqinqmp(mock_list: &[(&str, MQTYPE, &[u8], MQCC, MQRC)]) -> impl Fn(&mut MockFunctions) {
        |mock_library| {
            let mut seq = mockall::Sequence::new();
            mock_library.properties_ok(0xf0f0, 1, &mut seq);

            for (name, data_typ, data, cc, rc) in mock_list.iter().copied() {
                let data = data.to_owned();
                let name = name.to_owned();
                mock_library.expect_MQINQMP().once().in_sequence(&mut seq).returning(
                    move |_, _, mqimpo, _, _, typ, value_length, value, real_length, comp_code, reason| {
                        let mut_typ = unsafe { &mut *typ };
                        let mut_real_length = unsafe { &mut *real_length };
                        let mut_impo: &mut structs::MQIMPO = unsafe { &mut *mqimpo.cast() };
                        let maybe_name_mqcharv: Option<&mut sys::MQCHARV> =
                            unsafe { mut_impo.ReturnedName.VSPtr.as_mut().map(|_| &mut mut_impo.ReturnedName) };

                        // Set the returned real length
                        *mut_real_length = data.len().try_into().expect("i32 in range of usize");

                        // Copy the supplied name
                        if let Some(mut_name_mqcharv) = maybe_name_mqcharv {
                            let name_copied_length = std::cmp::min(
                                mut_name_mqcharv.VSBufSize.try_into().expect("i32 in range of usize"),
                                name.len(),
                            );
                            let mut_name: &mut [u8] =
                                unsafe { std::slice::from_raw_parts_mut(mut_name_mqcharv.VSPtr.cast(), name_copied_length) };
                            mut_name.copy_from_slice(&name.as_bytes()[..name_copied_length]);
                            mut_name_mqcharv.VSCCSID = 1208;
                            mut_name_mqcharv.VSLength = name_copied_length.try_into().expect("i32 in range of usize");
                        }

                        // Copy the supplied data into the returned value
                        let copied_length = std::cmp::min(*mut_real_length, value_length)
                            .try_into()
                            .expect("i32 in range of usize");
                        let mut_value: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(value.cast(), copied_length) };
                        mut_value.copy_from_slice(&data[..copied_length]);

                        // Set the data type
                        *mut_typ.as_mut() = data_typ;

                        MockFunctions::mqi_outcome(comp_code, reason, cc, rc);
                    },
                );
            }
        }
    }

    fn mqmhbuf(mock_data: &'static [u8]) -> impl Fn(&mut MockFunctions) {
        |mock_library| {
            let mut seq = mockall::Sequence::new();
            mock_library.properties_ok(0xf0f0, 1, &mut seq);
            mock_library
                .expect_MQMHBUF()
                .returning(|_, _, _, _, _, buf_len, buf_target, data_len, cc, rc| {
                    unsafe { MockFunctions::copy_to_mq_data(mock_data, buf_len, buf_target, data_len) };
                    MockFunctions::mqi_outcome_ok(cc, rc);
                })
                .once()
                .in_sequence(&mut seq);
        }
    }

    fn mqbufmh(mock_library: &mut MockFunctions) {
        let mut seq = mockall::Sequence::new();

        mock_library.properties_ok(0xf0f0, 1, &mut seq);
        mock_library
            .expect_MQBUFMH()
            .returning(|_, _, _, _, buffer_len, _, data_len, cc, rc| {
                unsafe {
                    *data_len = buffer_len;
                }
                MockFunctions::mqi_outcome_ok(cc, rc);
            })
            .once()
            .in_sequence(&mut seq);
    }

    fn properties_mocked(m: impl Fn(&mut MockFunctions)) -> ResultErr<Properties<Connection<Rc<MockFunctions>, ThreadNone>>> {
        Properties::new(mock::connect_ok(m), constants::MQCMHO_NONE)
    }

    #[test]
    fn property() -> Result<(), Box<dyn Error>> {
        // Test basic succesful retrieval of a byte property
        let value: Option<Vec<u8>> = properties_mocked(mqinqmp(&[("name", MQTYPE_BYTE_STRING, b"data", MQCC_OK, MQRC_NONE)]))?
            .property("_", MQIMPO::default())
            .warn_as_error()?;
        assert_ne!(value, None);

        // Property does not exist
        let value: Option<Vec<u8>> =
            properties_mocked(mqinqmp(&[("", MQTYPE(0), b"", MQCC_FAILED, MQRC_PROPERTY_NOT_AVAILABLE)]))?
                .property("_", MQIMPO::default())
                .warn_as_error()?;
        assert_eq!(value, None);

        // Failure response
        let result: ResultErr<Option<Vec<u8>>> =
            properties_mocked(mqinqmp(&[("", MQTYPE(0), b"", MQCC_FAILED, MQRC_CALL_IN_PROGRESS)]))?
                .property("_", MQIMPO::default())
                .warn_as_error();
        assert!(result.is_err_and(|err| matches!(err, Error(MQCC_FAILED, _, MQRC_CALL_IN_PROGRESS))));

        // Value too big for initial buffer - mqinqmp should be called twice
        let value: Option<Vec<u8>> = properties_mocked(mqinqmp(&[
            ("name", MQTYPE_BYTE_STRING, b"data", MQCC_FAILED, MQRC_PROPERTY_VALUE_TOO_BIG),
            ("name", MQTYPE_BYTE_STRING, b"data", MQCC_OK, MQRC_NONE),
        ]))?
        .property("_", MQIMPO::default())
        .warn_as_error()?;
        assert_ne!(value, None);

        // Value too big for initial buffer - mqinqmp should be called twice
        let value: Option<(Vec<u8>, Name<String>)> = properties_mocked(mqinqmp(&[
            ("name", MQTYPE_BYTE_STRING, b"data", MQCC_FAILED, MQRC_PROPERTY_NAME_TOO_BIG),
            ("name", MQTYPE_BYTE_STRING, b"data", MQCC_OK, MQRC_NONE),
        ]))?
        .property("_", MQIMPO::default())
        .warn_as_error()?;
        assert_ne!(value, None);

        // Name and value too big for initial buffer - mqinqmp should be called three times
        let value: Option<(Vec<u8>, Name<String>)> = properties_mocked(mqinqmp(&[
            ("name", MQTYPE_BYTE_STRING, b"data", MQCC_FAILED, MQRC_PROPERTY_VALUE_TOO_BIG),
            ("name", MQTYPE_BYTE_STRING, b"data", MQCC_FAILED, MQRC_PROPERTY_NAME_TOO_BIG),
            ("name", MQTYPE_BYTE_STRING, b"data", MQCC_OK, MQRC_NONE),
        ]))?
        .property("_", MQIMPO::default())
        .warn_as_error()?;
        assert_ne!(value, None);

        Ok(())
    }

    #[test]
    fn to_buffer() -> Result<(), Box<dyn Error>> {
        const MOCK_DATA: &[u8] = b"MOCK";

        let mut prop = properties_mocked(mqmhbuf(MOCK_DATA))?;
        let buffer = vec![0u8; usize::pow(2, 16)]; // 64k
        let (_, prop_buffer) = prop.to_buffer_mut("%", constants::MQMHBO_NONE, buffer).warn_as_error()?;
        assert_eq!(MOCK_DATA, prop_buffer);

        let prop = properties_mocked(mqmhbuf(MOCK_DATA))?;
        let buffer = vec![0u8; usize::pow(2, 16)]; // 64k
        let (_, prop_buffer) = prop.to_buffer("%", constants::MQMHBO_NONE, buffer).warn_as_error()?;
        assert_eq!(MOCK_DATA, prop_buffer);

        Ok(())
    }

    #[test]
    fn from_buffer() -> Result<(), Box<dyn Error>> {
        const MOCK_DATA: &[u8] = b"MOCK_FROM_BUFFER";
        const MOCK_MF: &MessageFormat = &MessageFormat {
            ccsid: CCSID(1208),
            encoding: constants::MQENC_NATIVE,
            fmt: TextEnc::Ascii(MQFMT_NONE),
        };

        let mut prop = properties_mocked(mqbufmh)?;
        let mut buffer_clone = MOCK_DATA.to_owned();
        let (same_format, same_buffer) = prop
            .from_buffer_mut(constants::MQBMHO_NONE, MOCK_MF, &mut buffer_clone)
            .warn_as_error()?;
        assert_eq!(same_buffer, MOCK_DATA);
        assert_eq!(&same_format, MOCK_MF);

        let mut prop = properties_mocked(mqbufmh)?;
        prop.from_buffer(constants::MQBMHO_NONE, MOCK_MF, MOCK_DATA).warn_as_error()?;

        Ok(())
    }
}
