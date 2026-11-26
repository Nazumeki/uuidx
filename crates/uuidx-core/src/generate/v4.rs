use super::{AllowedOptions, reject_unexpected};
use crate::{GenerateError, GenerationOptions, Uuid};
use uuid::Uuid as ExternalUuid;

pub(super) fn generate(options: &GenerationOptions) -> Result<Uuid, GenerateError> {
    reject_unexpected(options, AllowedOptions::default())?;
    Ok(Uuid::from_external(ExternalUuid::new_v4()))
}
