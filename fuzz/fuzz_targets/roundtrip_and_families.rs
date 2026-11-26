#![no_main]

use libfuzzer_sys::fuzz_target;
use uuidx_core::{
    IdentifierFamily, Uuid, UuidOutputFormat, UuidTextCase, format_uuid_with_case,
    inspect_identifier_as, inspect_uuid, parse_hex_array, parse_namespace, parse_uuid,
};

fuzz_target!(|data: &[u8]| {
    if let Ok(bytes) = <[u8; 16]>::try_from(data) {
        let uuid = Uuid::from_bytes(bytes);
        let _ = inspect_uuid(&uuid);
        for format in [
            UuidOutputFormat::Canonical,
            UuidOutputFormat::Simple,
            UuidOutputFormat::Urn,
            UuidOutputFormat::Braced,
        ] {
            for case in [UuidTextCase::Lower, UuidTextCase::Upper] {
                let formatted = format_uuid_with_case(&uuid, format, case);
                assert_eq!(parse_uuid(&formatted), Ok(uuid));
            }
        }
    }

    if let Ok(input) = std::str::from_utf8(data) {
        let _ = parse_namespace(input);
        let _ = parse_hex_array::<6>(input);
        let _ = parse_hex_array::<16>(input);
        for family in [
            IdentifierFamily::Uuid,
            IdentifierFamily::Ulid,
            IdentifierFamily::Nanoid,
            IdentifierFamily::Snowflake,
        ] {
            let _ = inspect_identifier_as(input, family);
        }
    }
});
