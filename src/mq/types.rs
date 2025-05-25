use crate::{structs, headers::TextEnc, core::CCSID, MqChar, MqStr};
use crate::types::{MQBYTE, MQCHAR};
use std::{
    fmt::{Debug, Display},
    str,
};

use libmqm_default as default;
use libmqm_sys::lib as sys;

use crate::types::{MQENC, MQRC};
use crate::constants;

use super::{connect_options::ProtectedSecret, headers::fmt::MQFMT_NONE};

macro_rules! impl_equivalent_type {
    ($new_type:path, [$($other_type:path),*]) => {
        $(
            impl_equivalent_type!($new_type, $other_type);
        )*
    };
    ($new_type:path, $other_type:path) => {
        impl PartialEq<$other_type> for $new_type {
            fn eq(&self, other: &$other_type) -> bool {
                other.0 == self.0
            }
        }

        impl From<$other_type> for $new_type {
            fn from(value: $other_type) -> Self {
                Self(value.0)
            }
        }

        impl AsRef<$new_type> for $other_type {
            fn as_ref(&self) -> &$new_type {
                // SAFETY: repr(transparent) ensures new type has same memory layout
                unsafe { &*std::ptr::from_ref(self).cast() }
            }
        }

        impl AsMut<$new_type> for $other_type {
            fn as_mut(&mut self) -> &mut $new_type {
                // SAFETY: repr(transparent) ensures new type has same memory layout
                unsafe { &mut *std::ptr::from_mut(self).cast() }
            }
        }
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
#[repr(transparent)]
pub struct CorrelationId(pub Identifier<24>);
#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
#[repr(transparent)]
pub struct MessageId(pub Identifier<24>);
#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
#[repr(transparent)]
pub struct GroupId(pub Identifier<24>);
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct MsgToken(pub [u8; sys::MQ_MSG_TOKEN_LENGTH]);

impl_equivalent_type!(CorrelationId, MessageId);
impl_equivalent_type!(MessageId, CorrelationId);

impl Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(AsRef::<DisplayId<24>>::as_ref(&self.0), f)
    }
}

impl Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(AsRef::<DisplayId<24>>::as_ref(&self.0), f)
    }
}

impl Display for GroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(AsRef::<DisplayId<24>>::as_ref(&self.0), f)
    }
}

/// Delegates `FromStr` to wrapped type implementation
macro_rules! impl_from_str {
    ($i:ident, $ty:ty) => {
        impl std::str::FromStr for $i {
            type Err = <$ty as std::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(<$ty as std::str::FromStr>::from_str(s)?))
            }
        }
    };
}

pub(crate) use impl_from_str;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct UserIdentifier(pub MqStr<12>);
impl_from_str!(UserIdentifier, MqStr<12>);

pub type StrucId = MqChar<4>;
pub type Fmt = MqChar<8>;

pub type Warning = (MQRC, &'static str);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MessageFormat {
    pub ccsid: CCSID,
    pub encoding: MQENC,
    pub fmt: TextEnc<Fmt>,
}

impl MessageFormat {
    #[must_use]
    #[allow(clippy::allow_attributes, clippy::missing_const_for_fn)]
    pub fn from_mqmd2(md: &structs::MQMD2) -> Self {
        Self {
            ccsid: CCSID(md.CodedCharSetId),
            encoding: MQENC(md.Encoding),
            fmt: TextEnc::Ascii(md.Format),
        }
    }

    #[must_use]
    pub fn into_mqmd2(&self) -> structs::MQMD2 {
        structs::MQMD2::new(sys::MQMD2 {
            CodedCharSetId: self.ccsid.0,
            Encoding: self.encoding.0,
            Format: *self.fmt.into_ascii().as_ref(),
            ..default::MQMD2_DEFAULT
        })
    }
}

pub const FORMAT_NONE: MessageFormat = MessageFormat {
    ccsid: CCSID(1208),
    encoding: constants::MQENC_NATIVE,
    fmt: TextEnc::Ascii(MQFMT_NONE),
};

pub type Identifier<const N: usize> = [MQBYTE; N];

#[repr(transparent)]
pub(super) struct DisplayId<const N: usize>(Identifier<N>);

impl<const N: usize> AsRef<DisplayId<N>> for Identifier<N> {
    fn as_ref(&self) -> &DisplayId<N> {
        // SAFETY: repr(transparent) ensures new type has same memory layout
        unsafe { &*std::ptr::from_ref(self).cast() }
    }
}

impl<const N: usize> DisplayId<N> {
    fn hex_fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte in self.0 {
            write!(fmt, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl<const N: usize> Display for DisplayId<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ID:")?;
        self.hex_fmt(f)
    }
}

impl<const N: usize> Debug for DisplayId<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("Identifier").field(&format_args!("{self}")).finish()
    }
}

impl Debug for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CorrelationId").field(&format_args!("{self}")).finish()
    }
}

impl Debug for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MessageId").field(&format_args!("{self}")).finish()
    }
}

impl Debug for GroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("GroupId").field(&format_args!("{self}")).finish()
    }
}

impl UserIdentifier {
    #[must_use]
    pub fn new(source: [MQCHAR; sys::MQ_USER_ID_LENGTH]) -> Option<Self> {
        Some(MqStr::from(source)).filter(MqStr::has_value).map(UserIdentifier)
    }
}

pub type ObjectName = MqStr<48>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ConnectionName(pub MqStr<264>);
impl_from_str!(ConnectionName, MqStr<264>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ChannelName(pub MqStr<20>);
impl_from_str!(ChannelName, MqStr<20>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct QueueName(pub ObjectName);
impl_from_str!(QueueName, ObjectName);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct QueueManagerName(pub ObjectName);
impl_from_str!(QueueManagerName, ObjectName);

impl_equivalent_type!(QueueManagerName, ReplyToQueueManagerName);
impl_equivalent_type!(ReplyToQueueManagerName, QueueManagerName);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ReplyToQueueManagerName(pub ObjectName);
impl_from_str!(ReplyToQueueManagerName, ObjectName);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ReplyToQueueName(pub ObjectName);
impl_from_str!(ReplyToQueueName, ObjectName);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct CipherSpec(pub MqStr<32>);
impl_from_str!(CipherSpec, MqStr<32>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct KeyRepo(pub MqStr<256>);
impl_from_str!(KeyRepo, MqStr<256>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ApplName(pub MqStr<28>);
impl_from_str!(ApplName, MqStr<28>);

#[derive(
    Debug, Clone, Copy, Default, PartialOrd, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From,
)]
#[repr(transparent)]
pub struct PutDate(pub MqStr<8>);
impl_from_str!(PutDate, MqStr<8>);

#[derive(
    Debug, Clone, Copy, Default, PartialOrd, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From,
)]
#[repr(transparent)]
pub struct PutTime(pub MqStr<8>);
impl_from_str!(PutTime, MqStr<8>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct AccountingToken(pub [MQBYTE; sys::MQ_ACCOUNTING_TOKEN_LENGTH]);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ApplIdentityData(pub MqStr<32>);
impl_from_str!(ApplIdentityData, MqStr<32>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ApplOriginData(pub MqStr<4>);
impl_from_str!(ApplOriginData, MqStr<4>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut)]
#[repr(transparent)]
pub struct KeyRepoPassword<T: ?Sized>(pub T);

impl<'a> KeyRepoPassword<ProtectedSecret<&'a str>> {
    #[must_use]
    pub const fn new(password: &'a str) -> Self {
        Self(ProtectedSecret::new(password))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct CryptoHardware(pub MqStr<256>);
impl_from_str!(CryptoHardware, MqStr<256>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct CertificateLabel(pub MqStr<64>);
impl_from_str!(CertificateLabel, MqStr<64>);

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::types::CorrelationId;

    #[test]
    fn correlation_id() {
        let cid = CorrelationId([0; 24]);
        assert_eq!(format!("{cid}"), "ID:000000000000000000000000000000000000000000000000");
        assert_eq!(
            format!("{cid:?}"),
            "CorrelationId(ID:000000000000000000000000000000000000000000000000)"
        );
    }
}
