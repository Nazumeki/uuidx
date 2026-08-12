use thiserror::Error;

pub const NANOID_STANDARD_ALPHABET: &str = "A-Za-z0-9_-";
pub const NANOID_STANDARD_LENGTH: usize = 21;
pub const NANOID_STANDARD_ENTROPY_BITS: u16 = 126;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NanoidParseError {
    #[error("NanoID input is empty")]
    Empty,

    #[error("invalid NanoID: expected 21 characters from the standard A-Za-z0-9_- alphabet")]
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NanoidInspection {
    pub normalized: String,
    pub length: usize,
    pub alphabet: &'static str,
    pub entropy_bits: u16,
}

pub fn inspect_nanoid(input: &str) -> Result<NanoidInspection, NanoidParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(NanoidParseError::Empty);
    }
    if trimmed.len() != NANOID_STANDARD_LENGTH
        || !trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(NanoidParseError::Invalid);
    }

    Ok(NanoidInspection {
        normalized: trimmed.to_owned(),
        length: NANOID_STANDARD_LENGTH,
        alphabet: NANOID_STANDARD_ALPHABET,
        entropy_bits: NANOID_STANDARD_ENTROPY_BITS,
    })
}
