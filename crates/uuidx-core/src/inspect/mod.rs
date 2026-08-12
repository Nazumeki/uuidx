use crate::{InspectableUuidVersion, Uuid, UuidVariant};

pub mod identifier;
pub mod nanoid;
pub mod snowflake;

#[cfg(feature = "ulid-inspect")]
pub mod ulid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitField {
    pub name: String,
    pub offset: u8,
    pub width: u8,
    pub value: u128,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeKind {
    Unicast,
    Multicast,
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Unicast => "unicast / may be hardware-derived",
            Self::Multicast => "multicast / locally generated",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameHashAlgorithm {
    Md5,
    Sha1,
}

impl std::fmt::Display for NameHashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Md5 => "MD5",
            Self::Sha1 => "SHA-1",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimestampInfo {
    pub unix_seconds: u64,
    pub unix_millis: u64,
    pub subsec_nanos: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UuidMetadata {
    None,
    Random,
    NameBased {
        algorithm: NameHashAlgorithm,
    },
    Time {
        timestamp: TimestampInfo,
        clock_sequence: Option<u16>,
        node_id: Option<[u8; 6]>,
        node_kind: Option<NodeKind>,
    },
    Custom {
        bytes: [u8; 16],
    },
    DceSecurity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UuidInspection {
    pub normalized: String,
    pub bytes: [u8; 16],
    pub version: InspectableUuidVersion,
    pub variant: UuidVariant,
    pub is_nil: bool,
    pub is_max: bool,
    pub fields: Vec<BitField>,
    pub metadata: UuidMetadata,
}

pub fn inspect_uuid(uuid: &Uuid) -> UuidInspection {
    let version = InspectableUuidVersion::from_uuid(uuid);
    let bytes = *uuid.as_bytes();
    let variant = variant_from_byte(bytes[8]);
    let metadata = metadata_for(uuid, version);
    let fields = layout_for(&bytes, version);

    UuidInspection {
        normalized: uuid.hyphenated(),
        bytes,
        version,
        variant,
        is_nil: uuid.is_nil(),
        is_max: uuid.is_max(),
        fields,
        metadata,
    }
}

fn variant_from_byte(byte: u8) -> UuidVariant {
    if byte & 0x80 == 0 {
        UuidVariant::Ncs
    } else if byte & 0xc0 == 0x80 {
        UuidVariant::Rfc9562
    } else if byte & 0xe0 == 0xc0 {
        UuidVariant::Microsoft
    } else {
        UuidVariant::Future
    }
}

fn metadata_for(uuid: &Uuid, version: InspectableUuidVersion) -> UuidMetadata {
    match version {
        InspectableUuidVersion::V1 | InspectableUuidVersion::V6 => {
            let timestamp = uuid.timestamp().map(|timestamp| {
                let (seconds, nanos) = timestamp.to_unix();
                TimestampInfo {
                    unix_seconds: seconds,
                    unix_millis: seconds.saturating_mul(1_000) + u64::from(nanos / 1_000_000),
                    subsec_nanos: nanos,
                }
            });
            let node_id = uuid.node_id();
            let node_kind = node_id.map(|node| {
                if node[0] & 0x01 == 0 {
                    NodeKind::Unicast
                } else {
                    NodeKind::Multicast
                }
            });
            UuidMetadata::Time {
                timestamp: timestamp.unwrap_or(TimestampInfo {
                    unix_seconds: 0,
                    unix_millis: 0,
                    subsec_nanos: 0,
                }),
                clock_sequence: Some(clock_sequence(uuid)),
                node_id,
                node_kind,
            }
        }
        InspectableUuidVersion::V7 => {
            let timestamp = uuid.timestamp().map(|timestamp| {
                let (seconds, nanos) = timestamp.to_unix();
                TimestampInfo {
                    unix_seconds: seconds,
                    unix_millis: seconds.saturating_mul(1_000) + u64::from(nanos / 1_000_000),
                    subsec_nanos: nanos,
                }
            });
            UuidMetadata::Time {
                timestamp: timestamp.unwrap_or(TimestampInfo {
                    unix_seconds: 0,
                    unix_millis: 0,
                    subsec_nanos: 0,
                }),
                clock_sequence: None,
                node_id: None,
                node_kind: None,
            }
        }
        InspectableUuidVersion::V3 => UuidMetadata::NameBased {
            algorithm: NameHashAlgorithm::Md5,
        },
        InspectableUuidVersion::V5 => UuidMetadata::NameBased {
            algorithm: NameHashAlgorithm::Sha1,
        },
        InspectableUuidVersion::V4 => UuidMetadata::Random,
        InspectableUuidVersion::V8 => UuidMetadata::Custom {
            bytes: *uuid.as_bytes(),
        },
        InspectableUuidVersion::V2 => UuidMetadata::DceSecurity,
        InspectableUuidVersion::Nil
        | InspectableUuidVersion::Max
        | InspectableUuidVersion::Unknown(_) => UuidMetadata::None,
    }
}

fn clock_sequence(uuid: &Uuid) -> u16 {
    let bytes = uuid.as_bytes();
    (u16::from(bytes[8] & 0x3f) << 8) | u16::from(bytes[9])
}

fn layout_for(bytes: &[u8; 16], version: InspectableUuidVersion) -> Vec<BitField> {
    match version {
        InspectableUuidVersion::V1 => vec![
            field("time_low", 0, 32, bits(bytes, 0, 32)),
            field("time_mid", 32, 16, bits(bytes, 32, 16)),
            field("version", 48, 4, bits(bytes, 48, 4)),
            field("time_hi", 52, 12, bits(bytes, 52, 12)),
            field("variant", 64, 2, bits(bytes, 64, 2)),
            field("clock_sequence", 66, 14, bits(bytes, 66, 14)),
            field("node", 80, 48, bits(bytes, 80, 48)),
        ],
        InspectableUuidVersion::V6 => vec![
            field("timestamp_high", 0, 48, bits(bytes, 0, 48)),
            field("version", 48, 4, bits(bytes, 48, 4)),
            field("timestamp_low", 52, 12, bits(bytes, 52, 12)),
            field("variant", 64, 2, bits(bytes, 64, 2)),
            field("clock_sequence", 66, 14, bits(bytes, 66, 14)),
            field("node", 80, 48, bits(bytes, 80, 48)),
        ],
        InspectableUuidVersion::V7 => vec![
            field("unix_timestamp_ms", 0, 48, bits(bytes, 0, 48)),
            field("version", 48, 4, bits(bytes, 48, 4)),
            field("rand_a", 52, 12, bits(bytes, 52, 12)),
            field("variant", 64, 2, bits(bytes, 64, 2)),
            field("rand_b", 66, 62, bits(bytes, 66, 62)),
        ],
        InspectableUuidVersion::V8 => vec![
            field("custom_a", 0, 48, bits(bytes, 0, 48)),
            field("version", 48, 4, bits(bytes, 48, 4)),
            field("custom_b", 52, 12, bits(bytes, 52, 12)),
            field("variant", 64, 2, bits(bytes, 64, 2)),
            field("custom_c", 66, 62, bits(bytes, 66, 62)),
        ],
        InspectableUuidVersion::V2
        | InspectableUuidVersion::V3
        | InspectableUuidVersion::V4
        | InspectableUuidVersion::V5
        | InspectableUuidVersion::Unknown(_) => vec![
            field("payload_a", 0, 48, bits(bytes, 0, 48)),
            field("version", 48, 4, bits(bytes, 48, 4)),
            field("payload_b", 52, 12, bits(bytes, 52, 12)),
            field("variant", 64, 2, bits(bytes, 64, 2)),
            field("payload_c", 66, 62, bits(bytes, 66, 62)),
        ],
        InspectableUuidVersion::Nil | InspectableUuidVersion::Max => {
            vec![field("value", 0, 128, bits(bytes, 0, 128))]
        }
    }
}

fn field(name: &str, offset: u8, width: u8, value: u128) -> BitField {
    BitField {
        name: name.to_owned(),
        offset,
        width,
        value,
    }
}

fn bits(bytes: &[u8; 16], offset: u8, width: u8) -> u128 {
    let mut value = 0u128;
    for bit in offset..offset.saturating_add(width) {
        let byte = bytes[(bit / 8) as usize];
        let bit_value = (byte >> (7 - (bit % 8))) & 1;
        value = (value << 1) | u128::from(bit_value);
    }
    value
}
