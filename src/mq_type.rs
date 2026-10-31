use std::{
    fmt::{Debug, Display},
    str,
};

use libmqm_default as default;
use libmqm_sys as mq;

use crate::{
    MqChar, MqStr, constants,
    header::TextEnc,
    macros::impl_from_str,
    string::{CCSID, MqStrError},
    structs,
    traits::Secret,
    types::{MQBYTE, MQCHAR, MQENC, MQRC},
};

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

/// Holds a secret value and redacts it from debug output.
///
/// This wrapper does not encrypt or otherwise protect the value in memory.
#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct ProtectedSecret<T: ?Sized>(T);

impl<T> ProtectedSecret<T> {
    /// Wrap a value so its debug representation is redacted.
    pub const fn new(secret: T) -> Self {
        Self(secret)
    }
}

impl<'t, T: ?Sized> Secret<'t, T> for ProtectedSecret<&'t T> {
    fn expose_secret(&self) -> &'t T {
        let Self(secret) = self;
        secret
    }
}

impl<T> std::fmt::Debug for ProtectedSecret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("ProtectedSecret")
            .field(&format_args!("{} <REDACTED>", std::any::type_name::<T>()))
            .finish()
    }
}

impl<T> From<T> for ProtectedSecret<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// An MQ correlation identifier used to associate related request and reply messages.
#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
#[repr(transparent)]
pub struct CorrelationId(
    /// The fixed-width identifier bytes stored in the message descriptor.
    pub Identifier<24>,
);
/// The unique identifier assigned to an MQ message.
#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
#[repr(transparent)]
pub struct MessageId(
    /// The fixed-width identifier bytes stored in the message descriptor.
    pub Identifier<24>,
);
/// An identifier shared by messages belonging to the same MQ message group.
#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
#[repr(transparent)]
pub struct GroupId(
    /// The fixed-width identifier bytes stored in the message descriptor.
    pub Identifier<24>,
);
/// An opaque MQ message token used for token-based message operations.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct MsgToken(
    /// The token bytes returned by MQ.
    pub [u8; mq::MQ_MSG_TOKEN_LENGTH],
);

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

/// The fixed-width user identifier stored in an MQ message descriptor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct UserIdentifier(
    /// The 12-character MQ user identifier.
    pub MqStr<12>,
);
impl_from_str!(UserIdentifier, MqStr<12>);

/// Four-character MQ structure identifier.
pub type StrucId = MqChar<4>;
/// Eight-character MQ message format identifier.
pub type Fmt = MqChar<8>;

/// An MQ warning reason code paired with the name of the MQ verb that returned it.
pub type Warning = (MQRC, &'static str);

/// Character set, numeric encoding, and format associated with message data.
///
/// These fields describe how to interpret the message body; they do not
/// transcode the body themselves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MessageFormat {
    /// Character-set identifier for the message data.
    pub ccsid: CCSID,
    /// Numeric encoding for integer and floating-point fields.
    pub encoding: MQENC,
    /// Message format identifier, tagged with its character encoding.
    pub fmt: TextEnc<Fmt>,
}

impl MessageFormat {
    /// Read the format fields from a message descriptor.
    #[must_use]
    #[allow(clippy::allow_attributes, clippy::missing_const_for_fn)]
    pub fn from_mqmd(md: &structs::MQMD) -> Self {
        Self {
            ccsid: CCSID(md.CodedCharSetId),
            encoding: MQENC(md.Encoding),
            fmt: TextEnc::Ascii(md.Format),
        }
    }

    /// Create a message descriptor initialized with this format's fields.
    #[must_use]
    pub fn into_mqmd(&self) -> structs::MQMD {
        structs::MQMD::new(mq::MQMD {
            CodedCharSetId: self.ccsid.0,
            Encoding: self.encoding.0,
            Format: *self.fmt.into_ascii().as_ref(),
            ..default::MQMD_DEFAULT
        })
    }
}

impl From<MqStr<8>> for MessageFormat {
    fn from(value: MqStr<8>) -> Self {
        Self {
            ccsid: CCSID::default(),
            encoding: MQENC::default(),
            fmt: TextEnc::Ascii(value.into()),
        }
    }
}

impl str::FromStr for MessageFormat {
    type Err = MqStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fmt: MqStr<8> = (*s).try_into()?;
        Ok(Self {
            ccsid: CCSID::default(),
            encoding: MQENC::default(),
            fmt: TextEnc::Ascii(fmt.into()),
        })
    }
}

/// Default message format used when no format has been specified.
pub const FORMAT_NONE: MessageFormat = MessageFormat {
    ccsid: CCSID(1208),
    encoding: constants::MQENC_NATIVE,
    fmt: TextEnc::Ascii(crate::header::fmt::MQFMT_NONE),
};

/// A fixed-width array of opaque MQ identifier bytes.
pub type Identifier<const N: usize> = [MQBYTE; N];

/// Displays an MQ identifier as hexadecimal bytes prefixed by `ID:`.
#[repr(transparent)]
pub struct DisplayId<const N: usize>(Identifier<N>);

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
    /// Create a user identifier when the source contains a nonblank value.
    #[must_use]
    pub fn new(source: [MQCHAR; mq::MQ_USER_ID_LENGTH]) -> Option<Self> {
        Some(MqStr::from(source)).filter(MqStr::has_value).map(UserIdentifier)
    }
}

/// A fixed-width 48-character MQ object name.
pub type ObjectName = MqStr<48>;

/// The host and port portion of an MQ client connection definition.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ConnectionName(
    /// The fixed-width connection name.
    pub MqStr<264>,
);
impl_from_str!(ConnectionName, MqStr<264>);

/// The name of an MQ channel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ChannelName(
    /// The fixed-width channel name.
    pub MqStr<20>,
);
impl_from_str!(ChannelName, MqStr<20>);

/// The name of an MQ queue.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct QueueName(
    /// The fixed-width queue name.
    pub ObjectName,
);
impl_from_str!(QueueName, ObjectName);

/// The name of an MQ queue manager.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct QueueManagerName(
    /// The fixed-width queue-manager name.
    pub ObjectName,
);
impl_from_str!(QueueManagerName, ObjectName);

impl_equivalent_type!(QueueManagerName, ReplyToQueueManagerName);
impl_equivalent_type!(ReplyToQueueManagerName, QueueManagerName);

/// The queue-manager name that identifies a reply destination.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ReplyToQueueManagerName(
    /// The fixed-width reply-to queue-manager name.
    pub ObjectName,
);
impl_from_str!(ReplyToQueueManagerName, ObjectName);

/// The queue name to which an application should send a reply.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ReplyToQueueName(
    /// The fixed-width reply-to queue name.
    pub ObjectName,
);
impl_from_str!(ReplyToQueueName, ObjectName);

/// The TLS cipher specification negotiated for an MQ channel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct CipherSpec(
    /// The fixed-width cipher specification name.
    pub MqStr<32>,
);
impl_from_str!(CipherSpec, MqStr<32>);

/// The path or name of the key repository used by an MQ TLS connection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct KeyRepo(
    /// The fixed-width key repository name.
    pub MqStr<256>,
);
impl_from_str!(KeyRepo, MqStr<256>);

/// The application name reported to MQ when connecting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ApplName(
    /// The fixed-width application name.
    pub MqStr<28>,
);
impl_from_str!(ApplName, MqStr<28>);

/// The date field in an MQ message descriptor.
#[derive(
    Debug, Clone, Copy, Default, PartialOrd, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From,
)]
#[repr(transparent)]
pub struct PutDate(
    /// The MQ date in `YYYYMMDD` form.
    pub MqStr<8>,
);
impl_from_str!(PutDate, MqStr<8>);

/// The time field in an MQ message descriptor.
#[derive(
    Debug, Clone, Copy, Default, PartialOrd, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From,
)]
#[repr(transparent)]
pub struct PutTime(
    /// The MQ time in `HHMMSShh` form.
    pub MqStr<8>,
);
impl_from_str!(PutTime, MqStr<8>);

/// An opaque accounting token carried in an MQ message descriptor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
pub struct AccountingToken(
    /// The fixed-width accounting-token bytes.
    pub [MQBYTE; mq::MQ_ACCOUNTING_TOKEN_LENGTH],
);

/// Application identity data carried in an MQ message descriptor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ApplIdentityData(
    /// The fixed-width application identity text.
    pub MqStr<32>,
);
impl_from_str!(ApplIdentityData, MqStr<32>);

/// Application origin data carried in an MQ message descriptor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct ApplOriginData(
    /// The fixed-width application origin text.
    pub MqStr<4>,
);
impl_from_str!(ApplOriginData, MqStr<4>);

/// A key-repository password value used when configuring an MQ TLS connection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut)]
#[repr(transparent)]
pub struct KeyRepoPassword<T: ?Sized>(
    /// The password value supplied to MQ.
    pub T,
);

impl<'a> KeyRepoPassword<ProtectedSecret<&'a str>> {
    /// Wrap a borrowed key-repository password with redacted debug output.
    #[must_use]
    pub const fn new(password: &'a str) -> Self {
        Self(ProtectedSecret::new(password))
    }
}

/// The name of the cryptographic hardware configuration used for TLS.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct CryptoHardware(
    /// The fixed-width cryptographic hardware name.
    pub MqStr<256>,
);
impl_from_str!(CryptoHardware, MqStr<256>);

/// The certificate label MQ should select from a key repository.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Deref, derive_more::DerefMut, derive_more::From)]
#[repr(transparent)]
pub struct CertificateLabel(
    /// The fixed-width certificate label.
    pub MqStr<64>,
);
impl_from_str!(CertificateLabel, MqStr<64>);

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::{
        header::{TextEnc, fmt},
        string::CCSID,
        types::CorrelationId,
    };

    use super::MessageFormat;

    #[test]
    fn correlation_id() {
        let cid = CorrelationId([0; 24]);
        assert_eq!(format!("{cid}"), "ID:000000000000000000000000000000000000000000000000");
        assert_eq!(
            format!("{cid:?}"),
            "CorrelationId(ID:000000000000000000000000000000000000000000000000)"
        );
    }

    #[test]
    fn message_format_from_mqstr() {
        let format = MessageFormat::from(crate::mqstr!("MQSTR"));
        assert_eq!(format.ccsid, CCSID::default());
        assert_eq!(format.encoding, crate::types::MQENC::default());
        assert_eq!(format.fmt, TextEnc::Ascii(fmt::MQFMT_STRING));
    }

    #[test]
    fn message_format_from_str() {
        let parsed: MessageFormat = "MQADMIN".parse().expect("valid format name");
        assert_eq!(parsed, MessageFormat::from(crate::mqstr!("MQADMIN")));

        let too_long = "TOO_LONG_9".parse::<MessageFormat>();
        assert!(too_long.is_err());
    }

    #[test]
    fn message_format_into_mqmd_roundtrip() {
        let format = MessageFormat::from(crate::mqstr!("MQSTR"));
        let mqmd = format.into_mqmd();
        assert_eq!(mqmd.CodedCharSetId, format.ccsid.0);
        assert_eq!(mqmd.Encoding, format.encoding.0);
        assert_eq!(MessageFormat::from_mqmd(&mqmd), format);
    }
}
