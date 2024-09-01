use crate::core::values::{MQCC, MQRC};
use crate::sys;
use crate::{HasMqNames, MqValue};
use std::fmt::{Debug, Display};

/// MQ API reason code (`MQRC_*`)
pub type ReasonCode = MqValue<MQRC>;

/// MQ API completion code (`MQCC_*`)
pub type CompletionCode = MqValue<MQCC>;

impl Default for ReasonCode {
    fn default() -> Self {
        Self::from(sys::MQRC_NONE)
    }
}

impl Default for CompletionCode {
    fn default() -> Self {
        Self::from(sys::MQCC_UNKNOWN)
    }
}

impl ReasonCode {
    #[must_use]
    pub fn ibm_reference_url(&self, language: &str, version: Option<&str>) -> Option<String> {
        let name = self.mq_primary_name()?.to_lowercase().replace('_', "-");
        let version = version.unwrap_or("latest");
        let code = self.value();
        Some(format!(
            "https://www.ibm.com/docs/{language}/ibm-mq/{version}?topic=codes-{code}-{code:04x}-rc{code}-{name}"
        ))
    }
}
/// A value returned from an MQ API call, optionally with a warning `ReasonCode`
#[derive(Debug, Clone, derive_more::Deref, derive_more::DerefMut, derive_more::AsRef, derive_more::AsMut)]
#[must_use]
pub struct Completion<T>(
    #[deref]
    #[deref_mut]
    #[as_ref]
    #[as_mut]
    pub T,
    pub Option<(ReasonCode, &'static str)>,
);

impl<T> Completion<T> {
    pub const fn new(value: T) -> Self {
        Self(value, None)
    }

    pub const fn new_warning(value: T, warning: (ReasonCode, &'static str)) -> Self {
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
    #[allow(clippy::missing_const_for_fn)]
    pub fn warning(&self) -> Option<(ReasonCode, &'static str)> {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self(value, Some((warning, verb))) => write!(f, "MQCC_WARNING: {verb} {warning} {value}"),
            Self(value, None) => write!(f, "MQCC_OK: {value}"),
        }
    }
}

/// MQ failure with `CompCode` != `MQCC_OK`. Has the associated verb and `ReasonCode`.
#[derive(Debug, derive_more::Error, derive_more::Display)]
#[display("{_0}: {_1} - {_2}")]
pub struct Error(pub CompletionCode, pub &'static str, pub ReasonCode);

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

pub trait WithMQError {
    fn mqi_error(&self) -> Option<&Error>;
}

impl WithMQError for Error {
    fn mqi_error(&self) -> Option<&Error> {
        Some(self)
    }
}

/// Extends a `ResultCompErr` with additional methods to handle warnings.
pub trait ResultCompErrExt<T, E> {
    /// Maps the the value of the MQI API Result, maintaining the `Completion` wrapper with any associated warning.
    fn map_completion<U, F: FnOnce(T) -> U>(self, op: F) -> ResultCompErr<U, E>;

    /// Discards the completion
    fn discard_warning(self) -> Result<T, E>;

    /// Returns the contained `Ok(Completion(..))` value, discarding any warning and consumes the `self` value.
    ///
    /// This function can panic, so use it with caution.
    ///
    /// # Panic
    /// Panics if the value is an `Err`, with a panic message provided by the `Err`'s value.
    fn unwrap_completion(self) -> T;
}

impl<T, E> ResultCompErrExt<T, E> for ResultCompErr<T, E>
where
    E: std::fmt::Debug, // for unwrap_completion
{
    fn map_completion<U, F: FnOnce(T) -> U>(self, op: F) -> ResultCompErr<U, E> {
        self.map(|mq| mq.map(op))
    }

    #[allow(clippy::unwrap_used)]
    fn unwrap_completion(self) -> T {
        self.unwrap().discard_warning()
    }

    fn discard_warning(self) -> Result<T, E> {
        self.map(Completion::discard_warning)
    }
}

impl<T, E: From<Error>> ResultCompExt<T, E> for ResultCompErr<T, E> {
    fn warn_as_error(self) -> Result<T, E> {
        match self {
            Ok(Completion(_, Some((warn_cc, verb)))) => {
                Err(E::from(Error(CompletionCode::from(sys::MQCC_WARNING), verb, warn_cc)))
            }
            other => other.map(|Completion(value, ..)| value),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sys;
    use crate::ReasonCode;

    #[test]
    fn mqmd_new() {
        let d = sys::MQMD2::default();
        assert_eq!(d.Version, 2);
    }

    #[test]
    fn reason_code_display() {
        assert_eq!(ReasonCode::from(sys::MQRC_Q_MGR_ACTIVE).to_string(), "MQRC_Q_MGR_ACTIVE");
        assert_eq!(ReasonCode::from(sys::MQRC_NONE).to_string(), "MQRC_NONE");
        assert_eq!(ReasonCode::from(-1).to_string(), "-1");
    }

    #[test]
    fn ibm_reference_url() {
        assert_eq!(
            ReasonCode::from(sys::MQRC_Q_ALREADY_EXISTS).ibm_reference_url("en", None),
            Some("https://www.ibm.com/docs/en/ibm-mq/latest?topic=codes-2290-08f2-rc2290-mqrc-q-already-exists".to_owned())
        );
    }
}
