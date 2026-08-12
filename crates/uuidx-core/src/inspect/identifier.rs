use std::fmt;

use crate::{
    NanoidInspection, SnowflakeInspection, inspect_nanoid, inspect_snowflake, inspect_uuid,
    parse_uuid,
};

#[cfg(feature = "ulid-inspect")]
use crate::{UlidInspection, inspect_ulid};

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
