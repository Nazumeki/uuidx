use crate::{GenerateError, GenerationOptions, Uuid};
use uuid::Uuid as ExternalUuid;

pub(super) fn generate(options: &GenerationOptions) -> Result<Uuid, GenerateError> {
    reject_unused(options)?;
    let node = options.node.unwrap_or_else(random_node);

    match options.timestamp {
        Some(timestamp) => {
            let timestamp = uuid::Timestamp::try_from(timestamp).map_err(|_| {
                GenerateError::InvalidTimestamp {
                    version: options.version,
                }
            })?;
            Ok(Uuid::from_external(ExternalUuid::new_v6(timestamp, &node)))
        }
        None => Ok(Uuid::from_external(ExternalUuid::now_v6(&node))),
    }
}

pub(crate) fn random_node() -> [u8; 6] {
    let random = ExternalUuid::new_v4();
    let mut node = [0u8; 6];
    node.copy_from_slice(&random.as_bytes()[..6]);
    node[0] |= 0x01;
    node
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
    if options.custom.is_some() {
        return Err(GenerateError::UnexpectedOption {
            version: options.version,
            option: "--custom",
        });
    }
    Ok(())
}
