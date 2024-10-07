#![allow(clippy::allow_attributes)]

use mqi::connect_options::{Credentials, CredentialsSecret, ProtectedSecret};

#[cfg(feature = "link")]
pub const fn mq_library() -> libmqm_sys::link::LinkedMq {
    libmqm_sys::link::LinkedMq
}

#[cfg(all(feature = "dlopen2", not(feature = "link")))]
pub fn mq_library() -> std::sync::Arc<dl::DebugContainer> {
    use std::sync::Arc;

    use dlopen2::wrapper::Container;
    use libmqm_sys::dlopen2::LoadMqm as _;

    Arc::new(dl::DebugContainer(unsafe {
        Container::load_mqm_default().expect("Loading of default MQM should work")
    }))
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
