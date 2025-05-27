/// Implement a method to set the mimimum version required for a [`MqStruct`] structure
macro_rules! impl_min_version {
    ([$($lt:lifetime),*], $ty:ty) => {
        impl <$($lt, )*> $ty {
            #[inline]
            #[doc = "Sets the `Version` field to the minimum required version"]
            pub fn set_min_version(&mut self, version: $crate::types::MQLONG) {
                self.Version = std::cmp::max(self.Version, version);
            }
        }
    };
}

pub(crate) use impl_min_version;
