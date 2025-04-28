use std::{marker::PhantomData, num::NonZero, ptr};

use libmqm_sys::Mqi;
use libmqm_default as default;
use crate::types::{MQCMHO, MQDMPO, MQIMPO, MQSMPO, MQTYPE, MQMHBO, MQBMHO};

use crate::prelude::*;
use crate::core::{MessageHandle, WriteRaw};
use crate::properties_options::{NameUsage, PropertyValue, PropertyParam, PropertyState, SetProperty};
use crate::{core, sys, constants, Completion, Conn, InqBuffer};

use crate::{EncodedString, Error, MqStruct};
use crate::{ResultComp, ResultCompErr, ResultErr};

use super::types::MessageFormat;
use super::Buffer;

#[derive(Debug)]
pub struct Properties<C: Conn> {
    handle: core::MessageHandle,
    connection: C,
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
fn inqmp<'a, 'b, A: core::Library<MQ: Mqi>>(
    mq: &core::MqFunctions<A>,
    connection_handle: Option<core::ConnectionHandle>,
    message_handle: &core::MessageHandle,
    mqimpo: &mut MqStruct<sys::MQIMPO>,
    name: &MqStruct<sys::MQCHARV>,
    mqpd: &mut MqStruct<sys::MQPD>,
    value_type: &mut MQTYPE,
    mut value: InqBuffer<'a, u8>,
    max_value_size: Option<NonZero<usize>>,
    mut returned_name: Option<InqBuffer<'b, sys::MQCHAR>>,
    max_name_size: Option<NonZero<usize>>,
) -> ResultCompErr<(InqBuffer<'a, u8>, Option<InqBuffer<'b, sys::MQCHAR>>), core::MqInqError> {
    if let Some(rn) = returned_name.as_mut() {
        let rn_ref = rn.as_mut();
        mqimpo.ReturnedName.VSPtr = rn_ref.as_mut_ptr().cast();
        mqimpo.ReturnedName.VSBufSize = rn_ref.len().try_into().expect("length should convert to usize");
    } else {
        mqimpo.ReturnedName.VSPtr = ptr::null_mut();
    }

    match (
        mq.mqinqmp(
            connection_handle,
            message_handle,
            mqimpo,
            name,
            mqpd,
            value_type,
            Some(value.as_mut()),
        ),
        returned_name,
    ) {
        (Err(core::MqInqError::Length(length, Error(.., constants::MQRC_PROPERTY_VALUE_TOO_BIG))), rn)
            if max_value_size.is_none_or(|max_len| Into::<usize>::into(max_len) > value.len()) =>
        {
            let len = length.try_into().expect("length should convert to usize");
            let value_vec = InqBuffer::Owned(vec![0; len]);
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
        (Err(core::MqInqError::Length(length, Error(.., constants::MQRC_PROPERTY_NAME_TOO_BIG))), Some(rn))
            if max_name_size.is_none_or(|max_len| Into::<usize>::into(max_len) > rn.len()) =>
        {
            let len = length.try_into().expect("length should convert to usize");
            let name_vec = InqBuffer::Owned(vec![0; len]);
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
        let mqcmho = sys::MQCMHO {
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
            impo: MqStruct::new(sys::MQIMPO {
                Options: options.0,
                ..default::MQIMPO_DEFAULT
            }),
            value_type: MQTYPE::default(),
            mqpd: MqStruct::new(default::MQPD_DEFAULT),
            name_required: NameUsage::default(),
        };

        let mut inq_value_buffer = InqBuffer::Slice(val_return_buffer.as_mut_slice());
        inq_value_buffer = match P::max_value_size() {
            Some(max_size) => inq_value_buffer.truncate(max_size.into()),
            None => inq_value_buffer,
        };
        let name = MqStruct::from_encoded_str(name);

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

            let mqi_inqmp = inqmp(
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
            });

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
        let mut mqdmpo = MqStruct::new(default::MQDMPO_DEFAULT);
        *mqdmpo.Options.as_mut() = options;

        let name_mqcharv = MqStruct::from_encoded_str(name);

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
        let mut mqpd = MqStruct::new(default::MQPD_DEFAULT);
        let mut mqsmpo = MqStruct::new(default::MQSMPO_DEFAULT);
        *mqsmpo.Options.as_mut() = location;
        let (data, value_type) = value.apply_mqsetmp(&mut mqpd, &mut mqsmpo);

        let name_mqcharv = MqStruct::from_encoded_str(name);
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

    pub fn close(self) -> ResultErr<()> {
        let mut s = self;
        let mqdmho = default::MQDMHO_DEFAULT;
        s.connection.mq().mqdltmh(Some(s.connection.handle()), &mut s.handle, &mqdmho)
    }

    pub fn to_buffer<'a, A: Buffer<'a, impl WriteRaw<sys::MQBYTE>>>(
        &self,
        name: &(impl EncodedString + ?Sized),
        options: MQMHBO,
        buffer: A,
    ) -> ResultCompErr<(MessageFormat, A), core::MqInqError> {
        let read_only_options = options - constants::MQMHBO_DELETE_PROPERTIES;
        let mut buf = buffer;
        let mut mhbo = MqStruct::new(default::MQMHBO_DEFAULT);
        *mhbo.Options.as_mut() = read_only_options;
        let mut mqmd = MqStruct::new(default::MQMD2_DEFAULT);
        let name_mqcharv = MqStruct::from_encoded_str(name);

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

    pub fn to_buffer_mut<'a, A: Buffer<'a, impl WriteRaw<sys::MQBYTE>>>(
        &mut self,
        name: &(impl EncodedString + ?Sized),
        options: MQMHBO,
        buffer: A,
    ) -> ResultCompErr<(MessageFormat, A), core::MqInqError> {
        let mut buf = buffer;
        let mhbo = MqStruct::new(sys::MQMHBO {
            Options: options.0,
            ..default::MQMHBO_DEFAULT
        });
        let mut mqmd = MqStruct::new(default::MQMD2_DEFAULT);
        let name_mqcharv = MqStruct::from_encoded_str(name);

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

    pub fn from_buffer(&mut self, options: MQBMHO, format: &MessageFormat, buffer: &[sys::MQBYTE]) -> ResultComp<()> {
        // Drop the delete properties option as this fn does not modify the buffer
        let options_read_only = options - constants::MQBMHO_DELETE_PROPERTIES;
        let mut mqmd = format.into_mqmd2();
        let bmho = MqStruct::new(sys::MQBMHO {
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
        buffer: &'a mut [sys::MQBYTE],
    ) -> ResultComp<(MessageFormat, &'a [sys::MQBYTE])> {
        let mut mqmd = format.into_mqmd2();
        let bmho = MqStruct::new(sys::MQBMHO {
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

    use crate::{
        headers::{fmt::MQFMT_NONE, TextEnc},
        core::CCSID,
        prelude::*,
        constants,
        test::mock::{self, MockFunctions},
        types::MessageFormat,
        Completion, Connection, ResultComp, ThreadNone,
    };
    use crate::types::MQENC;

    use super::Properties;

    fn with_mqmhbuf_mocked<F>(mock_data: &'static [u8], f: F) -> ResultComp<()>
    where
        F: FnOnce(&mut Properties<Connection<Rc<MockFunctions>, ThreadNone>>) -> ResultComp<()>,
    {
        let mock_connection = mock::connect_ok(|mock_library| {
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
        });

        let mut properties = Properties::new(mock_connection, constants::MQCMHO_NONE)?;

        f(&mut properties).warn_as_error()?;

        Ok(Completion::new(()))
    }

    #[test]
    pub fn to_buffer() -> Result<(), Box<dyn Error>> {
        const MOCK_DATA: &[u8] = b"MOCK";
        with_mqmhbuf_mocked(MOCK_DATA, |prop| {
            let buffer = vec![0u8; usize::pow(2, 16)]; // 64k
            let (_, prop_buffer) = prop.to_buffer_mut("%", constants::MQMHBO_NONE, buffer).warn_as_error()?;
            assert_eq!(MOCK_DATA, prop_buffer);
            Ok(Completion::new(()))
        })
        .warn_as_error()?;

        with_mqmhbuf_mocked(MOCK_DATA, |prop| {
            let buffer = vec![0u8; usize::pow(2, 16)]; // 64k
            let (_, prop_buffer) = prop.to_buffer("%", constants::MQMHBO_NONE, buffer).warn_as_error()?;
            assert_eq!(MOCK_DATA, prop_buffer);
            Ok(Completion::new(()))
        })
        .warn_as_error()?;

        Ok(())
    }

    pub fn with_mqbufmh_mocked<F>(data: &'static [u8], f: F) -> ResultComp<()>
    where
        F: FnOnce(&mut Properties<Connection<Rc<MockFunctions>, ThreadNone>>, MessageFormat, &mut [u8]) -> ResultComp<()>,
    {
        let connection = mock::connect_ok(|mock_library| {
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
        });

        let mut properties = Properties::new(connection, constants::MQCMHO_NONE)?;

        let mut buffer = data.to_owned();

        let mf = MessageFormat {
            ccsid: CCSID(1208),
            encoding: MQENC::default(),
            fmt: TextEnc::Ascii(MQFMT_NONE),
        };

        f(&mut properties, mf, &mut buffer).warn_as_error()?;

        Ok(Completion::new(()))
    }

    #[test]
    pub fn from_buffer() -> Result<(), Box<dyn Error>> {
        const MOCK_DATA: &[u8] = b"MOCK_FROM_BUFFER";

        with_mqbufmh_mocked(MOCK_DATA, |properties, mf, buffer| {
            let buffer_clone = buffer.to_owned();
            let (same_format, same_buffer) = properties
                .from_buffer_mut(constants::MQBMHO_NONE, &mf, buffer)
                .warn_as_error()?;
            assert_eq!(same_buffer, buffer_clone);
            assert_eq!(same_format, mf);
            Ok(Completion::new(()))
        })
        .warn_as_error()?;

        with_mqbufmh_mocked(MOCK_DATA, |properties, mf, buffer| {
            properties.from_buffer(constants::MQBMHO_NONE, &mf, buffer)
        })
        .warn_as_error()?;

        Ok(())
    }
}
