/// Implement a method to set the mimimum version required for a [`MqStruct`] structure
macro_rules! impl_mqstruct_min_version {
    ($ty:ty) => {
        impl $crate::MqStruct<'_, $ty> {
            #[inline]
            #[doc = "Sets the `Version` field to the minimum required version"]
            pub fn set_min_version(&mut self, version: sys::MQLONG) {
                self.Version = std::cmp::max(self.Version, version);
            }
        }
    };
}

pub(crate) use impl_mqstruct_min_version;
