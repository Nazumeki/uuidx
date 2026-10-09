use std::{
    str::FromStr,
    time::{Duration, UNIX_EPOCH},
};

use uuidx_core::{
    GeneratableUuidVersion, GenerateError, GenerationOptions, InspectableUuidVersion,
    NameHashAlgorithm, NodeKind, Uuid, UuidMetadata, UuidOutputFormat, UuidVariant, format_uuid,
    generate_uuid, inspect_uuid, parse_hex_array, parse_uuid,
};

fn generate(version: GeneratableUuidVersion) -> Uuid {
    generate_uuid(&GenerationOptions::new(version)).expect("generation should succeed")
}

fn versioned_uuid(version: u8) -> Uuid {
    let mut bytes = [0x11; 16];
    bytes[6] = (bytes[6] & 0x0f) | (version << 4);
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

fn versioned_uuid_with_variant(version: u8, variant_byte: u8) -> Uuid {
    let mut bytes = *versioned_uuid(version).as_bytes();
    bytes[8] = variant_byte;
    Uuid::from_bytes(bytes)
}

fn assert_unexpected(options: GenerationOptions, expected_option: &'static str) {
    assert!(matches!(
        generate_uuid(&options),
        Err(GenerateError::UnexpectedOption { option, .. }) if option == expected_option
    ));
}

#[test]
fn only_supported_versions_are_generatable() {
    for (version, expected) in [
        (GeneratableUuidVersion::V3, InspectableUuidVersion::V3),
        (GeneratableUuidVersion::V4, InspectableUuidVersion::V4),
        (GeneratableUuidVersion::V5, InspectableUuidVersion::V5),
        (GeneratableUuidVersion::V6, InspectableUuidVersion::V6),
        (GeneratableUuidVersion::V7, InspectableUuidVersion::V7),
        (GeneratableUuidVersion::V8, InspectableUuidVersion::V8),
    ] {
        let uuid = if matches!(
            version,
            GeneratableUuidVersion::V3 | GeneratableUuidVersion::V5
        ) {
            let mut options = GenerationOptions::new(version);
            options.namespace = Some(Uuid::NAMESPACE_DNS);
            options.name = Some(b"uuidx".to_vec());
            generate_uuid(&options).expect("name-based generation should succeed")
        } else if version == GeneratableUuidVersion::V8 {
            let mut options = GenerationOptions::new(version);
            options.custom = Some([0xabu8; 16]);
            generate_uuid(&options).expect("v8 generation should succeed")
        } else {
            generate(version)
        };

        let inspection = inspect_uuid(&uuid);
        assert_eq!(inspection.version, expected);
        assert_eq!(inspection.variant, UuidVariant::Rfc9562);
    }
}

#[test]
fn v5_is_deterministic_and_reports_sha1() {
    let mut options = GenerationOptions::new(GeneratableUuidVersion::V5);
    options.namespace = Some(Uuid::NAMESPACE_DNS);
    options.name = Some(b"example.com".to_vec());

    let first = generate_uuid(&options).expect("v5 generation should succeed");
    let second = generate_uuid(&options).expect("v5 generation should succeed");
    assert_eq!(first, second);
    assert_eq!(
        inspect_uuid(&first).metadata,
        UuidMetadata::NameBased {
            algorithm: NameHashAlgorithm::Sha1
        }
    );
}

#[test]
fn v3_matches_the_rfc_vector_and_reports_md5() {
    let mut options = GenerationOptions::new(GeneratableUuidVersion::V3);
    options.namespace = Some(Uuid::NAMESPACE_DNS);
    options.name = Some(b"example.org".to_vec());

    let first = generate_uuid(&options).expect("v3 generation should succeed");
    let second = generate_uuid(&options).expect("v3 generation should succeed");
    assert_eq!(first, second);
    assert_eq!(first.to_string(), "04738bdf-b25a-3829-a801-b21a1d25095b");
    assert_eq!(
        inspect_uuid(&first).metadata,
        UuidMetadata::NameBased {
            algorithm: NameHashAlgorithm::Md5
        }
    );
}

#[test]
fn name_and_custom_generators_require_their_payloads() {
    let v3_error = generate_uuid(&GenerationOptions::new(GeneratableUuidVersion::V3))
        .expect_err("v3 should require namespace and name");
    assert!(matches!(
        v3_error,
        GenerateError::MissingNameArguments { .. }
    ));

    let name_error = generate_uuid(&GenerationOptions::new(GeneratableUuidVersion::V5))
        .expect_err("v5 should require namespace and name");
    assert!(matches!(
        name_error,
        GenerateError::MissingNameArguments { .. }
    ));

    let custom_error = generate_uuid(&GenerationOptions::new(GeneratableUuidVersion::V8))
        .expect_err("v8 should require custom bytes");
    assert!(matches!(
        custom_error,
        GenerateError::MissingCustomBytes { .. }
    ));

    let mut namespace_only = GenerationOptions::new(GeneratableUuidVersion::V5);
    namespace_only.namespace = Some(Uuid::NAMESPACE_DNS);
    assert!(matches!(
        generate_uuid(&namespace_only),
        Err(GenerateError::MissingNameArguments { .. })
    ));

    let mut name_only = GenerationOptions::new(GeneratableUuidVersion::V5);
    name_only.name = Some(b"uuidx".to_vec());
    assert!(matches!(
        generate_uuid(&name_only),
        Err(GenerateError::MissingNameArguments { .. })
    ));
}

#[test]
fn generators_reject_options_outside_their_contracts() {
    let mut v4 = GenerationOptions::new(GeneratableUuidVersion::V4);
    v4.namespace = Some(Uuid::NAMESPACE_DNS);
    assert_unexpected(v4, "--namespace");
    let mut v4 = GenerationOptions::new(GeneratableUuidVersion::V4);
    v4.name = Some(b"uuidx".to_vec());
    assert_unexpected(v4, "--name");
    let mut v4 = GenerationOptions::new(GeneratableUuidVersion::V4);
    v4.node = Some([0; 6]);
    assert_unexpected(v4, "--node");
    let mut v4 = GenerationOptions::new(GeneratableUuidVersion::V4);
    v4.timestamp = Some(UNIX_EPOCH);
    assert_unexpected(v4, "--timestamp");
    let mut v4 = GenerationOptions::new(GeneratableUuidVersion::V4);
    v4.custom = Some([0; 16]);
    assert_unexpected(v4, "--custom");

    let mut v5 = GenerationOptions::new(GeneratableUuidVersion::V5);
    v5.namespace = Some(Uuid::NAMESPACE_DNS);
    v5.name = Some(b"uuidx".to_vec());
    v5.node = Some([0; 6]);
    assert_unexpected(v5, "--node");
    let mut v5 = GenerationOptions::new(GeneratableUuidVersion::V5);
    v5.namespace = Some(Uuid::NAMESPACE_DNS);
    v5.name = Some(b"uuidx".to_vec());
    v5.timestamp = Some(UNIX_EPOCH);
    assert_unexpected(v5, "--timestamp");
    let mut v5 = GenerationOptions::new(GeneratableUuidVersion::V5);
    v5.namespace = Some(Uuid::NAMESPACE_DNS);
    v5.name = Some(b"uuidx".to_vec());
    v5.custom = Some([0; 16]);
    assert_unexpected(v5, "--custom");

    let mut v3 = GenerationOptions::new(GeneratableUuidVersion::V3);
    v3.namespace = Some(Uuid::NAMESPACE_DNS);
    v3.name = Some(b"uuidx".to_vec());
    v3.node = Some([0; 6]);
    assert_unexpected(v3, "--node");
    let mut v3 = GenerationOptions::new(GeneratableUuidVersion::V3);
    v3.namespace = Some(Uuid::NAMESPACE_DNS);
    v3.name = Some(b"uuidx".to_vec());
    v3.timestamp = Some(UNIX_EPOCH);
    assert_unexpected(v3, "--timestamp");
    let mut v3 = GenerationOptions::new(GeneratableUuidVersion::V3);
    v3.namespace = Some(Uuid::NAMESPACE_DNS);
    v3.name = Some(b"uuidx".to_vec());
    v3.custom = Some([0; 16]);
    assert_unexpected(v3, "--custom");

    let mut v6 = GenerationOptions::new(GeneratableUuidVersion::V6);
    v6.namespace = Some(Uuid::NAMESPACE_DNS);
    assert_unexpected(v6, "--namespace");
    let mut v6 = GenerationOptions::new(GeneratableUuidVersion::V6);
    v6.name = Some(b"uuidx".to_vec());
    assert_unexpected(v6, "--name");
    let mut v6 = GenerationOptions::new(GeneratableUuidVersion::V6);
    v6.custom = Some([0; 16]);
    assert_unexpected(v6, "--custom");

    let mut v7 = GenerationOptions::new(GeneratableUuidVersion::V7);
    v7.namespace = Some(Uuid::NAMESPACE_DNS);
    assert_unexpected(v7, "--namespace");
    let mut v7 = GenerationOptions::new(GeneratableUuidVersion::V7);
    v7.name = Some(b"uuidx".to_vec());
    assert_unexpected(v7, "--name");
    let mut v7 = GenerationOptions::new(GeneratableUuidVersion::V7);
    v7.node = Some([0; 6]);
    assert_unexpected(v7, "--node");
    let mut v7 = GenerationOptions::new(GeneratableUuidVersion::V7);
    v7.custom = Some([0; 16]);
    assert_unexpected(v7, "--custom");

    let mut v8 = GenerationOptions::new(GeneratableUuidVersion::V8);
    v8.custom = Some([0; 16]);
    v8.namespace = Some(Uuid::NAMESPACE_DNS);
    assert_unexpected(v8, "--namespace");
    let mut v8 = GenerationOptions::new(GeneratableUuidVersion::V8);
    v8.custom = Some([0; 16]);
    v8.name = Some(b"uuidx".to_vec());
    assert_unexpected(v8, "--name");
    let mut v8 = GenerationOptions::new(GeneratableUuidVersion::V8);
    v8.custom = Some([0; 16]);
    v8.node = Some([0; 6]);
    assert_unexpected(v8, "--node");
    let mut v8 = GenerationOptions::new(GeneratableUuidVersion::V8);
    v8.custom = Some([0; 16]);
    v8.timestamp = Some(UNIX_EPOCH);
    assert_unexpected(v8, "--timestamp");
}

#[test]
fn timestamp_generators_reject_pre_epoch_values() {
    let before_epoch = UNIX_EPOCH - Duration::from_secs(1);
    let mut v6 = GenerationOptions::new(GeneratableUuidVersion::V6);
    v6.timestamp = Some(before_epoch);
    assert!(matches!(
        generate_uuid(&v6),
        Err(GenerateError::InvalidTimestamp {
            version: GeneratableUuidVersion::V6
        })
    ));

    let mut v7 = GenerationOptions::new(GeneratableUuidVersion::V7);
    v7.timestamp = Some(before_epoch);
    assert!(matches!(
        generate_uuid(&v7),
        Err(GenerateError::InvalidTimestamp {
            version: GeneratableUuidVersion::V7
        })
    ));
}

#[test]
fn v6_preserves_timestamp_and_node_metadata() {
    let mut options = GenerationOptions::new(GeneratableUuidVersion::V6);
    options.timestamp = Some(UNIX_EPOCH + Duration::from_secs(1_700_000_000));
    options.node = Some([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]);

    let uuid = generate_uuid(&options).expect("v6 generation should succeed");
    let inspection = inspect_uuid(&uuid);
    assert_eq!(inspection.version, InspectableUuidVersion::V6);
    match inspection.metadata {
        UuidMetadata::Time {
            node_id,
            node_kind,
            timestamp,
            ..
        } => {
            assert_eq!(node_id, Some([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]));
            assert_eq!(node_kind, Some(NodeKind::Unicast));
            assert_eq!(timestamp.unix_seconds, 1_700_000_000);
        }
        _ => panic!("expected time metadata"),
    }
}

#[test]
fn v6_default_node_is_locally_generated() {
    let inspection = inspect_uuid(&generate(GeneratableUuidVersion::V6));
    match inspection.metadata {
        UuidMetadata::Time {
            node_id: Some(node_id),
            node_kind: Some(NodeKind::Multicast),
            ..
        } => assert_ne!(node_id, [0; 6]),
        _ => panic!("expected a locally generated node"),
    }
}

#[test]
fn v7_uses_unix_millisecond_timestamp() {
    let millis = 1_700_000_123_456;
    let mut options = GenerationOptions::new(GeneratableUuidVersion::V7);
    options.timestamp = Some(UNIX_EPOCH + Duration::from_millis(millis));

    let uuid = generate_uuid(&options).expect("v7 generation should succeed");
    let inspection = inspect_uuid(&uuid);
    assert_eq!(inspection.version, InspectableUuidVersion::V7);
    match inspection.metadata {
        UuidMetadata::Time { timestamp, .. } => assert_eq!(timestamp.unix_millis, millis),
        _ => panic!("expected time metadata"),
    }
}

#[test]
fn v8_sets_version_and_retains_final_payload() {
    let mut options = GenerationOptions::new(GeneratableUuidVersion::V8);
    options.custom = Some([0xabu8; 16]);

    let uuid = generate_uuid(&options).expect("v8 generation should succeed");
    let inspection = inspect_uuid(&uuid);
    assert_eq!(inspection.version, InspectableUuidVersion::V8);
    assert_eq!(inspection.variant, UuidVariant::Rfc9562);
    assert_eq!(inspection.bytes[0], 0xab);
    assert_eq!(inspection.bytes[15], 0xab);
    assert!(matches!(inspection.metadata, UuidMetadata::Custom { .. }));
}

#[test]
fn parsing_and_formatting_round_trip() {
    let uuid = parse_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc").expect("valid UUID");
    for format in [
        UuidOutputFormat::Canonical,
        UuidOutputFormat::Simple,
        UuidOutputFormat::Urn,
        UuidOutputFormat::Braced,
    ] {
        let formatted = format_uuid(&uuid, format);
        assert_eq!(
            parse_uuid(&formatted).expect("formatted UUID should parse"),
            uuid
        );
    }
}

#[test]
fn public_format_and_type_contracts_are_stable() {
    let uuid = parse_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc").unwrap();

    assert_eq!(uuid.get_version_num(), 7);
    assert_eq!(uuid.hyphenated(), "018f2c0b-6c5b-7d2e-8f4a-123456789abc");
    assert_eq!(uuid.simple(), "018f2c0b6c5b7d2e8f4a123456789abc");
    assert_eq!(uuid.urn(), "urn:uuid:018f2c0b-6c5b-7d2e-8f4a-123456789abc");
    assert_eq!(uuid.to_string(), uuid.hyphenated());

    assert_eq!(
        format_uuid(&uuid, UuidOutputFormat::Canonical),
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc"
    );
    assert_eq!(
        format_uuid(&uuid, UuidOutputFormat::Simple),
        "018f2c0b6c5b7d2e8f4a123456789abc"
    );
    assert_eq!(
        format_uuid(&uuid, UuidOutputFormat::Urn),
        "urn:uuid:018f2c0b-6c5b-7d2e-8f4a-123456789abc"
    );
    assert_eq!(
        format_uuid(&uuid, UuidOutputFormat::Braced),
        "{018f2c0b-6c5b-7d2e-8f4a-123456789abc}"
    );

    for (format, expected) in [
        (UuidOutputFormat::Canonical, "canonical"),
        (UuidOutputFormat::Simple, "simple"),
        (UuidOutputFormat::Urn, "urn"),
        (UuidOutputFormat::Braced, "braced"),
    ] {
        assert_eq!(format.to_string(), expected);
    }

    for (version, expected) in [
        (GeneratableUuidVersion::V3, "v3"),
        (GeneratableUuidVersion::V4, "v4"),
        (GeneratableUuidVersion::V5, "v5"),
        (GeneratableUuidVersion::V6, "v6"),
        (GeneratableUuidVersion::V7, "v7"),
        (GeneratableUuidVersion::V8, "v8"),
    ] {
        assert_eq!(version.to_string(), expected);
    }
    for (version, expected) in [
        (InspectableUuidVersion::V1, "v1"),
        (InspectableUuidVersion::V2, "v2"),
        (InspectableUuidVersion::V3, "v3"),
        (InspectableUuidVersion::V4, "v4"),
        (InspectableUuidVersion::V5, "v5"),
        (InspectableUuidVersion::V6, "v6"),
        (InspectableUuidVersion::V7, "v7"),
        (InspectableUuidVersion::V8, "v8"),
        (InspectableUuidVersion::Nil, "nil"),
        (InspectableUuidVersion::Max, "max"),
        (InspectableUuidVersion::Unknown(12), "unknown(12)"),
    ] {
        assert_eq!(version.to_string(), expected);
    }

    assert_eq!(
        Uuid::NAMESPACE_DNS.to_string(),
        "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
    );
    assert_eq!(
        Uuid::NAMESPACE_URL.to_string(),
        "6ba7b811-9dad-11d1-80b4-00c04fd430c8"
    );
    assert_eq!(
        Uuid::NAMESPACE_OID.to_string(),
        "6ba7b812-9dad-11d1-80b4-00c04fd430c8"
    );
    assert_eq!(
        Uuid::NAMESPACE_X500.to_string(),
        "6ba7b814-9dad-11d1-80b4-00c04fd430c8"
    );
}

#[test]
fn hex_parser_reports_shape_and_character_errors() {
    assert_eq!(
        parse_hex_array::<3>("0102ff").expect("valid hex"),
        [1, 2, 255]
    );
    assert_eq!(parse_hex_array::<2>(" 0102 ").expect("trimmed hex"), [1, 2]);
    assert!(matches!(
        parse_hex_array::<3>("0102"),
        Err(uuidx_core::HexError::WrongLength {
            expected: 3,
            actual: 2
        })
    ));
    assert!(matches!(
        parse_hex_array::<3>("0102xz"),
        Err(uuidx_core::HexError::Invalid { .. })
    ));
    assert!(matches!(
        parse_hex_array::<3>("010"),
        Err(uuidx_core::HexError::Invalid { .. })
    ));
}

#[test]
fn parser_and_format_enums_cover_aliases_and_errors() {
    for value in ["3", "v3", "V3"] {
        assert_eq!(
            GeneratableUuidVersion::from_str(value).unwrap(),
            GeneratableUuidVersion::V3
        );
    }
    for value in ["4", "v4", "V4"] {
        assert_eq!(
            GeneratableUuidVersion::from_str(value).unwrap(),
            GeneratableUuidVersion::V4
        );
    }
    assert!(GeneratableUuidVersion::from_str("nil").is_err());
    assert!(GeneratableUuidVersion::from_str("max").is_err());
    assert!(GeneratableUuidVersion::from_str("v1").is_err());
    assert!(GeneratableUuidVersion::from_str("future").is_err());

    assert_eq!(InspectableUuidVersion::V1.number(), Some(1));
    assert_eq!(InspectableUuidVersion::Nil.number(), Some(0));
    assert_eq!(InspectableUuidVersion::Max.number(), Some(15));
    assert_eq!(InspectableUuidVersion::Unknown(9).number(), Some(9));
    assert_eq!(InspectableUuidVersion::Unknown(9).to_string(), "unknown(9)");

    for (value, expected) in [
        ("canonical", UuidOutputFormat::Canonical),
        ("hyphenated", UuidOutputFormat::Canonical),
        ("simple", UuidOutputFormat::Simple),
        ("hex", UuidOutputFormat::Simple),
        ("urn", UuidOutputFormat::Urn),
        ("braced", UuidOutputFormat::Braced),
        ("brace", UuidOutputFormat::Braced),
    ] {
        assert_eq!(UuidOutputFormat::from_str(value).unwrap(), expected);
    }
    assert!(UuidOutputFormat::from_str("invalid").is_err());
}

#[test]
fn uuid_wrapper_and_parser_report_special_and_invalid_values() {
    let uuid = parse_uuid(" 018f2c0b-6c5b-7d2e-8f4a-123456789abc ").unwrap();
    assert_eq!(uuid.to_string(), "018f2c0b-6c5b-7d2e-8f4a-123456789abc");
    assert_eq!(uuid.as_bytes().len(), 16);
    let nil = parse_uuid("00000000-0000-0000-0000-000000000000").unwrap();
    let max = parse_uuid("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap();
    assert!(nil.is_nil());
    assert!(max.is_max());
    assert_eq!(inspect_uuid(&nil).metadata, UuidMetadata::None);
    assert_eq!(inspect_uuid(&max).metadata, UuidMetadata::None);
    assert_eq!(inspect_uuid(&nil).fields[0].width, 128);
    assert_eq!(inspect_uuid(&max).fields[0].width, 128);
    assert!(matches!(
        parse_uuid(""),
        Err(uuidx_core::ParseUuidError::Empty)
    ));
    assert!(matches!(
        parse_uuid("  \t  "),
        Err(uuidx_core::ParseUuidError::Empty)
    ));
    assert!(matches!(
        parse_uuid("not-a-uuid"),
        Err(uuidx_core::ParseUuidError::Invalid(_))
    ));
}

#[test]
fn inspection_classifies_all_variants_and_unknown_versions() {
    for (byte, expected) in [
        (0x00, UuidVariant::Ncs),
        (0x80, UuidVariant::Rfc9562),
        (0xc0, UuidVariant::Microsoft),
        (0xe0, UuidVariant::Future),
    ] {
        assert_eq!(
            inspect_uuid(&versioned_uuid_with_variant(4, byte)).variant,
            expected
        );
    }

    let unknown = inspect_uuid(&versioned_uuid(9));
    assert_eq!(unknown.version, InspectableUuidVersion::Unknown(9));
    assert_eq!(unknown.fields.len(), 5);
    assert_eq!(unknown.metadata, UuidMetadata::None);
}

#[test]
fn inspection_layouts_match_each_uuid_family() {
    for (version, expected_len, expected_first) in [
        (InspectableUuidVersion::V1, 7, "time_low"),
        (InspectableUuidVersion::V2, 5, "payload_a"),
        (InspectableUuidVersion::V3, 5, "payload_a"),
        (InspectableUuidVersion::V4, 5, "payload_a"),
        (InspectableUuidVersion::V5, 5, "payload_a"),
        (InspectableUuidVersion::V6, 6, "timestamp_high"),
        (InspectableUuidVersion::V7, 5, "unix_timestamp_ms"),
        (InspectableUuidVersion::V8, 5, "custom_a"),
        (InspectableUuidVersion::Unknown(9), 5, "payload_a"),
    ] {
        let uuid = match version {
            InspectableUuidVersion::Unknown(number) => versioned_uuid(number),
            InspectableUuidVersion::V1
            | InspectableUuidVersion::V2
            | InspectableUuidVersion::V3
            | InspectableUuidVersion::V4
            | InspectableUuidVersion::V5
            | InspectableUuidVersion::V6
            | InspectableUuidVersion::V7
            | InspectableUuidVersion::V8 => versioned_uuid(version.number().unwrap()),
            InspectableUuidVersion::Nil | InspectableUuidVersion::Max => unreachable!(),
        };
        let inspection = inspect_uuid(&uuid);
        assert_eq!(inspection.fields.len(), expected_len);
        assert_eq!(inspection.fields[0].name, expected_first);
    }
}

#[test]
fn display_names_are_stable() {
    assert_eq!(UuidVariant::Ncs.to_string(), "NCS");
    assert_eq!(UuidVariant::Rfc9562.to_string(), "RFC 9562");
    assert_eq!(UuidVariant::Microsoft.to_string(), "Microsoft");
    assert_eq!(UuidVariant::Future.to_string(), "future");
    assert_eq!(
        NodeKind::Unicast.to_string(),
        "unicast / may be hardware-derived"
    );
    assert_eq!(
        NodeKind::Multicast.to_string(),
        "multicast / locally generated"
    );
    assert_eq!(NameHashAlgorithm::Md5.to_string(), "MD5");
    assert_eq!(NameHashAlgorithm::Sha1.to_string(), "SHA-1");
}

#[test]
fn legacy_versions_are_inspectable_without_generation_support() {
    for (number, expected, metadata) in [
        (1, InspectableUuidVersion::V1, "time"),
        (2, InspectableUuidVersion::V2, "dce"),
        (3, InspectableUuidVersion::V3, "md5"),
    ] {
        let inspection = inspect_uuid(&versioned_uuid(number));
        assert_eq!(inspection.version, expected);
        match (metadata, inspection.metadata) {
            ("time", UuidMetadata::Time { .. })
            | ("dce", UuidMetadata::DceSecurity)
            | (
                "md5",
                UuidMetadata::NameBased {
                    algorithm: NameHashAlgorithm::Md5,
                },
            ) => {}
            _ => panic!("unexpected metadata"),
        }
    }
}

#[test]
fn bit_layout_exposes_stable_offsets() {
    let fields = inspect_uuid(&generate(GeneratableUuidVersion::V7)).fields;
    assert_eq!(fields.len(), 5);
    assert_eq!(fields[0].name, "unix_timestamp_ms");
    assert_eq!((fields[0].offset, fields[0].width), (0, 48));
    assert_eq!(
        (
            fields[1].name.as_str(),
            fields[1].offset,
            fields[1].width,
            fields[1].value
        ),
        ("version", 48, 4, 7)
    );
    assert_eq!(
        (fields[2].name.as_str(), fields[2].offset, fields[2].width),
        ("rand_a", 52, 12)
    );
    assert_eq!(
        (
            fields[3].name.as_str(),
            fields[3].offset,
            fields[3].width,
            fields[3].value
        ),
        ("variant", 64, 2, 2)
    );
    assert_eq!(
        (fields[4].name.as_str(), fields[4].offset, fields[4].width),
        ("rand_b", 66, 62)
    );
}

#[cfg(feature = "ulid-inspect")]
#[test]
fn ulid_inspection_reports_timestamp_and_layout() {
    let inspection = uuidx_core::inspect_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAV").expect("valid ULID");
    assert_eq!(inspection.normalized, "01ARZ3NDEKTSV4RRFFQ69G5FAV");
    assert_eq!(inspection.bytes.len(), 16);
    assert_eq!(inspection.timestamp_ms, 1_469_922_850_259);
    assert_eq!(inspection.fields.len(), 2);
    assert_eq!(
        (
            inspection.fields[0].name.as_str(),
            inspection.fields[0].offset,
            inspection.fields[0].width,
        ),
        ("timestamp_ms", 0, 48)
    );
    assert_eq!(
        (
            inspection.fields[1].name.as_str(),
            inspection.fields[1].offset,
            inspection.fields[1].width,
        ),
        ("random", 48, 80)
    );
}

#[cfg(feature = "ulid-inspect")]
#[test]
fn ulid_parser_reports_empty_and_invalid_values() {
    assert!(matches!(
        uuidx_core::inspect_ulid(""),
        Err(uuidx_core::UlidParseError::Empty)
    ));
    assert!(matches!(
        uuidx_core::inspect_ulid("not-a-ulid"),
        Err(uuidx_core::UlidParseError::Invalid(_))
    ));
}

#[test]
fn nanoid_inspection_recognizes_the_standard_profile() {
    let inspection = uuidx_core::inspect_nanoid("  V1StGXR8_Z5jdHi6B-myT  ").unwrap();
    assert_eq!(inspection.normalized, "V1StGXR8_Z5jdHi6B-myT");
    assert_eq!(inspection.length, 21);
    assert_eq!(inspection.alphabet, "A-Za-z0-9_-");
    assert_eq!(inspection.entropy_bits, 126);
}

#[test]
fn nanoid_inspection_rejects_nonstandard_shapes() {
    assert_eq!(
        uuidx_core::inspect_nanoid(""),
        Err(uuidx_core::NanoidParseError::Empty)
    );
    for input in [
        "too-short",
        "V1StGXR8_Z5jdHi6B-my!",
        "V1StGXR8_Z5jdHi6B-myTT",
    ] {
        assert_eq!(
            uuidx_core::inspect_nanoid(input),
            Err(uuidx_core::NanoidParseError::Invalid)
        );
    }
}

#[test]
fn snowflake_inspection_decodes_the_twitter_layout() {
    let timestamp_ms = 1_700_000_000_000_u64;
    let value = ((timestamp_ms - uuidx_core::TWITTER_SNOWFLAKE_EPOCH_MS) << 22)
        | (17_u64 << 17)
        | (23_u64 << 12)
        | 3_210;
    let inspection = uuidx_core::inspect_snowflake(&value.to_string()).unwrap();

    assert_eq!(inspection.normalized, value.to_string());
    assert_eq!(inspection.value, value);
    assert_eq!(inspection.epoch_ms, 1_288_834_974_657);
    assert_eq!(inspection.timestamp_ms, timestamp_ms);
    assert_eq!(inspection.datacenter_id, 17);
    assert_eq!(inspection.worker_id, 23);
    assert_eq!(inspection.sequence, 3_210);
    assert_eq!(inspection.fields.len(), 5);
    assert_eq!(
        inspection
            .fields
            .iter()
            .map(|field| (field.name.as_str(), field.offset, field.width))
            .collect::<Vec<_>>(),
        vec![
            ("sign", 0, 1),
            ("timestamp_delta_ms", 1, 41),
            ("datacenter_id", 42, 5),
            ("worker_id", 47, 5),
            ("sequence", 52, 12),
        ]
    );
}

#[test]
fn snowflake_inspection_normalizes_and_rejects_invalid_values() {
    assert_eq!(
        uuidx_core::inspect_snowflake(" 42 ").unwrap().normalized,
        "42"
    );
    assert_eq!(
        uuidx_core::inspect_snowflake(""),
        Err(uuidx_core::SnowflakeParseError::Empty)
    );
    for input in ["-1", "+1", "1.5", "00042", "9223372036854775808"] {
        assert_eq!(
            uuidx_core::inspect_snowflake(input),
            Err(uuidx_core::SnowflakeParseError::Invalid)
        );
    }
}

#[test]
fn identifier_inspection_detects_each_supported_family() {
    assert!(matches!(
        uuidx_core::inspect_identifier("018f2c0b-6c5b-7d2e-8f4a-123456789abc"),
        Ok(uuidx_core::IdentifierInspection::Uuid(_))
    ));
    #[cfg(feature = "ulid-inspect")]
    assert!(matches!(
        uuidx_core::inspect_identifier("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        Ok(uuidx_core::IdentifierInspection::Ulid(_))
    ));
    assert!(matches!(
        uuidx_core::inspect_identifier("V1StGXR8_Z5jdHi6B-myT"),
        Ok(uuidx_core::IdentifierInspection::Nanoid(_))
    ));
    assert!(matches!(
        uuidx_core::inspect_identifier("1724552287438348288"),
        Ok(uuidx_core::IdentifierInspection::Snowflake(_))
    ));
    assert_eq!(
        uuidx_core::inspect_identifier("not-an-identifier"),
        Err(uuidx_core::IdentifierParseError)
    );
}

#[test]
fn identifier_family_parses_aliases_and_displays_names() {
    use uuidx_core::IdentifierFamily;

    assert_eq!(
        IdentifierFamily::from_str("UUID").unwrap(),
        IdentifierFamily::Uuid
    );
    assert_eq!(
        IdentifierFamily::from_str("nano-id").unwrap(),
        IdentifierFamily::Nanoid
    );
    assert_eq!(
        IdentifierFamily::from_str("snowflake").unwrap(),
        IdentifierFamily::Snowflake
    );
    assert_eq!(IdentifierFamily::Uuid.to_string(), "uuid");
    assert_eq!(IdentifierFamily::Nanoid.to_string(), "nanoid");
    assert!(IdentifierFamily::from_str("objectid").is_err());

    #[cfg(feature = "ulid-inspect")]
    {
        assert_eq!(
            IdentifierFamily::from_str("ulid").unwrap(),
            IdentifierFamily::Ulid
        );
        assert_eq!(IdentifierFamily::Ulid.to_string(), "ulid");
    }
}

#[test]
fn inspect_identifier_as_forces_the_requested_family() {
    use uuidx_core::{IdentifierFamily, IdentifierInspection, IdentifierTypeError};

    let uuid = "018f2c0b-6c5b-7d2e-8f4a-123456789abc";
    let nanoid = "V1StGXR8_Z5jdHi6B-myT";

    assert!(matches!(
        uuidx_core::inspect_identifier_as(uuid, IdentifierFamily::Uuid),
        Ok(IdentifierInspection::Uuid(_))
    ));
    assert!(matches!(
        uuidx_core::inspect_identifier_as(nanoid, IdentifierFamily::Nanoid),
        Ok(IdentifierInspection::Nanoid(_))
    ));
    assert!(matches!(
        uuidx_core::inspect_identifier_as("1724552287438348288", IdentifierFamily::Snowflake),
        Ok(IdentifierInspection::Snowflake(_))
    ));
    assert!(matches!(
        uuidx_core::inspect_identifier_as(nanoid, IdentifierFamily::Uuid),
        Err(IdentifierTypeError::Uuid(_))
    ));
    assert!(matches!(
        uuidx_core::inspect_identifier_as(uuid, IdentifierFamily::Nanoid),
        Err(IdentifierTypeError::Nanoid(_))
    ));

    #[cfg(feature = "ulid-inspect")]
    assert!(matches!(
        uuidx_core::inspect_identifier_as("01ARZ3NDEKTSV4RRFFQ69G5FAV", IdentifierFamily::Ulid),
        Ok(IdentifierInspection::Ulid(_))
    ));
}

#[test]
fn inspection_normalized_reflects_each_family() {
    use uuidx_core::{IdentifierFamily, inspect_identifier_as};

    assert_eq!(
        inspect_identifier_as(
            " 018F2C0B-6C5B-7D2E-8F4A-123456789ABC ",
            IdentifierFamily::Uuid
        )
        .unwrap()
        .normalized(),
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc"
    );
    assert_eq!(
        inspect_identifier_as("  V1StGXR8_Z5jdHi6B-myT ", IdentifierFamily::Nanoid)
            .unwrap()
            .normalized(),
        "V1StGXR8_Z5jdHi6B-myT"
    );
}

#[test]
fn inspection_kind_and_type_errors_cover_every_family() {
    use uuidx_core::{IdentifierFamily, IdentifierTypeError, inspect_identifier_as};

    assert_eq!(
        inspect_identifier_as(
            "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
            IdentifierFamily::Uuid
        )
        .unwrap()
        .kind(),
        "uuid"
    );
    assert_eq!(
        inspect_identifier_as("V1StGXR8_Z5jdHi6B-myT", IdentifierFamily::Nanoid)
            .unwrap()
            .kind(),
        "nanoid"
    );
    assert_eq!(
        inspect_identifier_as("1724552287438348288", IdentifierFamily::Snowflake)
            .unwrap()
            .kind(),
        "snowflake"
    );
    assert!(matches!(
        inspect_identifier_as("not-a-snowflake", IdentifierFamily::Snowflake),
        Err(IdentifierTypeError::Snowflake(_))
    ));

    #[cfg(feature = "ulid-inspect")]
    {
        assert_eq!(
            inspect_identifier_as("01ARZ3NDEKTSV4RRFFQ69G5FAV", IdentifierFamily::Ulid)
                .unwrap()
                .kind(),
            "ulid"
        );
        assert!(matches!(
            inspect_identifier_as("not-a-ulid", IdentifierFamily::Ulid),
            Err(IdentifierTypeError::Ulid(_))
        ));
    }
}
