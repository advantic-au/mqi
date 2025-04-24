#![allow(dead_code)]

#[cfg(feature = "link")]
#[must_use]
pub const fn mq_library() -> libmqm_sys::link::LinkedMq {
    libmqm_sys::link::LinkedMq
}

#[cfg(all(feature = "dlopen2", not(feature = "link")))]
#[must_use]
pub fn mq_library() -> std::sync::Arc<dl::DebugContainer> {
    use std::sync::Arc;

    use dlopen2::wrapper::Container;
    use libmqm_sys::dlopen2::LoadMqm as _;

    Arc::new(dl::DebugContainer(unsafe {
        Container::load_mqm_default().expect("Loading of default MQM should work")
    }))
}

#[cfg(feature = "dlopen2")]
mod dl {
    use libmqm_sys::dlopen2::MqmContainer;
    use crate::core::Library;

    // dlopen2 Container doesn't implement Debug so create a wrapper
    pub struct DebugContainer(pub MqmContainer);

    impl Library for DebugContainer {
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
