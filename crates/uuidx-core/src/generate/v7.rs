use std::time::UNIX_EPOCH;

use crate::{GenerateError, GenerationOptions, Uuid};
use uuid::Uuid as ExternalUuid;

pub(super) fn generate(options: &GenerationOptions) -> Result<Uuid, GenerateError> {
    reject_unused(options)?;

    match options.timestamp {
        Some(timestamp) => {
            let duration = timestamp.duration_since(UNIX_EPOCH).map_err(|_| {
                GenerateError::InvalidTimestamp {
                    version: options.version,
                }
            })?;
            let timestamp = uuid::Timestamp::from_unix(
                uuid::NoContext,
                duration.as_secs(),
                duration.subsec_nanos(),
            );
            Ok(Uuid::from_external(ExternalUuid::new_v7(timestamp)))
        }
        None => Ok(Uuid::from_external(ExternalUuid::now_v7())),
    }
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
    if options.custom.is_some() {
        return Err(GenerateError::UnexpectedOption {
            version: options.version,
            option: "--custom",
        });
    }
    Ok(())
}
