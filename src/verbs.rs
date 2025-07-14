use std::ptr;

use libmqm_sys::{self as mq, Mqi};
#[cfg(feature = "tracing")]
use {
    crate::support::outcome::{tracing_outcome, tracing_outcome_basic},
    tracing::instrument,
};

use super::{
    Library, MqFunctions, ReadRaw, WriteRaw,
    handle::{ConnectionHandle, MessageHandle, ObjectHandle, SubscriptionHandle},
};
use crate::{
    CCSID, result::Error, MQMD, MqStr, result::ResultComp, result::ResultCompErr, result::ResultErr, constants,
    support::outcome::{MqiOutcome, MqiOutcomeVoid},
    types::{MQCO, MQDCC, MQLONG, MQOO, MQOP, MQSR, MQSTAT, MQTYPE, MQXA},
};

#[derive(Debug, derive_more::From, derive_more::Error, derive_more::Display)]
pub enum MqInqError {
    #[display("{}, length: {}", _1, _0)]
    Length(MQLONG, Error),
    #[from]
    #[display("{_0}")]
    MQ(Error),
}

impl From<MqInqError> for Error {
    fn from(value: MqInqError) -> Self {
        let (MqInqError::Length(_, error) | MqInqError::MQ(error)) = value;
        error
    }
}

impl<L: Library<MQ: Mqi>> MqFunctions<L> {
    /// Connects an application program to a queue manager.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqconn(&self, qm_name: &MqStr<48>) -> ResultComp<ConnectionHandle> {
        let mut outcome = MqiOutcome::<ConnectionHandle>::with_verb("MQCONN");

        unsafe {
            self.0.lib().MQCONN(
                qm_name.as_ref(),
                outcome.value.mut_raw_handle(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Connects an application program to a queue manager. It provides control on the method of connection.
    ///
    /// # Safety
    /// Consumers of [`mqconnx`](Self::mqconnx) must ensure the [`MQCNO`](mq::MQCNO) structure is populated with valid pointers and offsets
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub unsafe fn mqconnx(&self, qm_name: &MqStr<48>, mqcno: &mut mq::MQCNO) -> ResultComp<ConnectionHandle> {
        let mut outcome = MqiOutcome::<ConnectionHandle>::with_verb("MQCONNX");
        unsafe {
            self.0.lib().MQCONNX(
                qm_name.as_ref(),
                mqcno,
                outcome.value.mut_raw_handle(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Breaks the connection between the queue manager and the application program,
    /// and is the inverse of the mqconn or mqconnx call
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqdisc(&self, connection: &mut ConnectionHandle) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQDISC");
        unsafe {
            self.0
                .lib()
                .MQDISC(connection.mut_raw_handle(), &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Establishes access to an object
    ///
    /// # Safety
    /// Consumers of [`mqopen`](Self::mqopen) must ensure the [`MQOD`](mq::MQOD) structure is populated with valid pointers and offsets
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub unsafe fn mqopen(
        &self,
        connection_handle: ConnectionHandle,
        mqod: &mut mq::MQOD,
        options: MQOO,
    ) -> ResultComp<ObjectHandle> {
        let mut outcome = MqiOutcome::<ObjectHandle>::with_verb("MQOPEN");

        unsafe {
            self.0.lib().MQOPEN(
                connection_handle.raw_handle(),
                mqod,
                options.0,
                outcome.value.mut_raw_handle(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Puts one message on a queue, or distribution list, or to a topic
    ///
    /// # Safety
    /// Consumers of [`mqput1`](Self::mqput1) must ensure the [`MQPMO`](mq::MQPMO) and [`MQOD`](mq::MQOD) structures are populated with valid pointers and offsets
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(body, self)))]
    #[allow(clippy::allow_attributes, clippy::similar_names)]
    pub unsafe fn mqput1(
        &self,
        connection_handle: ConnectionHandle,
        mqod: &mut mq::MQOD,
        mqmd: Option<&mut impl MQMD>,
        pmo: &mut mq::MQPMO,
        body: &[mq::MQBYTE],
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQPUT1");
        unsafe {
            self.0.lib().MQPUT1(
                connection_handle.raw_handle(),
                mqod,
                mqmd.map_or_else(ptr::null_mut, |md| ptr::from_mut(md).cast()),
                pmo,
                size_of_val(body)
                    .try_into()
                    .expect("body length should not exceed maximum positive MQLONG"),
                ptr::from_ref(body).cast_mut().cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Relinquishes access to an object, and is the inverse of the mqopen and mqsub calls
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqclose(
        &self,
        connection_handle: ConnectionHandle,
        object_handle: &mut ObjectHandle,
        options: MQCO,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQCLOSE");
        unsafe {
            self.0.lib().MQCLOSE(
                connection_handle.raw_handle(),
                object_handle.mut_raw_handle(),
                options.0,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Indicates to the queue manager that the application has reached a sync point, and that all the
    /// message gets and puts that have occurred since the last sync point are to be made permanent.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqcmit(&self, connection_handle: ConnectionHandle) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQCMIT");
        unsafe {
            self.0
                .lib()
                .MQCMIT(connection_handle.raw_handle(), &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Puts a message on a queue or distribution list, or to a topic. The queue, distribution list,
    /// or topic must already be open.
    ///
    /// # Safety
    /// Consumers of [`mqput`](Self::mqput) must ensure the [`MQPMO`](mq::MQPMO) structure is populated with valid pointers and offsets
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(body, self)))]
    pub unsafe fn mqput(
        &self,
        connection_handle: ConnectionHandle,
        object_handle: &ObjectHandle,
        mqmd: Option<&mut impl MQMD>,
        pmo: &mut mq::MQPMO,
        body: &[mq::MQBYTE],
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQPUT");

        unsafe {
            self.0.lib().MQPUT(
                connection_handle.raw_handle(),
                object_handle.raw_handle(),
                mqmd.map_or_else(ptr::null_mut, |md| ptr::from_mut(md).cast()),
                pmo,
                size_of_val(body)
                    .try_into()
                    .expect("body length should not exceed maximum positive MQLONG"),
                ptr::from_ref(body).cast_mut().cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Retrieves a message from a local queue that has been opened using the mqopen call
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(body, self)))]
    pub fn mqget(
        &self,
        connection_handle: ConnectionHandle,
        object_handle: &ObjectHandle,
        mqmd: Option<&mut impl MQMD>,
        gmo: &mut mq::MQGMO,
        body: &mut (impl WriteRaw<mq::MQBYTE> + ?Sized),
    ) -> ResultComp<MQLONG> {
        let mut outcome = MqiOutcome::with_verb("MQGET");
        unsafe {
            self.0.lib().MQGET(
                connection_handle.raw_handle(),
                object_handle.raw_handle(),
                mqmd.map_or_else(ptr::null_mut, |md| ptr::from_mut(md).cast()),
                gmo,
                size_of_val(body)
                    .try_into()
                    .expect("body length should not exceed maximum positive MQLONG"),
                ptr::from_mut(body).cast(),
                &mut outcome.value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome_basic(&outcome);
        outcome.into()
    }

    /// Returns an array of integers and a set of character strings containing
    /// the attributes of an object
    #[cfg_attr(feature = "tracing", instrument(level = "trace", , skip(self, int_attr, text_attr)))]
    pub fn mqinq(
        &self,
        connection_handle: ConnectionHandle,
        object_handle: &ObjectHandle,
        selectors: &[MQXA],
        int_attr: &mut [impl WriteRaw<MQLONG>],
        text_attr: &mut [impl WriteRaw<mq::MQCHAR>],
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQINQ");
        unsafe {
            self.0.lib().MQINQ(
                connection_handle.raw_handle(),
                object_handle.raw_handle(),
                selectors
                    .len()
                    .try_into()
                    .expect("selectors count should not exceed maximum positive MQLONG"),
                selectors.as_ptr().cast_mut().cast(),
                int_attr
                    .len()
                    .try_into()
                    .expect("int_attr count should not exceed maximum positive MQLONG"),
                int_attr.as_mut_ptr().cast(),
                size_of_val(text_attr)
                    .try_into()
                    .expect("text_attr count should not exceed maximum positive MQLONG"),
                ptr::from_mut(text_attr).cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Register the applications subscription to a particular topic
    /// # Safety
    /// Consumers of [`mqsub`](Self::mqsub) must ensure the [`MQSD`](mq::MQSD) structure is populated with valid pointers and offsets
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub unsafe fn mqsub(
        &self,
        connection_handle: ConnectionHandle,
        mqsd: &mut mq::MQSD,
        object_handle: &mut ObjectHandle,
    ) -> ResultComp<SubscriptionHandle> {
        let mut outcome = MqiOutcome::<SubscriptionHandle>::with_verb("MQSUB");
        unsafe {
            self.0.lib().MQSUB(
                connection_handle.raw_handle(),
                mqsd,
                Some(object_handle.mut_raw_handle()),
                outcome.value.mut_raw_handle(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Make a request for the retained publication, when the subscriber has been
    /// registered with `MQSO_PUBLICATIONS_ON_REQUEST`
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqsubrq(
        &self,
        connection_handle: ConnectionHandle,
        subscription_handle: &SubscriptionHandle,
        action: MQSR,
        mqsro: Option<&mut mq::MQSRO>,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQSUBRQ");
        unsafe {
            self.0.lib().MQSUBRQ(
                connection_handle.raw_handle(),
                subscription_handle.raw_handle(),
                action.0,
                mqsro,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Begins a unit of work that is coordinated by the queue manager, and that can
    /// involve external resource managers.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqbegin(&self, connection_handle: ConnectionHandle, mqbo: Option<&mut mq::MQBO>) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQBEGIN");
        unsafe {
            self.0
                .lib()
                .MQBEGIN(connection_handle.raw_handle(), mqbo, &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Indicates to the queue manager that all the message gets and puts that have
    /// occurred since the last sync point are to be backed out
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqback(&self, connection_handle: ConnectionHandle) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQBACK");
        unsafe {
            self.0
                .lib()
                .MQBACK(connection_handle.raw_handle(), &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Creates a message handle for use with mqsetmp, mqinqmp, and mqdltmp.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqcrtmh(&self, connection_handle: Option<ConnectionHandle>, cmho: &mq::MQCMHO) -> ResultErr<MessageHandle> {
        let mut outcome = MqiOutcome::<MessageHandle>::with_verb("MQCRTMH");
        unsafe {
            self.0.lib().MQCRTMH(
                connection_handle.map_or(mq::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                cmho,
                outcome.value.mut_raw_handle(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Deletes a message handle and is the inverse of the mqcrtmh call.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqdltmh(
        &self,
        connection_handle: Option<ConnectionHandle>,
        message_handle: &mut MessageHandle,
        dmho: &mq::MQDMHO,
    ) -> ResultErr<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQDLTMH");
        unsafe {
            self.0.lib().MQDLTMH(
                connection_handle.map_or(mq::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                message_handle.mut_raw_handle(),
                dmho,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Returns the value of a property of a message.
    ///
    /// # Safety
    /// Consumers of [`mqinqmp`](Self::mqinqmp) must ensure the name MQCHARV and MQIMPO structures are populated with valid pointers and offsets
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self, value)))]
    #[expect(clippy::too_many_arguments)]
    pub unsafe fn mqinqmp(
        &self,
        connection_handle: Option<ConnectionHandle>,
        message_handle: &MessageHandle,
        inq_prop_opts: &mut mq::MQIMPO,
        name: &mq::MQCHARV,
        prop_desc: &mut mq::MQPD,
        prop_type: &mut MQTYPE,
        value: Option<&mut (impl WriteRaw<mq::MQBYTE> + ?Sized)>,
    ) -> ResultCompErr<mq::MQLONG, MqInqError> {
        let mut outcome = MqiOutcome::with_verb("MQINQMP");
        let (out_len, out) = value.map_or((0, ptr::null_mut()), |out| {
            (
                size_of_val(out)
                    .try_into()
                    .expect("target value length should not exceed maximum positive MQLONG"),
                ptr::from_mut(out).cast(),
            )
        });
        unsafe {
            self.0.lib().MQINQMP(
                connection_handle.map_or(mq::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                message_handle.raw_handle(),
                inq_prop_opts,
                name,
                prop_desc,
                prop_type.as_mut(),
                out_len,
                out,
                &mut outcome.value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        match outcome.rc {
            constants::MQRC_PROPERTY_VALUE_TOO_BIG => {
                Err(MqInqError::Length(outcome.value, Error(outcome.cc, outcome.verb, outcome.rc)))
            }
            constants::MQRC_PROPERTY_NAME_TOO_BIG => Err(MqInqError::Length(
                inq_prop_opts.ReturnedName.VSLength,
                Error(outcome.cc, outcome.verb, outcome.rc),
            )),
            _ => outcome.into(),
        }
    }

    /// Deletes a property from a message handle and is the inverse of the mqsetmp call.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqdltmp(
        &self,
        connection_handle: Option<ConnectionHandle>,
        message_handle: &MessageHandle,
        delete_prop_opts: &mq::MQDMPO,
        name: &mq::MQCHARV,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQDLTMP");
        unsafe {
            self.0.lib().MQDLTMP(
                connection_handle.map_or(mq::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                message_handle.raw_handle(),
                delete_prop_opts,
                name,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Retrieve status information. The type of status information returned is
    /// determined by the `stat_type` value parameter
    ///
    /// # Safety
    /// Consumers of [`mqstat`](Self::mqstat) must ensure the [`MQSTS` MQCHARV](libmqm_sys::MQSTS) pointers are populated correctly.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub unsafe fn mqstat(&self, connection_handle: ConnectionHandle, stat_type: MQSTAT, sts: &mut mq::MQSTS) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQSTAT");
        unsafe {
            self.0.lib().MQSTAT(
                connection_handle.raw_handle(),
                stat_type.0,
                sts,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Set or modify a property of a message handle
    ///
    /// # Safety
    /// Consumers of [`mqsetmp`](Self::mqsetmp) must ensure the name [`MQCHARV`](mq::MQCHARV) pointer/offset is populated correctly.
    #[expect(clippy::too_many_arguments)]
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self, value)))]
    pub unsafe fn mqsetmp(
        &self,
        connection_handle: Option<ConnectionHandle>,
        message_handle: &MessageHandle,
        set_prop_opts: &mq::MQSMPO,
        name: &mq::MQCHARV,
        prop_desc: &mut mq::MQPD,
        prop_type: MQTYPE,
        value: &(impl ReadRaw + ?Sized),
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQSETMP");
        unsafe {
            self.0.lib().MQSETMP(
                connection_handle.map_or(mq::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                message_handle.raw_handle(),
                set_prop_opts,
                name,
                prop_desc,
                prop_type.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length should not exceed maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Changes the attributes of an object. The object must be a queue.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqset(
        &self,
        connection_handle: ConnectionHandle,
        object_handle: &ObjectHandle,
        selectors: &[MQXA],
        int_attr: &[mq::MQLONG],
        text_attr: &[mq::MQCHAR],
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQSET");
        unsafe {
            self.0.lib().MQSET(
                connection_handle.raw_handle(),
                object_handle.raw_handle(),
                selectors
                    .len()
                    .try_into()
                    .expect("selectors count should not exceed maximum positive MQLONG"),
                selectors.as_ptr().cast_mut().cast(),
                int_attr
                    .len()
                    .try_into()
                    .expect("int_attr count should not exceed maximum positive MQLONG"),
                int_attr.as_ptr().cast_mut(),
                text_attr
                    .len()
                    .try_into()
                    .expect("text_attr count should not exceed maximum positive MQLONG"),
                text_attr.as_ptr().cast_mut(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Registers a callback for the specified object handle and controls activation and changes to the callback
    ///
    /// # Safety
    /// Consumers of [`mqcb`](Self::mqcb) must populate the [`MQCBD`](libmqm_sys::MQCBD) structure with valid pointers
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub unsafe fn mqcb(
        &self,
        connection_handle: ConnectionHandle,
        operations: MQOP,
        callback_desc: Option<&mq::MQCBD>,
        object_handle: Option<&ObjectHandle>,
        mqmd: Option<&impl MQMD>,
        gmo: Option<&mq::MQGMO>,
    ) -> ResultErr<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQCB");
        unsafe {
            self.0.lib().MQCB(
                connection_handle.raw_handle(),
                operations.0,
                callback_desc,
                object_handle.map_or(mq::MQHO_NONE, |h| h.raw_handle()),
                mqmd.map_or_else(ptr::null_mut, |md| ptr::from_ref(md).cast_mut().cast()),
                gmo,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Performs controlling actions on callbacks and the object handles opened for a connection
    ///
    /// # Safety
    /// Consumers of [`mqctl`](Self::mqctl) must populate the [`MQCTLO`](libmqm_sys::MQCTLO) structure with valid pointers
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub unsafe fn mqctl(
        &self,
        connection_handle: ConnectionHandle,
        operation: MQOP,
        control_options: &mq::MQCTLO,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQCTL");
        unsafe {
            self.0.lib().MQCTL(
                connection_handle.raw_handle(),
                operation.0,
                control_options,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Converts a message handle into a buffer and is the inverse of the mqbufmh call
    ///
    /// # Safety
    /// Consumers of [`mqmhbuf`](Self::mqmhbuf) must populate the name MQCHARV structure with a valid pointer/offset
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(buffer, self)))]
    pub unsafe fn mqmhbuf(
        &self,
        connection_handle: Option<ConnectionHandle>,
        message_handle: &MessageHandle,
        mhbuf_options: &mq::MQMHBO,
        name: &mq::MQCHARV,
        mqmd: &mut impl MQMD,
        buffer: &mut (impl WriteRaw<mq::MQBYTE> + ?Sized),
    ) -> ResultCompErr<mq::MQLONG, MqInqError> {
        let mut outcome = MqiOutcome::with_verb("MQMHBUF");
        unsafe {
            self.0.lib().MQMHBUF(
                connection_handle.map_or(mq::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                message_handle.raw_handle(),
                mhbuf_options,
                name,
                ptr::from_mut(mqmd).cast(),
                size_of_val(buffer)
                    .try_into()
                    .expect("buffer length should not exceed maximum positive MQLONG"),
                ptr::from_mut(buffer).cast(),
                &mut outcome.value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        match outcome.rc {
            constants::MQRC_PROPERTY_VALUE_TOO_BIG => {
                Err(MqInqError::Length(outcome.value, Error(outcome.cc, outcome.verb, outcome.rc)))
            }
            _ => outcome.into(),
        }
    }

    /// Converts a buffer into a message handle and is the inverse of the mqmhbuf call
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(buffer, self)))]
    pub fn mqbufmh(
        &self,
        connection_handle: Option<ConnectionHandle>,
        message_handle: &MessageHandle,
        bufmh_options: &mq::MQBMHO,
        mqmd: &mut impl MQMD,
        buffer: &[mq::MQBYTE],
    ) -> ResultComp<MQLONG> {
        let mut outcome = MqiOutcome::with_verb("MQBUFMH");
        unsafe {
            self.0.lib().MQBUFMH(
                connection_handle.map_or(mq::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                message_handle.raw_handle(),
                bufmh_options,
                ptr::from_mut(mqmd).cast(),
                size_of_val(buffer)
                    .try_into()
                    .expect("buffer length should not exceed maximum positive MQLONG"),
                ptr::from_ref(buffer).cast_mut().cast(),
                &mut outcome.value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Converts characters from one character set to another
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(source, target, self)))]
    pub fn mqxcnvc(
        &self,
        connection_handle: Option<ConnectionHandle>,
        options: MQDCC,
        source_ccsid: CCSID,
        source: &[mq::MQCHAR],
        target_ccsid: CCSID,
        target: &mut (impl WriteRaw<mq::MQCHAR> + ?Sized),
    ) -> ResultComp<MQLONG> {
        let mut outcome = MqiOutcome::with_verb("MQXCNVC");
        unsafe {
            self.0.lib().MQXCNVC(
                connection_handle.map_or(mq::MQHC_DEF_HCONN, |h| h.raw_handle()),
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
                &mut outcome.value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }
}
