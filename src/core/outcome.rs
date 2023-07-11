use crate::prelude::*;
use std::ops::{Deref, DerefMut};

use crate::sys;
use crate::{Completion, CompletionCode, Error, ReasonCode, ResultComp, ResultErr};

#[derive(Default)]
pub struct MQIOutcome<T> {
    /// MQI verb that caused the failure
    pub verb: &'static str,
    /// Completion code of the MQI function call
    pub cc: CompletionCode,
    /// Reason code of the MQI function call
    pub rc: ReasonCode,
    /// Return valie of the MQI function call
    pub value: T,
}

pub type MQIOutcomeVoid = MQIOutcome<()>;

impl<T: Default> MQIOutcome<T> {
    #[must_use]
    pub fn with_verb(verb: &'static str) -> Self {
        Self {
            verb,
            ..Self::default()
        }
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

impl<T> Deref for MQIOutcome<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for MQIOutcome<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> From<MQIOutcome<T>> for ResultComp<T> {
    fn from(outcome: MQIOutcome<T>) -> Self {
        let MQIOutcome { cc, rc, value, verb } = outcome;
        match cc.mq_value() {
            sys::MQCC_OK => Ok(Completion(value, None, verb)),
            sys::MQCC_WARNING => Ok(Completion(value, Some(rc), verb)),
            _ => Err(Error(cc, verb, rc)),
        }
    }
}

impl<T> From<MQIOutcome<T>> for ResultErr<T> {
    fn from(outcome: MQIOutcome<T>) -> Self {
        let MQIOutcome { cc, rc, value, verb } = outcome;
        match cc.mq_value() {
            sys::MQCC_OK => Ok(value),
            _ => Err(Error(cc, verb, rc)),
        }
    }
}

/// Traces the MQI outcome
#[cfg(feature = "tracing")]
pub fn tracing_outcome<T: std::fmt::Debug>(outcome: &MQIOutcome<T>) {
    let MQIOutcome { verb, cc, rc, value } = outcome;
    match cc.mq_value() {
        sys::MQCC_OK => tracing::event!(
            tracing::Level::DEBUG,
            value = ?value,
            cc_name = cc.mq_primary_name(),
            cc = cc.mq_value(),
            rc_name = rc.mq_primary_name(),
            rc = rc.mq_value(),
            verb
        ),
        sys::MQCC_WARNING => tracing::event!(
            tracing::Level::WARN,
            value = ?value,
            cc_name = cc.mq_primary_name(),
            cc = cc.mq_value(),
            rc_name = rc.mq_primary_name(),
            rc = rc.mq_value(),
            verb
        ),
        _ => tracing::event!(
            tracing::Level::ERROR,
            cc_name = cc.mq_primary_name(),
            cc = cc.mq_value(),
            rc_name = rc.mq_primary_name(),
            rc = rc.mq_value(),
            verb
        ),
    }
}

/// Traces the MQI outcome without the value
#[cfg(feature = "tracing")]
pub fn tracing_outcome_basic<T>(outcome: &MQIOutcome<T>) {
    let MQIOutcome { verb, cc, rc, .. } = outcome;
    match cc.mq_value() {
        sys::MQCC_OK => tracing::event!(
            tracing::Level::DEBUG,
            cc_name = cc.mq_primary_name(),
            cc = cc.mq_value(),
            rc_name = rc.mq_primary_name(),
            rc = rc.mq_value(),
            verb
        ),
        sys::MQCC_WARNING => tracing::event!(
            tracing::Level::WARN,
            cc_name = cc.mq_primary_name(),
            cc = cc.mq_value(),
            rc_name = rc.mq_primary_name(),
            rc = rc.mq_value(),
            verb
        ),
        _ => tracing::event!(
            tracing::Level::ERROR,
            cc_name = cc.mq_primary_name(),
            cc = cc.mq_value(),
            rc_name = rc.mq_primary_name(),
            rc = rc.mq_value(),
            verb
        ),
    }
}
