use std::{fmt, str::FromStr, time::SystemTime};

use uuid::Uuid as ExternalUuid;

pub type NodeId = [u8; 6];

/// UUID value owned by the core API.
///
/// The underlying `uuid` constructors stay private to this crate so the
/// generation policy cannot be bypassed through a re-export.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Uuid(ExternalUuid);

impl Uuid {
    pub const NAMESPACE_DNS: Self = Self(ExternalUuid::NAMESPACE_DNS);
    pub const NAMESPACE_URL: Self = Self(ExternalUuid::NAMESPACE_URL);
    pub const NAMESPACE_OID: Self = Self(ExternalUuid::NAMESPACE_OID);
    pub const NAMESPACE_X500: Self = Self(ExternalUuid::NAMESPACE_X500);

    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(ExternalUuid::from_bytes(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }

    pub fn get_version_num(&self) -> usize {
        self.0.get_version_num()
    }

    pub fn is_nil(&self) -> bool {
        self.0.is_nil()
    }

    pub fn is_max(&self) -> bool {
        self.0.is_max()
    }

    pub fn hyphenated(&self) -> String {
        self.0.hyphenated().to_string()
    }

    pub fn simple(&self) -> String {
        self.0.simple().to_string()
    }

    pub fn urn(&self) -> String {
        self.0.urn().to_string()
    }

    pub(crate) fn from_external(uuid: ExternalUuid) -> Self {
        Self(uuid)
    }

    pub(crate) fn external(&self) -> &ExternalUuid {
        &self.0
    }

    pub(crate) fn timestamp(&self) -> Option<uuid::Timestamp> {
        self.0.get_timestamp()
    }

    pub(crate) fn node_id(&self) -> Option<NodeId> {
        self.0.get_node_id()
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.hyphenated())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GeneratableUuidVersion {
    V4,
    V5,
    V6,
    V7,
    V8,
}

impl fmt::Display for GeneratableUuidVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::V4 => "v4",
            Self::V5 => "v5",
            Self::V6 => "v6",
            Self::V7 => "v7",
            Self::V8 => "v8",
        })
    }
}

impl FromStr for GeneratableUuidVersion {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "4" | "v4" => Ok(Self::V4),
            "5" | "v5" => Ok(Self::V5),
            "6" | "v6" => Ok(Self::V6),
            "7" | "v7" => Ok(Self::V7),
            "8" | "v8" => Ok(Self::V8),
            other => Err(format!("unsupported generation target `{other}`")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InspectableUuidVersion {
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    Nil,
    Max,
    Unknown(u8),
}

impl InspectableUuidVersion {
    pub fn from_uuid(uuid: &Uuid) -> Self {
        if uuid.is_nil() {
            return Self::Nil;
        }
        if uuid.is_max() {
            return Self::Max;
        }

        match uuid.get_version_num() {
            1 => Self::V1,
            2 => Self::V2,
            3 => Self::V3,
            4 => Self::V4,
            5 => Self::V5,
            6 => Self::V6,
            7 => Self::V7,
            8 => Self::V8,
            value => Self::Unknown(value as u8),
        }
    }

    pub const fn number(self) -> Option<u8> {
        match self {
            Self::V1 => Some(1),
            Self::V2 => Some(2),
            Self::V3 => Some(3),
            Self::V4 => Some(4),
            Self::V5 => Some(5),
            Self::V6 => Some(6),
            Self::V7 => Some(7),
            Self::V8 => Some(8),
            Self::Nil => Some(0),
            Self::Max => Some(15),
            Self::Unknown(value) => Some(value),
        }
    }
}

impl fmt::Display for InspectableUuidVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V1 => f.write_str("v1"),
            Self::V2 => f.write_str("v2"),
            Self::V3 => f.write_str("v3"),
            Self::V4 => f.write_str("v4"),
            Self::V5 => f.write_str("v5"),
            Self::V6 => f.write_str("v6"),
            Self::V7 => f.write_str("v7"),
            Self::V8 => f.write_str("v8"),
            Self::Nil => f.write_str("nil"),
            Self::Max => f.write_str("max"),
            Self::Unknown(value) => write!(f, "unknown({value})"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GenerationOptions {
    pub version: GeneratableUuidVersion,
    pub namespace: Option<Uuid>,
    pub name: Option<Vec<u8>>,
    pub node: Option<NodeId>,
    pub timestamp: Option<SystemTime>,
    pub custom: Option<[u8; 16]>,
}

impl GenerationOptions {
    pub fn new(version: GeneratableUuidVersion) -> Self {
        Self {
            version,
            namespace: None,
            name: None,
            node: None,
            timestamp: None,
            custom: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UuidVariant {
    Ncs,
    Rfc9562,
    Microsoft,
    Future,
}

impl fmt::Display for UuidVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Ncs => "NCS",
            Self::Rfc9562 => "RFC 9562",
            Self::Microsoft => "Microsoft",
            Self::Future => "future",
        })
    }
}
