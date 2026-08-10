use std::{fmt, io};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{0}")]
    Usage(String),

    #[error("no input values were provided and stdin is interactive")]
    NoInput,

    #[error("input error: {0}")]
    Input(#[source] io::Error),

    #[error("output error: {0}")]
    Output(#[source] io::Error),

    #[error("JSON output error: {0}")]
    Json(#[source] serde_json::Error),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) | Self::NoInput => 2,
            Self::Input(_) | Self::Output(_) | Self::Json(_) => 3,
        }
    }
}

pub enum AppResult {
    Success,
    DataErrors,
    Failure(CliError),
}

impl AppResult {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Success => 0,
            Self::DataErrors => 1,
            Self::Failure(error) => error.exit_code(),
        }
    }
}

impl fmt::Debug for AppResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => f.write_str("Success"),
            Self::DataErrors => f.write_str("DataErrors"),
            Self::Failure(error) => f.debug_tuple("Failure").field(error).finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_errors_map_to_usage_or_process_exit_codes() {
        assert_eq!(CliError::Usage("bad option".to_owned()).exit_code(), 2);
        assert_eq!(CliError::NoInput.exit_code(), 2);
        assert_eq!(CliError::Input(io::Error::other("read")).exit_code(), 3);
        assert_eq!(CliError::Output(io::Error::other("write")).exit_code(), 3);
        assert_eq!(
            CliError::Json(serde_json::from_str::<serde_json::Value>("{").unwrap_err()).exit_code(),
            3
        );
    }

    #[test]
    fn app_results_map_to_success_data_and_failure_codes() {
        assert_eq!(AppResult::Success.exit_code(), 0);
        assert_eq!(AppResult::DataErrors.exit_code(), 1);
        assert_eq!(
            AppResult::Failure(CliError::Usage("bad option".to_owned())).exit_code(),
            2
        );
    }

    #[test]
    fn app_result_debug_output_keeps_failure_context() {
        assert_eq!(format!("{:?}", AppResult::Success), "Success");
        assert_eq!(format!("{:?}", AppResult::DataErrors), "DataErrors");
        assert_eq!(
            format!(
                "{:?}",
                AppResult::Failure(CliError::Usage("bad option".to_owned()))
            ),
            "Failure(Usage(\"bad option\"))"
        );
    }
}
