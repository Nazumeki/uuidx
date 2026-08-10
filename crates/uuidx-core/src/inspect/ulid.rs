use thiserror::Error;

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
}

pub fn inspect_ulid(input: &str) -> Result<UlidInspection, UlidParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(UlidParseError::Empty);
    }

    let ulid = ulid::Ulid::from_string(trimmed)
        .map_err(|error| UlidParseError::Invalid(error.to_string()))?;
    Ok(UlidInspection {
        normalized: ulid.to_string(),
        bytes: ulid.to_bytes(),
        timestamp_ms: ulid.timestamp_ms(),
        random: ulid.random(),
    })
}
