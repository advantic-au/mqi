use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use libmqm_sys as mq;

/// Message Descriptor - contains the attributes that describe the message
pub type MQMD = MqStruct<'static, mq::MQMD>;
/// Message Descriptor version 1 - original version of the message descriptor
pub type MQMD1 = MqStruct<'static, mq::MQMD1>;

/// Get Message Options - used to control the behavior of MQGET operations
pub type MQGMO = MqStruct<'static, mq::MQGMO>;
/// Put Message Options - used to control the behavior of MQPUT operations
pub type MQPMO<'a> = MqStruct<'a, mq::MQPMO>;

/// Connection Options - used to control how applications connect to a queue manager
pub type MQCNO<'a> = MqStruct<'a, mq::MQCNO>;
/// SSL/TLS Configuration Options - used to configure secure connections
pub type MQSCO<'a> = MqStruct<'a, mq::MQSCO>;
/// Connection Security Parameters - used for user ID and password authentication
pub type MQCSP<'a> = MqStruct<'a, mq::MQCSP>;
/// Channel Definition - defines the attributes of a channel
pub type MQCD<'a> = MqStruct<'a, mq::MQCD>;
#[cfg(feature = "mqc_9_3_0_0")]
/// Balancing Options - used for connection balancing in a multi-instance queue manager
pub type MQBNO = MqStruct<'static, mq::MQBNO>;

/// Subscription Descriptor - used to define a subscription to a topic
pub type MQSD<'a> = MqStruct<'a, mq::MQSD>;
/// Object Descriptor - used to identify the object to be opened
pub type MQOD<'a> = MqStruct<'a, mq::MQOD>;
/// Authentication Information Record - used for passing authentication information
pub type MQAIR<'a> = MqStruct<'a, mq::MQAIR>;

/// Variable-length character string - used for variable-length string parameters
pub type MQCHARV<'a> = MqStruct<'a, mq::MQCHARV>;

/// Inquire Message Property Options - used to control inquiries of message properties
pub type MQIMPO<'a> = MqStruct<'a, mq::MQIMPO>;
/// Delete Message Property Options - used to control deletion of message properties
pub type MQDMPO = MqStruct<'static, mq::MQDMPO>;
/// Set Message Property Options - used to control setting of message properties
pub type MQSMPO = MqStruct<'static, mq::MQSMPO>;

/// Message Handle to Buffer Options - used when converting message properties to a buffer
pub type MQMHBO = MqStruct<'static, mq::MQMHBO>;
/// Buffer to Message Handle Options - used when converting a buffer to message properties
pub type MQBMHO = MqStruct<'static, mq::MQBMHO>;

/// Property Descriptor - used to define the attributes of a property
pub type MQPD = MqStruct<'static, mq::MQPD>;

/// Subscription Request Options - used to control subscription requests
pub type MQSRO = MqStruct<'static, mq::MQSRO>;

/// Status Information - provides status information about an asynchronous put operation
pub type MQSTS<'a> = MqStruct<'a, mq::MQSTS>;

/// Callback Context - provides the context for a callback function
pub type MQCBC<'a> = MqStruct<'a, mq::MQCBC>;
/// Callback Descriptor - used to define a callback function
pub type MQCBD<'a> = MqStruct<'a, mq::MQCBD>;

/// Begin Options - used to control the behavior of the MQBEGIN call
pub type MQBO = MqStruct<'static, mq::MQBO>;
/// Control Options - used to control the behavior of the MQCTL call
pub type MQCTLO<'a> = MqStruct<'a, mq::MQCTLO>;

/// MQ structure holding a `T` with an associated lifetime for pointer fields
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct MqStruct<'ptr, T> {
    pub(super) struc: T,
    _marker: PhantomData<&'ptr mut ()>, // Lifetime reference required for pointers in the MQ structure
}

impl<T> MqStruct<'_, T> {
    pub const fn new(struc: T) -> Self {
        Self {
            struc,
            _marker: PhantomData,
        }
    }
}

impl<T> DerefMut for MqStruct<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.struc
    }
}

impl<T> Deref for MqStruct<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.struc
    }
}

/// Implement a method to set the mimimum version required for a [`MqStruct`] structure
macro_rules! impl_min_version {
    ([$($lt:lifetime),*], $ty:ty) => {
        impl <$($lt, )*> $ty {
            #[inline]
            #[doc = "Sets the `Version` field to a minimum"]
            pub fn set_min_version(&mut self, version: $crate::types::MQLONG) {
                self.Version = std::cmp::max(self.Version, version);
            }
        }
    };
}

impl_min_version!([], MQGMO);

pub(crate) use impl_min_version;
