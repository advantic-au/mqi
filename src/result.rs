use std::fmt::{Debug, Display};

use crate::{
    Connection, Library,
    connection::ConnectionEither,
    constants,
    types::{MQCC, MQRC},
};

/// A value returned from an MQ API call, optionally with a warning [`MQRC`]
#[derive(Debug, Clone, derive_more::Deref, derive_more::DerefMut, derive_more::AsRef, derive_more::AsMut)]
#[must_use]
pub struct Completion<T>(
    #[deref]
    #[deref_mut]
    #[as_ref]
    #[as_mut]
    pub T,
    pub Option<(MQRC, &'static str)>,
);

impl<T> Completion<T> {
    pub const fn new(value: T) -> Self {
        Self(value, None)
    }

    pub const fn new_warning(value: T, warning: (MQRC, &'static str)) -> Self {
        Self(value, Some(warning))
    }
}

impl<T: std::process::Termination> std::process::Termination for Completion<T> {
    fn report(self) -> std::process::ExitCode {
        self.discard_warning().report()
    }
}

impl<T> Completion<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> Completion<U> {
        let Self(value, warning) = self;
        Completion(op(value), warning)
    }

    /// Discards the `MQCC_WARNING` in the Completion
    pub fn discard_warning(self) -> T {
        let Self(value, ..) = self;
        value
    }

    /// Returns the reason code associated with the warning. Returns `None` when no warning is issued.
    #[must_use]
    pub const fn warning(&self) -> Option<(MQRC, &'static str)> {
        let Self(_, warning) = self;
        *warning
    }
}

impl<I: Iterator> Iterator for Completion<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let iter = &mut **self;
        iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let iter = &**self;
        iter.size_hint()
    }
}

impl<T: Display> Display for Completion<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self(value, Some((warning, verb))) => write!(f, "MQCC_WARNING: {verb} {warning} {value}"),
            Self(value, None) => write!(f, "MQCC_OK: {value}"),
        }
    }
}

/// MQ failure with [`MQCC`] != `MQCC_OK`. Has the associated verb and [`MQRC`].
#[derive(Debug, derive_more::Error, derive_more::Display)]
#[display("{_0}: {_1} - {_2}")]
pub struct Error(pub MQCC, pub &'static str, pub MQRC);

/// Result of an MQI API call wrapped in a `Completion` for warnings
pub type ResultCompErr<T, E> = Result<Completion<T>, E>;
/// Result of an MQI API call wrapped in a `Completion` for warnings and with an MQ `Error` for errors
pub type ResultComp<T> = Result<Completion<T>, Error>;
/// Result of an MQI API call with an MQ `Error`
pub type ResultErr<T> = Result<T, Error>;

/// Extends a `ResultComp` with additional methods to handle warnings.
pub trait ResultCompExt<T, E> {
    /// Converts the MQ warning in the `Ok(Completion(..))` into an `Err`.
    fn warn_as_error(self) -> Result<T, E>;
}

pub trait WithMqError {
    fn mqi_error(&self) -> Option<&Error>;
}

impl WithMqError for Error {
    fn mqi_error(&self) -> Option<&Error> {
        Some(self)
    }
}

/// Extends a `ResultCompErr` with additional methods to handle warnings.
pub trait ResultCompErrExt<T, E> {
    /// Maps the value of the MQI API Result, maintaining the `Completion` wrapper with any associated warning.
    fn map_completion<U, F: FnOnce(T) -> U>(self, op: F) -> ResultCompErr<U, E>;

    /// Discards the completion
    fn discard_warning(self) -> Result<T, E>;

    /// Returns the contained `Ok(Completion(..))` value, discarding any warning and consumes the `self` value.
    ///
    /// This function can panic, so use it with caution.
    ///
    /// ## Panic
    /// Panics if the value is an `Err`, with a panic message provided by the `Err`'s value.
    fn unwrap_completion(self) -> T
    where
        E: std::fmt::Debug;

    /// [Leak](Connection::leak) the connection if the [Completion] contains a [`MQRC_ALREADY_CONNECTED`](constants::MQRC_ALREADY_CONNECTED) warning
    fn leak_already_connected<L: Library<MQ: libmqm_sys::Mqi> + Clone, H>(
        self,
    ) -> ResultCompErr<ConnectionEither<'static, L, H>, E>
    where
        T: Into<Connection<L, H>>;
}

impl<T, E> ResultCompErrExt<T, E> for ResultCompErr<T, E> {
    fn map_completion<U, F: FnOnce(T) -> U>(self, op: F) -> ResultCompErr<U, E> {
        self.map(|mq| mq.map(op))
    }

    #[expect(clippy::unwrap_used)]
    fn unwrap_completion(self) -> T
    where
        E: std::fmt::Debug,
    {
        self.unwrap().discard_warning()
    }

    fn discard_warning(self) -> Result<T, E> {
        self.map(Completion::discard_warning)
    }

    fn leak_already_connected<L: Library<MQ: libmqm_sys::Mqi> + Clone, H>(
        self,
    ) -> ResultCompErr<ConnectionEither<'static, L, H>, E>
    where
        T: Into<Connection<L, H>>,
    {
        self.map(|comp| match comp {
            Completion(connection, warn @ Some((constants::MQRC_ALREADY_CONNECTED, _))) => {
                Completion(ConnectionEither::Ref(connection.into().leak()), warn)
            }
            other => other.map(|c| ConnectionEither::Owned(c.into())),
        })
    }
}

impl<T, E: From<Error>> ResultCompExt<T, E> for ResultCompErr<T, E> {
    fn warn_as_error(self) -> Result<T, E> {
        match self {
            Ok(Completion(_, Some((warn_cc, verb)))) => Err(E::from(Error(constants::MQCC_WARNING, verb, warn_cc))),
            other => other.map(|Completion(value, ..)| value),
        }
    }
}

#[cfg(test)]
mod test {
    #[cfg(feature = "mock")]
    #[test]
    fn leak_already_connected() {
        use super::*;
        use crate::test::mock;

        let connection = mock::connect_ok(|_| {});
        let subject_warn: ResultComp<_> = Ok(Completion(connection, Some((constants::MQRC_ALREADY_CONNECTED, "verb"))));
        let result = subject_warn.leak_already_connected();
        assert!(matches!(result, Ok(Completion(ConnectionEither::Ref(_), _))));

        let connection = mock::connect_ok(|_| {});
        let subject: ResultComp<_> = Ok(Completion(connection, None));
        let result = subject.leak_already_connected();
        assert!(matches!(result, Ok(Completion(ConnectionEither::Owned(_), _))));
    }
}
