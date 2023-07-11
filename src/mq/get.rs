use crate::constants::mapping;
use crate::impl_constant_lookup;
use crate::sys;
use num_enum::TryFromPrimitive;

use crate::constants;

#[repr(i32)]
#[derive(Default)]
pub enum GetWait {
    #[default]
    NoWait = sys::MQGMO_NO_WAIT,
    Wait(sys::MQLONG) = sys::MQGMO_WAIT,
}

#[repr(i32)]
#[derive(Default)]
pub enum Quiesce {
    #[default]
    NoFail = 0,
    Fail = sys::MQGMO_FAIL_IF_QUIESCING,
}

#[repr(i32)]
#[derive(Default)]
pub enum SyncPoint {
    #[default]
    Default = 0,
    Enabled = sys::MQGMO_SYNCPOINT,
    Disabled = sys::MQGMO_NO_SYNCPOINT,
    IfPersistent = sys::MQGMO_SYNCPOINT_IF_PERSISTENT,
}

#[repr(i32)]
#[derive(Default)]
pub enum GetBrowse {
    #[default]
    Get = 0,
    BrowseFirst = sys::MQGMO_BROWSE_FIRST,
    BrowseNext = sys::MQGMO_BROWSE_NEXT,
    BrowseMsgUnderCursor = sys::MQGMO_BROWSE_MSG_UNDER_CURSOR,
    GetMsgUnderCursor = sys::MQGMO_MSG_UNDER_CURSOR, // TODO: mark / unmark
}

#[repr(i32)]
#[derive(Clone, Copy, TryFromPrimitive)]
pub enum Locking {
    Default = 0,
    Lock = sys::MQGMO_LOCK,
    Unlock = sys::MQGMO_UNLOCK,
}

impl constants::MQConstant for Locking {
    fn mq_value(&self) -> sys::MQLONG {
        *self as sys::MQLONG
    }
}

impl_constant_lookup!(Locking, mapping::MQGMO_CONST);

struct Options {
    gmo: sys::MQGMO,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            gmo: sys::MQGMO {
                Version: sys::MQGMO_VERSION_4,
                ..sys::MQGMO::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        debug_assert_eq!(Options::default().gmo.Version, sys::MQGMO_VERSION_4);
    }
}
