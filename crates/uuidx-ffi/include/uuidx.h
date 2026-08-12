#ifndef UUIDX_H
#define UUIDX_H

#include <stddef.h>
#include <stdint.h>

#if defined(_WIN32) && defined(UUIDX_SHARED)
#  if defined(UUIDX_BUILDING_LIBRARY)
#    define UUIDX_API __declspec(dllexport)
#  else
#    define UUIDX_API __declspec(dllimport)
#  endif
#else
#  define UUIDX_API
#endif

#ifdef __cplusplus
extern "C" {
#endif

#define UUIDX_ABI_VERSION 1u

typedef int32_t uuidx_error_code_t;
#define UUIDX_ERROR_OK 0
#define UUIDX_ERROR_NULL_POINTER 1
#define UUIDX_ERROR_INVALID_UTF8 2
#define UUIDX_ERROR_INVALID_UUID 3
#define UUIDX_ERROR_INVALID_VERSION 4
#define UUIDX_ERROR_INVALID_FORMAT 5
#define UUIDX_ERROR_INVALID_ARGUMENT 6
#define UUIDX_ERROR_BUFFER_TOO_SMALL 7
#define UUIDX_ERROR_GENERATION 8
#define UUIDX_ERROR_PANIC 9
#define UUIDX_ERROR_INTERNAL 10

typedef int32_t uuidx_format_t;
#define UUIDX_FORMAT_CANONICAL 0
#define UUIDX_FORMAT_SIMPLE 1
#define UUIDX_FORMAT_URN 2
#define UUIDX_FORMAT_BRACED 3

typedef int32_t uuidx_namespace_t;
#define UUIDX_NAMESPACE_DNS 1
#define UUIDX_NAMESPACE_URL 2
#define UUIDX_NAMESPACE_OID 3
#define UUIDX_NAMESPACE_X500 4

typedef uint8_t uuidx_variant_t;
#define UUIDX_VARIANT_NCS 0u
#define UUIDX_VARIANT_RFC9562 1u
#define UUIDX_VARIANT_MICROSOFT 2u
#define UUIDX_VARIANT_FUTURE 3u

typedef uint8_t uuidx_metadata_kind_t;
#define UUIDX_METADATA_NONE 0u
#define UUIDX_METADATA_RANDOM 1u
#define UUIDX_METADATA_NAME_BASED 2u
#define UUIDX_METADATA_TIME 3u
#define UUIDX_METADATA_CUSTOM 4u
#define UUIDX_METADATA_DCE_SECURITY 5u

typedef uint8_t uuidx_hash_algorithm_t;
#define UUIDX_HASH_NONE 0u
#define UUIDX_HASH_MD5 1u
#define UUIDX_HASH_SHA1 2u

typedef uint8_t uuidx_node_kind_t;
#define UUIDX_NODE_NONE 0u
#define UUIDX_NODE_UNICAST 1u
#define UUIDX_NODE_MULTICAST 2u

typedef uint8_t uuidx_field_name_t;
#define UUIDX_FIELD_UNKNOWN 0u
#define UUIDX_FIELD_TIME_LOW 1u
#define UUIDX_FIELD_TIME_MID 2u
#define UUIDX_FIELD_VERSION 3u
#define UUIDX_FIELD_TIME_HI 4u
#define UUIDX_FIELD_VARIANT 5u
#define UUIDX_FIELD_CLOCK_SEQUENCE 6u
#define UUIDX_FIELD_NODE 7u
#define UUIDX_FIELD_TIMESTAMP_HIGH 8u
#define UUIDX_FIELD_TIMESTAMP_LOW 9u
#define UUIDX_FIELD_UNIX_TIMESTAMP_MS 10u
#define UUIDX_FIELD_RAND_A 11u
#define UUIDX_FIELD_RAND_B 12u
#define UUIDX_FIELD_CUSTOM_A 13u
#define UUIDX_FIELD_CUSTOM_B 14u
#define UUIDX_FIELD_CUSTOM_C 15u
#define UUIDX_FIELD_PAYLOAD_A 16u
#define UUIDX_FIELD_PAYLOAD_B 17u
#define UUIDX_FIELD_PAYLOAD_C 18u
#define UUIDX_FIELD_VALUE 19u

typedef struct uuidx_error uuidx_error_t;

typedef struct uuidx_uuid {
    uint8_t bytes[16];
} uuidx_uuid_t;

/*
 * An optional byte input is absent when its pointer is NULL and its length is
 * zero. A non-NULL pointer with a zero length is a present empty input.
 * `has_timestamp` must be zero or one. All unused fields should be zeroed.
 */
typedef struct uuidx_generation_options {
    int32_t version;
    const uuidx_uuid_t *namespace_uuid;
    const uint8_t *name;
    size_t name_length;
    const uint8_t *node;
    size_t node_length;
    uint8_t has_timestamp;
    uint64_t timestamp_seconds;
    uint32_t timestamp_nanoseconds;
    const uint8_t *custom;
    size_t custom_length;
} uuidx_generation_options_t;

typedef struct uuidx_bit_field {
    uuidx_field_name_t name;
    uint8_t offset;
    uint8_t width;
    uint8_t reserved;
    uint64_t value_high;
    uint64_t value_low;
} uuidx_bit_field_t;

typedef struct uuidx_inspection {
    uint8_t version;
    uuidx_variant_t variant;
    uint8_t is_nil;
    uint8_t is_max;
    uuidx_metadata_kind_t metadata_kind;
    uuidx_hash_algorithm_t hash_algorithm;
    uint8_t has_timestamp;
    uint8_t has_clock_sequence;
    uint8_t has_node;
    uuidx_node_kind_t node_kind;
    uint8_t field_count;
    uint8_t reserved;
    uint16_t clock_sequence;
    uint32_t subsec_nanos;
    uint64_t unix_seconds;
    uint64_t unix_millis;
    uint8_t node_id[6];
    uint8_t custom[16];
    uuidx_bit_field_t fields[7];
} uuidx_inspection_t;

UUIDX_API uint32_t uuidx_abi_version(void);

/*
 * Fallible calls return NULL on success. On failure they return one owned
 * error. The message is borrowed and remains valid until uuidx_error_free.
 * Embedded NUL bytes in internal messages are replaced with '?'.
 * Rust panics are converted to UUIDX_ERROR_PANIC when the library is built
 * with an unwinding panic strategy; panic-abort builds terminate immediately.
 */
UUIDX_API uuidx_error_code_t uuidx_error_code(const uuidx_error_t *error);
UUIDX_API const char *uuidx_error_message(const uuidx_error_t *error);
UUIDX_API void uuidx_error_free(uuidx_error_t *error);

UUIDX_API uuidx_error_t *uuidx_uuid_parse(
    const uint8_t *input,
    size_t input_length,
    uuidx_uuid_t *output
);

/*
 * For a valid UUID and format, output_length receives the required byte count
 * including the NUL. Pass output = NULL and output_capacity = 0 to query it.
 */
UUIDX_API uuidx_error_t *uuidx_uuid_format(
    const uuidx_uuid_t *uuid,
    uuidx_format_t format,
    char *output,
    size_t output_capacity,
    size_t *output_length
);

UUIDX_API uuidx_error_t *uuidx_uuid_namespace(
    uuidx_namespace_t namespace_kind,
    uuidx_uuid_t *output
);

UUIDX_API uuidx_error_t *uuidx_uuid_generate(
    const uuidx_generation_options_t *options,
    uuidx_uuid_t *output
);

UUIDX_API uuidx_error_t *uuidx_uuid_inspect(
    const uuidx_uuid_t *uuid,
    uuidx_inspection_t *output
);

#ifdef __cplusplus
}
#endif

#endif
