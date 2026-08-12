//! C-compatible bindings for UUID parsing, formatting, generation, and inspection.
//!
//! The public ABI is declared in `include/uuidx.h`. UUID values and inspection
//! records are written into caller-owned storage. Errors are returned as opaque
//! allocations and must be released with [`uuidx_error_free`]. Error messages
//! are borrowed from their error object and remain valid until it is released.
//!
//! Operational entry points use `catch_unwind` to prevent unwinding across the
//! C boundary when the library is built with an unwinding panic strategy. A
//! process-wide `panic = "abort"` build profile still aborts before a panic can
//! be caught.

use std::{
    ffi::{CString, c_char},
    panic::{AssertUnwindSafe, catch_unwind},
    ptr, slice, str,
    time::{Duration, UNIX_EPOCH},
};

use uuidx_core::{
    GeneratableUuidVersion, GenerationOptions, NameHashAlgorithm, NodeKind, Uuid, UuidMetadata,
    UuidOutputFormat, UuidVariant, format_uuid, generate_uuid, inspect_uuid, parse_uuid,
};

/// ABI version implemented by this library.
pub const UUIDX_ABI_VERSION: u32 = 1;

/// The operation completed successfully.
pub const UUIDX_ERROR_OK: i32 = 0;
/// A required pointer was null.
pub const UUIDX_ERROR_NULL_POINTER: i32 = 1;
/// A byte string was not valid UTF-8.
pub const UUIDX_ERROR_INVALID_UTF8: i32 = 2;
/// UUID text could not be parsed.
pub const UUIDX_ERROR_INVALID_UUID: i32 = 3;
/// A generation version was not supported.
pub const UUIDX_ERROR_INVALID_VERSION: i32 = 4;
/// A formatting mode was not supported.
pub const UUIDX_ERROR_INVALID_FORMAT: i32 = 5;
/// An argument had an invalid value or length.
pub const UUIDX_ERROR_INVALID_ARGUMENT: i32 = 6;
/// A caller-owned output buffer was too small.
pub const UUIDX_ERROR_BUFFER_TOO_SMALL: i32 = 7;
/// UUID generation rejected the supplied options.
pub const UUIDX_ERROR_GENERATION: i32 = 8;
/// A Rust panic was caught before crossing the C ABI boundary.
pub const UUIDX_ERROR_PANIC: i32 = 9;
/// The library encountered an unexpected internal value.
pub const UUIDX_ERROR_INTERNAL: i32 = 10;

/// Canonical hyphenated UUID output.
pub const UUIDX_FORMAT_CANONICAL: i32 = 0;
/// Simple 32-character hexadecimal UUID output.
pub const UUIDX_FORMAT_SIMPLE: i32 = 1;
/// `urn:uuid:` UUID output.
pub const UUIDX_FORMAT_URN: i32 = 2;
/// Braced canonical UUID output.
pub const UUIDX_FORMAT_BRACED: i32 = 3;

/// The RFC DNS namespace.
pub const UUIDX_NAMESPACE_DNS: i32 = 1;
/// The RFC URL namespace.
pub const UUIDX_NAMESPACE_URL: i32 = 2;
/// The RFC OID namespace.
pub const UUIDX_NAMESPACE_OID: i32 = 3;
/// The RFC X.500 namespace.
pub const UUIDX_NAMESPACE_X500: i32 = 4;

/// NCS UUID variant.
pub const UUIDX_VARIANT_NCS: u8 = 0;
/// RFC 9562 UUID variant.
pub const UUIDX_VARIANT_RFC9562: u8 = 1;
/// Microsoft UUID variant.
pub const UUIDX_VARIANT_MICROSOFT: u8 = 2;
/// Reserved future UUID variant.
pub const UUIDX_VARIANT_FUTURE: u8 = 3;

/// No version-specific metadata.
pub const UUIDX_METADATA_NONE: u8 = 0;
/// Random UUID metadata.
pub const UUIDX_METADATA_RANDOM: u8 = 1;
/// Name-based UUID metadata.
pub const UUIDX_METADATA_NAME_BASED: u8 = 2;
/// Time-based UUID metadata.
pub const UUIDX_METADATA_TIME: u8 = 3;
/// Custom UUID metadata.
pub const UUIDX_METADATA_CUSTOM: u8 = 4;
/// DCE Security UUID metadata.
pub const UUIDX_METADATA_DCE_SECURITY: u8 = 5;

/// No name-hash algorithm applies.
pub const UUIDX_HASH_NONE: u8 = 0;
/// MD5 name hashing.
pub const UUIDX_HASH_MD5: u8 = 1;
/// SHA-1 name hashing.
pub const UUIDX_HASH_SHA1: u8 = 2;

/// No node classification applies.
pub const UUIDX_NODE_NONE: u8 = 0;
/// A unicast, potentially hardware-derived node.
pub const UUIDX_NODE_UNICAST: u8 = 1;
/// A multicast, locally generated node.
pub const UUIDX_NODE_MULTICAST: u8 = 2;

/// Unknown bit-field name.
pub const UUIDX_FIELD_UNKNOWN: u8 = 0;
/// `time_low` bit field.
pub const UUIDX_FIELD_TIME_LOW: u8 = 1;
/// `time_mid` bit field.
pub const UUIDX_FIELD_TIME_MID: u8 = 2;
/// `version` bit field.
pub const UUIDX_FIELD_VERSION: u8 = 3;
/// `time_hi` bit field.
pub const UUIDX_FIELD_TIME_HI: u8 = 4;
/// `variant` bit field.
pub const UUIDX_FIELD_VARIANT: u8 = 5;
/// `clock_sequence` bit field.
pub const UUIDX_FIELD_CLOCK_SEQUENCE: u8 = 6;
/// `node` bit field.
pub const UUIDX_FIELD_NODE: u8 = 7;
/// `timestamp_high` bit field.
pub const UUIDX_FIELD_TIMESTAMP_HIGH: u8 = 8;
/// `timestamp_low` bit field.
pub const UUIDX_FIELD_TIMESTAMP_LOW: u8 = 9;
/// `unix_timestamp_ms` bit field.
pub const UUIDX_FIELD_UNIX_TIMESTAMP_MS: u8 = 10;
/// `rand_a` bit field.
pub const UUIDX_FIELD_RAND_A: u8 = 11;
/// `rand_b` bit field.
pub const UUIDX_FIELD_RAND_B: u8 = 12;
/// `custom_a` bit field.
pub const UUIDX_FIELD_CUSTOM_A: u8 = 13;
/// `custom_b` bit field.
pub const UUIDX_FIELD_CUSTOM_B: u8 = 14;
/// `custom_c` bit field.
pub const UUIDX_FIELD_CUSTOM_C: u8 = 15;
/// `payload_a` bit field.
pub const UUIDX_FIELD_PAYLOAD_A: u8 = 16;
/// `payload_b` bit field.
pub const UUIDX_FIELD_PAYLOAD_B: u8 = 17;
/// `payload_c` bit field.
pub const UUIDX_FIELD_PAYLOAD_C: u8 = 18;
/// Whole UUID `value` bit field.
pub const UUIDX_FIELD_VALUE: u8 = 19;

/// A UUID represented as 16 bytes in network byte order.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UuidxUuid {
    /// UUID bytes in the order used by the textual representation.
    pub bytes: [u8; 16],
}

impl UuidxUuid {
    fn into_core(self) -> Uuid {
        Uuid::from_bytes(self.bytes)
    }
}

impl From<Uuid> for UuidxUuid {
    fn from(value: Uuid) -> Self {
        Self {
            bytes: *value.as_bytes(),
        }
    }
}

/// UUID generation options.
///
/// Optional byte inputs are absent when their pointer is null and their length
/// is zero. A non-null pointer with a zero length represents a present empty
/// byte string, which is meaningful for `name`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UuidxGenerationOptions {
    /// UUID version number. Supported values are 3 through 8.
    pub version: i32,
    /// Optional namespace UUID used by versions 3 and 5.
    pub namespace_uuid: *const UuidxUuid,
    /// Optional name bytes used by versions 3 and 5.
    pub name: *const u8,
    /// Number of bytes available at `name`.
    pub name_length: usize,
    /// Optional six-byte node identifier used by version 6.
    pub node: *const u8,
    /// Number of bytes available at `node`.
    pub node_length: usize,
    /// One when timestamp fields are present, zero otherwise.
    pub has_timestamp: u8,
    /// Unix timestamp whole seconds.
    pub timestamp_seconds: u64,
    /// Nanoseconds within `timestamp_seconds`, less than one billion.
    pub timestamp_nanoseconds: u32,
    /// Optional 16-byte custom payload used by version 8.
    pub custom: *const u8,
    /// Number of bytes available at `custom`.
    pub custom_length: usize,
}

impl Default for UuidxGenerationOptions {
    fn default() -> Self {
        Self {
            version: 0,
            namespace_uuid: ptr::null(),
            name: ptr::null(),
            name_length: 0,
            node: ptr::null(),
            node_length: 0,
            has_timestamp: 0,
            timestamp_seconds: 0,
            timestamp_nanoseconds: 0,
            custom: ptr::null(),
            custom_length: 0,
        }
    }
}

/// One named bit range in an inspected UUID.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UuidxBitField {
    /// One of the `UUIDX_FIELD_*` constants.
    pub name: u8,
    /// Offset from the most significant UUID bit.
    pub offset: u8,
    /// Width in bits.
    pub width: u8,
    /// Reserved; currently zero.
    pub reserved: u8,
    /// Most significant 64 bits of the value.
    pub value_high: u64,
    /// Least significant 64 bits of the value.
    pub value_low: u64,
}

/// Fixed-size UUID inspection result.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UuidxInspection {
    /// UUID version nibble. Nil is 0 and max is 15.
    pub version: u8,
    /// One of the `UUIDX_VARIANT_*` constants.
    pub variant: u8,
    /// One when the UUID is nil.
    pub is_nil: u8,
    /// One when the UUID is max.
    pub is_max: u8,
    /// One of the `UUIDX_METADATA_*` constants.
    pub metadata_kind: u8,
    /// One of the `UUIDX_HASH_*` constants.
    pub hash_algorithm: u8,
    /// One when timestamp fields are populated.
    pub has_timestamp: u8,
    /// One when `clock_sequence` is populated.
    pub has_clock_sequence: u8,
    /// One when `node_id` is populated.
    pub has_node: u8,
    /// One of the `UUIDX_NODE_*` constants.
    pub node_kind: u8,
    /// Number of populated entries in `fields`.
    pub field_count: u8,
    /// Reserved; currently zero.
    pub reserved: u8,
    /// Clock sequence for applicable time-based UUIDs.
    pub clock_sequence: u16,
    /// Nanoseconds within `unix_seconds`.
    pub subsec_nanos: u32,
    /// Whole seconds since the Unix epoch.
    pub unix_seconds: u64,
    /// Whole milliseconds since the Unix epoch.
    pub unix_millis: u64,
    /// Node identifier for applicable time-based UUIDs.
    pub node_id: [u8; 6],
    /// Final custom bytes for version 8 UUIDs.
    pub custom: [u8; 16],
    /// Bit-field layout. Only the first `field_count` entries are populated.
    pub fields: [UuidxBitField; 7],
}

/// Opaque error returned by fallible ABI calls.
pub struct UuidxError {
    code: i32,
    message: CString,
}

#[derive(Debug)]
struct FfiError {
    code: i32,
    message: String,
}

impl FfiError {
    fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

type FfiResult<T = ()> = Result<T, FfiError>;

/// Returns the ABI version implemented by this library.
#[unsafe(no_mangle)]
pub extern "C" fn uuidx_abi_version() -> u32 {
    UUIDX_ABI_VERSION
}

/// Returns the numeric code stored in an error object.
///
/// A null pointer returns [`UUIDX_ERROR_NULL_POINTER`].
///
/// # Safety
///
/// `error` must be null or point to a live error returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuidx_error_code(error: *const UuidxError) -> i32 {
    if error.is_null() {
        UUIDX_ERROR_NULL_POINTER
    } else {
        // SAFETY: Required by this function's contract.
        unsafe { (*error).code }
    }
}

/// Returns a borrowed NUL-terminated message from an error object.
///
/// A null pointer returns an empty string. The returned pointer is valid until
/// the error is passed to [`uuidx_error_free`].
///
/// # Safety
///
/// `error` must be null or point to a live error returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuidx_error_message(error: *const UuidxError) -> *const c_char {
    if error.is_null() {
        c"".as_ptr()
    } else {
        // SAFETY: Required by this function's contract.
        unsafe { (*error).message.as_ptr() }
    }
}

/// Releases an error returned by this library. A null pointer is accepted.
///
/// # Safety
///
/// `error` must be null or a pointer returned by this library that has not
/// previously been released.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuidx_error_free(error: *mut UuidxError) {
    if !error.is_null() {
        // SAFETY: Required by this function's contract; reconstructing the box
        // transfers the allocation back to Rust for destruction.
        unsafe { drop(Box::from_raw(error)) };
    }
}

/// Parses UTF-8 UUID text into a caller-owned UUID value.
///
/// Returns null on success or an owned error that must be released with
/// [`uuidx_error_free`].
///
/// # Safety
///
/// `input` must point to `input_length` readable bytes and `output` must point
/// to writable storage for one [`UuidxUuid`]. All pointers must remain valid for
/// the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuidx_uuid_parse(
    input: *const u8,
    input_length: usize,
    output: *mut UuidxUuid,
) -> *mut UuidxError {
    ffi_call(|| {
        require_output(output, "output")?;
        // SAFETY: The caller contract provides readable input bytes.
        let input = unsafe { required_bytes(input, input_length, "input")? };
        let input = str::from_utf8(input).map_err(|error| {
            FfiError::new(
                UUIDX_ERROR_INVALID_UTF8,
                format!("input is not valid UTF-8: {error}"),
            )
        })?;
        let uuid = parse_uuid(input)
            .map_err(|error| FfiError::new(UUIDX_ERROR_INVALID_UUID, error.to_string()))?;
        // SAFETY: `output` was validated above and is caller-owned writable storage.
        unsafe { ptr::write(output, uuid.into()) };
        Ok(())
    })
}

/// Formats a UUID into a caller-owned NUL-terminated UTF-8 buffer.
///
/// `output_length` receives the required size including the NUL terminator.
/// Passing a null `output` with a zero capacity performs a size query. Returns
/// null on success or an owned error that must be released.
///
/// # Safety
///
/// `uuid` and `output_length` must point to readable/writable values. When
/// `output` is non-null, it must point to `output_capacity` writable bytes.
/// Pointers must remain valid for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuidx_uuid_format(
    uuid: *const UuidxUuid,
    format: i32,
    output: *mut c_char,
    output_capacity: usize,
    output_length: *mut usize,
) -> *mut UuidxError {
    ffi_call(|| {
        let uuid = read_uuid(uuid)?;
        require_output(output_length, "output_length")?;
        let format = parse_format(format)?;
        let formatted = format_uuid(&uuid, format);
        let required = formatted
            .len()
            .checked_add(1)
            .ok_or_else(|| FfiError::new(UUIDX_ERROR_INTERNAL, "formatted UUID is too long"))?;
        // SAFETY: `output_length` was validated and is caller-owned writable storage.
        unsafe { ptr::write(output_length, required) };

        if output.is_null() {
            return if output_capacity == 0 {
                Ok(())
            } else {
                Err(FfiError::new(
                    UUIDX_ERROR_NULL_POINTER,
                    "output is null but output_capacity is nonzero",
                ))
            };
        }
        if output_capacity < required {
            return Err(FfiError::new(
                UUIDX_ERROR_BUFFER_TOO_SMALL,
                format!("output buffer needs {required} bytes including the NUL terminator"),
            ));
        }
        if output_capacity > isize::MAX as usize {
            return Err(FfiError::new(
                UUIDX_ERROR_INVALID_ARGUMENT,
                "output_capacity exceeds the maximum supported object size",
            ));
        }

        // SAFETY: The caller contract and capacity checks guarantee writable
        // space. The source is a distinct Rust allocation.
        unsafe {
            ptr::copy_nonoverlapping(formatted.as_ptr(), output.cast::<u8>(), formatted.len());
            output.add(formatted.len()).write(0);
        }
        Ok(())
    })
}

/// Writes one of the four RFC namespace UUID constants.
///
/// Returns null on success or an owned error that must be released.
///
/// # Safety
///
/// `output` must point to writable storage for one [`UuidxUuid`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuidx_uuid_namespace(
    namespace_kind: i32,
    output: *mut UuidxUuid,
) -> *mut UuidxError {
    ffi_call(|| {
        require_output(output, "output")?;
        let uuid = match namespace_kind {
            UUIDX_NAMESPACE_DNS => Uuid::NAMESPACE_DNS,
            UUIDX_NAMESPACE_URL => Uuid::NAMESPACE_URL,
            UUIDX_NAMESPACE_OID => Uuid::NAMESPACE_OID,
            UUIDX_NAMESPACE_X500 => Uuid::NAMESPACE_X500,
            value => {
                return Err(FfiError::new(
                    UUIDX_ERROR_INVALID_ARGUMENT,
                    format!("unsupported namespace kind {value}"),
                ));
            }
        };
        // SAFETY: `output` was validated above and is caller-owned writable storage.
        unsafe { ptr::write(output, uuid.into()) };
        Ok(())
    })
}

/// Generates a UUID using the supplied options.
///
/// Returns null on success or an owned error that must be released.
///
/// # Safety
///
/// `options` must point to a valid [`UuidxGenerationOptions`] and every non-null
/// nested pointer must describe readable storage of its corresponding length.
/// `output` must point to writable storage for one [`UuidxUuid`]. All pointers
/// must remain valid for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuidx_uuid_generate(
    options: *const UuidxGenerationOptions,
    output: *mut UuidxUuid,
) -> *mut UuidxError {
    ffi_call(|| {
        require_output(options, "options")?;
        require_output(output, "output")?;
        // SAFETY: The caller contract provides a readable options value. Copying
        // it lets us read every input before writing potentially aliased output.
        let options = unsafe { *options };
        // SAFETY: Nested pointer validity is part of the caller contract and
        // lengths are validated by `optional_bytes`.
        let options = unsafe { generation_options(options)? };
        let uuid = generate_uuid(&options)
            .map_err(|error| FfiError::new(UUIDX_ERROR_GENERATION, error.to_string()))?;
        // SAFETY: `output` was validated above and is caller-owned writable storage.
        unsafe { ptr::write(output, uuid.into()) };
        Ok(())
    })
}

/// Inspects a UUID and writes a fixed-size summary and bit-field layout.
///
/// Returns null on success or an owned error that must be released.
///
/// # Safety
///
/// `uuid` must point to a readable [`UuidxUuid`] and `output` must point to
/// writable storage for one [`UuidxInspection`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuidx_uuid_inspect(
    uuid: *const UuidxUuid,
    output: *mut UuidxInspection,
) -> *mut UuidxError {
    ffi_call(|| {
        let uuid = read_uuid(uuid)?;
        require_output(output, "output")?;
        let inspection = inspection_from_core(&uuid)?;
        // SAFETY: `output` was validated above and is caller-owned writable storage.
        unsafe { ptr::write(output, inspection) };
        Ok(())
    })
}

fn ffi_call(operation: impl FnOnce() -> FfiResult) -> *mut UuidxError {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(())) => ptr::null_mut(),
        Ok(Err(error)) => owned_error(error.code, error.message),
        Err(_) => owned_error(UUIDX_ERROR_PANIC, "uuidx caught a Rust panic"),
    }
}

fn owned_error(code: i32, message: impl Into<String>) -> *mut UuidxError {
    let sanitized = message
        .into()
        .into_bytes()
        .into_iter()
        .map(|byte| if byte == 0 { b'?' } else { byte })
        .collect::<Vec<_>>();
    let message = CString::new(sanitized).unwrap_or_default();
    Box::into_raw(Box::new(UuidxError { code, message }))
}

fn require_output<T>(pointer: *const T, name: &str) -> FfiResult {
    if pointer.is_null() {
        Err(FfiError::new(
            UUIDX_ERROR_NULL_POINTER,
            format!("{name} must not be null"),
        ))
    } else {
        Ok(())
    }
}

unsafe fn required_bytes<'a>(pointer: *const u8, length: usize, name: &str) -> FfiResult<&'a [u8]> {
    if pointer.is_null() {
        return Err(FfiError::new(
            UUIDX_ERROR_NULL_POINTER,
            format!("{name} must not be null"),
        ));
    }
    if length > isize::MAX as usize {
        return Err(FfiError::new(
            UUIDX_ERROR_INVALID_ARGUMENT,
            format!("{name} length exceeds the maximum supported object size"),
        ));
    }
    // SAFETY: Pointer validity is required by the caller and length is bounded
    // to the maximum Rust slice size above.
    Ok(unsafe { slice::from_raw_parts(pointer, length) })
}

unsafe fn optional_bytes<'a>(
    pointer: *const u8,
    length: usize,
    name: &str,
) -> FfiResult<Option<&'a [u8]>> {
    if pointer.is_null() {
        return if length == 0 {
            Ok(None)
        } else {
            Err(FfiError::new(
                UUIDX_ERROR_NULL_POINTER,
                format!("{name} is null but its length is nonzero"),
            ))
        };
    }
    // SAFETY: Forwarded caller contract.
    unsafe { required_bytes(pointer, length, name).map(Some) }
}

fn read_uuid(pointer: *const UuidxUuid) -> FfiResult<Uuid> {
    require_output(pointer, "uuid")?;
    // SAFETY: The caller contract for each entry point provides a live UUID.
    Ok(unsafe { (*pointer).into_core() })
}

fn parse_format(format: i32) -> FfiResult<UuidOutputFormat> {
    match format {
        UUIDX_FORMAT_CANONICAL => Ok(UuidOutputFormat::Canonical),
        UUIDX_FORMAT_SIMPLE => Ok(UuidOutputFormat::Simple),
        UUIDX_FORMAT_URN => Ok(UuidOutputFormat::Urn),
        UUIDX_FORMAT_BRACED => Ok(UuidOutputFormat::Braced),
        value => Err(FfiError::new(
            UUIDX_ERROR_INVALID_FORMAT,
            format!("unsupported UUID format {value}"),
        )),
    }
}

fn parse_version(version: i32) -> FfiResult<GeneratableUuidVersion> {
    match version {
        3 => Ok(GeneratableUuidVersion::V3),
        4 => Ok(GeneratableUuidVersion::V4),
        5 => Ok(GeneratableUuidVersion::V5),
        6 => Ok(GeneratableUuidVersion::V6),
        7 => Ok(GeneratableUuidVersion::V7),
        8 => Ok(GeneratableUuidVersion::V8),
        value => Err(FfiError::new(
            UUIDX_ERROR_INVALID_VERSION,
            format!("unsupported generation version {value}"),
        )),
    }
}

unsafe fn generation_options(input: UuidxGenerationOptions) -> FfiResult<GenerationOptions> {
    let mut output = GenerationOptions::new(parse_version(input.version)?);
    output.namespace = if input.namespace_uuid.is_null() {
        None
    } else {
        // SAFETY: Nested pointer validity is required by the caller contract.
        Some(unsafe { (*input.namespace_uuid).into_core() })
    };
    // SAFETY: Nested pointer validity is required by the caller contract.
    output.name =
        unsafe { optional_bytes(input.name, input.name_length, "name")? }.map(<[u8]>::to_vec);

    // SAFETY: Nested pointer validity is required by the caller contract.
    output.node = unsafe { optional_bytes(input.node, input.node_length, "node")? }
        .map(|node| {
            node.try_into().map_err(|_| {
                FfiError::new(
                    UUIDX_ERROR_INVALID_ARGUMENT,
                    format!("node must contain exactly 6 bytes, got {}", node.len()),
                )
            })
        })
        .transpose()?;

    output.timestamp = match input.has_timestamp {
        0 => None,
        1 if input.timestamp_nanoseconds < 1_000_000_000 => UNIX_EPOCH.checked_add(Duration::new(
            input.timestamp_seconds,
            input.timestamp_nanoseconds,
        )),
        1 => {
            return Err(FfiError::new(
                UUIDX_ERROR_INVALID_ARGUMENT,
                "timestamp_nanoseconds must be less than 1000000000",
            ));
        }
        value => {
            return Err(FfiError::new(
                UUIDX_ERROR_INVALID_ARGUMENT,
                format!("has_timestamp must be 0 or 1, got {value}"),
            ));
        }
    };
    if input.has_timestamp == 1 && output.timestamp.is_none() {
        return Err(FfiError::new(
            UUIDX_ERROR_INVALID_ARGUMENT,
            "timestamp is outside the supported range",
        ));
    }

    // SAFETY: Nested pointer validity is required by the caller contract.
    output.custom = unsafe { optional_bytes(input.custom, input.custom_length, "custom")? }
        .map(|custom| {
            custom.try_into().map_err(|_| {
                FfiError::new(
                    UUIDX_ERROR_INVALID_ARGUMENT,
                    format!(
                        "custom payload must contain exactly 16 bytes, got {}",
                        custom.len()
                    ),
                )
            })
        })
        .transpose()?;
    Ok(output)
}

fn inspection_from_core(uuid: &Uuid) -> FfiResult<UuidxInspection> {
    let source = inspect_uuid(uuid);
    let mut output =
        UuidxInspection {
            version: source.version.number().ok_or_else(|| {
                FfiError::new(UUIDX_ERROR_INTERNAL, "inspection returned no UUID version")
            })?,
            variant: variant_value(source.variant),
            is_nil: u8::from(source.is_nil),
            is_max: u8::from(source.is_max),
            field_count: source.fields.len().try_into().map_err(|_| {
                FfiError::new(UUIDX_ERROR_INTERNAL, "too many UUID inspection fields")
            })?,
            ..UuidxInspection::default()
        };

    if source.fields.len() > output.fields.len() {
        return Err(FfiError::new(
            UUIDX_ERROR_INTERNAL,
            "too many UUID inspection fields",
        ));
    }
    for (target, field) in output.fields.iter_mut().zip(&source.fields) {
        target.name = field_name(&field.name)?;
        target.offset = field.offset;
        target.width = field.width;
        target.value_high = (field.value >> 64) as u64;
        target.value_low = field.value as u64;
    }

    match source.metadata {
        UuidMetadata::None => {}
        UuidMetadata::Random => output.metadata_kind = UUIDX_METADATA_RANDOM,
        UuidMetadata::NameBased { algorithm } => {
            output.metadata_kind = UUIDX_METADATA_NAME_BASED;
            output.hash_algorithm = match algorithm {
                NameHashAlgorithm::Md5 => UUIDX_HASH_MD5,
                NameHashAlgorithm::Sha1 => UUIDX_HASH_SHA1,
            };
        }
        UuidMetadata::Time {
            timestamp,
            clock_sequence,
            node_id,
            node_kind,
        } => {
            output.metadata_kind = UUIDX_METADATA_TIME;
            output.has_timestamp = 1;
            output.unix_seconds = timestamp.unix_seconds;
            output.unix_millis = timestamp.unix_millis;
            output.subsec_nanos = timestamp.subsec_nanos;
            if let Some(clock_sequence) = clock_sequence {
                output.has_clock_sequence = 1;
                output.clock_sequence = clock_sequence;
            }
            if let Some(node_id) = node_id {
                output.has_node = 1;
                output.node_id = node_id;
            }
            output.node_kind = match node_kind {
                Some(NodeKind::Unicast) => UUIDX_NODE_UNICAST,
                Some(NodeKind::Multicast) => UUIDX_NODE_MULTICAST,
                None => UUIDX_NODE_NONE,
            };
        }
        UuidMetadata::Custom { bytes } => {
            output.metadata_kind = UUIDX_METADATA_CUSTOM;
            output.custom = bytes;
        }
        UuidMetadata::DceSecurity => output.metadata_kind = UUIDX_METADATA_DCE_SECURITY,
    }
    Ok(output)
}

fn variant_value(variant: UuidVariant) -> u8 {
    match variant {
        UuidVariant::Ncs => UUIDX_VARIANT_NCS,
        UuidVariant::Rfc9562 => UUIDX_VARIANT_RFC9562,
        UuidVariant::Microsoft => UUIDX_VARIANT_MICROSOFT,
        UuidVariant::Future => UUIDX_VARIANT_FUTURE,
    }
}

fn field_name(name: &str) -> FfiResult<u8> {
    match name {
        "time_low" => Ok(UUIDX_FIELD_TIME_LOW),
        "time_mid" => Ok(UUIDX_FIELD_TIME_MID),
        "version" => Ok(UUIDX_FIELD_VERSION),
        "time_hi" => Ok(UUIDX_FIELD_TIME_HI),
        "variant" => Ok(UUIDX_FIELD_VARIANT),
        "clock_sequence" => Ok(UUIDX_FIELD_CLOCK_SEQUENCE),
        "node" => Ok(UUIDX_FIELD_NODE),
        "timestamp_high" => Ok(UUIDX_FIELD_TIMESTAMP_HIGH),
        "timestamp_low" => Ok(UUIDX_FIELD_TIMESTAMP_LOW),
        "unix_timestamp_ms" => Ok(UUIDX_FIELD_UNIX_TIMESTAMP_MS),
        "rand_a" => Ok(UUIDX_FIELD_RAND_A),
        "rand_b" => Ok(UUIDX_FIELD_RAND_B),
        "custom_a" => Ok(UUIDX_FIELD_CUSTOM_A),
        "custom_b" => Ok(UUIDX_FIELD_CUSTOM_B),
        "custom_c" => Ok(UUIDX_FIELD_CUSTOM_C),
        "payload_a" => Ok(UUIDX_FIELD_PAYLOAD_A),
        "payload_b" => Ok(UUIDX_FIELD_PAYLOAD_B),
        "payload_c" => Ok(UUIDX_FIELD_PAYLOAD_C),
        "value" => Ok(UUIDX_FIELD_VALUE),
        other => Err(FfiError::new(
            UUIDX_ERROR_INTERNAL,
            format!("unknown inspection field {other}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn ffi_call_converts_panics_to_owned_errors() {
        let error = ffi_call(|| panic!("test panic"));
        assert!(!error.is_null());
        // SAFETY: `ffi_call` returned a live error owned by this test.
        unsafe {
            assert_eq!(uuidx_error_code(error), UUIDX_ERROR_PANIC);
            assert_eq!(
                CStr::from_ptr(uuidx_error_message(error)).to_bytes(),
                b"uuidx caught a Rust panic"
            );
            uuidx_error_free(error);
        }
    }

    #[test]
    fn owned_error_sanitizes_embedded_nuls() {
        let error = owned_error(UUIDX_ERROR_INTERNAL, "before\0after");
        // SAFETY: `owned_error` returned a live error owned by this test.
        unsafe {
            assert_eq!(uuidx_error_code(error), UUIDX_ERROR_INTERNAL);
            assert_eq!(
                CStr::from_ptr(uuidx_error_message(error)).to_bytes(),
                b"before?after"
            );
            uuidx_error_free(error);
        }
    }

    #[test]
    fn unknown_inspection_field_is_an_internal_error() {
        let error = field_name("not-a-core-field").unwrap_err();
        assert_eq!(error.code, UUIDX_ERROR_INTERNAL);
        assert_eq!(error.message, "unknown inspection field not-a-core-field");
    }
}
