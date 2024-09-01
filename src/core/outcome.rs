use crate::{sys, ReasonCode, ResultCompErr};
use crate::{Completion, CompletionCode, Error};

#[derive(Default, Clone, derive_more::Deref, derive_more::DerefMut)]
pub struct MQIOutcome<T> {
    /// MQI verb that caused the failure
    pub verb: &'static str,
    /// Completion code of the MQI function call
    pub cc: CompletionCode,
    /// Reason code of the MQI function call
    pub rc: ReasonCode,
    /// Return value of the MQI function call
    #[deref]
    #[deref_mut]
    pub value: T,
}

pub type MQIOutcomeVoid = MQIOutcome<()>;

impl<T: Default> MQIOutcome<T> {
    #[must_use]
    pub fn with_verb(verb: &'static str) -> Self {
        Self { verb, ..Self::default() }
    }
}
impl<T> MQIOutcome<T> {
    #[must_use]
    pub fn new(verb: &'static str, value: T) -> Self {
        Self {
            verb,
            value,
            rc: ReasonCode::default(),
            cc: CompletionCode::default(),
        }
    }
}

impl<T, E: From<Error>> From<MQIOutcome<T>> for ResultCompErr<T, E> {
    fn from(outcome: MQIOutcome<T>) -> Self {
        let MQIOutcome { cc, rc, value, verb } = outcome;
        match cc.value() {
            sys::MQCC_OK => Ok(Completion::new(value)),
            sys::MQCC_WARNING => Ok(Completion::new_warning(value, (rc, verb))),
            _ => Err(Error(cc, verb, rc).into()),
        }
    }
}

impl<T, E: From<Error>> From<MQIOutcome<T>> for Result<T, E> {
    fn from(outcome: MQIOutcome<T>) -> Self {
        let MQIOutcome { cc, rc, value, verb } = outcome;
        match cc.value() {
            sys::MQCC_OK => Ok(value),
            _ => Err(Error(cc, verb, rc).into()),
        }
    }
}

/// Traces the MQI outcome
#[cfg(feature = "tracing")]
pub fn tracing_outcome<T: std::fmt::Debug>(outcome: &MQIOutcome<T>) {
    use crate::HasMqNames as _;

    let MQIOutcome { verb, cc, rc, value } = outcome;
    match cc.value() {
        sys::MQCC_OK => tracing::event!(
            tracing::Level::DEBUG,
            value = ?value,
            cc_name = cc.mq_primary_name(),
            cc = cc.value(),
            rc_name = rc.mq_primary_name(),
            rc = rc.value(),
            verb
        ),
        sys::MQCC_WARNING => tracing::event!(
            tracing::Level::WARN,
            value = ?value,
            cc_name = cc.mq_primary_name(),
            cc = cc.value(),
            rc_name = rc.mq_primary_name(),
            rc = rc.value(),
            verb
        ),
        _ => tracing::event!(
            tracing::Level::ERROR,
            cc_name = cc.mq_primary_name(),
            cc = cc.value(),
            rc_name = rc.mq_primary_name(),
            rc = rc.value(),
            verb
        ),
    }
}

/// Traces the MQI outcome without the value
#[cfg(feature = "tracing")]
pub fn tracing_outcome_basic<T>(outcome: &MQIOutcome<T>) {
    use crate::HasMqNames as _;

    let MQIOutcome { verb, cc, rc, .. } = outcome;
    match cc.value() {
        sys::MQCC_OK => tracing::event!(
            tracing::Level::DEBUG,
            cc_name = cc.mq_primary_name(),
            cc = cc.value(),
            rc_name = rc.mq_primary_name(),
            rc = rc.value(),
            verb
        ),
        sys::MQCC_WARNING => tracing::event!(
            tracing::Level::WARN,
            cc_name = cc.mq_primary_name(),
            cc = cc.value(),
            rc_name = rc.mq_primary_name(),
            rc = rc.value(),
            verb
        ),
        _ => tracing::event!(
            tracing::Level::ERROR,
            cc_name = cc.mq_primary_name(),
            cc = cc.value(),
            rc_name = rc.mq_primary_name(),
            rc = rc.value(),
            verb
        ),
    }
}
