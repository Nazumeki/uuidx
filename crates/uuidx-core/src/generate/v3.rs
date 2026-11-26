use super::{AllowedOptions, reject_unexpected};
use crate::{GenerateError, GenerationOptions, Uuid};
use uuid::Uuid as ExternalUuid;

pub(super) fn generate(options: &GenerationOptions) -> Result<Uuid, GenerateError> {
    let namespace = options
        .namespace
        .as_ref()
        .ok_or(GenerateError::MissingNameArguments {
            version: options.version,
        })?;
    let name = options
        .name
        .as_deref()
        .ok_or(GenerateError::MissingNameArguments {
            version: options.version,
        })?;

    reject_unexpected(
        options,
        AllowedOptions {
            name: true,
            ..AllowedOptions::default()
        },
    )?;
    Ok(Uuid::from_external(ExternalUuid::new_v3(
        namespace.external(),
        name,
    )))
}
