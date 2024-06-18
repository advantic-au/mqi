mod struct_attach;

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

pub trait StructBuilder<T>: StructType<T> {
    fn build(&self) -> Self::Struct<'_>;
}

pub trait StructOptionBuilder<T>: StructType<T> {
    fn option_build(&self) -> Option<Self::Struct<'_>>;
}

pub trait StructType<T> {
    type Struct<'a>: Deref<Target = T> + DerefMut
    where
        Self: 'a;
}

impl<T> StructType<T> for MqStruct<'_, T> {
    type Struct<'a> = Self where Self: 'a;
}

impl<'ptr, T: Clone> StructBuilder<T> for MqStruct<'ptr, T> {
    fn build(&self) -> Self::Struct<'_> {
        self.clone()
    }
}

impl<'ptr, T: Clone> StructOptionBuilder<T> for MqStruct<'ptr, T> {
    fn option_build(&self) -> Option<Self::Struct<'_>> {
        Some(self.build())
    }
}

impl<T, E> StructType<T> for MqStructSelfRef<T, E> {
    type Struct<'a> = MqStruct<'a, T> where Self: 'a;
}

impl<T: Clone, E> StructBuilder<T> for MqStructSelfRef<T, E> {
    fn build(&self) -> Self::Struct<'_> {
        MqStruct::new(self.0.clone())
    }
}

impl<T: Clone, E> StructOptionBuilder<T> for MqStructSelfRef<T, E> {
    fn option_build(&self) -> Option<Self::Struct<'_>> {
        Some(self.build())
    }
}

#[derive(Debug, Clone)]
pub struct MqStructSelfRef<R, E>(R, E);

impl<R, E> MqStructSelfRef<R, E> {
    pub const fn new(referer: R, referee: E) -> Self {
        Self(referer, referee)
    }
}

impl<R, E: Deref> MqStructSelfRef<R, E> {
    pub fn referred(&self) -> &E::Target {
        &self.1
    }
}

impl<R, E> Deref for MqStructSelfRef<R, E> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R, E> DerefMut for MqStructSelfRef<R, E> {
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
    fn option_build(&self) -> Option<Self::Struct<'_>> {
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

        assert!(a.is_none());
    }
}
