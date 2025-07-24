use libmqm_sys::{self as mq, Exits, exits};
#[cfg(feature = "tracing")]
use {crate::support::outcome::tracing_outcome, tracing::instrument};

use crate::{ExitsLibrary, MqFunctions, result::ResultComp, support::outcome::MqiOutcomeVoid, types};

impl<L: ExitsLibrary> MqFunctions<L> {
    /// Register Entry Point
    ///
    /// ## Safety
    /// Consumers of [`mqxep`](Self::mqxep) must ensure
    /// * [`MQIEP::Version`](exits::MQIEP::Version) <= [`MQIEP_CURRENT_VERSION`](exits::MQIEP_CURRENT_VERSION)
    /// * `entry_point` points to a valid function
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub unsafe fn mqxep(
        &self,
        config: &exits::MQIEP,
        reason: types::MQXR,
        function: types::MQXF,
        entry_point: mq::PMQFUNC,
        exit_opts: Option<&exits::MQXEPO>,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQXEP");
        unsafe {
            self.0.lib().MQXEP(
                (&raw const *config).cast_mut(),
                reason.0,
                function.0,
                entry_point,
                exit_opts,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Add Component Entry Point
    ///
    /// ## Safety
    /// Consumers of [`mqzep`](Self::mqzep) must ensure
    /// * [`MQIEP::Version`](exits::MQIEP::Version) <= [`MQIEP_CURRENT_VERSION`]
    /// * `entry_point` points to a valid function
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub unsafe fn mqzep(&self, config: &exits::MQIEP, function: mq::MQLONG, entry_point: mq::PMQFUNC) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQZEP");
        unsafe {
            self.0.lib().MQZEP(
                (&raw const *config).cast_mut(),
                function,
                entry_point,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }
}
