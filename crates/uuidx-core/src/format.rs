use std::{fmt, str::FromStr};

use crate::{HexError, Uuid};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UuidOutputFormat {
    Canonical,
    Simple,
    Urn,
    Braced,
}

impl fmt::Display for UuidOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Canonical => "canonical",
            Self::Simple => "simple",
            Self::Urn => "urn",
            Self::Braced => "braced",
        })
    }
}

impl FromStr for UuidOutputFormat {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "canonical" | "hyphenated" => Ok(Self::Canonical),
            "simple" | "hex" => Ok(Self::Simple),
            "urn" => Ok(Self::Urn),
            "braced" | "brace" => Ok(Self::Braced),
            other => Err(format!("unsupported UUID output format `{other}`")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UuidTextCase {
    Lower,
    Upper,
}

impl fmt::Display for UuidTextCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Lower => "lower",
            Self::Upper => "upper",
        })
    }
}

impl FromStr for UuidTextCase {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "lower" | "lowercase" => Ok(Self::Lower),
            "upper" | "uppercase" => Ok(Self::Upper),
            other => Err(format!("unsupported UUID text case `{other}`")),
        }
    }
}

pub fn format_uuid(uuid: &Uuid, format: UuidOutputFormat) -> String {
    format_uuid_with_case(uuid, format, UuidTextCase::Lower)
}

pub fn format_uuid_with_case(uuid: &Uuid, format: UuidOutputFormat, case: UuidTextCase) -> String {
    let inner = uuid.external();
    match (format, case) {
        (UuidOutputFormat::Canonical, UuidTextCase::Lower) => inner.hyphenated().to_string(),
        (UuidOutputFormat::Canonical, UuidTextCase::Upper) => format!("{:X}", inner.hyphenated()),
        (UuidOutputFormat::Simple, UuidTextCase::Lower) => inner.simple().to_string(),
        (UuidOutputFormat::Simple, UuidTextCase::Upper) => format!("{:X}", inner.simple()),
        (UuidOutputFormat::Urn, UuidTextCase::Lower) => inner.urn().to_string(),
        (UuidOutputFormat::Urn, UuidTextCase::Upper) => {
            format!("urn:uuid:{:X}", inner.hyphenated())
        }
        (UuidOutputFormat::Braced, UuidTextCase::Lower) => inner.braced().to_string(),
        (UuidOutputFormat::Braced, UuidTextCase::Upper) => format!("{:X}", inner.braced()),
    }
}

pub fn parse_hex_array<const N: usize>(input: &str) -> Result<[u8; N], HexError> {
    let trimmed = input.trim();
    let bytes = hex::decode(trimmed).map_err(|error| map_hex_error(error, trimmed.len()))?;

    if bytes.len() != N {
        return Err(HexError::WrongLength {
            expected: N,
            actual: bytes.len(),
        });
    }

    let mut output = [0u8; N];
    output.copy_from_slice(&bytes);
    Ok(output)
}

fn map_hex_error(error: hex::FromHexError, input_length: usize) -> HexError {
    match error {
        hex::FromHexError::InvalidHexCharacter { index, .. } => HexError::Invalid { offset: index },
        hex::FromHexError::OddLength => HexError::Invalid {
            offset: input_length.saturating_sub(1),
        },
        hex::FromHexError::InvalidStringLength => HexError::Invalid {
            offset: input_length,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_every_hex_decoder_error_shape() {
        assert_eq!(
            map_hex_error(
                hex::FromHexError::InvalidHexCharacter { c: 'z', index: 2 },
                4,
            ),
            HexError::Invalid { offset: 2 }
        );
        assert_eq!(
            map_hex_error(hex::FromHexError::OddLength, 3),
            HexError::Invalid { offset: 2 }
        );
        assert_eq!(
            map_hex_error(hex::FromHexError::OddLength, 0),
            HexError::Invalid { offset: 0 }
        );
        assert_eq!(
            map_hex_error(hex::FromHexError::InvalidStringLength, 4),
            HexError::Invalid { offset: 4 }
        );
    }

    #[test]
    fn text_case_round_trips_aliases_and_display() {
        assert_eq!(
            UuidTextCase::from_str("lower").unwrap(),
            UuidTextCase::Lower
        );
        assert_eq!(
            UuidTextCase::from_str("UPPERCASE").unwrap(),
            UuidTextCase::Upper
        );
        assert_eq!(UuidTextCase::Lower.to_string(), "lower");
        assert_eq!(UuidTextCase::Upper.to_string(), "upper");
        assert!(UuidTextCase::from_str("title").is_err());
    }

    #[test]
    fn case_switches_only_uuid_digits() {
        let uuid = crate::parse_uuid("936da01f-9abd-4d9d-80c7-02af85c822a8").unwrap();

        assert_eq!(
            format_uuid_with_case(&uuid, UuidOutputFormat::Canonical, UuidTextCase::Upper),
            "936DA01F-9ABD-4D9D-80C7-02AF85C822A8"
        );
        assert_eq!(
            format_uuid_with_case(&uuid, UuidOutputFormat::Simple, UuidTextCase::Upper),
            "936DA01F9ABD4D9D80C702AF85C822A8"
        );
        assert_eq!(
            format_uuid_with_case(&uuid, UuidOutputFormat::Urn, UuidTextCase::Upper),
            "urn:uuid:936DA01F-9ABD-4D9D-80C7-02AF85C822A8"
        );
        assert_eq!(
            format_uuid_with_case(&uuid, UuidOutputFormat::Braced, UuidTextCase::Upper),
            "{936DA01F-9ABD-4D9D-80C7-02AF85C822A8}"
        );

        for format in [
            UuidOutputFormat::Canonical,
            UuidOutputFormat::Simple,
            UuidOutputFormat::Urn,
            UuidOutputFormat::Braced,
        ] {
            assert_eq!(
                format_uuid_with_case(&uuid, format, UuidTextCase::Lower),
                format_uuid(&uuid, format)
            );
        }
    }
}
