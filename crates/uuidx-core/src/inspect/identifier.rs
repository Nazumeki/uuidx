use std::{fmt, str::FromStr};

use thiserror::Error;

use crate::{
    NanoidInspection, NanoidParseError, ParseUuidError, SnowflakeInspection, SnowflakeParseError,
    inspect_nanoid, inspect_snowflake, inspect_uuid, parse_uuid,
};

#[cfg(feature = "ulid-inspect")]
use crate::{UlidInspection, UlidParseError, inspect_ulid};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentifierInspection {
    Uuid(super::UuidInspection),
    #[cfg(feature = "ulid-inspect")]
    Ulid(UlidInspection),
    Nanoid(NanoidInspection),
    Snowflake(SnowflakeInspection),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdentifierParseError;

impl fmt::Display for IdentifierParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "ulid-inspect")]
        return f.write_str(
            "input is not a supported UUID, ULID, standard NanoID, or Twitter Snowflake",
        );

        #[cfg(not(feature = "ulid-inspect"))]
        f.write_str("input is not a supported UUID, standard NanoID, or Twitter Snowflake")
    }
}

impl std::error::Error for IdentifierParseError {}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IdentifierFamily {
    Uuid,
    #[cfg(feature = "ulid-inspect")]
    Ulid,
    Nanoid,
    Snowflake,
}

impl fmt::Display for IdentifierFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Uuid => "uuid",
            #[cfg(feature = "ulid-inspect")]
            Self::Ulid => "ulid",
            Self::Nanoid => "nanoid",
            Self::Snowflake => "snowflake",
        })
    }
}

impl FromStr for IdentifierFamily {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "uuid" => Ok(Self::Uuid),
            #[cfg(feature = "ulid-inspect")]
            "ulid" => Ok(Self::Ulid),
            "nanoid" | "nano-id" => Ok(Self::Nanoid),
            "snowflake" => Ok(Self::Snowflake),
            other => Err(format!("unsupported identifier family `{other}`")),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdentifierTypeError {
    #[error("{0}")]
    Uuid(#[from] ParseUuidError),

    #[cfg(feature = "ulid-inspect")]
    #[error("{0}")]
    Ulid(#[from] UlidParseError),

    #[error("{0}")]
    Nanoid(#[from] NanoidParseError),

    #[error("{0}")]
    Snowflake(#[from] SnowflakeParseError),
}

impl IdentifierInspection {
    pub fn normalized(&self) -> &str {
        match self {
            Self::Uuid(inspection) => &inspection.normalized,
            #[cfg(feature = "ulid-inspect")]
            Self::Ulid(inspection) => &inspection.normalized,
            Self::Nanoid(inspection) => &inspection.normalized,
            Self::Snowflake(inspection) => &inspection.normalized,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Uuid(_) => "uuid",
            #[cfg(feature = "ulid-inspect")]
            Self::Ulid(_) => "ulid",
            Self::Nanoid(_) => "nanoid",
            Self::Snowflake(_) => "snowflake",
        }
    }
}

pub fn inspect_identifier_as(
    input: &str,
    family: IdentifierFamily,
) -> Result<IdentifierInspection, IdentifierTypeError> {
    match family {
        IdentifierFamily::Uuid => {
            let uuid = parse_uuid(input)?;
            Ok(IdentifierInspection::Uuid(inspect_uuid(&uuid)))
        }
        #[cfg(feature = "ulid-inspect")]
        IdentifierFamily::Ulid => Ok(IdentifierInspection::Ulid(inspect_ulid(input)?)),
        IdentifierFamily::Nanoid => Ok(IdentifierInspection::Nanoid(inspect_nanoid(input)?)),
        IdentifierFamily::Snowflake => {
            Ok(IdentifierInspection::Snowflake(inspect_snowflake(input)?))
        }
    }
}

pub fn inspect_identifier(input: &str) -> Result<IdentifierInspection, IdentifierParseError> {
    if let Ok(uuid) = parse_uuid(input) {
        return Ok(IdentifierInspection::Uuid(inspect_uuid(&uuid)));
    }
    #[cfg(feature = "ulid-inspect")]
    if let Ok(ulid) = inspect_ulid(input) {
        return Ok(IdentifierInspection::Ulid(ulid));
    }
    if let Ok(nanoid) = inspect_nanoid(input) {
        return Ok(IdentifierInspection::Nanoid(nanoid));
    }
    if let Ok(snowflake) = inspect_snowflake(input) {
        return Ok(IdentifierInspection::Snowflake(snowflake));
    }
    Err(IdentifierParseError)
}
