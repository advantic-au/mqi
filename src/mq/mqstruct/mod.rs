mod struct_attach;
mod version;

pub(super) use version::*;

use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// MQ structure holding a `T` with an associated lifetime for pointer fields
#[derive(Default, Debug, Clone)]
#[repr(transparent)]
pub struct MqStruct<'ptr, T> {
    struc: T,
    _marker: PhantomData<&'ptr mut ()>, // Lifetime reference required for pointers in the MQ structure
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
