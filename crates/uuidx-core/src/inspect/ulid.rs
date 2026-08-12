use thiserror::Error;

use super::BitField;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UlidParseError {
    #[error("ULID input is empty")]
    Empty,

    #[error("invalid ULID: {0}")]
    Invalid(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UlidInspection {
    pub normalized: String,
    pub bytes: [u8; 16],
    pub timestamp_ms: u64,
    pub random: u128,
    pub fields: Vec<BitField>,
}

pub fn inspect_ulid(input: &str) -> Result<UlidInspection, UlidParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(UlidParseError::Empty);
    }

    let ulid = ulid::Ulid::from_string(trimmed)
        .map_err(|error| UlidParseError::Invalid(error.to_string()))?;
    let timestamp_ms = ulid.timestamp_ms();
    let random = ulid.random();
    Ok(UlidInspection {
        normalized: ulid.to_string(),
        bytes: ulid.to_bytes(),
        timestamp_ms,
        random,
        fields: vec![
            BitField {
                name: "timestamp_ms".to_owned(),
                offset: 0,
                width: 48,
                value: timestamp_ms.into(),
            },
            BitField {
                name: "random".to_owned(),
                offset: 48,
                width: 80,
                value: random,
            },
        ],
    })
}
