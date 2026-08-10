use serde::Serialize;
use uuidx_core::{BitField, Uuid, UuidInspection, UuidMetadata, UuidOutputFormat, inspect_uuid};

#[derive(Debug, Serialize)]
pub struct JsonRecord {
    pub schema_version: u8,
    pub operation: String,
    pub index: u64,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<JsonBitField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonError>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct JsonError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct JsonBitField {
    pub name: String,
    pub offset: u8,
    pub width: u8,
    pub value: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JsonMetadata {
    None,
    Random,
    NameBased {
        algorithm: String,
    },
    Time {
        unix_seconds: u64,
        unix_millis: u64,
        subsec_nanos: u32,
        clock_sequence: Option<u16>,
        node_id: Option<String>,
        node_kind: Option<String>,
    },
    Custom {
        bytes: String,
    },
    DceSecurity,
    #[cfg(feature = "ulid-inspect")]
    Ulid {
        timestamp_ms: u64,
        random: String,
        bytes: String,
    },
}

impl JsonRecord {
    pub fn generated(
        index: u64,
        uuid: &Uuid,
        value: &str,
        version: &str,
        format: UuidOutputFormat,
        warnings: &[String],
    ) -> Self {
        Self {
            schema_version: 1,
            operation: "generate".to_owned(),
            index,
            ok: true,
            input: None,
            value: Some(value.to_owned()),
            bytes: Some(hex::encode(uuid.as_bytes())),
            kind: Some("uuid".to_owned()),
            version: Some(version.to_owned()),
            format: Some(format.to_string()),
            metadata: None,
            fields: None,
            error: None,
            warnings: warnings.to_vec(),
        }
    }

    pub fn uuid_inspection(
        index: u64,
        input: &str,
        inspection: &UuidInspection,
        redact_sensitive: bool,
    ) -> Self {
        Self::from_inspection("inspect", index, input, inspection, redact_sensitive)
    }

    pub fn converted(
        index: u64,
        input: &str,
        uuid: &Uuid,
        value: &str,
        format: UuidOutputFormat,
    ) -> Self {
        let inspection = inspect_uuid(uuid);
        Self {
            schema_version: 1,
            operation: "convert".to_owned(),
            index,
            ok: true,
            input: Some(input.to_owned()),
            value: Some(value.to_owned()),
            bytes: Some(hex::encode(uuid.as_bytes())),
            kind: Some("uuid".to_owned()),
            version: Some(inspection.version.to_string()),
            metadata: None,
            fields: None,
            error: None,
            format: Some(format.to_string()),
            warnings: if inspection.version == uuidx_core::InspectableUuidVersion::V5 {
                vec!["UUID v5 uses legacy SHA-1 name hashing".to_owned()]
            } else {
                Vec::new()
            },
        }
    }

    pub fn validated(index: u64, input: &str, uuid: &Uuid) -> Self {
        let inspection = inspect_uuid(uuid);
        Self {
            schema_version: 1,
            operation: "validate".to_owned(),
            index,
            ok: true,
            input: Some(input.to_owned()),
            value: Some(inspection.normalized),
            bytes: Some(hex::encode(uuid.as_bytes())),
            kind: Some("uuid".to_owned()),
            version: Some(inspection.version.to_string()),
            format: None,
            metadata: None,
            fields: None,
            error: None,
            warnings: Vec::new(),
        }
    }

    pub fn error(operation: &str, index: u64, input: &str, code: &str, message: &str) -> Self {
        Self {
            schema_version: 1,
            operation: operation.to_owned(),
            index,
            ok: false,
            input: Some(input.to_owned()),
            value: None,
            bytes: None,
            kind: None,
            version: None,
            format: None,
            metadata: None,
            fields: None,
            error: Some(JsonError {
                code: code.to_owned(),
                message: message.to_owned(),
            }),
            warnings: Vec::new(),
        }
    }

    #[cfg(feature = "ulid-inspect")]
    pub fn ulid_inspection(
        index: u64,
        input: &str,
        inspection: &uuidx_core::UlidInspection,
    ) -> Self {
        Self {
            schema_version: 1,
            operation: "inspect".to_owned(),
            index,
            ok: true,
            input: Some(input.to_owned()),
            value: Some(inspection.normalized.clone()),
            bytes: Some(hex::encode(inspection.bytes)),
            kind: Some("ulid".to_owned()),
            version: None,
            format: None,
            metadata: Some(JsonMetadata::Ulid {
                timestamp_ms: inspection.timestamp_ms,
                random: format!("0x{:020x}", inspection.random),
                bytes: hex::encode(inspection.bytes),
            }),
            fields: None,
            error: None,
            warnings: vec!["ULID support is inspection-only".to_owned()],
        }
    }

    fn from_inspection(
        operation: &str,
        index: u64,
        input: &str,
        inspection: &UuidInspection,
        redact_sensitive: bool,
    ) -> Self {
        Self {
            schema_version: 1,
            operation: operation.to_owned(),
            index,
            ok: true,
            input: Some(input.to_owned()),
            value: Some(inspection.normalized.clone()),
            bytes: Some(hex::encode(inspection.bytes)),
            kind: Some("uuid".to_owned()),
            version: Some(inspection.version.to_string()),
            format: None,
            metadata: Some(metadata(inspection, redact_sensitive)),
            fields: Some(inspection.fields.iter().map(JsonBitField::from).collect()),
            error: None,
            warnings: warnings_for(inspection),
        }
    }
}

impl From<&BitField> for JsonBitField {
    fn from(field: &BitField) -> Self {
        Self {
            name: field.name.clone(),
            offset: field.offset,
            width: field.width,
            value: format!("0x{:x}", field.value),
        }
    }
}

fn metadata(inspection: &UuidInspection, redact_sensitive: bool) -> JsonMetadata {
    match &inspection.metadata {
        UuidMetadata::None => JsonMetadata::None,
        UuidMetadata::Random => JsonMetadata::Random,
        UuidMetadata::NameBased { algorithm } => JsonMetadata::NameBased {
            algorithm: algorithm.to_string(),
        },
        UuidMetadata::Time {
            timestamp,
            clock_sequence,
            node_id,
            node_kind,
        } => JsonMetadata::Time {
            unix_seconds: timestamp.unix_seconds,
            unix_millis: timestamp.unix_millis,
            subsec_nanos: timestamp.subsec_nanos,
            clock_sequence: *clock_sequence,
            node_id: if redact_sensitive {
                None
            } else {
                node_id.map(hex::encode)
            },
            node_kind: node_kind.map(|kind| kind.to_string()),
        },
        UuidMetadata::Custom { bytes } => JsonMetadata::Custom {
            bytes: hex::encode(bytes),
        },
        UuidMetadata::DceSecurity => JsonMetadata::DceSecurity,
    }
}

fn warnings_for(inspection: &UuidInspection) -> Vec<String> {
    match inspection.version {
        uuidx_core::InspectableUuidVersion::V1 => {
            vec!["UUID v1 exposes timestamp and node metadata".to_owned()]
        }
        uuidx_core::InspectableUuidVersion::V2 => {
            vec!["UUID v2 DCE Security semantics are outside RFC 9562".to_owned()]
        }
        uuidx_core::InspectableUuidVersion::V3 => {
            vec!["UUID v3 uses legacy MD5 name hashing".to_owned()]
        }
        uuidx_core::InspectableUuidVersion::V5 => {
            vec!["UUID v5 uses legacy SHA-1 name hashing".to_owned()]
        }
        _ => Vec::new(),
    }
}
