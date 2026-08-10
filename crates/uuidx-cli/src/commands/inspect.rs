use crate::{
    cli::{InspectArgs, InspectKindArg},
    errors::CliError,
    input,
    output::Output,
};

pub fn run(args: &InspectArgs, output: &mut Output) -> Result<bool, CliError> {
    let fail_fast = args.input.fail_fast;
    let kind = args.kind;
    let redact_sensitive = args.redact_sensitive;
    let show_layout = args.layout;
    let summary =
        input::for_each_record(&args.input, fail_fast, |index, value| {
            match inspect_value(kind, value) {
                Ok(InspectionValue::Uuid(uuid)) => {
                    output.inspected_uuid(index, value, &uuid, redact_sensitive, show_layout)?;
                    Ok(false)
                }
                #[cfg(feature = "ulid-inspect")]
                Ok(InspectionValue::Ulid(ulid)) => {
                    output.inspected_ulid(index, value, &ulid)?;
                    Ok(false)
                }
                Err(error) => {
                    output.record_error("inspect", index, value, "invalid_identifier", &error)?;
                    Ok(true)
                }
            }
        })?;
    Ok(summary.data_errors)
}

enum InspectionValue {
    Uuid(uuidx_core::UuidInspection),
    #[cfg(feature = "ulid-inspect")]
    Ulid(uuidx_core::UlidInspection),
}

fn inspect_value(kind: InspectKindArg, value: &str) -> Result<InspectionValue, String> {
    match kind {
        InspectKindArg::Uuid => uuidx_core::parse_uuid(value)
            .map(|uuid| InspectionValue::Uuid(uuidx_core::inspect_uuid(&uuid)))
            .map_err(|error| error.to_string()),
        #[cfg(feature = "ulid-inspect")]
        InspectKindArg::Ulid => uuidx_core::inspect_ulid(value)
            .map(InspectionValue::Ulid)
            .map_err(|error| error.to_string()),
        #[cfg(not(feature = "ulid-inspect"))]
        InspectKindArg::Ulid => {
            Err("ULID inspection support was disabled at build time".to_owned())
        }
        InspectKindArg::Auto => {
            if let Ok(uuid) = uuidx_core::parse_uuid(value) {
                return Ok(InspectionValue::Uuid(uuidx_core::inspect_uuid(&uuid)));
            }
            #[cfg(feature = "ulid-inspect")]
            if let Ok(ulid) = uuidx_core::inspect_ulid(value) {
                return Ok(InspectionValue::Ulid(ulid));
            }
            Err("input is neither a valid UUID nor a supported ULID".to_owned())
        }
    }
}
