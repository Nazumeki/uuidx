mod json;
mod mode;
mod plain;
mod pretty;

use std::io::{self, IsTerminal, Write};

use anstream::{
    AutoStream, ColorChoice,
    stream::{AsLockedWrite, RawStream},
};
use serde::Serialize;
use uuidx_core::{Uuid, UuidInspection, UuidOutputFormat};

use crate::{
    cli::{GlobalOptions, OutputModeArg},
    errors::CliError,
};

pub use mode::RenderMode;

pub(crate) trait OutputWriter: RawStream + AsLockedWrite {}

impl<T> OutputWriter for T where T: RawStream + AsLockedWrite {}

pub struct Output<WOut = io::Stdout, WErr = io::Stderr>
where
    WOut: OutputWriter,
    WErr: OutputWriter,
{
    mode: RenderMode,
    stdout: AutoStream<WOut>,
    stderr: AutoStream<WErr>,
}

impl Output<io::Stdout, io::Stderr> {
    pub fn new(options: &GlobalOptions) -> Self {
        Self::with_writers(options, io::stdout(), io::stderr())
    }
}

impl<WOut, WErr> Output<WOut, WErr>
where
    WOut: OutputWriter,
    WErr: OutputWriter,
{
    pub(crate) fn with_writers(options: &GlobalOptions, stdout: WOut, stderr: WErr) -> Self {
        let mode = resolve_mode(options.output);
        let color = resolve_color(mode);
        Self {
            mode,
            stdout: AutoStream::new(stdout, color),
            stderr: AutoStream::new(stderr, color),
        }
    }

    pub fn generated(
        &mut self,
        index: u64,
        uuid: &Uuid,
        version: &str,
        format: UuidOutputFormat,
        warnings: &[String],
    ) -> Result<(), CliError> {
        let value = uuidx_core::format_uuid(uuid, format);
        match self.mode {
            RenderMode::Plain => self.write_stdout(&plain::value(&value)),
            RenderMode::Pretty => {
                self.write_stdout(&pretty::generated(index, &value, version, warnings))
            }
            RenderMode::Json => self.write_json(&json::JsonRecord::generated(
                index, uuid, &value, version, format, warnings,
            )),
        }
    }

    pub fn inspected_uuid(
        &mut self,
        index: u64,
        input: &str,
        inspection: &UuidInspection,
        redact_sensitive: bool,
        show_layout: bool,
    ) -> Result<(), CliError> {
        match self.mode {
            RenderMode::Plain => self.write_stdout(&plain::value(&inspection.normalized)),
            RenderMode::Pretty => self.write_stdout(&pretty::inspection(
                index,
                input,
                inspection,
                redact_sensitive,
                show_layout,
            )),
            RenderMode::Json => self.write_json(&json::JsonRecord::uuid_inspection(
                index,
                input,
                inspection,
                redact_sensitive,
            )),
        }
    }

    #[cfg(feature = "ulid-inspect")]
    pub fn inspected_ulid(
        &mut self,
        index: u64,
        input: &str,
        inspection: &uuidx_core::UlidInspection,
    ) -> Result<(), CliError> {
        match self.mode {
            RenderMode::Plain => self.write_stdout(&plain::value(&inspection.normalized)),
            RenderMode::Pretty => {
                self.write_stdout(&pretty::ulid_inspection(index, input, inspection))
            }
            RenderMode::Json => {
                self.write_json(&json::JsonRecord::ulid_inspection(index, input, inspection))
            }
        }
    }

    pub fn converted(
        &mut self,
        index: u64,
        input: &str,
        uuid: &Uuid,
        value: &str,
        format: UuidOutputFormat,
    ) -> Result<(), CliError> {
        match self.mode {
            RenderMode::Plain => self.write_stdout(&plain::value(value)),
            RenderMode::Pretty => {
                self.write_stdout(&pretty::converted(index, input, value, format))
            }
            RenderMode::Json => self.write_json(&json::JsonRecord::converted(
                index, input, uuid, value, format,
            )),
        }
    }

    pub fn validated(&mut self, index: u64, input: &str, uuid: &Uuid) -> Result<(), CliError> {
        match self.mode {
            RenderMode::Plain => Ok(()),
            RenderMode::Pretty => self.write_stdout(&pretty::validated(index, input, uuid)),
            RenderMode::Json => self.write_json(&json::JsonRecord::validated(index, input, uuid)),
        }
    }

    pub fn record_error(
        &mut self,
        operation: &str,
        index: u64,
        input: &str,
        code: &str,
        message: &str,
    ) -> Result<(), CliError> {
        match self.mode {
            RenderMode::Json => self.write_json(&json::JsonRecord::error(
                operation, index, input, code, message,
            )),
            RenderMode::Pretty => self.write_stderr(&pretty::data_error(index, input, message)),
            RenderMode::Plain => self.write_stderr(&plain::data_error(index, input, message)),
        }
    }

    pub fn top_level_error(&mut self, error: &str) -> Result<(), CliError> {
        self.write_stderr(&pretty::top_level_error(error))
    }

    pub fn flush(&mut self) -> Result<(), CliError> {
        self.stdout.flush().map_err(CliError::Output)?;
        self.stderr.flush().map_err(CliError::Output)
    }

    fn write_stdout(&mut self, value: &str) -> Result<(), CliError> {
        self.stdout
            .write_all(value.as_bytes())
            .and_then(|_| self.stdout.write_all(b"\n"))
            .map_err(CliError::Output)
    }

    fn write_stderr(&mut self, value: &str) -> Result<(), CliError> {
        self.stderr
            .write_all(value.as_bytes())
            .and_then(|_| self.stderr.write_all(b"\n"))
            .map_err(CliError::Output)
    }

    fn write_json<T: Serialize>(&mut self, value: &T) -> Result<(), CliError> {
        serde_json::to_writer(&mut self.stdout, value).map_err(CliError::Json)?;
        self.stdout.write_all(b"\n").map_err(CliError::Output)
    }
}

fn resolve_mode(mode: OutputModeArg) -> RenderMode {
    match mode {
        OutputModeArg::Auto => {
            if std::io::stdout().is_terminal() {
                RenderMode::Pretty
            } else {
                RenderMode::Plain
            }
        }
        OutputModeArg::Pretty => RenderMode::Pretty,
        OutputModeArg::Plain => RenderMode::Plain,
        OutputModeArg::Json => RenderMode::Json,
    }
}

fn resolve_color(output: RenderMode) -> ColorChoice {
    if output == RenderMode::Pretty {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuidx_core::{inspect_uuid, parse_uuid};

    fn output(mode: OutputModeArg) -> Output<Vec<u8>, Vec<u8>> {
        Output::with_writers(&GlobalOptions { output: mode }, Vec::new(), Vec::new())
    }

    #[test]
    fn explicit_output_modes_are_stable() {
        assert_eq!(resolve_mode(OutputModeArg::Pretty), RenderMode::Pretty);
        assert_eq!(resolve_mode(OutputModeArg::Plain), RenderMode::Plain);
        assert_eq!(resolve_mode(OutputModeArg::Json), RenderMode::Json);
    }

    #[test]
    fn color_is_automatic_for_pretty_and_disabled_for_machine_output() {
        assert_eq!(resolve_color(RenderMode::Plain), ColorChoice::Never);
        assert_eq!(resolve_color(RenderMode::Json), ColorChoice::Never);
        assert_eq!(resolve_color(RenderMode::Pretty), ColorChoice::Auto);
    }

    #[test]
    fn render_methods_support_each_output_mode() {
        let uuid = parse_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc").unwrap();
        let inspection = inspect_uuid(&uuid);
        let warnings = vec!["test warning".to_owned()];

        for mode in [
            OutputModeArg::Plain,
            OutputModeArg::Pretty,
            OutputModeArg::Json,
        ] {
            let mut output = output(mode);
            output
                .generated(0, &uuid, "v7", UuidOutputFormat::Canonical, &warnings)
                .unwrap();
            output
                .inspected_uuid(0, uuid.to_string().as_str(), &inspection, false, true)
                .unwrap();
            output
                .converted(
                    0,
                    uuid.to_string().as_str(),
                    &uuid,
                    &uuid.simple(),
                    UuidOutputFormat::Simple,
                )
                .unwrap();
            output.validated(0, &uuid.to_string(), &uuid).unwrap();
            output
                .record_error("test", 0, "bad", "invalid", "invalid value")
                .unwrap();
            output.top_level_error("top-level failure").unwrap();
            output.flush().unwrap();
        }
    }

    #[cfg(feature = "ulid-inspect")]
    #[test]
    fn render_methods_support_ulid_inspection() {
        let inspection = uuidx_core::inspect_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
        let mut output = output(OutputModeArg::Json);
        output
            .inspected_ulid(0, &inspection.normalized, &inspection)
            .unwrap();
    }
}
