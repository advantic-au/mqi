use crate::{
    Completion, Error, ResultCompErr, constants,
    types::{MQCC, MQRC},
};

#[derive(Clone, derive_more::Deref, derive_more::DerefMut)]
pub struct MqiOutcome<T> {
    /// MQI verb that caused the failure
    pub verb: &'static str,
    /// Completion code of the MQI function call
    pub cc: MQCC,
    /// Reason code of the MQI function call
    pub rc: MQRC,
    /// Return value of the MQI function call
    #[deref]
    #[deref_mut]
    pub value: T,
}

impl<T: Default> Default for MqiOutcome<T> {
    fn default() -> Self {
        Self {
            verb: Default::default(),
            cc: constants::MQCC_UNKNOWN,
            rc: constants::MQRC_NONE,
            value: Default::default(),
        }
    }
}

pub type MqiOutcomeVoid = MqiOutcome<()>;

impl<T: Default> MqiOutcome<T> {
    #[must_use]
    pub fn with_verb(verb: &'static str) -> Self {
        Self { verb, ..Self::default() }
    }
}
impl<T> MqiOutcome<T> {
    #[must_use]
    pub const fn new(verb: &'static str, value: T) -> Self {
        Self {
            verb,
            value,
            rc: constants::MQRC_NONE,
            cc: constants::MQCC_UNKNOWN,
        }
    }
}

impl<T, E: From<Error>> From<MqiOutcome<T>> for ResultCompErr<T, E> {
    fn from(outcome: MqiOutcome<T>) -> Self {
        let MqiOutcome { cc, rc, value, verb } = outcome;
        match cc {
            constants::MQCC_OK => Ok(Completion::new(value)),
            constants::MQCC_WARNING => Ok(Completion::new_warning(value, (rc, verb))),
            _ => Err(Error(cc, verb, rc).into()),
        }
    }
}

impl<T, E: From<Error>> From<MqiOutcome<T>> for Result<T, E> {
    fn from(outcome: MqiOutcome<T>) -> Self {
        let MqiOutcome { cc, rc, value, verb } = outcome;
        match cc {
            constants::MQCC_OK => Ok(value),
            _ => Err(Error(cc, verb, rc).into()),
        }
    }
}

/// Traces the MQI outcome
#[cfg(feature = "tracing")]
pub fn tracing_outcome<T: std::fmt::Debug>(outcome: &MqiOutcome<T>) {
    use libmqm_constants::lookup::HasMqNames as _;

    let MqiOutcome { verb, cc, rc, value } = outcome;
    match *cc {
        constants::MQCC_OK => tracing::event!(
            tracing::Level::DEBUG,
            value = ?value,
            cc_name = cc.mq_primary_name(),
            cc = cc.0,
            rc_name = rc.mq_primary_name(),
            rc = rc.0,
            verb
        ),
        constants::MQCC_WARNING => tracing::event!(
            tracing::Level::WARN,
            value = ?value,
            cc_name = cc.mq_primary_name(),
            cc = cc.0,
            rc_name = rc.mq_primary_name(),
            rc = rc.0,
            verb
        ),
        _ => tracing::event!(
            tracing::Level::ERROR,
            cc_name = cc.mq_primary_name(),
            cc = cc.0,
            rc_name = rc.mq_primary_name(),
            rc = rc.0,
            verb
        ),
    }
}

/// Traces the MQI outcome without the value
#[cfg(feature = "tracing")]
pub fn tracing_outcome_basic<T>(outcome: &MqiOutcome<T>) {
    use libmqm_constants::lookup::HasMqNames as _;

    let MqiOutcome { verb, cc, rc, .. } = outcome;
    match *cc {
        constants::MQCC_OK => tracing::event!(
            tracing::Level::DEBUG,
            cc_name = cc.mq_primary_name(),
            cc = cc.0,
            rc_name = rc.mq_primary_name(),
            rc = rc.0,
            verb
        ),
        constants::MQCC_WARNING => tracing::event!(
            tracing::Level::WARN,
            cc_name = cc.mq_primary_name(),
            cc = cc.0,
            rc_name = rc.mq_primary_name(),
            rc = rc.0,
            verb
        ),
        _ => tracing::event!(
            tracing::Level::ERROR,
            cc_name = cc.mq_primary_name(),
            cc = cc.0,
            rc_name = rc.mq_primary_name(),
            rc = rc.0,
            verb
        ),
    }
}
