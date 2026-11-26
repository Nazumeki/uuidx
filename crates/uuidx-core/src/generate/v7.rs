use std::{sync::Mutex, time::UNIX_EPOCH};

use super::{AllowedOptions, reject_unexpected};
use crate::{GenerateError, GenerationOptions, Uuid};
use uuid::Uuid as ExternalUuid;

static V7_CONTEXT: Mutex<uuid::ContextV7> = Mutex::new(uuid::ContextV7::new());

pub(super) fn generate(options: &GenerationOptions) -> Result<Uuid, GenerateError> {
    reject_unexpected(
        options,
        AllowedOptions {
            timestamp: true,
            ..AllowedOptions::default()
        },
    )?;

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
        None => Ok(Uuid::from_external(ExternalUuid::new_v7(
            uuid::Timestamp::now(&V7_CONTEXT),
        ))),
    }
}
