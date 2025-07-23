use libmqm_default as default;
use libmqm_sys as mq;

use super::option;
use crate::{
    Object,
    connection::AsConnection,
    constants,
    header::TextEnc,
    prelude::*,
    result::{Completion, Error, ResultComp, ResultCompErr},
    string::{CCSID, StrCcsidCow},
    structs,
    traits::{Buffer, WriteRaw},
    types,
};

#[cfg(feature = "mqai")]
mod mqai {
    use libmqm_default as default;
    use libmqm_sys as mq;

    use super::option;
    use crate::{
        MqaiLibrary, Object, bag,
        connection::AsConnection,
        constants,
        prelude::*,
        result::{Completion, Error, ResultComp},
        structs,
    };

    impl<C: AsConnection> Object<C>
    where
        C::Lib: crate::MqaiLibrary,
    {
        pub fn get_bag_with<R: option::GetBagAttr>(
            &self,
            options: &impl option::GetOption,
            bag: &mut bag::Bag<bag::Owned, impl MqaiLibrary>,
        ) -> ResultComp<Option<R>> {
            let mut param = option::GetParam {
                md: structs::MQMD::new(default::MQMD_DEFAULT),
                gmo: structs::MQGMO::new(default::MQGMO_DEFAULT),
            };
            let mut no_msg_available = false;

            options.apply_param(&mut param);
            assert!(param.gmo.Version <= mq::MQGMO_CURRENT_VERSION);
            assert!(param.md.Version <= mq::MQMD_CURRENT_VERSION);

            let result = R::get_bag_extract(&mut param, |param| {
                let connection = self.connection.as_connection();
                let conn = connection.as_connection();
                let mqi_get_bag = conn
                    .mq
                    .mq_get_bag(conn.handle, self.handle(), &mut *param.md, &mut param.gmo, Some(&*bag));
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
            bag: &mut bag::Bag<bag::Owned, impl MqaiLibrary>,
        ) -> ResultComp<bool> {
            self.get_bag_with::<()>(options, bag).map_completion(|o| o.is_some())
        }
    }
}

impl<C: AsConnection> Object<C> {
    /// Get a message from an object returning a slice of data.
    ///
    /// This function uses the [MQGET](libmqm_sys::MQGET) verb. A retun code of [`MQRC_NO_MSG_AVAILABLE`](mq::MQRC_NO_MSG_AVAILABLE)
    /// is translated to a return value of [`None`].
    ///
    /// ## Panics
    /// * An [`MQMD`](mq::MQMD) or [`MQGMO`](mq::MQGMO) Version exceeds the compiled MQ client
    /// * The MQ client returns an invalid data length
    pub fn get_data<'b, R>(&self, options: &impl option::GetOption, buffer: &'b mut [R]) -> ResultComp<Option<&'b [R]>>
    where
        R: WriteRaw<u8>,
    {
        self.get_as(options, buffer)
            .map_completion(|o| o.map(|buffer: &mut [R]| &*buffer))
    }

    /// Get a message from an object returning a slice of data in a tuple with a [`GetAttr`](option::GetAttr).
    ///
    /// This function uses the [MQGET](libmqm_sys::MQGET) verb. A retun code of [`MQRC_NO_MSG_AVAILABLE`](mq::MQRC_NO_MSG_AVAILABLE)
    /// is translated to a return value of [`None`].
    ///
    /// ## Panics
    /// * An [`MQMD`](mq::MQMD) or [`MQGMO`](mq::MQGMO) Version exceeds the compiled MQ client
    /// * The MQ client returns an invalid data length
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

    /// Get a string message from an object returning a [`StrCcsidCow`].
    ///
    /// No conversion of the message is performed by default.
    ///
    /// This function uses the [MQGET](libmqm_sys::MQGET) verb. A retun code of [`MQRC_NO_MSG_AVAILABLE`](mq::MQRC_NO_MSG_AVAILABLE)
    /// is translated to a return value of [`None`].
    ///
    /// ## Panics
    /// * An [`MQMD`](mq::MQMD) or [`MQGMO`](mq::MQGMO) Version exceeds the compiled MQ client
    /// * The MQ client returns an invalid data length
    pub fn get_string<'b>(
        &self,
        options: &impl option::GetOption,
        buffer: impl Buffer<'b, u8>,
    ) -> ResultCompErr<Option<StrCcsidCow<'b>>, super::GetStringCcsidError> {
        self.get_as(options, buffer)
    }

    /// Get a string message from an object returning a ([`StrCcsidCow`], [impl `GetAttr`](option::GetAttr)) tuple.
    ///
    /// No conversion of the message is performed by default.
    ///
    /// This function uses the [MQGET](libmqm_sys::MQGET) verb. A retun code of [`MQRC_NO_MSG_AVAILABLE`](mq::MQRC_NO_MSG_AVAILABLE)
    /// is translated to a return value of [`None`].
    ///
    /// ## Panics
    /// * An [`MQMD`](mq::MQMD) or [`MQGMO`](mq::MQGMO) Version exceeds the compiled MQ client
    /// * The MQ client returns an invalid data length
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

    /// Get a message from an object returning a usually inferred [`GetValue`](option::GetValue).
    ///
    /// This function uses the [MQGET](libmqm_sys::MQGET) verb. A retun code of [`MQRC_NO_MSG_AVAILABLE`](mq::MQRC_NO_MSG_AVAILABLE)
    /// is translated to a return value of [`None`].
    ///
    /// ## Panics
    /// * An [`MQMD`](mq::MQMD) or [`MQGMO`](mq::MQGMO) Version exceeds the compiled MQ client
    /// * The MQ client returns an invalid data length
    pub fn get_as<'b, V, R, B>(&self, options: &impl option::GetOption, buffer: B) -> ResultCompErr<Option<V>, V::Error>
    where
        B: Buffer<'b, R>,
        V: option::GetValue<'b, R, B>,
        R: WriteRaw<u8>,
    {
        let mut param = option::GetParam {
            md: structs::MQMD::new(default::MQMD_DEFAULT),
            gmo: structs::MQGMO::new(default::MQGMO_DEFAULT),
        };
        let mut no_msg_available = false;

        options.apply_param(&mut param);
        param.gmo.set_min_version(mq::MQGMO_VERSION_3); // Required for ReturnLength

        assert!(param.gmo.Version <= mq::MQGMO_CURRENT_VERSION);
        assert!(param.md.Version <= mq::MQMD_CURRENT_VERSION);

        let result = V::get_consume(&mut param, |param| {
            let mut buffer = buffer;
            let write_area = match V::get_max_data_size() {
                Some(max_len) => &mut buffer.as_mut()[..max_len.into()],
                None => buffer.as_mut(),
            };

            let connection = self.connection.as_connection();
            let mqi_get = connection
                .mq
                .mqget(
                    connection.handle,
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
                                    .expect("length of buffer should be within bounds of MQLONG"),
                                length,
                            ),
                            returned_length => returned_length.0,
                        },
                    )
                })
                .map_completion(|(message_length, data_length)| option::GetState {
                    buffer,
                    data_length: data_length.try_into().expect("data length should be within bounds of usize"),
                    message_length: message_length
                        .try_into()
                        .expect("message length should be within bounds of usize"),
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
