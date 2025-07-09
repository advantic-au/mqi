use libmqm_default as default;

use super::Object;
use crate::{ResultComp, ResultCompErr, constants, option::Conn, prelude::*, structs, types};

pub(super) mod option {
    use libmqm_constants::types::MQOO;

    use crate::{Error, ResultComp, ResultCompErr, structs};

    pub struct OpenParamOption<'a, T> {
        pub mqod: structs::MQOD<'a>,
        pub options: T,
    }

    pub type OpenParam<'a> = OpenParamOption<'a, MQOO>;

    /// A trait that manipulates the parameters to the [`MQOPEN`](`::libmqm_sys::MQOPEN`) function
    #[diagnostic::on_unimplemented(
        message = "{Self} does not implement `OpenOption` so it can't be used as an argument for MQI open"
    )]
    /// # Safety
    /// This trait can directly manipulate the [`MQOD`](structs::MQOD) structure which is used by [`MQOPEN`](libmqm_sys::MQOPEN).
    /// Incorrect values in the [`MQOD`](structs::MQOD) can lead to undefined behaviour.
    /// Implementations of the trait must ensure that pointers and offsets contained in the structure point to active data.
    pub unsafe trait OpenOption<'oo, T> {
        fn apply_param(&self, param: &mut OpenParamOption<'oo, T>);
    }

    /// # Safety
    /// This trait can directly manipulate the [`MQOD`](structs::MQOD) structure which is used by [`MQOPEN`](libmqm_sys::MQOPEN).
    /// Incorrect values in the [`MQOD`](structs::MQOD) can lead to undefined behaviour.
    /// Implementations of the trait must ensure that pointers and offsets contained in the structure point to active data.
    pub unsafe trait OpenValue<S> {
        type Error: From<Error> + std::fmt::Debug;

        fn open_consume<'oo, F>(param: &mut OpenParam<'oo>, mqi: F) -> ResultCompErr<Self, Self::Error>
        where
            F: FnOnce(&mut OpenParam<'oo>) -> ResultComp<S>,
            Self: std::marker::Sized;
    }

    /// # Safety
    /// This trait can directly manipulate the [`MQOD`](structs::MQOD) structure which is used by [`MQOPEN`](libmqm_sys::MQOPEN).
    /// Incorrect values in the [`MQOD`](structs::MQOD) can lead to undefined behaviour.
    /// Implementations of the trait must ensure that pointers and offsets contained in the structure point to active data.
    pub unsafe trait OpenAttr<S, O> {
        fn open_extract<'a, F>(param: &mut OpenParamOption<'a, O>, mqi: F) -> ResultComp<(Self, S)>
        where
            F: FnOnce(&mut OpenParamOption<'a, O>) -> ResultComp<S>,
            Self: Sized;
    }
}

impl<C: Conn> Object<C> {
    /// Establish access and return an MQ object ([`Object`])
    pub fn open<'oo>(connection: C, open_option: &impl option::OpenOption<'oo, types::MQOO>) -> ResultComp<Self> {
        Self::open_as(connection, open_option)
    }

    /// Establish access and return an MQ object ([`Object`]) and type inferred [`OpenAttr`](option::OpenAttr) in a tuple
    pub fn open_with<'oo, A>(connection: C, open_option: &impl option::OpenOption<'oo, types::MQOO>) -> ResultComp<(Self, A)>
    where
        A: option::OpenAttr<Self, types::MQOO>,
    {
        Self::open_as(connection, open_option)
    }

    /// Establish access and return a type inferred value of [`OpenValue`]
    pub(super) fn open_as<'oo, R>(
        connection: C,
        open_option: &impl option::OpenOption<'oo, types::MQOO>,
    ) -> ResultCompErr<R, <R as option::OpenValue<Self>>::Error>
    where
        R: option::OpenValue<Self>,
    {
        let mut oo = option::OpenParamOption {
            mqod: structs::MQOD::new(default::MQOD_DEFAULT),
            options: constants::MQOO_BIND_AS_Q_DEF,
        };
        open_option.apply_param(&mut oo);
        // SAFETY: Implementors of option::OpenOption must ensure MQOD structure is populated correctly for mqopen
        R::open_consume(&mut oo, |option::OpenParamOption { mqod, options }| unsafe {
            connection
                .mq()
                .mqopen(connection.handle(), mqod, *options)
                .map_completion(|handle| Self {
                    handle,
                    connection,
                    close_options: constants::MQCO_NONE,
                })
        })
    }
}
