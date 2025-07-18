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
    use crate::{Library, Object, bag, connection::AsConnection, header::TextEnc, result::ResultComp, structs, types};

    impl<C: AsConnection> Object<C>
    where
        C::Lib: Library<MQ: mq::Mqai>,
    {
        pub fn put_bag<'po>(
            &self,
            put_options: &impl option::PutOption<'po>,
            format: TextEnc<types::Fmt>,
            bag: &bag::Bag<impl bag::BagDrop, impl Library<MQ: mq::Mqai>>,
        ) -> ResultComp<()> {
            self.put_bag_with(put_options, format, bag)
        }

        pub fn put_bag_with<'po, R>(
            &self,
            put_options: &impl option::PutOption<'po>,
            format: TextEnc<types::Fmt>,
            bag: &bag::Bag<impl bag::BagDrop, impl Library<MQ: mq::Mqai>>,
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
    pub fn put_message<'po>(
        &self,
        put_options: &impl option::PutOption<'po>,
        message: &(impl option::PutMessage + ?Sized),
    ) -> ResultComp<()> {
        self.put_message_with(put_options, message)
    }

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
    /// Put a message to a queue or topic
    #[inline]
    pub fn put_message<'po, 'oo>(
        &self,
        open_options: &impl OpenOption<'oo, MQPMO>,
        put_options: &impl option::PutOption<'po>,
        message: &(impl option::PutMessage + ?Sized),
    ) -> ResultComp<()> {
        self.put_message_with(open_options, put_options, message)
    }

    /// Put a message to a queue or topic with a specified return type that implements [`PutAttr`](option::PutAttr).
    ///
    /// Type inference of the return value may not always work so you may have to explicitly state the return type using the
    /// `put_message_with::<Type>` syntax.
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
