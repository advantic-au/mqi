use crate::{
    Object,
    connection::AsConnection,
    result::{Error, ResultComp, ResultCompErr},
    structs, types,
};

#[derive(Debug)]
/// Values returned by an MQ subscription call.
pub struct SubscribeState<C: AsConnection> {
    /// Subscription handle created by MQ.
    pub subscription: super::function::Subscription<C>,
    /// Optional queue object opened or associated with the subscription.
    pub object: Option<Object<C>>,
}

/// Subscription descriptor and output settings assembled for `MQSUB`.
#[derive(Debug, Clone)]
pub struct SubscribeParam<'a> {
    /// Topic, selection, and subscription settings passed to MQ.
    pub sd: structs::MQSD<'a>,
    /// Close options retained for the subscription handle.
    pub close_options: types::MQCO,
    /// Whether an existing object handle was provided by the caller.
    pub provided_object: types::MQLONG,
}

/// Request options and status returned by an MQ subscription request.
#[derive(Debug, Clone)]
pub struct SubscribeRequestParam {
    /// Subscription request options passed to `MQSUBRQ`.
    pub sro: structs::MQSRO,
    /// Subscription request status returned by MQ.
    pub sr: types::MQSR,
}

/// Converts the result of `MQSUB` into a caller-selected value.
pub trait SubscribeValue<C: AsConnection> {
    /// Error returned when MQSUB succeeds but the requested value cannot be extracted.
    type Error: From<Error> + std::fmt::Debug;

    /// # Errors
    /// Returns an error if MQSUB fails or the subscription value cannot be extracted.
    fn subscribe_consume<'so, F>(param: &mut SubscribeParam<'so>, mqi: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut SubscribeParam<'so>) -> ResultComp<SubscribeState<C>>,
        Self: std::marker::Sized;
}

/// Extracts an additional typed attribute from a subscription operation.
pub trait SubscribeAttr<C: AsConnection> {
    /// # Errors
    /// Returns an error if MQSUB fails or the requested attribute cannot be extracted.
    fn subscribe_extract<'so, F>(param: &mut SubscribeParam<'so>, mqi: F) -> ResultComp<(Self, SubscribeState<C>)>
    where
        F: FnOnce(&mut SubscribeParam<'so>) -> ResultComp<SubscribeState<C>>,
        Self: Sized;
}

/// A trait that manipulates the parameters to the [`MQSUB`](`libmqm_sys::MQSUB`) function
#[diagnostic::on_unimplemented(
    message = "{Self} does not implement `SubscribeOption` so it can't be used as an argument for MQI subscribe"
)]
/// ## Safety
/// This trait can directly manipulate the [`MQSD`](structs::MQSD) structure which is used by [`MQSUB`](libmqm_sys::MQSUB).
/// Incorrect values in the [`MQSD`](structs::MQSD) can lead to undefined behaviour.
///
/// Implementations of [`SubscribeOption`] must ensure that pointers and offsets contained in the structure point to active data.
pub unsafe trait SubscribeOption<'so> {
    /// Apply this option to the subscription descriptor and handle settings.
    fn apply_param(&self, param: &mut SubscribeParam<'so>);
}

/// Configures an MQ subscription request (`MQSUBRQ`).
pub trait SubscribeRequestOption {
    /// Apply this option to the subscription-request options.
    fn apply_param(&self, param: &mut SubscribeRequestParam);
}
