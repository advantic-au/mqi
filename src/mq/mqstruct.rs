use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr,
};

use crate::{sys, ApplName, MqStr};

/// MQ structure holding a `T` with an associated lifetime for pointer fields
#[derive(Default, Debug, Clone)]
pub struct MqStruct<'ptr, T> {
    struc: T,
    _marker: PhantomData<&'ptr mut ()>, // Lifetime reference required for pointers in the MQ structure
}

pub trait StructProvider<T> {
    fn struc(&self) -> Option<MqStruct<T>>;
}

impl<'a, T: Clone> StructProvider<T> for MqStruct<'a, T> {
    fn struc(&self) -> Option<MqStruct<T>> {
        Some(self.clone())
    }
}

#[derive(Debug, Clone, Default)]
pub struct NoStruct;

impl<T> StructProvider<T> for NoStruct {
    fn struc(&self) -> Option<MqStruct<T>> {
        None
    }
}

impl<T> MqStruct<'_, T> {
    pub fn new(struc: T) -> Self {
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

impl<'ptr> MqStruct<'ptr, sys::MQCNO> {
    pub(super) fn set_csp(&mut self, csp: Option<&'ptr sys::MQCSP>) {
        self.SecurityParmsPtr = csp.map_or(ptr::null_mut(), |mqcsp| ptr::addr_of!(*mqcsp).cast_mut().cast());
    }

    pub(super) fn set_sco(&mut self, sco: Option<&'ptr sys::MQSCO>) {
        self.SSLConfigPtr = sco.map_or(ptr::null_mut(), |mqsco| ptr::addr_of!(*mqsco).cast_mut().cast());
    }

    pub(super) fn set_app_name(&mut self, app: Option<&ApplName>) {
        app.unwrap_or(&MqStr::empty()).copy_into_mqchar(&mut self.ApplName);
    }
}
