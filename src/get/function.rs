use crate::{
    Buffer, CCSID, Completion, Conn, Error, ResultComp, ResultCompErr, StrCcsidCow, WriteRaw, headers::TextEnc, prelude::*,
    structs,
};

use super::option;
use crate::{Object, constants, types};

use libmqm_default as default;

#[cfg(feature = "mqai")]
mod mqai {
    use super::option;
    use libmqm_default as default;
    use libmqm_sys::Mqai;

    use crate::Conn;
    use crate::{Bag, Completion, Error, Library, Object, Owned, ResultComp, constants, prelude::*, structs};

    impl<C: Conn> Object<C>
    where
        C::Lib: crate::Library<MQ: libmqm_sys::Mqai>,
    {
        pub fn get_bag_with<R: option::GetBagAttr>(
            &self,
            options: &impl option::GetOption,
            bag: &mut Bag<Owned, impl Library<MQ: Mqai>>,
        ) -> ResultComp<Option<R>> {
            let mut param = option::GetParam {
                md: structs::MQMD2::new(default::MQMD2_DEFAULT),
                gmo: structs::MQGMO::new(default::MQGMO_DEFAULT),
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

        pub fn get_bag(
            &self,
            options: &impl option::GetOption,
            bag: &mut Bag<Owned, impl Library<MQ: Mqai>>,
        ) -> ResultComp<bool> {
            self.get_bag_with::<()>(options, bag).map_completion(|o| o.is_some())
        }
    }
}

impl<C: Conn> Object<C> {
    /// This function uses the [`MQGET`](libmqm_sys::MQGET) MQ API function.
    pub fn get_data<'b, R>(&self, options: &impl option::GetOption, buffer: &'b mut [R]) -> ResultComp<Option<&'b [R]>>
    where
        R: WriteRaw<u8>,
    {
        self.get_as(options, buffer)
            .map_completion(|o| o.map(|buffer: &mut [R]| &*buffer))
    }

    /// This function uses the [`MQGET`](libmqm_sys::MQGET) MQ API function.
    pub fn get_data_with<'b, A, R>(
        &self,
        options: &impl option::GetOption,
        buffer: &'b mut [R],
    ) -> ResultComp<Option<(&'b [R], A)>>
    where
        A: option::GetAttr<'b, R>,
        R: WriteRaw<u8>,
    {
        self.get_as(options, buffer)
            .map_completion(|o| o.map(|(buffer, attr): (&mut [R], A)| (&*buffer, attr)))
    }

    /// This function uses the [`MQGET`](libmqm_sys::MQGET) MQ API function.
    pub fn get_string<'b>(
        &self,
        options: &impl option::GetOption,
        buffer: impl Buffer<'b, u8>,
    ) -> ResultCompErr<Option<StrCcsidCow<'b>>, super::GetStringCcsidError> {
        self.get_as(options, buffer)
    }

    /// This function uses the [`MQGET`](libmqm_sys::MQGET) MQ API function.
    pub fn get_string_with<'b, A>(
        &self,
        options: &impl option::GetOption,
        buffer: impl Buffer<'b, u8>,
    ) -> ResultCompErr<Option<(StrCcsidCow<'b>, A)>, super::GetStringCcsidError>
    where
        A: option::GetAttr<'b, u8>,
    {
        self.get_as(options, buffer)
    }

    /// This function uses the [`MQGET`](libmqm_sys::MQGET) MQ API function.
    pub fn get_as<'b, V, R, B>(&self, options: &impl option::GetOption, buffer: B) -> ResultCompErr<Option<V>, V::Error>
    where
        B: Buffer<'b, R>,
        V: option::GetValue<'b, R, B>,
        R: WriteRaw<u8>,
    {
        use libmqm_sys as mq;

        let mut param = option::GetParam {
            md: structs::MQMD2::new(default::MQMD2_DEFAULT),
            gmo: structs::MQGMO::new(mq::MQGMO {
                Version: mq::MQGMO_VERSION_3, // Version 3 for ReturnedLength
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
                            constants::MQRL_UNDEFINED => std::cmp::min(
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
                .map_completion(|(message_length, data_length)| option::GetState {
                    buffer,
                    data_length: data_length
                        .try_into()
                        .expect("data length should be within positive usize range"),
                    message_length: message_length
                        .try_into()
                        .expect("message length should be within positive usize range"),
                    format: types::MessageFormat {
                        ccsid: CCSID(param.md.CodedCharSetId),
                        encoding: types::MQENC(param.md.Encoding),
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
