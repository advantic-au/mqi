use std::borrow::Cow;

use crate::{result::ResultComp, structs, types::MessageFormat};

/// A trait that provides a rendered message for the [`mqput`](`crate::MqFunctions::mqput`) function
#[diagnostic::on_unimplemented(message = "{Self} does not implement `PutMessage` so it can't be used as an argument for MQI put")]
pub trait PutMessage {
    fn render(&self) -> Cow<'_, [u8]>;
    fn format(&self) -> MessageFormat;
}

pub type PutParam<'a> = (structs::MQMD, structs::MQPMO<'a>);

/// Options accepted by the message put functions.
///
/// Options can be combined in tuples. Wrap an individual option in
/// `Option<T>` to apply it conditionally at runtime: `Some(value)` applies the
/// option and `None` skips it. Skipping an option does not undo a setting
/// supplied by another option in the tuple. Tuple options are applied from
/// right to left, so the leftmost option takes precedence when options set the
/// same field.
///
/// This trait manipulates the parameters to [`MQPUT`](libmqm_sys::MQPUT).
/// It is also used by [`MQPUT1`](libmqm_sys::MQPUT1).
///
/// ```
/// use mqi::{constants, put::PutOption};
///
/// fn accepts_put_option<'a>(_: &impl PutOption<'a>) {}
///
/// let use_syncpoint = true;
/// let syncpoint = use_syncpoint.then_some(constants::MQPMO_SYNCPOINT);
/// accepts_put_option(&syncpoint);
/// ```
///
/// ## Safety
/// This trait can directly manipulate the [`MQPMO`](structs::MQPMO) structure which is used by [`MQPUT`](libmqm_sys::MQPUT)
/// and [`MQPUT1`](libmqm_sys::MQPUT1). Incorrect values in the [`MQPMO`](structs::MQPMO) can lead to undefined behaviour.
///
/// Implementations of the [`PutOption`] trait must ensure that pointers and offsets contained in the structure point to active data.
#[diagnostic::on_unimplemented(message = "{Self} does not implement `PutOption` so it can't be used as an argument for MQI put")]
pub unsafe trait PutOption<'po> {
    fn apply_param(&self, param: &mut PutParam<'po>);
}

/// ## Safety
/// This trait can directly manipulate the [`MQPMO`](structs::MQPMO) structure which is used by [`MQPUT`](libmqm_sys::MQPUT)
/// and [`MQPUT1`](libmqm_sys::MQPUT1). Incorrect values in the [`MQPMO`](structs::MQPMO) can lead to undefined behaviour.
///
/// Implementations of the [`PutAttr`] trait must ensure that pointers and offsets contained in the structure point to active data.
pub unsafe trait PutAttr {
    fn put_extract<'p, F>(param: &mut PutParam<'p>, mqi: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut PutParam<'p>) -> ResultComp<()>,
        Self: Sized;
}
