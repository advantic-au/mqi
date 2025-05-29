use std::ptr;

use libmqm_sys::{Exits, lib as sys};
#[cfg(feature = "tracing")]
use {super::tracing_outcome, tracing::instrument};

use super::{ConnectionHandle, Library, MqFunctions, MqiOutcome, WriteRaw};
use crate::{
    CCSID, ResultComp,
    types::{MQDCC, MQLONG},
};

impl<L: Library<MQ: Exits>> MqFunctions<L> {
    /// Converts characters from one character set to another
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(source, target, self)))]
    pub fn mqxcnvc(
        &self,
        connection_handle: Option<ConnectionHandle>,
        options: MQDCC,
        source_ccsid: CCSID,
        source: &[sys::MQCHAR],
        target_ccsid: CCSID,
        target: &mut (impl WriteRaw<sys::MQCHAR> + ?Sized),
    ) -> ResultComp<MQLONG> {
        let mut outcome = MqiOutcome::with_verb("MQXCNVC");
        unsafe {
            self.0.lib().MQXCNVC(
                connection_handle.map_or(sys::MQHC_DEF_HCONN, |h| h.raw_handle()),
                options.0,
                source_ccsid.0,
                size_of_val(source)
                    .try_into()
                    .expect("usize length of source should convert into MQLONG"),
                ptr::from_ref(source).cast_mut().cast(),
                target_ccsid.0,
                size_of_val(target)
                    .try_into()
                    .expect("usize length of target should convert into MQLONG"),
                ptr::from_mut(target).cast(),
                &raw mut outcome.value,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }
}
