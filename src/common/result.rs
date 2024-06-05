#![allow(clippy::module_name_repetitions)]

use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};
use thiserror::Error;
use crate::{constants::{mapping, MQConstant}, impl_constant_lookup, HasMqNames};
use crate::sys;

/// MQ API reason code (`MQRC_*`)
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ReasonCode(pub sys::MQLONG);

#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub struct CompletionCode(pub sys::MQLONG);

impl ReasonCode {
    #[must_use]
    pub fn ibm_reference_url(&self, language: &str, version: Option<&str>) -> Option<String> {
        let name = self.mq_primary_name()?.to_lowercase().replace('_', "-");
        let version = version.unwrap_or("latest");
        let code = self.mq_value();
        Some(format!("https://www.ibm.com/docs/{language}/ibm-mq/{version}?topic=codes-{code}-{code:04x}-rc{code}-{name}"))
    }
}

impl_constant_lookup!(CompletionCode, mapping::MQCC_CONST);
impl_constant_lookup!(ReasonCode, mapping::MQRC_FULL_CONST);

impl MQConstant for CompletionCode {
    fn mq_value(&self) -> sys::MQLONG {
        let Self(value) = self;
        *value
    }
}

impl MQConstant for ReasonCode {
    fn mq_value(&self) -> sys::MQLONG {
        let Self(value) = self;
        *value
    }
}

impl Debug for ReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(rc) = self;
        let code = format!("{} = {rc}", self.mq_primary_name().unwrap_or("*UNKNOWN*"));
        f.debug_tuple("ReasonCode").field(&code).finish()
    }
}

impl Display for ReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(rc) = self;
        match self.mq_primary_name() {
            Some(name) => write!(f, "{name} = {rc}"),
            None => write!(f, "{rc}"),
        }
    }
}

impl Debug for CompletionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(cc) = self;
        let code = self.mq_primary_name().map_or(format!("{cc} (Unknown)"), ToOwned::to_owned);
        f.debug_tuple("CompletionCode").field(&code).finish()
    }
}

impl Display for CompletionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(cc) = self;
        match self.mq_primary_name() {
            Some(name) => write!(f, "{name}"),
            None => write!(f, "{cc}"),
        }
    }
}

/// A value returned from an MQ API call, optionally with a warning `ReasonCode`
#[derive(Debug, Clone)]
pub struct Completion<T>(pub T, pub Option<ReasonCode>, pub &'static str);

impl<T> std::process::Termination for Completion<T>
where
    T: std::process::Termination,
{
    fn report(self) -> std::process::ExitCode {
        let Self(value, ..) = self;
        value.report()
    }
}

impl<T> Completion<T> {
    pub fn map<U, F>(self, op: F) -> Completion<U>
    where
        F: FnOnce(T) -> U,
    {
        let Self(value, warning, verb) = self;
        Completion(op(value), warning, verb)
    }

    /// Returns the reason code associated with the warning. Returns `None` when no warning is issued.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn warning(&self) -> Option<ReasonCode> {
        let Self(_, warning, _) = self;
        *warning
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
            Self(value, Some(warning), verb) => write!(f, "MQCC_WARNING: {verb} {warning} {value}"),
            Self(value, None, verb) => write!(f, "MQCC_OK: {verb} {value}"),
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
        let Completion(result, ..) = self.unwrap();
        result
    }
}

impl<T> ResultCompExt<T> for ResultComp<T> {
    fn warn_as_error(self) -> ResultErr<T> {
        match self {
            Ok(Completion(_, Some(warn_cc), verb)) => Err(Error(CompletionCode(sys::MQCC_WARNING), verb, warn_cc)),
            other => other.map(|Completion(value, ..)| value),
        }
    }
}

impl Default for ReasonCode {
    fn default() -> Self {
        Self(sys::MQRC_NONE)
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
            ReasonCode(sys::MQRC_Q_MGR_ACTIVE).to_string(),
            "MQRC_Q_MGR_ACTIVE = 2222"
        );
        assert_eq!(ReasonCode(sys::MQRC_NONE).to_string(), "MQRC_NONE = 0");
        assert_eq!(ReasonCode(-1).to_string(), "-1");
    }

    #[test]
    fn ibm_reference_url() {
        assert_eq!(
            ReasonCode(sys::MQRC_Q_ALREADY_EXISTS).ibm_reference_url("en", None),
            Some("https://www.ibm.com/docs/en/ibm-mq/latest?topic=codes-2290-08f2-rc2290-mqrc-q-already-exists".to_owned())
        );
    }
}
