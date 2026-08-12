use std::{
    ffi::{CStr, c_char},
    mem::MaybeUninit,
    ptr,
};

use uuidx_ffi::*;

const HEADER: &str = include_str!("../include/uuidx.h");

unsafe fn error_details(error: *mut UuidxError) -> (i32, String) {
    assert!(!error.is_null());
    // SAFETY: The test received this live error from the FFI crate.
    let code = unsafe { uuidx_error_code(error) };
    // SAFETY: The message is borrowed until the error is freed below.
    let message = unsafe { CStr::from_ptr(uuidx_error_message(error)) }
        .to_string_lossy()
        .into_owned();
    // SAFETY: The live error has not previously been released.
    unsafe { uuidx_error_free(error) };
    (code, message)
}

unsafe fn assert_error(error: *mut UuidxError, expected_code: i32, expected_message: &str) {
    // SAFETY: The caller passes an owned error returned by the FFI crate.
    let (code, message) = unsafe { error_details(error) };
    assert_eq!(code, expected_code);
    assert!(
        message.contains(expected_message),
        "expected `{message}` to contain `{expected_message}`"
    );
}

fn parse(value: &[u8]) -> UuidxUuid {
    let mut output = MaybeUninit::uninit();
    // SAFETY: Input and output pointers are valid for their declared sizes.
    let error = unsafe { uuidx_uuid_parse(value.as_ptr(), value.len(), output.as_mut_ptr()) };
    assert!(error.is_null());
    // SAFETY: Successful parsing initialized the output.
    unsafe { output.assume_init() }
}

fn inspect(uuid: &UuidxUuid) -> UuidxInspection {
    let mut output = MaybeUninit::uninit();
    // SAFETY: Input and output pointers reference live values.
    let error = unsafe { uuidx_uuid_inspect(uuid, output.as_mut_ptr()) };
    assert!(error.is_null());
    // SAFETY: Successful inspection initialized the output.
    unsafe { output.assume_init() }
}

fn generate(options: &UuidxGenerationOptions) -> UuidxUuid {
    let mut output = MaybeUninit::uninit();
    // SAFETY: The options and output pointers are live. Each caller keeps any
    // nested input referenced by the options alive for this call.
    let error = unsafe { uuidx_uuid_generate(options, output.as_mut_ptr()) };
    assert!(error.is_null());
    // SAFETY: Successful generation initialized the output.
    unsafe { output.assume_init() }
}

fn field_names(inspection: &UuidxInspection) -> Vec<u8> {
    inspection.fields[..usize::from(inspection.field_count)]
        .iter()
        .map(|field| field.name)
        .collect()
}

#[test]
fn parses_and_formats_with_caller_owned_storage() {
    let uuid = parse(b"018f2c0b-6c5b-7d2e-8f4a-123456789abc");
    let mut required = 0;
    // SAFETY: UUID and length pointers are valid; null output with zero capacity is a query.
    let error =
        unsafe { uuidx_uuid_format(&uuid, UUIDX_FORMAT_URN, ptr::null_mut(), 0, &mut required) };
    assert!(error.is_null());
    assert_eq!(required, 46);

    let mut output = vec![0 as c_char; required];
    // SAFETY: The output buffer has the queried capacity.
    let error = unsafe {
        uuidx_uuid_format(
            &uuid,
            UUIDX_FORMAT_URN,
            output.as_mut_ptr(),
            output.len(),
            &mut required,
        )
    };
    assert!(error.is_null());
    // SAFETY: Successful formatting wrote a NUL-terminated string.
    assert_eq!(
        unsafe { CStr::from_ptr(output.as_ptr()) }.to_bytes(),
        b"urn:uuid:018f2c0b-6c5b-7d2e-8f4a-123456789abc"
    );
}

#[test]
fn reports_owned_errors_and_required_buffer_size() {
    let mut output = MaybeUninit::uninit();
    // SAFETY: Input and output pointers are valid for their declared sizes.
    let error = unsafe { uuidx_uuid_parse([0xff].as_ptr(), 1, output.as_mut_ptr()) };
    // SAFETY: The call returned one owned error.
    let (code, message) = unsafe { error_details(error) };
    assert_eq!(code, UUIDX_ERROR_INVALID_UTF8);
    assert!(message.contains("UTF-8"));

    let uuid = parse(b"018f2c0b-6c5b-7d2e-8f4a-123456789abc");
    let mut buffer = [0_i8; 4];
    let mut required = 0;
    // SAFETY: All supplied pointers are valid for their declared sizes.
    let error = unsafe {
        uuidx_uuid_format(
            &uuid,
            UUIDX_FORMAT_CANONICAL,
            buffer.as_mut_ptr(),
            buffer.len(),
            &mut required,
        )
    };
    // SAFETY: The call returned one owned error.
    let (code, _) = unsafe { error_details(error) };
    assert_eq!(code, UUIDX_ERROR_BUFFER_TOO_SMALL);
    assert_eq!(required, 37);
}

#[test]
fn error_objects_handle_embedded_nuls_and_null_accessors() {
    let mut output = MaybeUninit::uninit();
    // SAFETY: Input and output pointers are valid for their declared sizes.
    let error = unsafe { uuidx_uuid_parse(b"not\0uuid".as_ptr(), 8, output.as_mut_ptr()) };
    // SAFETY: The call returned one owned error.
    let (code, message) = unsafe { error_details(error) };
    assert_eq!(code, UUIDX_ERROR_INVALID_UUID);
    assert!(!message.is_empty());

    // SAFETY: All three error APIs explicitly accept null pointers.
    unsafe {
        assert_eq!(uuidx_error_code(ptr::null()), UUIDX_ERROR_NULL_POINTER);
        assert!(CStr::from_ptr(uuidx_error_message(ptr::null())).is_empty());
        uuidx_error_free(ptr::null_mut());
    }
}

#[test]
fn generates_name_based_random_and_custom_uuids() {
    let mut dns = MaybeUninit::uninit();
    // SAFETY: The output pointer references writable UUID storage.
    let error = unsafe { uuidx_uuid_namespace(UUIDX_NAMESPACE_DNS, dns.as_mut_ptr()) };
    assert!(error.is_null());
    // SAFETY: Successful namespace lookup initialized the output.
    let dns = unsafe { dns.assume_init() };

    let name = b"example.org";
    let mut options = UuidxGenerationOptions {
        version: 3,
        namespace_uuid: &dns,
        name: name.as_ptr(),
        name_length: name.len(),
        ..UuidxGenerationOptions::default()
    };
    let mut output = MaybeUninit::uninit();
    // SAFETY: The options and all nested pointers are live, and output is writable.
    let error = unsafe { uuidx_uuid_generate(&options, output.as_mut_ptr()) };
    assert!(error.is_null());
    // SAFETY: Successful generation initialized the output.
    let v3 = unsafe { output.assume_init() };
    assert_eq!(v3, parse(b"04738bdf-b25a-3829-a801-b21a1d25095b"));

    options = UuidxGenerationOptions {
        version: 4,
        ..UuidxGenerationOptions::default()
    };
    // SAFETY: The options pointer is live, and output is writable.
    let error = unsafe { uuidx_uuid_generate(&options, output.as_mut_ptr()) };
    assert!(error.is_null());
    // SAFETY: Successful generation initialized the output.
    assert_eq!(unsafe { output.assume_init() }.bytes[6] >> 4, 4);

    let custom = [0xab; 16];
    options = UuidxGenerationOptions {
        version: 8,
        custom: custom.as_ptr(),
        custom_length: custom.len(),
        ..UuidxGenerationOptions::default()
    };
    // SAFETY: The options and custom pointers are live, and output is writable.
    let error = unsafe { uuidx_uuid_generate(&options, output.as_mut_ptr()) };
    assert!(error.is_null());
    // SAFETY: Successful generation initialized the output.
    let v8 = unsafe { output.assume_init() };
    assert_eq!(v8.bytes[0], 0xab);
    assert_eq!(v8.bytes[6] >> 4, 8);
}

#[test]
fn inspection_exposes_metadata_and_full_width_fields() {
    let uuid = parse(b"018f2c0b-6c5b-7d2e-8f4a-123456789abc");
    let mut inspection = MaybeUninit::uninit();
    // SAFETY: Input and output pointers reference live values.
    let error = unsafe { uuidx_uuid_inspect(&uuid, inspection.as_mut_ptr()) };
    assert!(error.is_null());
    // SAFETY: Successful inspection initialized the output.
    let inspection = unsafe { inspection.assume_init() };
    assert_eq!(inspection.version, 7);
    assert_eq!(inspection.variant, UUIDX_VARIANT_RFC9562);
    assert_eq!(inspection.metadata_kind, UUIDX_METADATA_TIME);
    assert_eq!(inspection.field_count, 5);
    assert_eq!(inspection.fields[0].name, UUIDX_FIELD_UNIX_TIMESTAMP_MS);
    assert_eq!(inspection.fields[0].width, 48);

    let nil = parse(b"00000000-0000-0000-0000-000000000000");
    let mut nil_inspection = MaybeUninit::uninit();
    // SAFETY: Input and output pointers reference live values.
    let error = unsafe { uuidx_uuid_inspect(&nil, nil_inspection.as_mut_ptr()) };
    assert!(error.is_null());
    // SAFETY: Successful inspection initialized the output.
    let nil_inspection = unsafe { nil_inspection.assume_init() };
    assert_eq!(nil_inspection.is_nil, 1);
    assert_eq!(nil_inspection.fields[0].name, UUIDX_FIELD_VALUE);
    assert_eq!(nil_inspection.fields[0].width, 128);
    assert_eq!(nil_inspection.fields[0].value_high, 0);
    assert_eq!(nil_inspection.fields[0].value_low, 0);
}

#[test]
fn validates_generation_shapes_before_calling_core() {
    let node = [1_u8; 5];
    let options = UuidxGenerationOptions {
        version: 6,
        node: node.as_ptr(),
        node_length: node.len(),
        ..UuidxGenerationOptions::default()
    };
    let mut output = MaybeUninit::uninit();
    // SAFETY: The options and nested input are live, and output is writable.
    let error = unsafe { uuidx_uuid_generate(&options, output.as_mut_ptr()) };
    // SAFETY: The call returned one owned error.
    let (code, message) = unsafe { error_details(error) };
    assert_eq!(code, UUIDX_ERROR_INVALID_ARGUMENT);
    assert!(message.contains("exactly 6 bytes"));

    let options = UuidxGenerationOptions {
        version: 2,
        ..UuidxGenerationOptions::default()
    };
    // SAFETY: The options pointer is live, and output is writable.
    let error = unsafe { uuidx_uuid_generate(&options, output.as_mut_ptr()) };
    // SAFETY: The call returned one owned error.
    let (code, _) = unsafe { error_details(error) };
    assert_eq!(code, UUIDX_ERROR_INVALID_VERSION);
}

#[test]
fn parse_validates_every_pointer_and_input_shape() {
    let mut output = MaybeUninit::uninit();

    // SAFETY: A null input is intentionally supplied to test the documented contract.
    let error = unsafe { uuidx_uuid_parse(ptr::null(), 0, output.as_mut_ptr()) };
    // SAFETY: The call returned one owned error.
    unsafe { assert_error(error, UUIDX_ERROR_NULL_POINTER, "input must not be null") };

    // SAFETY: The input is valid; a null output is intentionally supplied.
    let error = unsafe { uuidx_uuid_parse(b"nil".as_ptr(), 3, ptr::null_mut()) };
    // SAFETY: The call returned one owned error.
    unsafe { assert_error(error, UUIDX_ERROR_NULL_POINTER, "output must not be null") };

    // SAFETY: The implementation rejects an unrepresentable length before forming a slice.
    let error = unsafe {
        uuidx_uuid_parse(
            b"x".as_ptr(),
            (isize::MAX as usize).saturating_add(1),
            output.as_mut_ptr(),
        )
    };
    // SAFETY: The call returned one owned error.
    unsafe {
        assert_error(
            error,
            UUIDX_ERROR_INVALID_ARGUMENT,
            "length exceeds the maximum supported object size",
        )
    };

    // SAFETY: Input and output pointers are valid for their declared sizes.
    let error = unsafe { uuidx_uuid_parse(b"not-a-uuid".as_ptr(), 10, output.as_mut_ptr()) };
    // SAFETY: The call returned one owned error.
    unsafe { assert_error(error, UUIDX_ERROR_INVALID_UUID, "invalid UUID") };
}

#[test]
fn format_supports_every_mode_and_validates_buffer_contracts() {
    let uuid = parse(b"018f2c0b-6c5b-7d2e-8f4a-123456789abc");
    for (format, expected) in [
        (
            UUIDX_FORMAT_CANONICAL,
            "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        ),
        (UUIDX_FORMAT_SIMPLE, "018f2c0b6c5b7d2e8f4a123456789abc"),
        (
            UUIDX_FORMAT_URN,
            "urn:uuid:018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        ),
        (
            UUIDX_FORMAT_BRACED,
            "{018f2c0b-6c5b-7d2e-8f4a-123456789abc}",
        ),
    ] {
        let mut output = [0 as c_char; 48];
        let mut required = 0;
        // SAFETY: All pointers are valid, and the buffer is large enough for every format.
        let error = unsafe {
            uuidx_uuid_format(
                &uuid,
                format,
                output.as_mut_ptr(),
                output.len(),
                &mut required,
            )
        };
        assert!(error.is_null());
        assert_eq!(required, expected.len() + 1);
        // SAFETY: Successful formatting wrote a NUL-terminated string.
        assert_eq!(
            unsafe { CStr::from_ptr(output.as_ptr()) }.to_str().unwrap(),
            expected
        );
    }

    let mut required = 123;
    // SAFETY: A null UUID is intentionally supplied; the length pointer is valid.
    let error = unsafe {
        uuidx_uuid_format(
            ptr::null(),
            UUIDX_FORMAT_CANONICAL,
            ptr::null_mut(),
            0,
            &mut required,
        )
    };
    // SAFETY: The call returned one owned error.
    unsafe { assert_error(error, UUIDX_ERROR_NULL_POINTER, "uuid must not be null") };
    assert_eq!(required, 123);

    // SAFETY: The UUID is live; a null output-length pointer is intentionally supplied.
    let error = unsafe {
        uuidx_uuid_format(
            &uuid,
            UUIDX_FORMAT_CANONICAL,
            ptr::null_mut(),
            0,
            ptr::null_mut(),
        )
    };
    // SAFETY: The call returned one owned error.
    unsafe {
        assert_error(
            error,
            UUIDX_ERROR_NULL_POINTER,
            "output_length must not be null",
        )
    };

    // SAFETY: The UUID and output-length pointers are valid; the format is intentionally invalid.
    let error = unsafe { uuidx_uuid_format(&uuid, -1, ptr::null_mut(), 0, &mut required) };
    // SAFETY: The call returned one owned error.
    unsafe {
        assert_error(
            error,
            UUIDX_ERROR_INVALID_FORMAT,
            "unsupported UUID format -1",
        )
    };

    // SAFETY: A null buffer with nonzero capacity is intentionally supplied.
    let error = unsafe {
        uuidx_uuid_format(
            &uuid,
            UUIDX_FORMAT_CANONICAL,
            ptr::null_mut(),
            37,
            &mut required,
        )
    };
    // SAFETY: The call returned one owned error.
    unsafe {
        assert_error(
            error,
            UUIDX_ERROR_NULL_POINTER,
            "output_capacity is nonzero",
        )
    };
    assert_eq!(required, 37);

    let mut byte = 0 as c_char;
    // SAFETY: The implementation rejects the impossible object size before accessing the buffer.
    let error = unsafe {
        uuidx_uuid_format(
            &uuid,
            UUIDX_FORMAT_CANONICAL,
            &mut byte,
            (isize::MAX as usize).saturating_add(1),
            &mut required,
        )
    };
    // SAFETY: The call returned one owned error.
    unsafe {
        assert_error(
            error,
            UUIDX_ERROR_INVALID_ARGUMENT,
            "output_capacity exceeds the maximum supported object size",
        )
    };
}

#[test]
fn returns_all_namespaces_and_rejects_invalid_requests() {
    for (kind, expected) in [
        (
            UUIDX_NAMESPACE_DNS,
            b"6ba7b810-9dad-11d1-80b4-00c04fd430c8".as_slice(),
        ),
        (
            UUIDX_NAMESPACE_URL,
            b"6ba7b811-9dad-11d1-80b4-00c04fd430c8".as_slice(),
        ),
        (
            UUIDX_NAMESPACE_OID,
            b"6ba7b812-9dad-11d1-80b4-00c04fd430c8".as_slice(),
        ),
        (
            UUIDX_NAMESPACE_X500,
            b"6ba7b814-9dad-11d1-80b4-00c04fd430c8".as_slice(),
        ),
    ] {
        let mut output = MaybeUninit::uninit();
        // SAFETY: The output pointer references writable UUID storage.
        let error = unsafe { uuidx_uuid_namespace(kind, output.as_mut_ptr()) };
        assert!(error.is_null());
        // SAFETY: Successful lookup initialized the output.
        assert_eq!(unsafe { output.assume_init() }, parse(expected));
    }

    // SAFETY: A null output is intentionally supplied.
    let error = unsafe { uuidx_uuid_namespace(UUIDX_NAMESPACE_DNS, ptr::null_mut()) };
    // SAFETY: The call returned one owned error.
    unsafe { assert_error(error, UUIDX_ERROR_NULL_POINTER, "output must not be null") };

    let mut output = MaybeUninit::uninit();
    // SAFETY: The output pointer is valid; the namespace kind is intentionally invalid.
    let error = unsafe { uuidx_uuid_namespace(0, output.as_mut_ptr()) };
    // SAFETY: The call returned one owned error.
    unsafe {
        assert_error(
            error,
            UUIDX_ERROR_INVALID_ARGUMENT,
            "unsupported namespace kind 0",
        )
    };
}

#[test]
fn generation_supports_every_version_and_present_empty_names() {
    let dns = parse(b"6ba7b810-9dad-11d1-80b4-00c04fd430c8");
    let empty = [];
    let v3 = generate(&UuidxGenerationOptions {
        version: 3,
        namespace_uuid: &dns,
        name: empty.as_ptr(),
        name_length: 0,
        ..UuidxGenerationOptions::default()
    });
    assert_eq!(inspect(&v3).version, 3);

    let v4 = generate(&UuidxGenerationOptions {
        version: 4,
        ..UuidxGenerationOptions::default()
    });
    assert_eq!(inspect(&v4).version, 4);

    let name = b"example.org";
    let v5 = generate(&UuidxGenerationOptions {
        version: 5,
        namespace_uuid: &dns,
        name: name.as_ptr(),
        name_length: name.len(),
        ..UuidxGenerationOptions::default()
    });
    assert_eq!(v5, parse(b"aad03681-8b63-5304-89e0-8ca8f49461b5"));

    let node = [0x02, 0, 0, 0, 0, 1];
    let v6 = generate(&UuidxGenerationOptions {
        version: 6,
        node: node.as_ptr(),
        node_length: node.len(),
        has_timestamp: 1,
        timestamp_seconds: 1_700_000_000,
        timestamp_nanoseconds: 123_456_789,
        ..UuidxGenerationOptions::default()
    });
    assert_eq!(inspect(&v6).version, 6);

    let v7 = generate(&UuidxGenerationOptions {
        version: 7,
        has_timestamp: 1,
        timestamp_seconds: 1_700_000_000,
        timestamp_nanoseconds: 123_000_000,
        ..UuidxGenerationOptions::default()
    });
    assert_eq!(inspect(&v7).unix_millis, 1_700_000_000_123);

    let custom = [0xab; 16];
    let v8 = generate(&UuidxGenerationOptions {
        version: 8,
        custom: custom.as_ptr(),
        custom_length: custom.len(),
        ..UuidxGenerationOptions::default()
    });
    assert_eq!(inspect(&v8).version, 8);
}

#[test]
fn generation_validates_pointer_length_and_timestamp_contracts() {
    let mut output = MaybeUninit::uninit();
    let defaults = UuidxGenerationOptions::default();

    // SAFETY: Null top-level pointers are intentionally supplied one at a time.
    let error = unsafe { uuidx_uuid_generate(ptr::null(), output.as_mut_ptr()) };
    // SAFETY: The call returned one owned error.
    unsafe { assert_error(error, UUIDX_ERROR_NULL_POINTER, "options must not be null") };
    // SAFETY: The options pointer is valid; a null output is intentional.
    let error = unsafe { uuidx_uuid_generate(&defaults, ptr::null_mut()) };
    // SAFETY: The call returned one owned error.
    unsafe { assert_error(error, UUIDX_ERROR_NULL_POINTER, "output must not be null") };

    for (options, expected) in [
        (
            UuidxGenerationOptions {
                version: 4,
                name_length: 1,
                ..UuidxGenerationOptions::default()
            },
            "name is null but its length is nonzero",
        ),
        (
            UuidxGenerationOptions {
                version: 4,
                node_length: 1,
                ..UuidxGenerationOptions::default()
            },
            "node is null but its length is nonzero",
        ),
        (
            UuidxGenerationOptions {
                version: 4,
                custom_length: 1,
                ..UuidxGenerationOptions::default()
            },
            "custom is null but its length is nonzero",
        ),
    ] {
        // SAFETY: The top-level pointers are valid; a nested null/length mismatch is intentional.
        let error = unsafe { uuidx_uuid_generate(&options, output.as_mut_ptr()) };
        // SAFETY: The call returned one owned error.
        unsafe { assert_error(error, UUIDX_ERROR_NULL_POINTER, expected) };
    }

    let byte = 0_u8;
    let options = UuidxGenerationOptions {
        version: 4,
        name: &byte,
        name_length: (isize::MAX as usize).saturating_add(1),
        ..UuidxGenerationOptions::default()
    };
    // SAFETY: The impossible length is rejected before the nested pointer is read.
    let error = unsafe { uuidx_uuid_generate(&options, output.as_mut_ptr()) };
    // SAFETY: The call returned one owned error.
    unsafe { assert_error(error, UUIDX_ERROR_INVALID_ARGUMENT, "name length exceeds") };

    let custom = [0_u8; 15];
    let options = UuidxGenerationOptions {
        version: 8,
        custom: custom.as_ptr(),
        custom_length: custom.len(),
        ..UuidxGenerationOptions::default()
    };
    // SAFETY: All supplied pointers are live for their declared lengths.
    let error = unsafe { uuidx_uuid_generate(&options, output.as_mut_ptr()) };
    // SAFETY: The call returned one owned error.
    unsafe { assert_error(error, UUIDX_ERROR_INVALID_ARGUMENT, "exactly 16 bytes") };

    for (options, expected) in [
        (
            UuidxGenerationOptions {
                version: 7,
                has_timestamp: 2,
                ..UuidxGenerationOptions::default()
            },
            "has_timestamp must be 0 or 1",
        ),
        (
            UuidxGenerationOptions {
                version: 7,
                has_timestamp: 1,
                timestamp_nanoseconds: 1_000_000_000,
                ..UuidxGenerationOptions::default()
            },
            "timestamp_nanoseconds must be less",
        ),
        (
            UuidxGenerationOptions {
                version: 7,
                has_timestamp: 1,
                timestamp_seconds: u64::MAX,
                ..UuidxGenerationOptions::default()
            },
            "timestamp is outside the supported range",
        ),
    ] {
        // SAFETY: The pointers are valid; each timestamp shape is intentionally invalid.
        let error = unsafe { uuidx_uuid_generate(&options, output.as_mut_ptr()) };
        // SAFETY: The call returned one owned error.
        unsafe { assert_error(error, UUIDX_ERROR_INVALID_ARGUMENT, expected) };
    }
}

#[test]
fn generation_translates_core_option_errors() {
    let mut output = MaybeUninit::uninit();
    let options = UuidxGenerationOptions {
        version: 3,
        ..UuidxGenerationOptions::default()
    };
    // SAFETY: Options and output pointers are valid.
    let error = unsafe { uuidx_uuid_generate(&options, output.as_mut_ptr()) };
    // SAFETY: The call returned one owned error.
    unsafe {
        assert_error(
            error,
            UUIDX_ERROR_GENERATION,
            "requires --namespace and --name",
        )
    };

    let empty = [];
    let options = UuidxGenerationOptions {
        version: 4,
        name: empty.as_ptr(),
        name_length: 0,
        ..UuidxGenerationOptions::default()
    };
    // SAFETY: The present empty name pointer and output pointer are valid.
    let error = unsafe { uuidx_uuid_generate(&options, output.as_mut_ptr()) };
    // SAFETY: The call returned one owned error.
    unsafe { assert_error(error, UUIDX_ERROR_GENERATION, "--name is not valid") };
}

#[test]
fn inspection_validates_pointers_and_maps_every_family() {
    let uuid = parse(b"018f2c0b-6c5b-7d2e-8f4a-123456789abc");
    let mut output = MaybeUninit::uninit();
    // SAFETY: Null input and output pointers are intentionally supplied one at a time.
    let error = unsafe { uuidx_uuid_inspect(ptr::null(), output.as_mut_ptr()) };
    // SAFETY: The call returned one owned error.
    unsafe { assert_error(error, UUIDX_ERROR_NULL_POINTER, "uuid must not be null") };
    // SAFETY: The UUID pointer is valid; a null output is intentional.
    let error = unsafe { uuidx_uuid_inspect(&uuid, ptr::null_mut()) };
    // SAFETY: The call returned one owned error.
    unsafe { assert_error(error, UUIDX_ERROR_NULL_POINTER, "output must not be null") };

    let cases = [
        (
            "00000000-0000-1000-8000-020000000001",
            1,
            UUIDX_METADATA_TIME,
            UUIDX_FIELD_TIME_LOW,
        ),
        (
            "00000000-0000-2000-8000-000000000000",
            2,
            UUIDX_METADATA_DCE_SECURITY,
            UUIDX_FIELD_PAYLOAD_A,
        ),
        (
            "04738bdf-b25a-3829-a801-b21a1d25095b",
            3,
            UUIDX_METADATA_NAME_BASED,
            UUIDX_FIELD_PAYLOAD_A,
        ),
        (
            "00000000-0000-4000-8000-000000000000",
            4,
            UUIDX_METADATA_RANDOM,
            UUIDX_FIELD_PAYLOAD_A,
        ),
        (
            "aad03681-8b63-5304-89e0-8ca8f49461b5",
            5,
            UUIDX_METADATA_NAME_BASED,
            UUIDX_FIELD_PAYLOAD_A,
        ),
        (
            "00000000-0000-6000-8000-030000000001",
            6,
            UUIDX_METADATA_TIME,
            UUIDX_FIELD_TIMESTAMP_HIGH,
        ),
        (
            "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
            7,
            UUIDX_METADATA_TIME,
            UUIDX_FIELD_UNIX_TIMESTAMP_MS,
        ),
        (
            "abababab-abab-8bab-abab-abababababab",
            8,
            UUIDX_METADATA_CUSTOM,
            UUIDX_FIELD_CUSTOM_A,
        ),
        (
            "00000000-0000-9000-8000-000000000000",
            9,
            UUIDX_METADATA_NONE,
            UUIDX_FIELD_PAYLOAD_A,
        ),
    ];
    for (text, version, metadata, first_field) in cases {
        let inspection = inspect(&parse(text.as_bytes()));
        assert_eq!(inspection.version, version);
        assert_eq!(inspection.metadata_kind, metadata);
        assert_eq!(inspection.fields[0].name, first_field);
    }

    let v3 = inspect(&parse(b"04738bdf-b25a-3829-a801-b21a1d25095b"));
    assert_eq!(v3.hash_algorithm, UUIDX_HASH_MD5);
    let v5 = inspect(&parse(b"aad03681-8b63-5304-89e0-8ca8f49461b5"));
    assert_eq!(v5.hash_algorithm, UUIDX_HASH_SHA1);
    let v8 = inspect(&parse(b"abababab-abab-8bab-abab-abababababab"));
    assert_eq!(
        v8.custom,
        parse(b"abababab-abab-8bab-abab-abababababab").bytes
    );

    let unicast = inspect(&parse(b"00000000-0000-1000-8000-020000000001"));
    assert_eq!(unicast.has_timestamp, 1);
    assert_eq!(unicast.has_clock_sequence, 1);
    assert_eq!(unicast.has_node, 1);
    assert_eq!(unicast.node_id, [0x02, 0, 0, 0, 0, 1]);
    assert_eq!(unicast.node_kind, UUIDX_NODE_UNICAST);
    let multicast = inspect(&parse(b"00000000-0000-1000-8000-030000000001"));
    assert_eq!(multicast.node_kind, UUIDX_NODE_MULTICAST);

    assert_eq!(
        field_names(&unicast),
        [
            UUIDX_FIELD_TIME_LOW,
            UUIDX_FIELD_TIME_MID,
            UUIDX_FIELD_VERSION,
            UUIDX_FIELD_TIME_HI,
            UUIDX_FIELD_VARIANT,
            UUIDX_FIELD_CLOCK_SEQUENCE,
            UUIDX_FIELD_NODE,
        ]
    );
    assert_eq!(
        field_names(&inspect(&parse(b"00000000-0000-6000-8000-000000000000"))),
        [
            UUIDX_FIELD_TIMESTAMP_HIGH,
            UUIDX_FIELD_VERSION,
            UUIDX_FIELD_TIMESTAMP_LOW,
            UUIDX_FIELD_VARIANT,
            UUIDX_FIELD_CLOCK_SEQUENCE,
            UUIDX_FIELD_NODE,
        ]
    );
    assert_eq!(
        field_names(&inspect(&parse(b"018f2c0b-6c5b-7d2e-8f4a-123456789abc"))),
        [
            UUIDX_FIELD_UNIX_TIMESTAMP_MS,
            UUIDX_FIELD_VERSION,
            UUIDX_FIELD_RAND_A,
            UUIDX_FIELD_VARIANT,
            UUIDX_FIELD_RAND_B,
        ]
    );
    assert_eq!(
        field_names(&v8),
        [
            UUIDX_FIELD_CUSTOM_A,
            UUIDX_FIELD_VERSION,
            UUIDX_FIELD_CUSTOM_B,
            UUIDX_FIELD_VARIANT,
            UUIDX_FIELD_CUSTOM_C,
        ]
    );
    assert_eq!(
        field_names(&v3),
        [
            UUIDX_FIELD_PAYLOAD_A,
            UUIDX_FIELD_VERSION,
            UUIDX_FIELD_PAYLOAD_B,
            UUIDX_FIELD_VARIANT,
            UUIDX_FIELD_PAYLOAD_C,
        ]
    );
}

#[test]
fn inspection_maps_special_values_and_all_variants() {
    let nil = inspect(&parse(b"00000000-0000-0000-0000-000000000000"));
    assert_eq!(nil.version, 0);
    assert_eq!(nil.is_nil, 1);
    assert_eq!(nil.metadata_kind, UUIDX_METADATA_NONE);
    let max = inspect(&parse(b"ffffffff-ffff-ffff-ffff-ffffffffffff"));
    assert_eq!(max.version, 15);
    assert_eq!(max.is_max, 1);
    assert_eq!(max.variant, UUIDX_VARIANT_FUTURE);
    assert_eq!(max.fields[0].value_high, u64::MAX);
    assert_eq!(max.fields[0].value_low, u64::MAX);

    for (text, expected) in [
        (
            b"00000000-0000-4000-0000-000000000000".as_slice(),
            UUIDX_VARIANT_NCS,
        ),
        (
            b"00000000-0000-4000-8000-000000000000".as_slice(),
            UUIDX_VARIANT_RFC9562,
        ),
        (
            b"00000000-0000-4000-c000-000000000000".as_slice(),
            UUIDX_VARIANT_MICROSOFT,
        ),
        (
            b"00000000-0000-4000-e000-000000000000".as_slice(),
            UUIDX_VARIANT_FUTURE,
        ),
    ] {
        assert_eq!(inspect(&parse(text)).variant, expected);
    }
}

#[test]
fn ffi_struct_layouts_are_stable_for_the_c_abi() {
    use std::mem::{align_of, offset_of, size_of};

    assert_eq!(size_of::<UuidxUuid>(), 16);
    assert_eq!(align_of::<UuidxUuid>(), 1);
    assert_eq!(offset_of!(UuidxGenerationOptions, version), 0);
    assert!(offset_of!(UuidxGenerationOptions, namespace_uuid) >= size_of::<i32>());
    assert_eq!(offset_of!(UuidxBitField, name), 0);
    assert_eq!(size_of::<UuidxBitField>(), 24);
    assert_eq!(offset_of!(UuidxInspection, version), 0);
    assert!(offset_of!(UuidxInspection, fields) > offset_of!(UuidxInspection, custom));

    let defaults = UuidxGenerationOptions::default();
    assert_eq!(defaults.version, 0);
    assert!(defaults.namespace_uuid.is_null());
    assert!(defaults.name.is_null());
    assert!(defaults.node.is_null());
    assert!(defaults.custom.is_null());
    assert_eq!(defaults.has_timestamp, 0);
}

#[test]
fn checked_in_header_matches_the_exported_contract() {
    for (name, value) in [
        ("UUIDX_ABI_VERSION", "1u"),
        ("UUIDX_ERROR_OK", "0"),
        ("UUIDX_ERROR_NULL_POINTER", "1"),
        ("UUIDX_ERROR_INVALID_UTF8", "2"),
        ("UUIDX_ERROR_INVALID_UUID", "3"),
        ("UUIDX_ERROR_INVALID_VERSION", "4"),
        ("UUIDX_ERROR_INVALID_FORMAT", "5"),
        ("UUIDX_ERROR_INVALID_ARGUMENT", "6"),
        ("UUIDX_ERROR_BUFFER_TOO_SMALL", "7"),
        ("UUIDX_ERROR_GENERATION", "8"),
        ("UUIDX_ERROR_PANIC", "9"),
        ("UUIDX_ERROR_INTERNAL", "10"),
        ("UUIDX_FORMAT_CANONICAL", "0"),
        ("UUIDX_FORMAT_SIMPLE", "1"),
        ("UUIDX_FORMAT_URN", "2"),
        ("UUIDX_FORMAT_BRACED", "3"),
        ("UUIDX_NAMESPACE_DNS", "1"),
        ("UUIDX_NAMESPACE_URL", "2"),
        ("UUIDX_NAMESPACE_OID", "3"),
        ("UUIDX_NAMESPACE_X500", "4"),
        ("UUIDX_VARIANT_NCS", "0u"),
        ("UUIDX_VARIANT_RFC9562", "1u"),
        ("UUIDX_VARIANT_MICROSOFT", "2u"),
        ("UUIDX_VARIANT_FUTURE", "3u"),
        ("UUIDX_METADATA_NONE", "0u"),
        ("UUIDX_METADATA_RANDOM", "1u"),
        ("UUIDX_METADATA_NAME_BASED", "2u"),
        ("UUIDX_METADATA_TIME", "3u"),
        ("UUIDX_METADATA_CUSTOM", "4u"),
        ("UUIDX_METADATA_DCE_SECURITY", "5u"),
        ("UUIDX_HASH_NONE", "0u"),
        ("UUIDX_HASH_MD5", "1u"),
        ("UUIDX_HASH_SHA1", "2u"),
        ("UUIDX_NODE_NONE", "0u"),
        ("UUIDX_NODE_UNICAST", "1u"),
        ("UUIDX_NODE_MULTICAST", "2u"),
        ("UUIDX_FIELD_UNKNOWN", "0u"),
        ("UUIDX_FIELD_TIME_LOW", "1u"),
        ("UUIDX_FIELD_TIME_MID", "2u"),
        ("UUIDX_FIELD_VERSION", "3u"),
        ("UUIDX_FIELD_TIME_HI", "4u"),
        ("UUIDX_FIELD_VARIANT", "5u"),
        ("UUIDX_FIELD_CLOCK_SEQUENCE", "6u"),
        ("UUIDX_FIELD_NODE", "7u"),
        ("UUIDX_FIELD_TIMESTAMP_HIGH", "8u"),
        ("UUIDX_FIELD_TIMESTAMP_LOW", "9u"),
        ("UUIDX_FIELD_UNIX_TIMESTAMP_MS", "10u"),
        ("UUIDX_FIELD_RAND_A", "11u"),
        ("UUIDX_FIELD_RAND_B", "12u"),
        ("UUIDX_FIELD_CUSTOM_A", "13u"),
        ("UUIDX_FIELD_CUSTOM_B", "14u"),
        ("UUIDX_FIELD_CUSTOM_C", "15u"),
        ("UUIDX_FIELD_PAYLOAD_A", "16u"),
        ("UUIDX_FIELD_PAYLOAD_B", "17u"),
        ("UUIDX_FIELD_PAYLOAD_C", "18u"),
        ("UUIDX_FIELD_VALUE", "19u"),
    ] {
        let expected = format!("#define {name} {value}");
        assert!(
            HEADER.lines().any(|line| line.trim() == expected),
            "header is missing `{expected}`"
        );
    }

    for symbol in [
        "uuidx_abi_version(",
        "uuidx_error_code(",
        "uuidx_error_message(",
        "uuidx_error_free(",
        "uuidx_uuid_parse(",
        "uuidx_uuid_format(",
        "uuidx_uuid_namespace(",
        "uuidx_uuid_generate(",
        "uuidx_uuid_inspect(",
    ] {
        assert!(HEADER.contains(symbol), "header is missing `{symbol}`");
    }
    for declaration in [
        "typedef int32_t uuidx_error_code_t;",
        "typedef int32_t uuidx_format_t;",
        "typedef int32_t uuidx_namespace_t;",
        "typedef uint8_t uuidx_variant_t;",
        "typedef uint8_t uuidx_metadata_kind_t;",
        "typedef uint8_t uuidx_hash_algorithm_t;",
        "typedef uint8_t uuidx_node_kind_t;",
        "typedef uint8_t uuidx_field_name_t;",
        "typedef struct uuidx_error uuidx_error_t;",
        "typedef struct uuidx_uuid {",
        "typedef struct uuidx_generation_options {",
        "typedef struct uuidx_bit_field {",
        "typedef struct uuidx_inspection {",
    ] {
        assert!(
            HEADER.contains(declaration),
            "header is missing `{declaration}`"
        );
    }
    for field in [
        "uint8_t bytes[16];",
        "int32_t version;",
        "const uuidx_uuid_t *namespace_uuid;",
        "const uint8_t *name;",
        "size_t name_length;",
        "const uint8_t *node;",
        "size_t node_length;",
        "uint8_t has_timestamp;",
        "uint64_t timestamp_seconds;",
        "uint32_t timestamp_nanoseconds;",
        "const uint8_t *custom;",
        "size_t custom_length;",
        "uuidx_field_name_t name;",
        "uint8_t offset;",
        "uint8_t width;",
        "uint8_t reserved;",
        "uint64_t value_high;",
        "uint64_t value_low;",
        "uuidx_variant_t variant;",
        "uint8_t is_nil;",
        "uint8_t is_max;",
        "uuidx_metadata_kind_t metadata_kind;",
        "uuidx_hash_algorithm_t hash_algorithm;",
        "uint8_t has_clock_sequence;",
        "uint8_t has_node;",
        "uuidx_node_kind_t node_kind;",
        "uint8_t field_count;",
        "uint16_t clock_sequence;",
        "uint32_t subsec_nanos;",
        "uint64_t unix_seconds;",
        "uint64_t unix_millis;",
        "uint8_t node_id[6];",
        "uint8_t custom[16];",
        "uuidx_bit_field_t fields[7];",
    ] {
        assert!(
            HEADER.lines().any(|line| line.trim() == field),
            "header is missing field `{field}`"
        );
    }
    assert!(HEADER.contains("error. The message is borrowed"));
    assert!(HEADER.contains("uuidx_error_free"));
    assert_eq!(uuidx_abi_version(), UUIDX_ABI_VERSION);
}
