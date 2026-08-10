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
