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

use serde::{Serialize, de::DeserializeOwned};
use wasm_bindgen::prelude::*;

use crate::{
    error::ApiError,
    generation::GenerationInput,
    inspection::{IdentifierInspectInput, UuidInspectInput},
};

#[wasm_bindgen(js_name = generateUuid, skip_typescript)]
pub fn generate_uuid(version: &str, options: Option<JsValue>) -> Result<String, JsValue> {
    let options: GenerationInput = from_optional_js(options, "generation")?;
    generation::generate(version, options).map_err(ApiError::into_js)
}

#[wasm_bindgen(js_name = validateUuid, skip_typescript)]
pub fn validate_uuid(input: &str) -> bool {
    uuidx_core::parse_uuid(input).is_ok()
}

#[wasm_bindgen(js_name = validateIdentifier, skip_typescript)]
pub fn validate_identifier(input: &str, family: Option<String>) -> Result<bool, JsValue> {
    inspection::validate_identifier(input, family.as_deref()).map_err(ApiError::into_js)
}

#[wasm_bindgen(js_name = formatUuid, skip_typescript)]
pub fn format_uuid(
    input: &str,
    format: &str,
    text_case: Option<String>,
) -> Result<String, JsValue> {
    generation::format(input, format, text_case.as_deref()).map_err(ApiError::into_js)
}

#[wasm_bindgen(js_name = inspectUuid, skip_typescript)]
pub fn inspect_uuid(input: &str, options: Option<JsValue>) -> Result<JsValue, JsValue> {
    let options: UuidInspectInput = from_optional_js(options, "inspection")?;
    let inspection =
        inspection::inspect(input, options.redact_sensitive).map_err(ApiError::into_js)?;
    to_js_value(&inspection)
}

#[wasm_bindgen(js_name = inspectIdentifier, skip_typescript)]
pub fn inspect_identifier(input: &str, options: Option<JsValue>) -> Result<JsValue, JsValue> {
    let options: IdentifierInspectInput = from_optional_js(options, "inspection")?;
    let inspection = inspection::inspect_identifier(input, options).map_err(ApiError::into_js)?;
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

fn from_optional_js<T: Default + DeserializeOwned>(
    options: Option<JsValue>,
    context: &str,
) -> Result<T, JsValue> {
    match options {
        None => Ok(T::default()),
        Some(value) if value.is_null() || value.is_undefined() => Ok(T::default()),
        Some(value) => serde_wasm_bindgen::from_value(value).map_err(|error| {
            ApiError::new(
                "invalid_options",
                format!("invalid {context} options: {error}"),
            )
            .into_js()
        }),
    }
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
export type UuidTextCase = "lower" | "upper";
export type UuidVariant = "ncs" | "rfc9562" | "microsoft" | "future";

export interface GenerationOptions {
  namespace?: "dns" | "url" | "oid" | "x500" | string;
  name?: string;
  node?: "random" | string;
  timestampMs?: number;
  custom?: string;
  format?: UuidFormat;
  case?: UuidTextCase;
}

export interface UuidInspectOptions {
  redactSensitive?: boolean;
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
  fields: BitField[];
}

export function generateUuid(version: UuidVersion | string, options?: GenerationOptions): string;
export function validateUuid(input: string): boolean;
export function formatUuid(input: string, format: UuidFormat | string, textCase?: UuidTextCase | string): string;
export function inspectUuid(input: string, options?: UuidInspectOptions): UuidInspection;
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
  fields: BitField[];
}

export function inspectUlid(input: string): UlidInspection;
"#;

#[cfg(feature = "ulid-inspect")]
#[wasm_bindgen(typescript_custom_section)]
const IDENTIFIER_TYPES: &str = r#"
export type IdentifierFamily = "uuid" | "ulid" | "nanoid" | "snowflake";

export interface IdentifierInspectOptions {
  type?: IdentifierFamily | string;
  redactSensitive?: boolean;
}

export type IdentifierInspection =
  | UuidInspection
  | UlidInspection
  | NanoidInspection
  | SnowflakeInspection;

export function validateIdentifier(input: string, family?: IdentifierFamily | string): boolean;
export function inspectIdentifier(input: string, options?: IdentifierInspectOptions): IdentifierInspection;
"#;

#[cfg(not(feature = "ulid-inspect"))]
#[wasm_bindgen(typescript_custom_section)]
const IDENTIFIER_TYPES: &str = r#"
export type IdentifierFamily = "uuid" | "nanoid" | "snowflake";

export interface IdentifierInspectOptions {
  type?: IdentifierFamily | string;
  redactSensitive?: boolean;
}

export type IdentifierInspection =
  | UuidInspection
  | NanoidInspection
  | SnowflakeInspection;

export function validateIdentifier(input: string, family?: IdentifierFamily | string): boolean;
export function inspectIdentifier(input: string, options?: IdentifierInspectOptions): IdentifierInspection;
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_accepts_uuid_syntax_and_rejects_other_values() {
        assert!(validate_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc"));
        assert!(validate_uuid("018f2c0b6c5b7d2e8f4a123456789abc"));
        assert!(!validate_uuid("not-a-uuid"));
        assert!(validate_identifier("V1StGXR8_Z5jdHi6B-myT", None).unwrap());
        assert!(validate_identifier("V1StGXR8_Z5jdHi6B-myT", Some("nanoid".to_owned())).unwrap());
        assert!(!validate_identifier("V1StGXR8_Z5jdHi6B-myT", Some("uuid".to_owned())).unwrap());
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
            format_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc", "simple", None).unwrap(),
            "018f2c0b6c5b7d2e8f4a123456789abc"
        );
        assert_eq!(
            format_uuid(
                "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
                "simple",
                Some("upper".to_owned())
            )
            .unwrap(),
            "018F2C0B6C5B7D2E8F4A123456789ABC"
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
            assert_eq!(generate_uuid("v4", options.clone()).unwrap().len(), 36);
            assert_eq!(generate_uuid("v6", options.clone()).unwrap().len(), 36);
            assert_eq!(generate_uuid("v7", options).unwrap().len(), 36);
        }

        let options = Object::new();
        set(&options, "namespace", &JsValue::from_str("dns"));
        set(&options, "name", &JsValue::from_str("example.org"));
        set(&options, "format", &JsValue::from_str("simple"));
        set(&options, "case", &JsValue::from_str("upper"));
        assert_eq!(
            generate_uuid("v5", Some(options.into())).unwrap(),
            "AAD036818B63530489E08CA8F49461B5"
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
            format_uuid("not-a-uuid", "canonical", None).unwrap_err(),
            "invalid_uuid",
        );
        assert_error_code(
            format_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc", "compact", None).unwrap_err(),
            "unsupported_format",
        );
        assert_error_code(
            format_uuid(
                "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
                "canonical",
                Some("mixed".to_owned()),
            )
            .unwrap_err(),
            "unsupported_case",
        );
    }

    #[wasm_bindgen_test]
    fn uuid_inspection_serializes_to_a_plain_javascript_object() {
        let inspection = inspect_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc", None).unwrap();
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

        let options = Object::new();
        set(&options, "redactSensitive", &JsValue::TRUE);
        let redacted =
            inspect_uuid("11111111-1111-1111-9234-020000000001", Some(options.into())).unwrap();
        let metadata = Reflect::get(&redacted, &JsValue::from_str("metadata")).unwrap();
        assert!(
            Reflect::get(&metadata, &JsValue::from_str("nodeId"))
                .unwrap()
                .is_null()
        );

        assert_error_code(
            inspect_uuid("not-a-uuid", None).unwrap_err(),
            "invalid_uuid",
        );
        assert_error_code(
            inspect_uuid(
                "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
                Some(JsValue::from_str("bad")),
            )
            .unwrap_err(),
            "invalid_options",
        );
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
            let inspection = inspect_identifier(input, None).unwrap();
            assert_eq!(
                Reflect::get(&inspection, &JsValue::from_str("kind"))
                    .unwrap()
                    .as_string()
                    .as_deref(),
                Some(expected)
            );
        }

        let forced = Object::new();
        set(&forced, "type", &JsValue::from_str("uuid"));
        assert_error_code(
            inspect_identifier("V1StGXR8_Z5jdHi6B-myT", Some(forced.into())).unwrap_err(),
            "invalid_identifier",
        );
        assert_error_code(
            validate_identifier("V1StGXR8_Z5jdHi6B-myT", Some("objectid".to_owned())).unwrap_err(),
            "unsupported_identifier_type",
        );
        assert_error_code(
            inspect_identifier("not-an-identifier", None).unwrap_err(),
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
