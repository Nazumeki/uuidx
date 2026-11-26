use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuidx_core::{
    BitField, IdentifierFamily, IdentifierInspection, NanoidInspection, SnowflakeInspection,
    UuidInspection, UuidMetadata, UuidVariant, inspect_uuid, parse_uuid,
};

use crate::error::ApiError;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct UuidInspectInput {
    pub(crate) redact_sensitive: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct IdentifierInspectInput {
    #[serde(rename = "type")]
    pub(crate) kind: Option<String>,
    pub(crate) redact_sensitive: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UuidInspectionDto {
    kind: &'static str,
    value: String,
    bytes: String,
    version: String,
    variant: String,
    is_nil: bool,
    is_max: bool,
    fields: Vec<BitFieldDto>,
    metadata: MetadataDto,
}

#[derive(Debug, Serialize)]
pub(crate) struct BitFieldDto {
    name: String,
    offset: u8,
    width: u8,
    value: String,
}

#[derive(Debug, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub(crate) enum MetadataDto {
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
}

pub(crate) fn inspect(input: &str, redact_sensitive: bool) -> Result<UuidInspectionDto, ApiError> {
    let uuid = parse_uuid(input)?;
    Ok(UuidInspectionDto::from_core(
        inspect_uuid(&uuid),
        redact_sensitive,
    ))
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum IdentifierInspectionDto {
    Uuid(UuidInspectionDto),
    #[cfg(feature = "ulid-inspect")]
    Ulid(UlidInspectionDto),
    Nanoid(NanoidInspectionDto),
    Snowflake(SnowflakeInspectionDto),
}

pub(crate) fn parse_family(value: &str) -> Result<IdentifierFamily, ApiError> {
    IdentifierFamily::from_str(value)
        .map_err(|message| ApiError::new("unsupported_identifier_type", message))
}

pub(crate) fn validate_identifier(input: &str, family: Option<&str>) -> Result<bool, ApiError> {
    match family {
        Some(family) => {
            let family = parse_family(family)?;
            Ok(uuidx_core::inspect_identifier_as(input, family).is_ok())
        }
        None => Ok(uuidx_core::inspect_identifier(input).is_ok()),
    }
}

pub(crate) fn inspect_identifier(
    input: &str,
    options: IdentifierInspectInput,
) -> Result<IdentifierInspectionDto, ApiError> {
    let inspection = match options.kind.as_deref() {
        Some(kind) => {
            let family = parse_family(kind)?;
            uuidx_core::inspect_identifier_as(input, family)
                .map_err(|error| ApiError::new("invalid_identifier", error.to_string()))?
        }
        None => uuidx_core::inspect_identifier(input)
            .map_err(|error| ApiError::new("invalid_identifier", error.to_string()))?,
    };

    match inspection {
        IdentifierInspection::Uuid(inspection) => Ok(IdentifierInspectionDto::Uuid(
            UuidInspectionDto::from_core(inspection, options.redact_sensitive),
        )),
        #[cfg(feature = "ulid-inspect")]
        IdentifierInspection::Ulid(inspection) => {
            Ok(IdentifierInspectionDto::Ulid(ulid_dto(inspection)))
        }
        IdentifierInspection::Nanoid(inspection) => {
            Ok(IdentifierInspectionDto::Nanoid(inspection.into()))
        }
        IdentifierInspection::Snowflake(inspection) => {
            Ok(IdentifierInspectionDto::Snowflake(inspection.into()))
        }
    }
}

impl UuidInspectionDto {
    fn from_core(inspection: UuidInspection, redact_sensitive: bool) -> Self {
        Self {
            kind: "uuid",
            value: inspection.normalized,
            bytes: hex::encode(inspection.bytes),
            version: inspection.version.to_string(),
            variant: variant_name(inspection.variant).to_owned(),
            is_nil: inspection.is_nil,
            is_max: inspection.is_max,
            fields: inspection.fields.iter().map(BitFieldDto::from).collect(),
            metadata: MetadataDto::from_core(inspection.metadata, redact_sensitive),
        }
    }
}

fn variant_name(variant: UuidVariant) -> &'static str {
    match variant {
        UuidVariant::Ncs => "ncs",
        UuidVariant::Rfc9562 => "rfc9562",
        UuidVariant::Microsoft => "microsoft",
        UuidVariant::Future => "future",
    }
}

impl From<&BitField> for BitFieldDto {
    fn from(field: &BitField) -> Self {
        Self {
            name: field.name.clone(),
            offset: field.offset,
            width: field.width,
            value: format!("0x{:x}", field.value),
        }
    }
}

impl MetadataDto {
    fn from_core(metadata: UuidMetadata, redact_sensitive: bool) -> Self {
        match metadata {
            UuidMetadata::None => Self::None,
            UuidMetadata::Random => Self::Random,
            UuidMetadata::NameBased { algorithm } => Self::NameBased {
                algorithm: algorithm.to_string(),
            },
            UuidMetadata::Time {
                timestamp,
                clock_sequence,
                node_id,
                node_kind,
            } => Self::Time {
                unix_seconds: timestamp.unix_seconds,
                unix_millis: timestamp.unix_millis,
                subsec_nanos: timestamp.subsec_nanos,
                clock_sequence,
                node_id: if redact_sensitive {
                    None
                } else {
                    node_id.map(hex::encode)
                },
                node_kind: node_kind.map(|kind| kind.to_string()),
            },
            UuidMetadata::Custom { bytes } => Self::Custom {
                bytes: hex::encode(bytes),
            },
            UuidMetadata::DceSecurity => Self::DceSecurity,
        }
    }
}

#[cfg(feature = "ulid-inspect")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UlidInspectionDto {
    kind: &'static str,
    value: String,
    bytes: String,
    timestamp_ms: u64,
    random: String,
    fields: Vec<BitFieldDto>,
}

#[cfg(feature = "ulid-inspect")]
pub(crate) fn inspect_ulid(input: &str) -> Result<UlidInspectionDto, ApiError> {
    let inspection = uuidx_core::inspect_ulid(input)
        .map_err(|error| ApiError::new("invalid_ulid", error.to_string()))?;
    Ok(ulid_dto(inspection))
}

#[cfg(feature = "ulid-inspect")]
fn ulid_dto(inspection: uuidx_core::UlidInspection) -> UlidInspectionDto {
    UlidInspectionDto {
        kind: "ulid",
        value: inspection.normalized,
        bytes: hex::encode(inspection.bytes),
        timestamp_ms: inspection.timestamp_ms,
        random: format!("0x{:020x}", inspection.random),
        fields: inspection.fields.iter().map(BitFieldDto::from).collect(),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NanoidInspectionDto {
    kind: &'static str,
    value: String,
    length: usize,
    alphabet: &'static str,
    entropy_bits: u16,
}

impl From<NanoidInspection> for NanoidInspectionDto {
    fn from(inspection: NanoidInspection) -> Self {
        Self {
            kind: "nanoid",
            value: inspection.normalized,
            length: inspection.length,
            alphabet: inspection.alphabet,
            entropy_bits: inspection.entropy_bits,
        }
    }
}

pub(crate) fn inspect_nanoid(input: &str) -> Result<NanoidInspectionDto, ApiError> {
    uuidx_core::inspect_nanoid(input)
        .map(NanoidInspectionDto::from)
        .map_err(|error| ApiError::new("invalid_nanoid", error.to_string()))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SnowflakeInspectionDto {
    kind: &'static str,
    value: String,
    epoch: &'static str,
    epoch_ms: u64,
    timestamp_ms: u64,
    datacenter_id: u8,
    worker_id: u8,
    sequence: u16,
    fields: Vec<BitFieldDto>,
}

impl From<SnowflakeInspection> for SnowflakeInspectionDto {
    fn from(inspection: SnowflakeInspection) -> Self {
        Self {
            kind: "snowflake",
            value: inspection.normalized,
            epoch: "twitter",
            epoch_ms: inspection.epoch_ms,
            timestamp_ms: inspection.timestamp_ms,
            datacenter_id: inspection.datacenter_id,
            worker_id: inspection.worker_id,
            sequence: inspection.sequence,
            fields: inspection.fields.iter().map(BitFieldDto::from).collect(),
        }
    }
}

pub(crate) fn inspect_snowflake(input: &str) -> Result<SnowflakeInspectionDto, ApiError> {
    uuidx_core::inspect_snowflake(input)
        .map(SnowflakeInspectionDto::from)
        .map_err(|error| ApiError::new("invalid_snowflake", error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_inspection_is_javascript_safe_and_camel_cased() {
        let value =
            serde_json::to_value(inspect("018f2c0b-6c5b-7d2e-8f4a-123456789abc", false).unwrap())
                .unwrap();

        assert_eq!(value["kind"], "uuid");
        assert_eq!(value["version"], "v7");
        assert_eq!(value["variant"], "rfc9562");
        assert_eq!(value["isNil"], false);
        assert_eq!(value["metadata"]["type"], "time");
        assert!(value["metadata"]["unixMillis"].is_number());
        assert!(value["metadata"]["clockSequence"].is_null());
        assert!(value["metadata"]["nodeId"].is_null());
        assert_eq!(value["fields"][0]["value"], "0x18f2c0b6c5b");
        assert!(value["fields"][0]["value"].is_string());
    }

    #[test]
    fn variants_use_machine_readable_names() {
        for (variant, expected) in [
            (UuidVariant::Ncs, "ncs"),
            (UuidVariant::Rfc9562, "rfc9562"),
            (UuidVariant::Microsoft, "microsoft"),
            (UuidVariant::Future, "future"),
        ] {
            assert_eq!(variant_name(variant), expected);
        }
    }

    #[test]
    fn inspection_maps_metadata_families() {
        for (input, expected_type) in [
            ("00000000-0000-0000-0000-000000000000", "none"),
            ("11111111-1111-1111-9111-111111111111", "time"),
            ("11111111-1111-2111-9111-111111111111", "dce_security"),
            ("11111111-1111-3111-9111-111111111111", "name_based"),
            ("11111111-1111-4111-9111-111111111111", "random"),
            ("11111111-1111-5111-9111-111111111111", "name_based"),
            ("11111111-1111-8111-9111-111111111111", "custom"),
        ] {
            let value = serde_json::to_value(inspect(input, false).unwrap()).unwrap();
            assert_eq!(value["metadata"]["type"], expected_type);
        }
    }

    #[test]
    fn time_metadata_includes_clock_node_and_node_kind() {
        for (node, expected_kind) in [
            ("020000000001", "unicast / may be hardware-derived"),
            ("030000000001", "multicast / locally generated"),
        ] {
            let value = serde_json::to_value(
                inspect(&format!("11111111-1111-1111-9234-{node}"), false).unwrap(),
            )
            .unwrap();

            assert_eq!(value["metadata"]["clockSequence"], 0x1234);
            assert_eq!(value["metadata"]["nodeId"], node);
            assert_eq!(value["metadata"]["nodeKind"], expected_kind);

            let redacted = serde_json::to_value(
                inspect(&format!("11111111-1111-1111-9234-{node}"), true).unwrap(),
            )
            .unwrap();
            assert!(redacted["metadata"]["nodeId"].is_null());
            assert_eq!(redacted["metadata"]["nodeKind"], expected_kind);
        }
    }

    #[test]
    fn inspection_preserves_special_values_and_metadata_details() {
        let nil =
            serde_json::to_value(inspect("00000000-0000-0000-0000-000000000000", false).unwrap())
                .unwrap();
        assert_eq!(nil["isNil"], true);
        assert_eq!(nil["metadata"]["type"], "none");

        let max =
            serde_json::to_value(inspect("ffffffff-ffff-ffff-ffff-ffffffffffff", false).unwrap())
                .unwrap();
        assert_eq!(max["isMax"], true);
        assert_eq!(max["variant"], "future");

        let v5 =
            serde_json::to_value(inspect("11111111-1111-5111-9111-111111111111", false).unwrap())
                .unwrap();
        assert_eq!(v5["metadata"]["algorithm"], "SHA-1");

        let v8 =
            serde_json::to_value(inspect("11111111-1111-8111-9111-111111111111", false).unwrap())
                .unwrap();
        assert_eq!(v8["metadata"]["bytes"], "11111111111181119111111111111111");
    }

    #[test]
    fn inspection_rejects_invalid_inputs() {
        assert_eq!(
            inspect("not-a-uuid", false).unwrap_err().code(),
            "invalid_uuid"
        );
    }

    #[cfg(feature = "ulid-inspect")]
    #[test]
    fn ulid_inspection_uses_strings_for_eighty_bit_randomness() {
        let value =
            serde_json::to_value(inspect_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap()).unwrap();

        assert_eq!(value["kind"], "ulid");
        assert_eq!(value["timestampMs"], 1_469_922_850_259_u64);
        assert!(value["random"].as_str().unwrap().starts_with("0x"));
        assert_eq!(value["fields"].as_array().map(Vec::len), Some(2));
    }

    #[cfg(feature = "ulid-inspect")]
    #[test]
    fn ulid_inspection_rejects_invalid_inputs() {
        assert_eq!(
            inspect_ulid("not-a-ulid").unwrap_err().code(),
            "invalid_ulid"
        );
    }

    #[test]
    fn identifier_inspection_serializes_nanoid_and_snowflake_safely() {
        let nanoid = serde_json::to_value(
            inspect_identifier("V1StGXR8_Z5jdHi6B-myT", IdentifierInspectInput::default()).unwrap(),
        )
        .unwrap();
        assert_eq!(nanoid["kind"], "nanoid");
        assert_eq!(nanoid["entropyBits"], 126);

        let snowflake = serde_json::to_value(
            inspect_identifier("1724552287438348288", IdentifierInspectInput::default()).unwrap(),
        )
        .unwrap();
        assert_eq!(snowflake["kind"], "snowflake");
        assert_eq!(snowflake["value"], "1724552287438348288");
        assert!(snowflake["timestampMs"].is_number());
        assert_eq!(snowflake["fields"].as_array().map(Vec::len), Some(5));
    }

    #[test]
    fn identifier_specific_inspectors_return_stable_error_codes() {
        assert_eq!(
            inspect_identifier("bad", IdentifierInspectInput::default())
                .unwrap_err()
                .code(),
            "invalid_identifier"
        );
        assert_eq!(inspect_nanoid("bad").unwrap_err().code(), "invalid_nanoid");
        assert_eq!(
            inspect_snowflake("bad").unwrap_err().code(),
            "invalid_snowflake"
        );
    }

    #[test]
    fn forced_identifier_type_and_validation_enforce_family_contracts() {
        assert!(validate_identifier("018f2c0b-6c5b-7d2e-8f4a-123456789abc", None).unwrap());
        assert!(validate_identifier("V1StGXR8_Z5jdHi6B-myT", Some("nanoid")).unwrap());
        assert!(!validate_identifier("V1StGXR8_Z5jdHi6B-myT", Some("uuid")).unwrap());
        assert!(!validate_identifier("not-an-identifier", None).unwrap());
        assert_eq!(
            validate_identifier("V1StGXR8_Z5jdHi6B-myT", Some("objectid"))
                .unwrap_err()
                .code(),
            "unsupported_identifier_type"
        );

        let forced_uuid = serde_json::to_value(
            inspect_identifier(
                "11111111-1111-1111-9234-020000000001",
                IdentifierInspectInput {
                    kind: Some("uuid".to_owned()),
                    redact_sensitive: true,
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(forced_uuid["kind"], "uuid");
        assert!(forced_uuid["metadata"]["nodeId"].is_null());

        let forced_snowflake = serde_json::to_value(
            inspect_identifier(
                "1724552287438348288",
                IdentifierInspectInput {
                    kind: Some("snowflake".to_owned()),
                    redact_sensitive: false,
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(forced_snowflake["kind"], "snowflake");

        assert_eq!(
            inspect_identifier(
                "V1StGXR8_Z5jdHi6B-myT",
                IdentifierInspectInput {
                    kind: Some("uuid".to_owned()),
                    redact_sensitive: false,
                },
            )
            .unwrap_err()
            .code(),
            "invalid_identifier"
        );
        assert_eq!(
            inspect_identifier(
                "V1StGXR8_Z5jdHi6B-myT",
                IdentifierInspectInput {
                    kind: Some("objectid".to_owned()),
                    redact_sensitive: false,
                },
            )
            .unwrap_err()
            .code(),
            "unsupported_identifier_type"
        );

        #[cfg(feature = "ulid-inspect")]
        {
            assert!(validate_identifier("01ARZ3NDEKTSV4RRFFQ69G5FAV", Some("ulid")).unwrap());
            let forced_ulid = serde_json::to_value(
                inspect_identifier(
                    "01ARZ3NDEKTSV4RRFFQ69G5FAV",
                    IdentifierInspectInput {
                        kind: Some("ulid".to_owned()),
                        redact_sensitive: false,
                    },
                )
                .unwrap(),
            )
            .unwrap();
            assert_eq!(forced_ulid["kind"], "ulid");
        }
    }

    #[test]
    fn inspect_options_reject_unknown_fields() {
        assert!(serde_json::from_str::<UuidInspectInput>(r#"{"unknown":true}"#).is_err());
        assert!(serde_json::from_str::<IdentifierInspectInput>(r#"{"unknown":true}"#).is_err());
    }
}
