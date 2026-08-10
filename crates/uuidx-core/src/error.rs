use thiserror::Error;

use crate::{GeneratableUuidVersion, InspectableUuidVersion};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseUuidError {
    #[error("UUID input is empty")]
    Empty,

    #[error("invalid UUID: {0}")]
    Invalid(String),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HexError {
    #[error("expected exactly {expected} hexadecimal bytes, got {actual}")]
    WrongLength { expected: usize, actual: usize },

    #[error("invalid hexadecimal input at byte {offset}")]
    Invalid { offset: usize },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GenerateError {
    #[error("UUID version {version} is not supported for generation")]
    UnsupportedVersion { version: InspectableUuidVersion },

    #[error("UUID {version} generation requires --namespace and --name")]
    MissingNameArguments { version: GeneratableUuidVersion },

    #[error("UUID {version} generation requires a custom 16-byte payload")]
    MissingCustomBytes { version: GeneratableUuidVersion },

    #[error("UUID {version} generation received an invalid timestamp")]
    InvalidTimestamp { version: GeneratableUuidVersion },

    #[error("option {option} is not valid for UUID {version} generation")]
    UnexpectedOption {
        version: GeneratableUuidVersion,
        option: &'static str,
    },
}
