use crate::{
    values::{MQRC, MQENC, CCSID},
    headers::TextEnc,
    sys, MqStr,
};
use std::{
    fmt::{Debug, Display},
    mem, ptr, str,
};

use super::{headers::fmt::MQFMT_NONE, MqStruct};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Display, derive_more::From)]
pub struct CorrelationId(pub Identifier<24>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Display, derive_more::From)]
pub struct MessageId(pub Identifier<24>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Display, derive_more::From)]
pub struct GroupId(pub Identifier<24>);
#[derive(Debug, Clone, Copy)]
pub struct MsgToken(pub [u8; sys::MQ_MSG_TOKEN_LENGTH]);

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
pub struct UserIdentifier(pub MqStr<12>);
impl_from_str!(UserIdentifier, MqStr<12>);

pub type StrucId = [u8; 4];
pub type Fmt = [u8; 8];

pub type Warning = (MQRC, &'static str);

#[derive(Clone, Copy, Debug)]
pub struct MessageFormat {
    pub ccsid: CCSID,
    pub encoding: MQENC,
    pub fmt: TextEnc<Fmt>,
}

impl MessageFormat {
    #[must_use]
    pub fn from_mqmd2(md: &MqStruct<sys::MQMD2>) -> Self {
        Self {
            ccsid: CCSID(md.CodedCharSetId),
            encoding: MQENC(md.Encoding),
            fmt: TextEnc::Ascii(unsafe { mem::transmute::<[sys::MQCHAR; 8], [u8; 8]>(md.Format) }),
        }
    }
}

pub const FORMAT_NONE: MessageFormat = MessageFormat {
    ccsid: CCSID(1208),
    encoding: MQENC(sys::MQENC_NATIVE),
    fmt: TextEnc::Ascii(MQFMT_NONE),
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::From, derive_more::Deref)]
#[repr(transparent)]
pub struct Identifier<const N: usize>(pub [u8; N]);

impl<const N: usize> Identifier<N> {
    #[must_use]
    pub const fn from_ref(source: &[u8; N]) -> &Self {
        unsafe { &*ptr::from_ref(source).cast() }
    }

    fn hex_fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte in self.0 {
            write!(fmt, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl<const N: usize> Display for Identifier<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ID:")?;
        self.hex_fmt(f)
    }
}

impl<const N: usize> Debug for Identifier<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("Identifier").field(&format_args!("{self}")).finish()
    }
}

impl CorrelationId {
    #[must_use]
    pub const fn from_ref(src: &[u8; sys::MQ_CORREL_ID_LENGTH]) -> &Self {
        unsafe { &*ptr::from_ref(src).cast() }
    }
}

impl UserIdentifier {
    #[must_use]
    pub fn new(source: [sys::MQCHAR; sys::MQ_USER_ID_LENGTH]) -> Option<Self> {
        Some(MqStr::from(source)).filter(MqStr::has_value).map(UserIdentifier)
    }
}

pub type ObjectName = MqStr<48>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct ConnectionName(pub MqStr<264>);
impl_from_str!(ConnectionName, MqStr<264>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct ChannelName(pub MqStr<20>);
impl_from_str!(ChannelName, MqStr<20>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct QueueName(pub ObjectName);
impl_from_str!(QueueName, ObjectName);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct QueueManagerName(pub ObjectName);
impl_from_str!(QueueManagerName, ObjectName);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct CipherSpec(pub MqStr<32>);
impl_from_str!(CipherSpec, MqStr<32>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct KeyRepo(pub MqStr<256>);
impl_from_str!(KeyRepo, MqStr<256>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct CryptoHardware(pub MqStr<256>);
impl_from_str!(CryptoHardware, MqStr<256>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct CertificateLabel(pub MqStr<64>);
impl_from_str!(CertificateLabel, MqStr<64>);

#[cfg(test)]
mod tests {
    use super::Identifier;

    #[test]
    fn correlation_id() {
        let cid = Identifier([0; 24]);
        assert_eq!(format!("{cid}"), "ID:000000000000000000000000000000000000000000000000");
        assert_eq!(
            format!("{cid:?}"),
            "CorrelationId(ID:000000000000000000000000000000000000000000000000)"
        );
    }
}
