use std::{
    marker::{PhantomData, PhantomPinned}, ops::{Deref, DerefMut}, ptr
};

use crate::{sys, ApplName, MqStr};

/// MQ structure holding a `T` with an associated lifetime for pointer fields
#[derive(Default, Debug, Clone)]
pub struct MqStruct<'ptr, T> {
    struc: T,
    _marker: PhantomData<&'ptr mut ()>, // Lifetime reference required for pointers in the MQ structure
}

pub trait StructBuilder<T>: StructType<T> {
    fn build<'a>(&'a self) -> Self::Struct<'a>;

    fn option_build<'a>(&'a self) -> Option<Self::Struct<'a>> where Self: Clone {
        Some(Self::build(self))
    }
}

pub trait StructOptionBuilder<T>: StructType<T> {
    fn option_build<'a>(&'a self) -> Option<Self::Struct<'a>>;
}

pub trait StructType<T> {
    type Struct<'a>: Deref<Target = T> + DerefMut where Self: 'a;
}

impl<T> StructType<T> for MqStruct<'_, T> {
    type Struct<'a> = Self where Self: 'a;
}

impl<'ptr, T: Clone> StructBuilder<T> for MqStruct<'ptr, T> {
    fn build<'a>(&'a self) -> Self::Struct<'a> {
        self.clone()
    }    
}

impl<T, E> StructType<T> for MqRefer<T, E> {
    type Struct<'a> = Self where Self: 'a;
}


impl<T: Clone, E: Clone> StructBuilder<T> for MqRefer<T, E> {
    fn build<'a>(&'a self) -> Self::Struct<'a> {
        self.clone()
    }
}

pub type MqReferPinned<R, E> = MqRefer<R, std::pin::Pin<Box<(E, PhantomPinned)>>>;
#[derive(Debug, Clone)]
pub struct MqRefer<R, E>(R, E);

impl<R, E> MqRefer<R, E> {
    pub fn new(referer: R, referee: E) -> Self {
        Self(referer, referee)
    }
}

impl<R, E> Deref for MqRefer<R, E> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R, E> DerefMut for MqRefer<R, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct NoStruct;

impl<T> StructType<T> for NoStruct {
    type Struct<'a> = MqStruct<'static, T>;
}

impl<T> StructOptionBuilder<T> for NoStruct {
    fn option_build<'a>(&'a self) -> Option<Self::Struct<'a>> {
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

#[cfg(test)]
mod tests {
    use crate::{NoStruct, StructOptionBuilder};


    #[test]
    fn lifetime() {
        let a: Option<crate::MqStruct<i32>>;
        {
            let b = NoStruct;
            a = b.option_build();
        }

        assert!(a.is_none())
    }
}