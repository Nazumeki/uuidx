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

pub fn format_uuid(uuid: &Uuid, format: UuidOutputFormat) -> String {
    match format {
        UuidOutputFormat::Canonical => uuid.hyphenated(),
        UuidOutputFormat::Simple => uuid.simple(),
        UuidOutputFormat::Urn => uuid.urn(),
        UuidOutputFormat::Braced => format!("{{{}}}", uuid.hyphenated()),
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
}
