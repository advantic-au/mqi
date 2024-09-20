use std::fmt::Debug;
use std::mem::{size_of_val, MaybeUninit};
use std::ptr;

use super::values::{MQCO, MQDCC, MQOO, MQOP, MQSR, MQSTAT, MQTYPE, MQXA};
use super::{ConnectionHandle, Library, MqFunctions, MqiOutcome, MqiOutcomeVoid, MessageHandle, ObjectHandle, SubscriptionHandle};
use crate::{sys, Error, MqStr, ResultComp, ResultCompErr, ResultErr, MQMD};
use libmqm_sys::{function, Mqi};

#[cfg(feature = "tracing")]
use {
    super::{tracing_outcome, tracing_outcome_basic},
    tracing::instrument,
};

pub mod error {
    use crate::{sys, Error};

    #[derive(Debug, derive_more::From, derive_more::Error, derive_more::Display)]
    pub enum MqInqError {
        #[display("{}, length: {}", _1, _0)]
        Length(sys::MQLONG, Error),
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
}

impl<L: Library<MQ: function::Mqi>> MqFunctions<L> {
    /// Connects an application program to a queue manager.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqconn(&self, qm_name: &MqStr<48>) -> ResultComp<ConnectionHandle> {
        let mut outcome = MqiOutcome::<ConnectionHandle>::with_verb("MQCONN");

        unsafe {
            self.0.lib().MQCONN(
                qm_name.as_ref().as_ptr().cast_mut(),
                outcome.mut_raw_handle(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Connects an application program to a queue manager. It provides control on the method of connection.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqconnx(&self, qm_name: &MqStr<48>, mqcno: &mut sys::MQCNO) -> ResultComp<ConnectionHandle> {
        let mut outcome = MqiOutcome::<ConnectionHandle>::with_verb("MQCONNX");
        unsafe {
            self.0.lib().MQCONNX(
                qm_name.as_ref().as_ptr().cast_mut(),
                ptr::from_mut(mqcno).cast(),
                outcome.mut_raw_handle(),
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
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqopen(&self, connection_handle: ConnectionHandle, mqod: &mut sys::MQOD, options: MQOO) -> ResultComp<ObjectHandle> {
        let mut outcome = MqiOutcome::<ObjectHandle>::with_verb("MQOPEN");

        unsafe {
            self.0.lib().MQOPEN(
                connection_handle.raw_handle(),
                ptr::from_mut(mqod).cast(),
                options.0,
                outcome.mut_raw_handle(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Puts one message on a queue, or distribution list, or to a topic
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(body, self)))]
    pub fn mqput1<T: ?Sized>(
        &self,
        connection_handle: ConnectionHandle,
        mqod: &mut sys::MQOD,
        mqmd: Option<&mut impl MQMD>,
        pmo: &mut sys::MQPMO,
        body: &T,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQPUT1");
        unsafe {
            self.0.lib().MQPUT1(
                connection_handle.raw_handle(),
                ptr::from_mut(mqod).cast(),
                mqmd.map_or_else(ptr::null_mut, |md| ptr::from_mut(md).cast()),
                ptr::from_mut(pmo).cast(),
                size_of_val(body)
                    .try_into()
                    .expect("body length exceeds maximum positive MQLONG"),
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
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(body, self)))]
    pub fn mqput<T: ?Sized>(
        &self,
        connection_handle: ConnectionHandle,
        object_handle: &ObjectHandle,
        mqmd: Option<&mut impl MQMD>,
        pmo: &mut sys::MQPMO,
        body: &T,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQPUT");

        unsafe {
            self.0.lib().MQPUT(
                connection_handle.raw_handle(),
                object_handle.raw_handle(),
                mqmd.map_or_else(ptr::null_mut, |md| ptr::from_mut(md).cast()),
                ptr::from_mut(pmo).cast(),
                size_of_val(body)
                    .try_into()
                    .expect("body length exceeds maximum positive MQLONG"),
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
    pub fn mqget<T: ?Sized>(
        &self,
        connection_handle: ConnectionHandle,
        object_handle: &ObjectHandle,
        mqmd: Option<&mut impl MQMD>,
        gmo: &mut sys::MQGMO,
        body: &mut T,
    ) -> ResultComp<sys::MQLONG> {
        let mut outcome = MqiOutcome::with_verb("MQGET");
        unsafe {
            self.0.lib().MQGET(
                connection_handle.raw_handle(),
                object_handle.raw_handle(),
                mqmd.map_or_else(ptr::null_mut, |md| ptr::from_mut(md).cast()),
                ptr::from_mut(gmo).cast(),
                size_of_val(body)
                    .try_into()
                    .expect("body length exceeds maximum positive MQLONG"),
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
    #[cfg_attr(feature = "tracing", instrument(level = "trace", , skip(self)))]
    pub fn mqinq(
        &self,
        connection_handle: ConnectionHandle,
        object_handle: &ObjectHandle,
        selectors: &[MQXA],
        int_attr: &mut [MaybeUninit<sys::MQLONG>],
        text_attr: &mut [MaybeUninit<sys::MQCHAR>],
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQINQ");
        unsafe {
            self.0.lib().MQINQ(
                connection_handle.raw_handle(),
                object_handle.raw_handle(),
                selectors
                    .len()
                    .try_into()
                    .expect("selectors count exceeds maximum positive MQLONG"),
                selectors.as_ptr().cast_mut().cast(),
                int_attr
                    .len()
                    .try_into()
                    .expect("int_attr count exceeds maximum positive MQLONG"),
                int_attr.as_mut_ptr().cast(),
                text_attr
                    .len()
                    .try_into()
                    .expect("text_attr count exceeds maximum positive MQLONG"),
                text_attr.as_mut_ptr().cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Register the applications subscription to a particular topic
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqsub(
        &self,
        connection_handle: ConnectionHandle,
        mqsd: &mut sys::MQSD,
        object_handle: &mut ObjectHandle,
    ) -> ResultComp<SubscriptionHandle> {
        let mut outcome = MqiOutcome::<SubscriptionHandle>::with_verb("MQSUB");
        unsafe {
            self.0.lib().MQSUB(
                connection_handle.raw_handle(),
                ptr::from_mut(mqsd).cast(),
                object_handle.mut_raw_handle(),
                outcome.mut_raw_handle(),
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
        mqsro: &mut sys::MQSRO,
    ) -> ResultErr<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQSUBRQ");
        unsafe {
            self.0.lib().MQSUBRQ(
                connection_handle.raw_handle(),
                subscription_handle.raw_handle(),
                action.0,
                ptr::from_mut(mqsro).cast(),
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
    pub fn mqbegin(&self, connection_handle: ConnectionHandle, mqbo: &mut sys::MQBO) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQBEGIN");
        unsafe {
            self.0.lib().MQBEGIN(
                connection_handle.raw_handle(),
                ptr::from_mut(mqbo).cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
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
    pub fn mqcrtmh(&self, connection_handle: Option<ConnectionHandle>, cmho: &sys::MQCMHO) -> ResultErr<MessageHandle> {
        let mut outcome = MqiOutcome::<MessageHandle>::with_verb("MQCRTMH");
        unsafe {
            self.0.lib().MQCRTMH(
                connection_handle.map_or(sys::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                ptr::from_ref(cmho).cast_mut().cast(),
                outcome.mut_raw_handle(),
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
        dmho: &sys::MQDMHO,
    ) -> ResultErr<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQDLTMH");
        unsafe {
            self.0.lib().MQDLTMH(
                connection_handle.map_or(sys::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                message_handle.mut_raw_handle(),
                ptr::from_ref(dmho).cast_mut().cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Returns the value of a property of a message.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self, value)))]
    #[expect(clippy::too_many_arguments)]
    pub fn mqinqmp<T: ?Sized>(
        &self,
        connection_handle: Option<ConnectionHandle>,
        message_handle: &MessageHandle,
        inq_prop_opts: &mut sys::MQIMPO,
        name: &sys::MQCHARV,
        prop_desc: &mut sys::MQPD,
        prop_type: &mut MQTYPE,
        value: Option<&mut T>,
    ) -> ResultCompErr<sys::MQLONG, error::MqInqError> {
        let mut outcome = MqiOutcome::with_verb("MQINQMP");
        let (out_len, out) = value.map_or((0, ptr::null_mut()), |out| {
            (
                size_of_val(out)
                    .try_into()
                    .expect("target value length exceeds maximum positive MQLONG"),
                ptr::from_mut(out).cast(),
            )
        });
        unsafe {
            self.0.lib().MQINQMP(
                connection_handle.map_or(sys::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                message_handle.raw_handle(),
                ptr::from_mut(inq_prop_opts).cast(),
                ptr::from_ref(name).cast_mut().cast(),
                ptr::from_mut(prop_desc).cast(),
                ptr::from_mut(prop_type).cast(),
                out_len,
                out,
                &mut outcome.value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        match outcome.rc.value() {
            sys::MQRC_PROPERTY_VALUE_TOO_BIG => Err(error::MqInqError::Length(
                outcome.value,
                Error(outcome.cc, outcome.verb, outcome.rc),
            )),
            sys::MQRC_PROPERTY_NAME_TOO_BIG => Err(error::MqInqError::Length(
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
        delete_prop_opts: &sys::MQDMPO,
        name: &sys::MQCHARV,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQDLTMP");
        unsafe {
            self.0.lib().MQDLTMP(
                connection_handle.map_or(sys::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                message_handle.raw_handle(),
                ptr::from_ref(delete_prop_opts).cast_mut().cast(),
                ptr::from_ref(name).cast_mut().cast(),
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
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqstat(&self, connection_handle: ConnectionHandle, stat_type: MQSTAT, sts: &mut sys::MQSTS) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQSTAT");
        unsafe {
            self.0.lib().MQSTAT(
                connection_handle.raw_handle(),
                stat_type.0,
                ptr::from_mut(sts).cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Set or modify a property of a message handle
    #[expect(clippy::too_many_arguments)]
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqsetmp<T: Debug + ?Sized>(
        &self,
        connection_handle: Option<ConnectionHandle>,
        message_handle: &MessageHandle,
        set_prop_opts: &sys::MQSMPO,
        name: &sys::MQCHARV,
        prop_desc: &mut sys::MQPD,
        prop_type: MQTYPE,
        value: &T,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQSETMP");
        unsafe {
            self.0.lib().MQSETMP(
                connection_handle.map_or(sys::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                message_handle.raw_handle(),
                ptr::from_ref(set_prop_opts).cast_mut().cast(),
                ptr::from_ref(name).cast_mut().cast(),
                ptr::from_mut(prop_desc).cast(),
                prop_type.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
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
        int_attr: &[sys::MQLONG],
        text_attr: &[sys::MQCHAR],
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQSET");
        unsafe {
            self.0.lib().MQSET(
                connection_handle.raw_handle(),
                object_handle.raw_handle(),
                selectors
                    .len()
                    .try_into()
                    .expect("selectors count exceeds maximum positive MQLONG"),
                selectors.as_ptr().cast_mut().cast(),
                int_attr
                    .len()
                    .try_into()
                    .expect("int_attr count exceeds maximum positive MQLONG"),
                int_attr.as_ptr().cast_mut(),
                text_attr
                    .len()
                    .try_into()
                    .expect("text_attr count exceeds maximum positive MQLONG"),
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
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqcb(
        &self,
        connection_handle: ConnectionHandle,
        operations: MQOP,
        callback_desc: &sys::MQCBD,
        object_handle: Option<&ObjectHandle>,
        mqmd: Option<&impl MQMD>,
        gmo: Option<&sys::MQGMO>,
    ) -> ResultErr<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQCB");
        unsafe {
            self.0.lib().MQCB(
                connection_handle.raw_handle(),
                operations.0,
                ptr::from_ref(callback_desc).cast_mut().cast(),
                object_handle.map_or(sys::MQHO_NONE, |h| h.raw_handle()),
                mqmd.map_or_else(ptr::null_mut, |md| ptr::from_ref(md).cast_mut().cast()),
                gmo.map_or_else(ptr::null_mut, |mo| ptr::from_ref(mo).cast_mut().cast()),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Performs controlling actions on callbacks and the object handles opened for a connection
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqctl(&self, connection_handle: ConnectionHandle, operation: MQOP, control_options: &sys::MQCTLO) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("MQCTL");
        unsafe {
            self.0.lib().MQCTL(
                connection_handle.raw_handle(),
                operation.0,
                ptr::from_ref(control_options).cast_mut().cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Converts a message handle into a buffer and is the inverse of the mqbufmh call
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(buffer, self)))]
    pub fn mqmhbuf<T: ?Sized>(
        &self,
        connection_handle: Option<ConnectionHandle>,
        message_handle: &MessageHandle,
        mhbuf_options: &sys::MQMHBO,
        name: &sys::MQCHARV,
        mqmd: &mut impl MQMD,
        buffer: &mut T,
    ) -> ResultComp<sys::MQLONG> {
        let mut outcome = MqiOutcome::with_verb("MQMHBUF");
        unsafe {
            self.0.lib().MQMHBUF(
                connection_handle.map_or(sys::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                message_handle.raw_handle(),
                ptr::from_ref(mhbuf_options).cast_mut().cast(),
                ptr::from_ref(name).cast_mut().cast(),
                ptr::from_mut(mqmd).cast(),
                size_of_val(buffer)
                    .try_into()
                    .expect("buffer length exceeds maximum positive MQLONG"),
                ptr::from_mut(buffer).cast(),
                &mut outcome.value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Converts a buffer into a message handle and is the inverse of the mqmhbuf call
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(buffer, self)))]
    pub fn mqbufm<T: ?Sized>(
        &self,
        connection_handle: Option<ConnectionHandle>,
        message_handle: &MessageHandle,
        bufmh_options: &sys::MQBMHO,
        mqmd: &mut impl MQMD,
        buffer: &mut T,
    ) -> ResultErr<sys::MQLONG> {
        let mut outcome = MqiOutcome::with_verb("MQBUFMH");
        unsafe {
            self.0.lib().MQBUFMH(
                connection_handle.map_or(sys::MQHC_UNASSOCIATED_HCONN, |h| h.raw_handle()),
                message_handle.raw_handle(),
                ptr::from_ref(bufmh_options).cast_mut().cast(),
                ptr::from_mut(mqmd).cast(),
                size_of_val(buffer)
                    .try_into()
                    .expect("buffer length exceeds maximum positive MQLONG"),
                ptr::from_mut(buffer).cast(),
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
    pub fn mqxcnvc<T: ?Sized>(
        &self,
        connection_handle: Option<ConnectionHandle>,
        options: MQDCC,
        source_ccsid: sys::MQLONG,
        source: &T,
        target_ccsid: sys::MQLONG,
        target: &mut T,
    ) -> ResultComp<sys::MQLONG> {
        let mut outcome = MqiOutcome::with_verb("MQXCNVC");
        unsafe {
            self.0.lib().MQXCNVC(
                connection_handle.map_or(sys::MQHC_DEF_HCONN, |h| h.raw_handle()),
                options.value(),
                source_ccsid,
                size_of_val(source)
                    .try_into()
                    .expect("usize length of source converts into MQLONG"),
                ptr::from_ref(source).cast_mut().cast(),
                target_ccsid,
                size_of_val(target)
                    .try_into()
                    .expect("usize length of target converts into MQLONG"),
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
