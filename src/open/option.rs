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
