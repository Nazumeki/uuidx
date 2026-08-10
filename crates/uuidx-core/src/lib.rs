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
pub use format::{UuidOutputFormat, format_uuid, parse_hex_array};
pub use generate::generate_uuid;
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
