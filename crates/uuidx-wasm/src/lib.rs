#![cfg_attr(target_env = "msvc", allow(linker_messages))]

//! WebAssembly bindings for uuidx UUID operations.
//!
//! The JavaScript API exposes UUID generation, validation, formatting, and
//! structured inspection without duplicating the domain rules in
//! `uuidx-core`. Generation accepts an optional plain object, inspection
//! returns plain objects and arrays, and failures throw JavaScript `Error`
//! instances with a machine-readable `code` property.
//!
//! Build this crate for `wasm32-unknown-unknown`, then run `wasm-bindgen` (or a
//! bundler that invokes it) to produce the JavaScript module and declarations.

mod error;
mod generation;
mod inspection;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::{error::ApiError, generation::GenerationInput};

#[wasm_bindgen(js_name = generateUuid, skip_typescript)]
pub fn generate_uuid(version: &str, options: Option<JsValue>) -> Result<String, JsValue> {
    let options = match options {
        None => GenerationInput::default(),
        Some(value) if value.is_null() || value.is_undefined() => GenerationInput::default(),
        Some(value) => serde_wasm_bindgen::from_value(value).map_err(|error| {
            ApiError::new(
                "invalid_options",
                format!("invalid generation options: {error}"),
            )
            .into_js()
        })?,
    };

    generation::generate(version, options).map_err(ApiError::into_js)
}

#[wasm_bindgen(js_name = validateUuid, skip_typescript)]
pub fn validate_uuid(input: &str) -> bool {
    uuidx_core::parse_uuid(input).is_ok()
}

#[wasm_bindgen(js_name = formatUuid, skip_typescript)]
pub fn format_uuid(input: &str, format: &str) -> Result<String, JsValue> {
    generation::format(input, format).map_err(ApiError::into_js)
}

#[wasm_bindgen(js_name = inspectUuid, skip_typescript)]
pub fn inspect_uuid(input: &str) -> Result<JsValue, JsValue> {
    let inspection = inspection::inspect(input).map_err(ApiError::into_js)?;
    to_js_value(&inspection)
}

#[wasm_bindgen(js_name = inspectIdentifier, skip_typescript)]
pub fn inspect_identifier(input: &str) -> Result<JsValue, JsValue> {
    let inspection = inspection::inspect_identifier(input).map_err(ApiError::into_js)?;
    to_js_value(&inspection)
}

#[cfg(feature = "ulid-inspect")]
#[wasm_bindgen(js_name = inspectUlid, skip_typescript)]
pub fn inspect_ulid(input: &str) -> Result<JsValue, JsValue> {
    let inspection = inspection::inspect_ulid(input).map_err(ApiError::into_js)?;
    to_js_value(&inspection)
}

#[wasm_bindgen(js_name = inspectNanoid, skip_typescript)]
pub fn inspect_nanoid(input: &str) -> Result<JsValue, JsValue> {
    let inspection = inspection::inspect_nanoid(input).map_err(ApiError::into_js)?;
    to_js_value(&inspection)
}

#[wasm_bindgen(js_name = inspectSnowflake, skip_typescript)]
pub fn inspect_snowflake(input: &str) -> Result<JsValue, JsValue> {
    let inspection = inspection::inspect_snowflake(input).map_err(ApiError::into_js)?;
    to_js_value(&inspection)
}

fn to_js_value(value: &impl Serialize) -> Result<JsValue, JsValue> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(|error| {
            ApiError::new(
                "serialization_failed",
                format!("failed to serialize result: {error}"),
            )
            .into_js()
        })
}

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_TYPES: &str = r#"
export type UuidVersion = "v3" | "v4" | "v5" | "v6" | "v7" | "v8";
export type UuidFormat = "canonical" | "simple" | "urn" | "braced";
export type UuidVariant = "ncs" | "rfc9562" | "microsoft" | "future";

export interface GenerationOptions {
  namespace?: "dns" | "url" | "oid" | "x500" | string;
  name?: string;
  node?: "random" | string;
  timestampMs?: number;
  custom?: string;
  format?: UuidFormat;
}

export interface BitField {
  name: string;
  offset: number;
  width: number;
  value: string;
}

export type UuidMetadata =
  | { type: "none" }
  | { type: "random" }
  | { type: "name_based"; algorithm: string }
  | {
      type: "time";
      unixSeconds: number;
      unixMillis: number;
      subsecNanos: number;
      clockSequence: number | null;
      nodeId: string | null;
      nodeKind: string | null;
    }
  | { type: "custom"; bytes: string }
  | { type: "dce_security" };

export interface UuidInspection {
  kind: "uuid";
  value: string;
  bytes: string;
  version: string;
  variant: UuidVariant;
  isNil: boolean;
  isMax: boolean;
  fields: BitField[];
  metadata: UuidMetadata;
}

export interface NanoidInspection {
  kind: "nanoid";
  value: string;
  length: number;
  alphabet: "A-Za-z0-9_-";
  entropyBits: number;
}

export interface SnowflakeInspection {
  kind: "snowflake";
  value: string;
  epoch: "twitter";
  epochMs: number;
  timestampMs: number;
  datacenterId: number;
  workerId: number;
  sequence: number;
}

export function generateUuid(version: UuidVersion | string, options?: GenerationOptions): string;
export function validateUuid(input: string): boolean;
export function formatUuid(input: string, format: UuidFormat | string): string;
export function inspectUuid(input: string): UuidInspection;
export function inspectNanoid(input: string): NanoidInspection;
export function inspectSnowflake(input: string): SnowflakeInspection;
"#;

#[cfg(feature = "ulid-inspect")]
#[wasm_bindgen(typescript_custom_section)]
const ULID_TYPESCRIPT_TYPES: &str = r#"
export interface UlidInspection {
  kind: "ulid";
  value: string;
  bytes: string;
  timestampMs: number;
  random: string;
}

export function inspectUlid(input: string): UlidInspection;
"#;

#[cfg(feature = "ulid-inspect")]
#[wasm_bindgen(typescript_custom_section)]
const IDENTIFIER_TYPES: &str = r#"
export type IdentifierInspection =
  | UuidInspection
  | UlidInspection
  | NanoidInspection
  | SnowflakeInspection;

export function inspectIdentifier(input: string): IdentifierInspection;
"#;

#[cfg(not(feature = "ulid-inspect"))]
#[wasm_bindgen(typescript_custom_section)]
const IDENTIFIER_TYPES: &str = r#"
export type IdentifierInspection =
  | UuidInspection
  | NanoidInspection
  | SnowflakeInspection;

export function inspectIdentifier(input: string): IdentifierInspection;
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_accepts_uuid_syntax_and_rejects_other_values() {
        assert!(validate_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc"));
        assert!(validate_uuid("018f2c0b6c5b7d2e8f4a123456789abc"));
        assert!(!validate_uuid("not-a-uuid"));
    }

    #[test]
    fn host_safe_public_wrappers_delegate_successfully() {
        let generated = generate_uuid("v4", None).unwrap();
        assert_eq!(
            uuidx_core::parse_uuid(&generated)
                .unwrap()
                .get_version_num(),
            4
        );

        assert_eq!(
            format_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc", "simple").unwrap(),
            "018f2c0b6c5b7d2e8f4a123456789abc"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use js_sys::{Error, Object, Reflect};
    use serde::{Serialize, Serializer};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    struct SerializationFailure;

    impl Serialize for SerializationFailure {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(serde::ser::Error::custom("intentional test failure"))
        }
    }

    fn set(object: &Object, key: &str, value: &JsValue) {
        Reflect::set(object, &JsValue::from_str(key), value).expect("property should be set");
    }

    fn assert_error_code(error: JsValue, expected: &str) {
        assert!(error.is_instance_of::<Error>());
        assert_eq!(
            Reflect::get(&error, &JsValue::from_str("code"))
                .unwrap()
                .as_string()
                .as_deref(),
            Some(expected)
        );
    }

    #[wasm_bindgen_test]
    fn generation_decodes_javascript_options_and_defaults() {
        for options in [None, Some(JsValue::NULL), Some(JsValue::UNDEFINED)] {
            assert_eq!(generate_uuid("v4", options).unwrap().len(), 36);
        }

        let options = Object::new();
        set(&options, "namespace", &JsValue::from_str("dns"));
        set(&options, "name", &JsValue::from_str("example.org"));
        set(&options, "format", &JsValue::from_str("simple"));
        assert_eq!(
            generate_uuid("v5", Some(options.into())).unwrap(),
            "aad036818b63530489e08ca8f49461b5"
        );
    }

    #[wasm_bindgen_test]
    fn generation_returns_structured_javascript_errors() {
        assert_error_code(
            generate_uuid("v4", Some(JsValue::from_str("not an object"))).unwrap_err(),
            "invalid_options",
        );
        assert_error_code(
            generate_uuid("v2", None).unwrap_err(),
            "unsupported_version",
        );

        let options = Object::new();
        set(&options, "name", &JsValue::from_str("unused"));
        assert_error_code(
            generate_uuid("v4", Some(options.into())).unwrap_err(),
            "generation_failed",
        );
    }

    #[wasm_bindgen_test]
    fn formatting_returns_structured_javascript_errors() {
        assert_error_code(
            format_uuid("not-a-uuid", "canonical").unwrap_err(),
            "invalid_uuid",
        );
        assert_error_code(
            format_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc", "compact").unwrap_err(),
            "unsupported_format",
        );
    }

    #[wasm_bindgen_test]
    fn uuid_inspection_serializes_to_a_plain_javascript_object() {
        let inspection = inspect_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc").unwrap();
        assert_eq!(
            Reflect::get(&inspection, &JsValue::from_str("kind"))
                .unwrap()
                .as_string()
                .as_deref(),
            Some("uuid")
        );
        assert_eq!(
            Reflect::get(&inspection, &JsValue::from_str("isNil"))
                .unwrap()
                .as_bool(),
            Some(false)
        );

        assert_error_code(inspect_uuid("not-a-uuid").unwrap_err(), "invalid_uuid");
    }

    #[cfg(feature = "ulid-inspect")]
    #[wasm_bindgen_test]
    fn ulid_inspection_serializes_and_returns_structured_errors() {
        let inspection = inspect_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
        assert_eq!(
            Reflect::get(&inspection, &JsValue::from_str("kind"))
                .unwrap()
                .as_string()
                .as_deref(),
            Some("ulid")
        );

        assert_error_code(inspect_ulid("not-a-ulid").unwrap_err(), "invalid_ulid");
    }

    #[wasm_bindgen_test]
    fn identifier_inspection_detects_nanoid_and_snowflake() {
        for (input, expected) in [
            ("V1StGXR8_Z5jdHi6B-myT", "nanoid"),
            ("1724552287438348288", "snowflake"),
        ] {
            let inspection = inspect_identifier(input).unwrap();
            assert_eq!(
                Reflect::get(&inspection, &JsValue::from_str("kind"))
                    .unwrap()
                    .as_string()
                    .as_deref(),
                Some(expected)
            );
        }

        assert_error_code(
            inspect_identifier("not-an-identifier").unwrap_err(),
            "invalid_identifier",
        );
        assert_error_code(inspect_nanoid("bad").unwrap_err(), "invalid_nanoid");
        assert_error_code(inspect_snowflake("bad").unwrap_err(), "invalid_snowflake");
    }

    #[wasm_bindgen_test]
    fn serialization_failures_return_the_public_error_shape() {
        assert_error_code(
            to_js_value(&SerializationFailure).unwrap_err(),
            "serialization_failed",
        );
    }
}
