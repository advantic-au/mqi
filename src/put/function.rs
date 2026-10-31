use std::borrow::Cow;

use libmqm_default as default;
use libmqm_sys as mq;

use super::option;
use crate::{
    Connection, Library, Object,
    connection::AsConnection,
    constants,
    header::{TextEnc, fmt},
    open::{OpenOption, OpenParamOption},
    result::ResultComp,
    string::CCSID,
    structs,
    types::{MQPMO, MessageFormat},
};

impl option::PutMessage for str {
    fn render(&self) -> Cow<'_, [u8]> {
        self.as_bytes().into()
    }

    fn format(&self) -> MessageFormat {
        MessageFormat {
            ccsid: CCSID(1208),
            encoding: constants::MQENC_NATIVE,
            fmt: TextEnc::Ascii(fmt::MQFMT_STRING),
        }
    }
}

impl<B: AsRef<[u8]>> option::PutMessage for (B, MessageFormat) {
    fn render(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_ref())
    }

    fn format(&self) -> MessageFormat {
        self.1
    }
}

#[cfg(feature = "mqai")]
mod mqai {
    use libmqm_default as default;
    use libmqm_sys as mq;

    use super::option;
    use crate::{MqaiLibrary, Object, bag, connection::AsConnection, header::TextEnc, result::ResultComp, structs, types};

    impl<C: AsConnection> Object<C>
    where
        C::Lib: MqaiLibrary,
    {
        /// # Errors
        /// Returns an error if MQ cannot convert and put the bag.
        ///
        /// # Panics
        /// Panics if the put options configure MQMD or MQPMO versions unsupported by this build.
        pub fn put_bag<'po>(
            &self,
            put_options: &impl option::PutOption<'po>,
            format: TextEnc<types::Fmt>,
            bag: &bag::Bag<impl bag::BagDrop, impl MqaiLibrary>,
        ) -> ResultComp<()> {
            self.put_bag_with(put_options, format, bag)
        }

        /// # Errors
        /// Returns an error if MQ cannot convert and put the bag or extract the requested attribute.
        ///
        /// # Panics
        /// Panics if the put options configure MQMD or MQPMO versions unsupported by this build.
        pub fn put_bag_with<'po, R>(
            &self,
            put_options: &impl option::PutOption<'po>,
            format: TextEnc<types::Fmt>,
            bag: &bag::Bag<impl bag::BagDrop, impl MqaiLibrary>,
        ) -> ResultComp<R>
        where
            R: option::PutAttr,
        {
            let md = structs::MQMD::new(mq::MQMD {
                Format: format.into_ascii().into(),
                ..default::MQMD_DEFAULT
            });
            let mqpmo = structs::MQPMO::new(default::MQPMO_DEFAULT);

            let mut put_param = (md, mqpmo);
            put_options.apply_param(&mut put_param);
            assert!(put_param.0.Version <= mq::MQMD_CURRENT_VERSION);
            assert!(put_param.1.Version <= mq::MQPMO_CURRENT_VERSION);

            R::put_extract(&mut put_param, |(md, pmo)| {
                let connection = self.connection.as_connection();
                // SAFETY: Implementors of PutOption must ensure the MQPMO is correctly populate
                unsafe {
                    connection
                        .mq
                        .mq_put_bag(connection.handle, self.handle(), &mut **md, &mut *pmo, bag.handle())
                }
            })
        }
    }
}

impl<C: AsConnection> Object<C> {
    /// Put a message to an already-open queue or topic.
    ///
    /// Prefer this method when you will use the open object for multiple
    /// operations. Open it with [`Object::open`](crate::Object::open), passing
    /// the appropriate [`MQOO_OUTPUT`](crate::constants::MQOO_OUTPUT) option.
    /// For a one-off put without keeping an object open, use
    /// [`Connection::put_message`](crate::Connection::put_message), which uses
    /// MQPUT1.
    ///
    /// The message may be a string (`&str`, sent as UTF-8 with
    /// `MQFMT_STRING`) or a byte buffer paired with a
    /// [`MessageFormat`](crate::types::MessageFormat). `put_options` controls
    /// MQPMO behavior, independently of the options used to open the object.
    /// Wrap an individual put option in `Some` or `None` to apply or skip it
    /// based on runtime configuration. Skipping an option does not clear a
    /// setting supplied by another option in the tuple. Successful MQ warnings
    /// are preserved in the returned
    /// [`Completion`](crate::result::Completion).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mqi::{Object, constants, prelude::*, types::QueueName};
    ///
    /// # #[cfg(feature = "link")]
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let connection = mqi::connect_default(mqi::connection::NO_CONNECT_OPTIONS).warn_as_error()?;
    /// let queue = Object::open(
    ///     &connection,
    ///     &(QueueName(mqstr!("DEV.QUEUE.1")), constants::MQOO_OUTPUT),
    /// )
    /// .warn_as_error()?;
    ///
    /// queue.put_message(&(), "Hello, MQ").warn_as_error()?;
    /// # Ok(())
    /// # }
    /// # #[cfg(not(feature = "link"))]
    /// # fn main() {}
    /// ```
    ///
    /// # Errors
    /// Returns an error if MQ cannot put the message.
    ///
    /// # Panics
    /// Panics if the put options configure MQMD or MQPMO versions unsupported by this build.
    pub fn put_message<'po>(
        &self,
        put_options: &impl option::PutOption<'po>,
        message: &(impl option::PutMessage + ?Sized),
    ) -> ResultComp<()> {
        self.put_message_with(put_options, message)
    }

    /// Put a message to an open object and extract additional attributes.
    ///
    /// This is [`put_message`](Self::put_message) with a result value extracted
    /// through [`PutAttr`](crate::put::PutAttr). Use it when the MQPUT result
    /// includes an attribute you need, such as a generated message identifier.
    /// The put options, message formats, and warning behavior are the same as
    /// for [`put_message`](Self::put_message).
    ///
    /// # Errors
    /// Returns an error if MQ cannot put the message or extract the requested attribute.
    ///
    /// # Panics
    /// Panics if the put options configure MQMD or MQPMO versions unsupported by this build.
    pub fn put_message_with<'po, R>(
        &self,
        put_options: &impl option::PutOption<'po>,
        message: &(impl option::PutMessage + ?Sized),
    ) -> ResultComp<R>
    where
        R: option::PutAttr,
    {
        put(put_options, message, |(md, pmo), data| {
            let connection = self.connection.as_connection();
            // SAFETY: Implementors of PutOption must ensure the MQPMO is correctly populated
            unsafe {
                connection
                    .mq
                    .mqput(connection.handle, self.handle(), Some(&mut **md), pmo, data)
            }
        })
    }
}

fn put<'po, T, F>(options: &impl option::PutOption<'po>, message: &(impl option::PutMessage + ?Sized), put: F) -> ResultComp<T>
where
    T: option::PutAttr,
    F: FnOnce(&mut option::PutParam, &[u8]) -> ResultComp<()>,
{
    let MessageFormat {
        ccsid: CCSID(ccsid),
        encoding,
        fmt,
    } = message.format();
    let md = structs::MQMD::new(mq::MQMD {
        CodedCharSetId: ccsid,
        Encoding: encoding.0,
        Format: *fmt.into_ascii().as_ref(),
        ..default::MQMD_DEFAULT
    });
    let mqpmo = structs::MQPMO::new(default::MQPMO_DEFAULT);

    let mut put_param = (md, mqpmo);

    options.apply_param(&mut put_param);
    assert!(put_param.0.Version <= mq::MQMD_CURRENT_VERSION);
    assert!(put_param.1.Version <= mq::MQPMO_CURRENT_VERSION);

    T::put_extract(&mut put_param, |param| put(param, &message.render()))
}

impl<L: Library<MQ: mq::Mqi>, H> Connection<L, H> {
    /// Put one message to a queue or topic without opening an object first.
    ///
    /// This method uses MQPUT1: it opens the destination, puts the message, and
    /// closes the destination as part of one MQ call. It is convenient for a
    /// one-off put. If you will perform multiple operations on the same queue or
    /// topic, open it once with [`Object::open`](crate::Object::open) and use
    /// [`Object::put_message`](crate::Object::put_message) instead.
    ///
    /// The first argument selects the destination and MQOO-style open options;
    /// the second provides MQPMO options. These are separate option groups and
    /// each can be a single value or a tuple. The message may be a string
    /// (`&str`, sent as UTF-8 with `MQFMT_STRING`) or a byte buffer paired with
    /// a [`MessageFormat`](crate::types::MessageFormat). Successful MQ warnings
    /// are preserved in the returned [`Completion`](crate::result::Completion).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mqi::{prelude::*, types::QueueName};
    ///
    /// # #[cfg(feature = "link")]
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let connection = mqi::connect_default(mqi::connection::NO_CONNECT_OPTIONS).warn_as_error()?;
    /// connection
    ///     .put_message(&QueueName(mqstr!("DEV.QUEUE.1")), &(), "Hello, MQ")
    ///     .warn_as_error()?;
    /// # Ok(())
    /// # }
    /// # #[cfg(not(feature = "link"))]
    /// # fn main() {}
    /// ```
    ///
    /// To supply a byte buffer and message format explicitly:
    ///
    /// ```no_run
    /// use mqi::{prelude::*, types::{MessageFormat, QueueName}};
    ///
    /// # #[cfg(feature = "link")]
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let connection = mqi::connect_default(mqi::connection::NO_CONNECT_OPTIONS).warn_as_error()?;
    /// let format = MessageFormat::from(mqstr!("MQSTR"));
    /// let bytes = b"Hello, MQ";
    /// connection
    ///     .put_message(
    ///         &QueueName(mqstr!("DEV.QUEUE.1")),
    ///         &(),
    ///         &(bytes.as_slice(), format),
    ///     )
    ///     .warn_as_error()?;
    /// # Ok(())
    /// # }
    /// # #[cfg(not(feature = "link"))]
    /// # fn main() {}
    /// ```
    ///
    /// # Errors
    /// Returns an error if MQ cannot open the destination and put the message.
    ///
    /// # Panics
    /// Panics if the open options configure an MQOD version or the put options configure an MQMD
    /// or MQPMO version unsupported by this build.
    #[inline]
    pub fn put_message<'po, 'oo>(
        &self,
        open_options: &impl OpenOption<'oo, MQPMO>,
        put_options: &impl option::PutOption<'po>,
        message: &(impl option::PutMessage + ?Sized),
    ) -> ResultComp<()> {
        self.put_message_with(open_options, put_options, message)
    }

    /// Put one message with MQPUT1 and extract additional attributes.
    ///
    /// This is [`put_message`](Self::put_message) with a result value extracted
    /// through [`PutAttr`](crate::put::PutAttr), such as a generated message
    /// identifier. The destination options, put options, message formats, and
    /// warning behavior are the same as for [`put_message`](Self::put_message).
    /// The attribute type is inferred from the expected result type; specify
    /// `put_message_with::<AttributeType>` if inference needs help.
    ///
    /// # Errors
    /// Returns an error if MQ cannot open the destination, put the message, or extract the attribute.
    ///
    /// # Panics
    /// Panics if the open options configure an MQOD version or the put options configure an MQMD
    /// or MQPMO version unsupported by this build.
    pub fn put_message_with<'po, 'oo, R>(
        &self,
        open_options: &impl OpenOption<'oo, MQPMO>,
        put_options: &impl option::PutOption<'po>,
        message: &(impl option::PutMessage + ?Sized),
    ) -> ResultComp<R>
    where
        R: option::PutAttr,
    {
        let mut open_params = OpenParamOption {
            mqod: structs::MQOD::new(default::MQOD_DEFAULT),
            options: MQPMO::default(),
        };
        open_options.apply_param(&mut open_params);
        assert!(open_params.mqod.Version <= mq::MQOD_CURRENT_VERSION);
        put(put_options, message, |(md, pmo), data| {
            let pmo_options: &mut MQPMO = pmo.Options.as_mut();
            pmo_options.insert(open_params.options);

            // SAFETY: Implementors of OpenOption and PutOption must ensure the MQOD and MQPMO are populated correctly
            unsafe { self.mq.mqput1(self.handle, &mut open_params.mqod, Some(&mut **md), pmo, data) }
        })
    }
}
