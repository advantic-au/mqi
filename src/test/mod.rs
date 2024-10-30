use crate::core::Library;

pub mod mock;

impl Library for mock::MockFunctions {
    type MQ = Self;

    fn lib(&self) -> &Self::MQ {
        self
    }
}
