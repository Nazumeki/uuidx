use crate::{GenerateError, GenerationOptions, Uuid};
use uuid::Uuid as ExternalUuid;

pub(super) fn generate(options: &GenerationOptions) -> Result<Uuid, GenerateError> {
    reject_unused(options)?;
    Ok(Uuid::from_external(ExternalUuid::new_v4()))
}

fn reject_unused(options: &GenerationOptions) -> Result<(), GenerateError> {
    if options.namespace.is_some() {
        return Err(GenerateError::UnexpectedOption {
            version: options.version,
            option: "--namespace",
        });
    }
    if options.name.is_some() {
        return Err(GenerateError::UnexpectedOption {
            version: options.version,
            option: "--name",
        });
    }
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
