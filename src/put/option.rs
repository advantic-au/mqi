use std::borrow::Cow;

use crate::{ResultComp, structs, types::MessageFormat};

/// A trait that provides a rendered message for the [`mqput`](`crate::MqFunctions::mqput`) function
#[diagnostic::on_unimplemented(message = "{Self} does not implement `PutMessage` so it can't be used as an argument for MQI put")]
pub trait PutMessage {
    fn render(&self) -> Cow<'_, [u8]>;
    fn format(&self) -> MessageFormat;
}

pub type PutParam<'a> = (structs::MQMD2, structs::MQPMO<'a>);

/// A trait that manipulates the parameters to the [`MQPUT`](`::libmqm_sys::MQPUT`) function
///
/// # Safety
/// This trait can directly manipulate the [`MQPMO`](structs::MQPMO) structure which is used by [`MQPUT`](libmqm_sys::MQPUT)
/// and [`MQPUT1`](libmqm_sys::MQPUT1). Incorrect values in the [`MQPMO`](structs::MQPMO) can lead to undefined behaviour.
///
/// Implementations of the [`PutOption`] trait must ensure that pointers and offsets contained in the structure point to active data.
#[diagnostic::on_unimplemented(message = "{Self} does not implement `PutOption` so it can't be used as an argument for MQI put")]
pub unsafe trait PutOption<'po> {
    fn apply_param(&self, param: &mut PutParam<'po>);
}

/// # Safety
/// This trait can directly manipulate the [`MQPMO`](structs::MQPMO) structure which is used by [`MQPUT`](libmqm_sys::MQPUT)
/// and [`MQPUT1`](libmqm_sys::MQPUT1). Incorrect values in the [`MQPMO`](structs::MQPMO) can lead to undefined behaviour.
///
/// Implementations of the [`PutAttr`] trait must ensure that pointers and offsets contained in the structure point to active data.
pub unsafe trait PutAttr {
    fn put_bag_extract<'p, F>(param: &mut PutParam<'p>, mqi: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut PutParam<'p>) -> ResultComp<()>,
        Self: Sized;
}
