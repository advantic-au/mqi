use crate::{
    Object,
    connection::AsConnection,
    result::{Error, ResultComp, ResultCompErr},
    structs, types,
};

pub struct SubscribeState<C: AsConnection> {
    pub subscription: super::function::Subscription<C>,
    pub object: Option<Object<C>>,
}

#[derive(Debug)]
pub struct SubscribeParam<'a> {
    pub sd: structs::MQSD<'a>,
    pub close_options: types::MQCO,
    pub provided_object: types::MQLONG,
}

#[derive(Debug)]
pub struct SubscribeRequestParam {
    pub sro: structs::MQSRO,
    pub sr: types::MQSR,
}

pub trait SubscribeValue<C: AsConnection> {
    type Error: From<Error> + std::fmt::Debug;

    fn subscribe_consume<'so, F>(param: &mut SubscribeParam<'so>, mqi: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut SubscribeParam<'so>) -> ResultComp<SubscribeState<C>>,
        Self: std::marker::Sized;
}

pub trait SubscribeAttr<C: AsConnection> {
    fn subscribe_extract<'so, F>(param: &mut SubscribeParam<'so>, mqi: F) -> ResultComp<(Self, SubscribeState<C>)>
    where
        F: FnOnce(&mut SubscribeParam<'so>) -> ResultComp<SubscribeState<C>>,
        Self: Sized;
}

/// A trait that manipulates the parameters to the [`MQSUB`](`libmqm_sys::MQSUB`) function
#[diagnostic::on_unimplemented(
    message = "{Self} does not implement `SubscribeOption` so it can't be used as an argument for MQI subscribe"
)]
/// # Safety
/// This trait can directly manipulate the [`MQSD`](structs::MQSD) structure which is used by [`MQSUB`](libmqm_sys::MQSUB).
/// Incorrect values in the [`MQSD`](structs::MQSD) can lead to undefined behaviour.
///
/// Implementations of [`SubscribeOption`] must ensure that pointers and offsets contained in the structure point to active data.
pub unsafe trait SubscribeOption<'so> {
    fn apply_param(&self, param: &mut SubscribeParam<'so>);
}

pub trait SubscribeRequestOption {
    fn apply_param(&self, param: &mut SubscribeRequestParam);
}
