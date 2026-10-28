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
    /// Get a message body as bytes into a caller-provided buffer.
    ///
    /// The buffer capacity limits how many bytes can be returned. Size it for
    /// the largest message your application accepts. If you need to inspect the
    /// full message length and implement a retry with a larger buffer, use
    /// [`get_as`](Self::get_as) with a custom
    /// [`GetValue`](crate::get::GetValue); its `GetState` argument exposes both
    /// the data length and full message length. MQ may report truncation as a
    /// warning or error depending on the get options.
    ///
    /// The [`GetOption`](crate::get::GetOption) controls matching, waiting, and other
    /// MQGET behavior. [`GetWait::Wait`](super::GetWait::Wait) takes a duration
    /// in milliseconds; [`GetWait::NoWait`](super::GetWait::NoWait) checks
    /// immediately. Wrap an option in `Some` or `None` to enable or skip it
    /// based on runtime configuration. A skipped option does not clear a
    /// setting supplied by another option in the tuple. A return code of
    /// [`MQRC_NO_MSG_AVAILABLE`](mq::MQRC_NO_MSG_AVAILABLE) becomes `Ok(None)`;
    /// a retrieved message becomes `Ok(Some(bytes))`. MQ warnings are preserved
    /// in the [`Completion`](crate::result::Completion).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mqi::{
    ///     Object, constants,
    ///     get::GetWait,
    ///     prelude::*,
    ///     types::QueueName,
    /// };
    ///
    /// # let connection = mqi::connect_default(mqi::connection::NO_CONNECT_OPTIONS).warn_as_error()?;
    /// let queue = Object::open(
    ///     &connection,
    ///     &(QueueName(mqstr!("DEV.QUEUE.1")), constants::MQOO_INPUT_AS_Q_DEF),
    /// )
    /// .warn_as_error()?;
    ///
    /// let mut buffer = vec![0; 64 * 1024];
    /// if let Some(message) = queue
    ///     .get_data(&GetWait::Wait(5_000), &mut buffer)
    ///     .warn_as_error()?
    /// {
    ///     println!("received {} bytes", message.len());
    /// }
    /// # Ok::<(), mqi::result::Error>(())
    /// ```
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

    /// Get a message body as bytes and extract additional attributes.
    ///
    /// This is [`get_data`](Self::get_data) plus a value extracted through
    /// [`GetAttr`](crate::get::GetAttr), for example the message descriptor or
    /// message format. It returns `Ok(None)` when no message is available and
    /// `Ok(Some((bytes, attributes)))` when a message is received.
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

    /// Get an MQ string message into a caller-provided buffer.
    ///
    /// The message must use the `MQFMT_STRING` format; otherwise the call
    /// returns [`GetStringCcsidError::UnexpectedFormat`](super::GetStringCcsidError::UnexpectedFormat).
    /// This method does not
    /// convert the message to UTF-8; the returned
    /// [`StrCcsidCow`](crate::string::StrCcsidCow) retains the message CCSID and
    /// encoding information. Use [`get_as`](Self::get_as) with `Cow<str>` when
    /// you specifically require UTF-8 text.
    ///
    /// The buffer capacity limits how much message data is returned. A
    /// [`GetWait`](super::GetWait) controls whether to wait; `Wait` uses
    /// milliseconds and `NoWait` checks immediately. `Ok(None)` means no
    /// message was available before the wait ended.
    /// MQ warnings are preserved in the [`Completion`](crate::result::Completion).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mqi::{
    ///     Object, constants,
    ///     get::GetWait,
    ///     prelude::*,
    ///     types::QueueName,
    /// };
    ///
    /// # let connection = mqi::connect_default(mqi::connection::NO_CONNECT_OPTIONS).warn_as_error()?;
    /// let queue = Object::open(
    ///     &connection,
    ///     &(QueueName(mqstr!("DEV.QUEUE.1")), constants::MQOO_INPUT_AS_Q_DEF),
    /// )
    /// .warn_as_error()?;
    ///
    /// let mut buffer = vec![0; 64 * 1024];
    /// if let Some(message) = queue
    ///     .get_string(&GetWait::Wait(5_000), buffer.as_mut_slice())
    ///     .warn_as_error()?
    /// {
    ///     println!("{message:?}");
    /// }
    /// # Ok::<(), mqi::get::GetStringCcsidError>(())
    /// ```
    ///
    /// For messages with another format, use [`get_data`](Self::get_data).
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

    /// Get an MQ string message and extract additional attributes.
    ///
    /// This is [`get_string`](Self::get_string) plus values extracted through
    /// [`GetAttr`](crate::get::GetAttr), such as the MQ message format. The body
    /// must use `MQFMT_STRING`; its CCSID is retained rather than converted.
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

    /// Get a message and construct the return value selected by its expected type.
    ///
    /// This is the general form behind [`get_data`](Self::get_data) and
    /// [`get_string`](Self::get_string). The expected return type selects a
    /// [`GetValue`](crate::get::GetValue) implementation; built-in choices include
    /// byte slices, byte vectors, `Cow<str>`, and
    /// [`StrCcsidCow`](crate::string::StrCcsidCow). Use it to request UTF-8
    /// parsing or a custom result type. `Cow<str>` requires `MQFMT_STRING` and
    /// UTF-8 CCSID 1208; use `StrCcsidCow` when the message may use another CCSID.
    /// A no-message result is `Ok(None)`.
    ///
    /// The supplied buffer still limits the data available to the selected
    /// value. MQ warnings remain in the returned [`Completion`](crate::result::Completion).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::borrow::Cow;
    /// use mqi::{Object, constants, get::GetWait, prelude::*, types::QueueName};
    ///
    /// # let connection = mqi::connect_default(mqi::connection::NO_CONNECT_OPTIONS).warn_as_error()?;
    /// let queue = Object::open(
    ///     &connection,
    ///     &(QueueName(mqstr!("DEV.QUEUE.1")), constants::MQOO_INPUT_AS_Q_DEF),
    /// )
    /// .warn_as_error()?;
    ///
    /// let mut buffer = vec![0; 64 * 1024];
    /// let message: Option<Cow<'_, str>> = queue
    ///     .get_as(&GetWait::Wait(5_000), buffer.as_mut_slice())
    ///     .warn_as_error()?;
    /// # Ok::<(), mqi::get::GetStringError>(())
    /// ```
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
