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
use uuidx_core::{IdentifierInspection, Uuid, UuidInspection, UuidOutputFormat, UuidTextCase};

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
        case: UuidTextCase,
        warnings: &[String],
    ) -> Result<(), CliError> {
        let value = uuidx_core::format_uuid_with_case(uuid, format, case);
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
        show_layout: bool,
    ) -> Result<(), CliError> {
        match self.mode {
            RenderMode::Plain => self.write_stdout(&plain::value(&inspection.normalized)),
            RenderMode::Pretty => self.write_stdout(&pretty::ulid_inspection(
                index,
                input,
                inspection,
                show_layout,
            )),
            RenderMode::Json => {
                self.write_json(&json::JsonRecord::ulid_inspection(index, input, inspection))
            }
        }
    }

    pub fn inspected_nanoid(
        &mut self,
        index: u64,
        input: &str,
        inspection: &uuidx_core::NanoidInspection,
        show_layout: bool,
    ) -> Result<(), CliError> {
        match self.mode {
            RenderMode::Plain => self.write_stdout(&plain::value(&inspection.normalized)),
            RenderMode::Pretty => self.write_stdout(&pretty::nanoid_inspection(
                index,
                input,
                inspection,
                show_layout,
            )),
            RenderMode::Json => self.write_json(&json::JsonRecord::nanoid_inspection(
                index, input, inspection,
            )),
        }
    }

    pub fn inspected_snowflake(
        &mut self,
        index: u64,
        input: &str,
        inspection: &uuidx_core::SnowflakeInspection,
        show_layout: bool,
    ) -> Result<(), CliError> {
        match self.mode {
            RenderMode::Plain => self.write_stdout(&plain::value(&inspection.normalized)),
            RenderMode::Pretty => self.write_stdout(&pretty::snowflake_inspection(
                index,
                input,
                inspection,
                show_layout,
            )),
            RenderMode::Json => self.write_json(&json::JsonRecord::snowflake_inspection(
                index, input, inspection,
            )),
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

    pub fn validated(
        &mut self,
        index: u64,
        input: &str,
        inspection: &IdentifierInspection,
    ) -> Result<(), CliError> {
        match self.mode {
            RenderMode::Plain => Ok(()),
            RenderMode::Pretty => self.write_stdout(&pretty::validated(index, input, inspection)),
            RenderMode::Json => {
                self.write_json(&json::JsonRecord::validated(index, input, inspection))
            }
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
pub(crate) struct TestWriter {
    fail_write: bool,
    fail_flush: bool,
}

#[cfg(test)]
impl TestWriter {
    pub(crate) const fn working() -> Self {
        Self {
            fail_write: false,
            fail_flush: false,
        }
    }

    pub(crate) const fn failing_write() -> Self {
        Self {
            fail_write: true,
            fail_flush: false,
        }
    }

    pub(crate) const fn failing_flush() -> Self {
        Self {
            fail_write: false,
            fail_flush: true,
        }
    }
}

#[cfg(test)]
impl Write for TestWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.fail_write {
            Err(io::Error::other("test write failure"))
        } else {
            Ok(buffer.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush {
            Err(io::Error::other("test flush failure"))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
pub(crate) fn test_output(
    mode: OutputModeArg,
    stdout: TestWriter,
    stderr: TestWriter,
) -> Output<Box<dyn Write>, Box<dyn Write>> {
    Output::with_writers(
        &GlobalOptions { output: mode },
        Box::new(stdout) as Box<dyn Write>,
        Box::new(stderr) as Box<dyn Write>,
    )
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
        assert_eq!(resolve_mode(OutputModeArg::Auto), RenderMode::Plain);
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
                .generated(
                    0,
                    &uuid,
                    "v7",
                    UuidOutputFormat::Canonical,
                    UuidTextCase::Lower,
                    &warnings,
                )
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
            output
                .validated(
                    0,
                    &uuid.to_string(),
                    &IdentifierInspection::Uuid(inspection.clone()),
                )
                .unwrap();
            output
                .record_error("test", 0, "bad", "invalid", "invalid value")
                .unwrap();
            output.top_level_error("top-level failure").unwrap();
            output.flush().unwrap();
        }
    }

    #[test]
    fn write_and_flush_failures_preserve_their_error_categories() {
        let uuid = parse_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc").unwrap();

        let mut plain = test_output(
            OutputModeArg::Plain,
            TestWriter::failing_write(),
            TestWriter::working(),
        );
        assert!(matches!(
            plain.generated(
                0,
                &uuid,
                "v7",
                UuidOutputFormat::Canonical,
                UuidTextCase::Lower,
                &[],
            ),
            Err(CliError::Output(_))
        ));

        let mut pretty = test_output(
            OutputModeArg::Pretty,
            TestWriter::working(),
            TestWriter::failing_write(),
        );
        assert!(matches!(
            pretty.record_error("inspect", 0, "bad", "invalid", "bad value"),
            Err(CliError::Output(_))
        ));

        let mut json = test_output(
            OutputModeArg::Json,
            TestWriter::failing_write(),
            TestWriter::working(),
        );
        assert!(matches!(
            json.generated(
                0,
                &uuid,
                "v7",
                UuidOutputFormat::Canonical,
                UuidTextCase::Lower,
                &[],
            ),
            Err(CliError::Json(_))
        ));

        let mut stdout_flush = test_output(
            OutputModeArg::Plain,
            TestWriter::failing_flush(),
            TestWriter::working(),
        );
        assert!(matches!(stdout_flush.flush(), Err(CliError::Output(_))));

        let mut stderr_flush = test_output(
            OutputModeArg::Plain,
            TestWriter::working(),
            TestWriter::failing_flush(),
        );
        assert!(matches!(stderr_flush.flush(), Err(CliError::Output(_))));
    }

    #[cfg(feature = "ulid-inspect")]
    #[test]
    fn render_methods_support_ulid_inspection() {
        let inspection = uuidx_core::inspect_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
        for mode in [
            OutputModeArg::Plain,
            OutputModeArg::Pretty,
            OutputModeArg::Json,
        ] {
            let mut output = output(mode);
            output
                .inspected_ulid(0, &inspection.normalized, &inspection, true)
                .unwrap();
        }
    }

    #[test]
    fn render_methods_support_nanoid_and_snowflake_inspection() {
        let nanoid = uuidx_core::inspect_nanoid("V1StGXR8_Z5jdHi6B-myT").unwrap();
        let snowflake = uuidx_core::inspect_snowflake("1724552287438348288").unwrap();
        for mode in [
            OutputModeArg::Plain,
            OutputModeArg::Pretty,
            OutputModeArg::Json,
        ] {
            let mut output = output(mode);
            output
                .inspected_nanoid(0, &nanoid.normalized, &nanoid, true)
                .unwrap();
            output
                .inspected_snowflake(1, &snowflake.normalized, &snowflake, true)
                .unwrap();
        }
    }
}
