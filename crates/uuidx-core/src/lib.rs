//! UUID domain logic used by the `uuidx` command-line application.
//!
//! This crate intentionally has no CLI, terminal, or JSON dependencies. It
//! owns the UUID contract while the companion CLI crate owns presentation and
//! process orchestration.

mod error;
mod format;
mod generate;
mod inspect;
mod parse;
mod types;

pub use error::{GenerateError, HexError, ParseUuidError};
pub use format::{
    UuidOutputFormat, UuidTextCase, format_uuid, format_uuid_with_case, parse_hex_array,
};
pub use generate::generate_uuid;
pub use inspect::identifier::{
    IdentifierFamily, IdentifierInspection, IdentifierParseError, IdentifierTypeError,
    inspect_identifier, inspect_identifier_as,
};
pub use inspect::nanoid::{
    NANOID_STANDARD_ALPHABET, NANOID_STANDARD_ENTROPY_BITS, NANOID_STANDARD_LENGTH,
    NanoidInspection, NanoidParseError, inspect_nanoid,
};
pub use inspect::snowflake::{
    SnowflakeInspection, SnowflakeParseError, TWITTER_SNOWFLAKE_EPOCH_MS, inspect_snowflake,
};
pub use inspect::{
    BitField, NameHashAlgorithm, NodeKind, TimestampInfo, UuidInspection, UuidMetadata,
    inspect_uuid,
};
pub use parse::parse_uuid;
pub use types::{
    GeneratableUuidVersion, GenerationOptions, InspectableUuidVersion, Uuid, UuidVariant,
};

#[cfg(feature = "ulid-inspect")]
pub use inspect::ulid::{UlidInspection, UlidParseError, inspect_ulid};
