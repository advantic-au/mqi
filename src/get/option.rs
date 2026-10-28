use std::num::NonZero;

use crate::{
    macros::all_option_tuples,
    result::{ResultComp, ResultCompErr},
    structs,
    traits::Buffer,
    types,
};

#[derive(Debug, Clone)]
pub struct GetParam {
    pub md: structs::MQMD,
    pub gmo: structs::MQGMO,
}

/// ## Examples
/// Implements [`GetValue`] for a fixed array of bytes
///
/// ```
/// use std::num::NonZero;
/// use mqi::{get, prelude::*, traits::Buffer, result::Error, result::ResultComp};
///
/// pub struct Fixed<const N: usize>(pub [u8; N]);
///
/// impl<'b, const N: usize, B> get::GetValue<'b, u8, B> for Fixed<N> {
///    type Error = Error;
///
///    fn get_consume<F>(param: &mut get::GetParam, get: F) -> ResultComp<Self>
///    where
///        F: FnOnce(&mut get::GetParam) -> ResultComp<get::GetState<B>>,
///        B: Buffer<'b, u8>,
///    {
///        get(param).map_completion(|state| {
///            // Copy the message data into the fixed array
///            let msg_data: &[u8] = &state.buffer.as_ref()[..state.data_length];
///            let mut target = Self([0; N]);
///            target.0[..state.data_length].copy_from_slice(msg_data);
///            target
///        })
///    }
///
///    fn get_max_data_size() -> Option<NonZero<usize>> {
///        NonZero::new(N)
///    }
/// }
/// ```
pub trait GetValue<'b, R, B>: std::marker::Sized {
    type Error: std::fmt::Debug;

    /// Execute and consumes the result of the provided `get` function, creating `Self` from the [`GetState`]
    fn get_consume<F>(param: &mut GetParam, get: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        B: Buffer<'b, R>;

    /// The maximum size in bytes `Self` can consume from a `get` function call
    #[must_use]
    #[inline]
    fn get_max_data_size() -> Option<NonZero<usize>> {
        None
    }
}

/// Options accepted by the message get functions.
///
/// Options can be combined in tuples. Wrap an individual option in
/// `Option<T>` to apply it conditionally at runtime: `Some(value)` applies the
/// option and `None` skips it. Skipping an option does not undo a setting
/// supplied by another option in the tuple. Tuple options are applied from
/// right to left, so the leftmost option takes precedence when options set the
/// same field.
///
/// This trait manipulates the parameters to [`MQGET`](libmqm_sys::MQGET).
///
/// ```
/// use mqi::get::{GetOption, GetWait};
///
/// fn accepts_get_option(_: &impl GetOption) {}
///
/// let wait_for_message = true;
/// let wait = wait_for_message.then_some(GetWait::Wait(5_000));
/// accepts_get_option(&wait);
/// ```
#[diagnostic::on_unimplemented(message = "{Self} does not implement `GetOption` so it can't be used as an argument for MQI get")]
pub trait GetOption {
    fn apply_param(&self, param: &mut GetParam);
}

pub trait GetAttr<'b, R> {
    fn get_extract<F, B>(param: &mut GetParam, get: F) -> ResultComp<(Self, GetState<B>)>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<GetState<B>>,
        B: Buffer<'b, R>,
        Self: Sized;
}

#[cfg(feature = "mqai")]
pub trait GetBagAttr {
    fn get_bag_extract<F>(param: &mut GetParam, mqi: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut GetParam) -> ResultComp<()>,
        Self: Sized;
}

#[derive(Debug, Clone)]
pub struct GetState<B> {
    /// The buffer holding the message data from the `MQGET` call.
    pub buffer: B,
    /// The length of the message data returned by the `MQGET` call, confined by buffer size
    pub data_length: usize,
    /// The full length of the message data unconfined by buffer size
    pub message_length: usize,
    /// The format of the returned message
    pub format: types::MessageFormat,
}

impl<B> GetState<B> {
    pub fn into_truncated_buffer<'b, R>(self) -> B
    where
        B: Buffer<'b, R>,
    {
        self.buffer.truncate(self.data_length)
    }
}

all_option_tuples!(GetOption, GetParam);
