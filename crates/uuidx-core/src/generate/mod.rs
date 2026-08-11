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
