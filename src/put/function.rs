use std::borrow::Cow;

use libmqm_default as default;
use libmqm_sys::{MQMD2, Mqi};

use super::option;
use crate::{
    CCSID, Conn, ConnectionHandle, Library, MqFunctions, Object, ResultComp, constants,
    headers::{TextEnc, fmt},
    open::{OpenOption, OpenParamOption},
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
    // use super::option;
    use libmqm_default as default;
    use libmqm_sys::{MQMD2, Mqai};

    use super::option;
    use crate::{Conn, Library, Object, ResultComp, bag, headers::TextEnc, structs, types};

    impl<C: Conn> Object<C>
    where
        C::Lib: Library<MQ: Mqai>,
    {
        pub fn put_bag<'po>(
            &self,
            put_options: &impl option::PutOption<'po>,
            format: TextEnc<types::Fmt>,
            bag: &bag::Bag<impl bag::BagDrop, impl Library<MQ: Mqai>>,
        ) -> ResultComp<()> {
            self.put_bag_with(put_options, format, bag)
        }

        pub fn put_bag_with<'po, R>(
            &self,
            put_options: &impl option::PutOption<'po>,
            format: TextEnc<types::Fmt>,
            bag: &bag::Bag<impl bag::BagDrop, impl Library<MQ: Mqai>>,
        ) -> ResultComp<R>
        where
            R: option::PutAttr,
        {
            let md = structs::MQMD2::new(MQMD2 {
                Format: format.into_ascii().into(),
                ..default::MQMD2_DEFAULT
            });
            let mqpmo = structs::MQPMO::new(default::MQPMO_DEFAULT);

            let mut put_param = (md, mqpmo);
            put_options.apply_param(&mut put_param);
            R::put_bag_extract(&mut put_param, |(md, pmo)| {
                let connection = self.connection();
                // SAFETY: Implementors of PutOption must ensure the MQPMO is correctly populate
                unsafe {
                    connection
                        .mq()
                        .mq_put_bag(connection.handle(), self.handle(), &mut **md, &mut *pmo, bag.handle())
                }
            })
        }
    }
}

impl<C: Conn> Object<C> {
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
            let connection = self.connection();
            // SAFETY: Implementors of PutOption must ensure the MQPMO is correctly populated
            unsafe {
                connection
                    .mq()
                    .mqput(connection.handle(), self.handle(), Some(&mut **md), pmo, data)
            }
        })
    }
}

pub fn put_message_with<'po, 'oo, R>(
    functions: &MqFunctions<impl Library<MQ: Mqi>>,
    handle: ConnectionHandle,
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
    put(put_options, message, |(md, pmo), data| {
        let pmo_options: &mut MQPMO = pmo.Options.as_mut();
        pmo_options.insert(open_params.options);

        // SAFETY: Implementors of OpenOption and PutOption must ensure the MQOD and MQPMO are populated correctly
        unsafe { functions.mqput1(handle, &mut open_params.mqod, Some(&mut **md), pmo, data) }
    })
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
    let md = structs::MQMD2::new(MQMD2 {
        CodedCharSetId: ccsid,
        Encoding: encoding.0,
        Format: *fmt.into_ascii().as_ref(),
        ..default::MQMD2_DEFAULT
    });
    let mqpmo = structs::MQPMO::new(default::MQPMO_DEFAULT);

    let mut put_param = (md, mqpmo);

    options.apply_param(&mut put_param);
    T::put_bag_extract(&mut put_param, |param| put(param, &message.render()))
}
