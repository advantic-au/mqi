#![allow(clippy::allow_attributes)]

use mqi::{
    connect_options::{Credentials, CredentialsSecret, ProtectedSecret},
    core::Library,
};

#[path = "../../src/test/mock.rs"]
pub mod mock;

impl Library for mock::MockFunctions {
    type MQ = Self;

    fn lib(&self) -> &Self::MQ {
        self
    }
}

#[allow(dead_code)]
pub fn credentials_app() -> CredentialsSecret<'static, ProtectedSecret<&'static str>> {
    Credentials::user("app", "app")
}

#[cfg(feature = "dlopen2")]
mod dl {
    use libmqm_sys::dlopen2::MqmContainer;
    use mqi::core::Library;

    // dlopen2 Container doesn't implement Debug so create a wrapper
    pub struct DebugContainer(pub MqmContainer);

    impl mqi::core::Library for DebugContainer {
        type MQ = <MqmContainer as Library>::MQ;

        fn lib(&self) -> &Self::MQ {
            self.0.lib()
        }
    }

    impl std::fmt::Debug for DebugContainer {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("MqmContainer")
        }
    }
}
