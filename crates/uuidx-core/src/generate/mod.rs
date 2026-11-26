mod v3;
mod v4;
mod v5;
mod v6;
mod v7;
mod v8;

use crate::{GeneratableUuidVersion, GenerateError, GenerationOptions, Uuid};

pub fn generate_uuid(options: &GenerationOptions) -> Result<Uuid, GenerateError> {
    match options.version {
        GeneratableUuidVersion::V3 => v3::generate(options),
        GeneratableUuidVersion::V4 => v4::generate(options),
        GeneratableUuidVersion::V5 => v5::generate(options),
        GeneratableUuidVersion::V6 => v6::generate(options),
        GeneratableUuidVersion::V7 => v7::generate(options),
        GeneratableUuidVersion::V8 => v8::generate(options),
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct AllowedOptions {
    pub name: bool,
    pub node: bool,
    pub timestamp: bool,
    pub custom: bool,
}

pub(super) fn reject_unexpected(
    options: &GenerationOptions,
    allowed: AllowedOptions,
) -> Result<(), GenerateError> {
    if !allowed.name {
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
    }
    if !allowed.node && options.node.is_some() {
        return Err(GenerateError::UnexpectedOption {
            version: options.version,
            option: "--node",
        });
    }
    if !allowed.timestamp && options.timestamp.is_some() {
        return Err(GenerateError::UnexpectedOption {
            version: options.version,
            option: "--timestamp",
        });
    }
    if !allowed.custom && options.custom.is_some() {
        return Err(GenerateError::UnexpectedOption {
            version: options.version,
            option: "--custom",
        });
    }
    Ok(())
}
