use std::fmt::Debug;
use std::mem::{size_of_val, MaybeUninit};
use std::ptr;

use super::{
    CallbackOperation, CloseOptions, ConnectionHandle, Library, MQFunctions, MQIOutcome, MQIOutcomeVoid, MessageHandle,
    ObjectHandle, OpenOptions, SubscriptionHandle, MQHO_NONE, UNNASSOCIATED_HCONN,
};
use crate::{impl_constant_lookup, mapping, sys, Attribute, Mask, MqValue, QMName, RawValue, ResultComp, ResultErr, MQMD};
use libmqm_sys::MQI;

#[cfg(feature = "tracing")]
use {
    super::{tracing_outcome, tracing_outcome_basic},
    tracing::instrument,
};

#[derive(Debug, Clone, Copy)]
pub struct SubReqAction;
impl_constant_lookup!(SubReqAction, mapping::MQSR_CONST);
impl RawValue for SubReqAction {
    type ValueType = sys::MQLONG;
}

#[derive(Clone, Copy)]
pub struct MqType;
impl_constant_lookup!(MqType, mapping::MQTYPE_CONST);
impl RawValue for MqType {
    type ValueType = sys::MQLONG;
}

// TODO: overaching Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash
#[derive(Clone, Copy)]
pub struct MqStat;
impl_constant_lookup!(MqStat, mapping::MQSTAT_CONST);
impl RawValue for MqStat {
    type ValueType = sys::MQLONG;
}

impl<L: Library> MQFunctions<L> {
    /// Connects an application program to a queue manager.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqconn(&self, qm_name: &QMName) -> ResultComp<ConnectionHandle> {
        let mut outcome = MQIOutcome::<ConnectionHandle>::with_verb("MQCONN");

        unsafe {
            self.0.MQCONN(
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
    pub fn mqconnx(&self, qm_name: &QMName, mqcno: &mut sys::MQCNO) -> ResultComp<ConnectionHandle> {
        let mut outcome = MQIOutcome::<ConnectionHandle>::with_verb("MQCONNX");
        unsafe {
            self.0.MQCONNX(
                qm_name.as_ref().as_ptr().cast_mut(),
                ptr::addr_of_mut!(*mqcno).cast(),
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
        let mut outcome = MQIOutcomeVoid::with_verb("MQDISC");
        unsafe {
            self.0
                .MQDISC(connection.mut_raw_handle(), &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Establishes access to an object
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqopen(
        &self,
        connection_handle: &ConnectionHandle,
        mqod: &mut sys::MQOD,
        options: Mask<OpenOptions>,
    ) -> ResultComp<ObjectHandle> {
        let mut outcome = MQIOutcome::<ObjectHandle>::with_verb("MQOPEN");

        unsafe {
            self.0.MQOPEN(
                connection_handle.raw_handle(),
                ptr::addr_of_mut!(*mqod).cast(),
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
    pub fn mqput1<T>(
        &self,
        connection_handle: &ConnectionHandle,
        mqod: &mut sys::MQOD,
        mqmd: Option<&mut impl MQMD>,
        pmo: &mut sys::MQPMO,
        body: &T,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQPUT1");
        unsafe {
            self.0.MQPUT1(
                connection_handle.raw_handle(),
                ptr::addr_of!(*mqod).cast_mut().cast(),
                mqmd.map_or_else(ptr::null_mut, |md| ptr::addr_of_mut!(*md).cast()),
                ptr::addr_of_mut!(*pmo).cast(),
                size_of_val(body)
                    .try_into()
                    .expect("body length exceeds maximum positive MQLONG"),
                ptr::addr_of!(*body).cast_mut().cast(),
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
        connection_handle: &ConnectionHandle,
        object_handle: &mut ObjectHandle,
        options: Mask<CloseOptions>,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQCLOSE");
        unsafe {
            self.0.MQCLOSE(
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
    pub fn mqcmit(&self, connection_handle: &ConnectionHandle) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQCMIT");
        unsafe {
            self.0
                .MQCMIT(connection_handle.raw_handle(), &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Puts a message on a queue or distribution list, or to a topic. The queue, distribution list,
    /// or topic must already be open.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(body, self)))]
    pub fn mqput<T>(
        &self,
        connection_handle: &ConnectionHandle,
        object_handle: &ObjectHandle,
        mqmd: Option<&mut impl MQMD>,
        pmo: &mut sys::MQPMO,
        body: &T,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQPUT");
        
        unsafe {
            self.0.MQPUT(
                connection_handle.raw_handle(),
                object_handle.raw_handle(),
                mqmd.map_or_else(ptr::null_mut, |md| ptr::addr_of_mut!(*md).cast()),
                ptr::addr_of_mut!(*pmo).cast(),
                size_of_val(body)
                    .try_into()
                    .expect("body length exceeds maximum positive MQLONG"),
                ptr::addr_of!(*body).cast_mut().cast(),
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
    pub fn mqget<T>(
        &self,
        connection_handle: &ConnectionHandle,
        object_handle: &ObjectHandle,
        mqmd: Option<&mut impl MQMD>,
        gmo: &mut sys::MQGMO,
        body: &mut T,
    ) -> ResultComp<sys::MQLONG> {
        let mut outcome = MQIOutcome::with_verb("MQGET");
        unsafe {
            self.0.MQGET(
                connection_handle.raw_handle(),
                object_handle.raw_handle(),
                mqmd.map_or_else(ptr::null_mut, |md| ptr::addr_of_mut!(*md).cast()),
                ptr::addr_of_mut!(*gmo).cast(),
                size_of_val(body)
                    .try_into()
                    .expect("body length exceeds maximum positive MQLONG"),
                ptr::addr_of_mut!(*body).cast(),
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
        connection_handle: &ConnectionHandle,
        object_handle: &ObjectHandle,
        selectors: &[MqValue<Attribute>],
        int_attr: &mut [MaybeUninit<sys::MQLONG>],
        text_attr: &mut [MaybeUninit<sys::MQCHAR>],
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQINQ");
        unsafe {
            self.0.MQINQ(
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
        connection_handle: &ConnectionHandle,
        mqsd: &mut sys::MQSD,
        object_handle: &mut ObjectHandle,
    ) -> ResultErr<SubscriptionHandle> {
        let mut outcome = MQIOutcome::<SubscriptionHandle>::with_verb("MQSUB");
        unsafe {
            self.0.MQSUB(
                connection_handle.raw_handle(),
                ptr::addr_of_mut!(*mqsd).cast(),
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
        connection_handle: &ConnectionHandle,
        subscription_handle: &SubscriptionHandle,
        action: MqValue<SubReqAction>,
        mqsro: &mut sys::MQSRO,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQSUBRQ");
        unsafe {
            self.0.MQSUBRQ(
                connection_handle.raw_handle(),
                subscription_handle.raw_handle(),
                action.0,
                ptr::addr_of_mut!(*mqsro).cast(),
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
    pub fn mqbegin(&self, connection_handle: &ConnectionHandle, mqbo: &mut sys::MQBO) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQBEGIN");
        unsafe {
            self.0.MQBEGIN(
                connection_handle.raw_handle(),
                ptr::addr_of_mut!(*mqbo).cast(),
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
    pub fn mqback(&self, connection_handle: &ConnectionHandle) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQBACK");
        unsafe {
            self.0
                .MQBACK(connection_handle.raw_handle(), &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Creates a message handle for use with mqsetmp, mqinqmp, and mqdltmp.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqcrtmh(
        &self,
        connection_handle: Option<&ConnectionHandle>,
        cmho: &sys::MQCMHO,
    ) -> ResultErr<MessageHandle> {
        let mut outcome = MQIOutcome::<MessageHandle>::with_verb("MQCRTMH");
        unsafe {
            self.0.MQCRTMH(
                connection_handle.unwrap_or(&UNNASSOCIATED_HCONN).raw_handle(),
                ptr::addr_of!(*cmho).cast_mut().cast(),
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
        connection_handle: Option<&ConnectionHandle>,
        message_handle: &mut MessageHandle,
        dmho: &sys::MQDMHO,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQDLTMH");
        unsafe {
            self.0.MQDLTMH(
                connection_handle.unwrap_or(&UNNASSOCIATED_HCONN).raw_handle(),
                message_handle.mut_raw_handle(),
                ptr::addr_of!(*dmho).cast_mut().cast(),
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
    #[allow(clippy::too_many_arguments)]
    pub fn mqinqmp<T: ?Sized>(
        &self,
        connection_handle: Option<&ConnectionHandle>,
        message_handle: &MessageHandle,
        inq_prop_opts: &mut sys::MQIMPO,
        name: &sys::MQCHARV,
        prop_desc: &mut sys::MQPD,
        prop_type: &mut MqValue<MqType>,
        value: &mut T,
    ) -> ResultComp<sys::MQLONG> {
        let mut outcome = MQIOutcome::with_verb("MQINQMP");
        unsafe {
            self.0.MQINQMP(
                connection_handle.unwrap_or(&UNNASSOCIATED_HCONN).raw_handle(),
                message_handle.raw_handle(),
                ptr::addr_of_mut!(*inq_prop_opts).cast(),
                ptr::addr_of!(*name).cast_mut().cast(),
                ptr::addr_of_mut!(*prop_desc).cast(),
                ptr::addr_of_mut!(*prop_type).cast(),
                size_of_val(value)
                    .try_into()
                    .expect("target value length exceeds maximum positive MQLONG"),
                ptr::addr_of_mut!(*value).cast(),
                &mut outcome.value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Deletes a property from a message handle and is the inverse of the mqsetmp call.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqdltmp(
        &self,
        connection_handle: &ConnectionHandle,
        message_handle: &MessageHandle,
        delete_prop_opts: &sys::MQDMPO,
        name: &sys::MQCHARV,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQDLTMP");
        unsafe {
            self.0.MQDLTMP(
                connection_handle.raw_handle(),
                message_handle.raw_handle(),
                ptr::addr_of!(*delete_prop_opts).cast_mut().cast(),
                ptr::addr_of!(*name).cast_mut().cast(),
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
    pub fn mqstat(&self, connection_handle: &ConnectionHandle, stat_type: MqValue<MqStat>) -> ResultErr<sys::MQSTS> {
        let mut outcome = MQIOutcome::<sys::MQSTS>::with_verb("MQSTAT");
        outcome.Version = 2;
        unsafe {
            self.0.MQSTAT(
                connection_handle.raw_handle(),
                stat_type.0,
                ptr::addr_of_mut!(outcome.value).cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Set or modify a property of a message handle
    #[allow(clippy::too_many_arguments)]
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqsetmp<T: Debug>(
        &self,
        connection_handle: &ConnectionHandle,
        message_handle: &MessageHandle,
        set_prop_opts: &sys::MQSMPO,
        name: &sys::MQCHARV,
        prop_desc: &mut sys::MQPD,
        prop_type: MqValue<MqType>,
        value: &T,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQSETMP");
        unsafe {
            self.0.MQSETMP(
                connection_handle.raw_handle(),
                message_handle.raw_handle(),
                ptr::addr_of!(*set_prop_opts).cast_mut().cast(),
                ptr::addr_of!(*name).cast_mut().cast(),
                ptr::addr_of_mut!(*prop_desc).cast(),
                prop_type.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::addr_of!(*value).cast_mut().cast(),
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
        connection_handle: &ConnectionHandle,
        object_handle: &ObjectHandle,
        selectors: &[MqValue<Attribute>],
        int_attr: &[sys::MQLONG],
        text_attr: &[sys::MQCHAR],
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQSET");
        unsafe {
            self.0.MQSET(
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
        connection_handle: &ConnectionHandle,
        operations: Mask<CallbackOperation>,
        callback_desc: &sys::MQCBD,
        object_handle: Option<&ObjectHandle>,
        mqmd: &impl MQMD,
        gmo: &sys::MQGMO,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQCB");
        unsafe {
            self.0.MQCB(
                connection_handle.raw_handle(),
                operations.0,
                ptr::addr_of!(*callback_desc).cast_mut().cast(),
                object_handle.unwrap_or(&MQHO_NONE).raw_handle(),
                ptr::addr_of!(*mqmd).cast_mut().cast(),
                ptr::addr_of!(*gmo).cast_mut().cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracking")]
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Performs controlling actions on callbacks and the object handles opened for a connection
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mqctl(
        &self,
        connection_handle: &ConnectionHandle,
        operation: MqValue<CallbackOperation>,
        control_options: &sys::MQCTLO,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("MQCTL");
        unsafe {
            self.0.MQCTL(
                connection_handle.raw_handle(),
                operation.0,
                ptr::addr_of!(*control_options).cast_mut().cast(),
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
        connection_handle: Option<&ConnectionHandle>,
        message_handle: &MessageHandle,
        mhbuf_options: &sys::MQMHBO,
        name: &sys::MQCHARV,
        mqmd: &mut impl MQMD,
        buffer: &mut T,
    ) -> ResultComp<sys::MQLONG> {
        let mut outcome = MQIOutcome::with_verb("MQMHBUF");
        unsafe {
            self.0.MQMHBUF(
                connection_handle.unwrap_or(&UNNASSOCIATED_HCONN).raw_handle(),
                message_handle.raw_handle(),
                ptr::addr_of!(*mhbuf_options).cast_mut().cast(),
                ptr::addr_of!(*name).cast_mut().cast(),
                ptr::addr_of_mut!(*mqmd).cast(),
                size_of_val(buffer)
                    .try_into()
                    .expect("buffer length exceeds maximum positive MQLONG"),
                ptr::addr_of_mut!(*buffer).cast(),
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
        connection_handle: Option<&ConnectionHandle>,
        message_handle: &MessageHandle,
        bufmh_options: &sys::MQBMHO,
        mqmd: &mut impl MQMD,
        buffer: &mut T,
    ) -> ResultErr<sys::MQLONG> {
        let mut outcome = MQIOutcome::with_verb("MQBUFMH");
        unsafe {
            self.0.MQBUFMH(
                connection_handle.unwrap_or(&UNNASSOCIATED_HCONN).raw_handle(),
                message_handle.raw_handle(),
                ptr::addr_of!(*bufmh_options).cast_mut().cast(),
                ptr::addr_of_mut!(*mqmd).cast(),
                size_of_val(buffer)
                    .try_into()
                    .expect("buffer length exceeds maximum positive MQLONG"),
                ptr::addr_of_mut!(*buffer).cast(),
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
