use super::{AllowedOptions, reject_unexpected};
use crate::{GenerateError, GenerationOptions, Uuid};
use uuid::Uuid as ExternalUuid;

pub(super) fn generate(options: &GenerationOptions) -> Result<Uuid, GenerateError> {
    let custom = options.custom.ok_or(GenerateError::MissingCustomBytes {
        version: options.version,
    })?;

    reject_unexpected(
        options,
        AllowedOptions {
            custom: true,
            ..AllowedOptions::default()
        },
    )?;

    Ok(Uuid::from_external(ExternalUuid::new_v8(custom)))
}
