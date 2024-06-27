#![allow(clippy::module_name_repetitions)]

use crate::sys;
use crate::{
    constants::mapping,
    impl_constant_lookup, HasMqNames, MqValue,
};
use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};
use thiserror::Error;

impl_constant_lookup!(MQRC, mapping::MQRC_FULL_CONST);
impl_constant_lookup!(MQCC, mapping::MQCC_CONST);

/// MQ API reason code (`MQRC_*`)
#[derive(Clone, Copy)]
pub struct MQRC;
pub type ReasonCode = MqValue<MQRC>;

/// MQ API completion code (`MQCC_*`)
#[derive(Clone, Copy)]
pub struct MQCC;
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
#[derive(Debug, Clone)]
#[must_use]
pub struct Completion<T>(pub T, pub Option<(ReasonCode, &'static str)>);

impl<T: std::process::Termination> std::process::Termination for Completion<T> {
    fn report(self) -> std::process::ExitCode {
        let Self(value, ..) = self;
        value.report()
    }
}

impl<T> Completion<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> Completion<U> {
        let Self(value, warning) = self;
        Completion(op(value), warning)
    }

    /// Returns the reason code associated with the warning. Returns `None` when no warning is issued.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn warning(&self) -> Option<&(ReasonCode, &'static str)> {
        let Self(_, warning) = self;
        warning.as_ref()
    }

    /// Discards the `MQCC_WARNING` in the Completion
    pub fn discard_warning(self) -> T {
        let Self(value, ..) = self;
        value
    }
}

impl<I: Iterator> Iterator for Completion<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let Self(value, ..) = self;
        value.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self(value, ..) = self;
        value.size_hint()
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

impl<T> AsMut<T> for Completion<T> {
    fn as_mut(&mut self) -> &mut T {
        let Self(value, ..) = self;
        value
    }
}

impl<T> AsRef<T> for Completion<T> {
    fn as_ref(&self) -> &T {
        let Self(value, ..) = self;
        value
    }
}

impl<T> Deref for Completion<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for Completion<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

/// MQ failure with `CompCode` != `MQCC_OK`. Has the associated verb and `ReasonCode`.
#[derive(Debug, Error)]
#[error("{0}: {1} - {2}")]
pub struct Error(pub CompletionCode, pub &'static str, pub ReasonCode);

/// Result of an MQI API call wrapped in a `Completion` for warnings
pub type ResultCompErr<T, E> = Result<Completion<T>, E>;
/// Result of an MQI API call wrapped in a `Completion` for warnings and with an MQ `Error` for errors
pub type ResultComp<T> = Result<Completion<T>, Error>;
/// Result of an MQI API call with an MQ `Error`
pub type ResultErr<T> = Result<T, Error>;

/// Extends a `ResultComp` with additional methods to handle warnings.
pub trait ResultCompExt<T> {
    /// Converts the MQ warning in the `Ok(Completion(..))` into an `Err`.
    fn warn_as_error(self) -> ResultErr<T>;
}

/// Extends a `ResultCompErr` with additional methods to handle warnings.
pub trait ResultCompErrExt<T, E> {
    /// Maps the the value of the MQI API Result, maintaining the `Completion` wrapper with any associated warning.
    fn map_completion<U, F: FnOnce(T) -> U>(self, op: F) -> ResultCompErr<U, E>;

    /// Discards the completion
    fn discard_completion(self) -> Result<T, E>;

    /// Returns the contained `Ok(Completion(..))` value, discarding any warning and consumes the `self` value.
    ///
    /// This function can panic, so use it with caution.
    ///
    /// # Panic
    /// Panics if the value is an `Err`, with a panic message provided by the `Err`'s value.
    fn unwrap_completion(self) -> T;
}

impl<T, E: std::fmt::Debug> ResultCompErrExt<T, E> for ResultCompErr<T, E> {

    fn map_completion<U, F: FnOnce(T) -> U>(self, op: F) -> ResultCompErr<U, E> {
        self.map(|mq| mq.map(op))
    }

    #[allow(clippy::unwrap_used)]
    fn unwrap_completion(self) -> T {
        self.unwrap().discard_warning()
    }
    
    fn discard_completion(self) -> Result<T, E> {
        self.map(|Completion(value, _)| value)
    }
}

impl<T> ResultCompExt<T> for ResultComp<T> {
    fn warn_as_error(self) -> ResultErr<T> {
        match self {
            Ok(Completion(_, Some((warn_cc, verb)))) => {
                Err(Error(CompletionCode::from(sys::MQCC_WARNING), verb, warn_cc))
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
        assert_eq!(
            ReasonCode::from(sys::MQRC_Q_MGR_ACTIVE).to_string(),
            "MQRC_Q_MGR_ACTIVE"
        );
        assert_eq!(ReasonCode::from(sys::MQRC_NONE).to_string(), "MQRC_NONE");
        assert_eq!(ReasonCode::from(-1).to_string(), "-1");
    }

    #[test]
    fn ibm_reference_url() {
        assert_eq!(
            ReasonCode::from(sys::MQRC_Q_ALREADY_EXISTS).ibm_reference_url("en", None),
            Some(
                "https://www.ibm.com/docs/en/ibm-mq/latest?topic=codes-2290-08f2-rc2290-mqrc-q-already-exists"
                    .to_owned()
            )
        );
    }
}
