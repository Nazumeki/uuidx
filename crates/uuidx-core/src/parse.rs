use crate::{ParseUuidError, Uuid};
use uuid::Uuid as ExternalUuid;

pub fn parse_uuid(input: &str) -> Result<Uuid, ParseUuidError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ParseUuidError::Empty);
    }

    ExternalUuid::parse_str(trimmed)
        .map(Uuid::from_external)
        .map_err(|error| ParseUuidError::Invalid(error.to_string()))
}

pub fn parse_namespace(input: &str) -> Result<Uuid, ParseUuidError> {
    let trimmed = input.trim();
    if trimmed.eq_ignore_ascii_case("dns") {
        Ok(Uuid::NAMESPACE_DNS)
    } else if trimmed.eq_ignore_ascii_case("url") {
        Ok(Uuid::NAMESPACE_URL)
    } else if trimmed.eq_ignore_ascii_case("oid") {
        Ok(Uuid::NAMESPACE_OID)
    } else if trimmed.eq_ignore_ascii_case("x500") {
        Ok(Uuid::NAMESPACE_X500)
    } else {
        parse_uuid(trimmed)
    }
}
