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

    reject_unused(options)?;
    Ok(Uuid::from_external(ExternalUuid::new_v5(
        namespace.external(),
        name,
    )))
}

fn reject_unused(options: &GenerationOptions) -> Result<(), GenerateError> {
    if options.node.is_some() {
        return Err(GenerateError::UnexpectedOption {
            version: options.version,
            option: "--node",
        });
    }
    if options.timestamp.is_some() {
        return Err(GenerateError::UnexpectedOption {
            version: options.version,
            option: "--timestamp",
        });
    }
    if options.custom.is_some() {
        return Err(GenerateError::UnexpectedOption {
            version: options.version,
            option: "--custom",
        });
    }
    Ok(())
}
