use std::{
    fs::File,
    io::{self, BufRead, BufReader, IsTerminal},
};

use crate::{cli::InputArgs, errors::CliError};

pub struct InputSummary {
    pub data_errors: bool,
}

pub fn for_each_record<F>(
    input: &InputArgs,
    fail_fast: bool,
    callback: F,
) -> Result<InputSummary, CliError>
where
    F: FnMut(u64, &str) -> Result<bool, CliError>,
{
    if !input.values.is_empty() {
        return process_values(input.values.iter().map(String::as_str), fail_fast, callback);
    }

    if let Some(path) = input.input.as_deref() {
        let file = File::open(path).map_err(CliError::Input)?;
        return process_reader(BufReader::new(file), fail_fast, callback);
    }

    if io::stdin().is_terminal() {
        return Err(CliError::NoInput);
    }

    process_reader(BufReader::new(io::stdin()), fail_fast, callback)
}

fn process_values<'a, I, F>(
    values: I,
    fail_fast: bool,
    mut callback: F,
) -> Result<InputSummary, CliError>
where
    I: IntoIterator<Item = &'a str>,
    F: FnMut(u64, &str) -> Result<bool, CliError>,
{
    let mut data_errors = false;
    let mut index = 0u64;
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        let record_error = callback(index, value)?;
        data_errors |= record_error;
        index += 1;
        if record_error && fail_fast {
            break;
        }
    }
    Ok(InputSummary { data_errors })
}

fn process_reader<R, F>(
    mut reader: R,
    fail_fast: bool,
    mut callback: F,
) -> Result<InputSummary, CliError>
where
    R: BufRead,
    F: FnMut(u64, &str) -> Result<bool, CliError>,
{
    let mut line = String::new();
    let mut index = 0u64;
    let mut data_errors = false;

    loop {
        line.clear();
        let read = reader.read_line(&mut line).map_err(CliError::Input)?;
        if read == 0 {
            break;
        }
        let value = line.trim();
        if value.is_empty() {
            continue;
        }

        let record_error = callback(index, value)?;
        data_errors |= record_error;
        index += 1;
        if record_error && fail_fast {
            break;
        }
    }

    Ok(InputSummary { data_errors })
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Cursor};

    use super::*;

    #[test]
    fn positional_values_skip_blanks_and_keep_record_indexes_contiguous() {
        let values = ["", " first ", "second", "  ", "third"];
        let mut records = Vec::new();
        let summary = process_values(values.iter().copied(), false, |index, value| {
            records.push((index, value.to_owned()));
            Ok(false)
        })
        .unwrap();

        assert!(!summary.data_errors);
        assert_eq!(
            records,
            vec![
                (0, "first".to_owned()),
                (1, "second".to_owned()),
                (2, "third".to_owned())
            ]
        );
    }

    #[test]
    fn positional_processing_accumulates_errors_and_supports_fail_fast() {
        let values = ["ok", "bad", "later"];
        let mut seen = Vec::new();
        let summary = process_values(values.iter().copied(), false, |index, value| {
            seen.push((index, value.to_owned()));
            Ok(value == "bad")
        })
        .unwrap();
        assert!(summary.data_errors);
        assert_eq!(
            seen,
            vec![
                (0, "ok".to_owned()),
                (1, "bad".to_owned()),
                (2, "later".to_owned())
            ]
        );

        let mut seen = Vec::new();
        let summary = process_values(values.iter().copied(), true, |index, value| {
            seen.push((index, value.to_owned()));
            Ok(value == "bad")
        })
        .unwrap();
        assert!(summary.data_errors);
        assert_eq!(seen, vec![(0, "ok".to_owned()), (1, "bad".to_owned())]);
    }

    #[test]
    fn reader_processing_skips_blank_lines_and_reports_callback_errors() {
        let mut records = Vec::new();
        let summary = process_reader(Cursor::new("\nfirst\n\nsecond\n"), false, |index, value| {
            records.push((index, value.to_owned()));
            Ok(value == "second")
        })
        .unwrap();

        assert!(summary.data_errors);
        assert_eq!(
            records,
            vec![(0, "first".to_owned()), (1, "second".to_owned())]
        );

        let mut records = Vec::new();
        let summary = process_reader(
            Cursor::new("first\nsecond\nthird\n"),
            true,
            |index, value| {
                records.push((index, value.to_owned()));
                Ok(value == "second")
            },
        )
        .unwrap();
        assert!(summary.data_errors);
        assert_eq!(
            records,
            vec![(0, "first".to_owned()), (1, "second".to_owned())]
        );
    }

    #[test]
    fn for_each_record_reads_files_and_reports_open_errors() {
        let path =
            std::env::temp_dir().join(format!("uuidx-input-test-{}.txt", std::process::id()));
        fs::write(&path, " first\n\nsecond\n").unwrap();
        let input = InputArgs {
            values: Vec::new(),
            input: Some(path.clone()),
            fail_fast: false,
        };
        let mut records = Vec::new();
        let summary = for_each_record(&input, false, |index, value| {
            records.push((index, value.to_owned()));
            Ok(false)
        })
        .unwrap();
        fs::remove_file(path).unwrap();
        assert!(!summary.data_errors);
        assert_eq!(
            records,
            vec![(0, "first".to_owned()), (1, "second".to_owned())]
        );

        let missing = InputArgs {
            values: Vec::new(),
            input: Some(std::env::temp_dir().join("uuidx-input-file-does-not-exist")),
            fail_fast: false,
        };
        assert!(matches!(
            for_each_record(&missing, false, |_, _| Ok(false)),
            Err(CliError::Input(_))
        ));
    }

    #[test]
    fn callback_errors_are_propagated_without_being_data_errors() {
        assert!(matches!(
            process_values(["value"].into_iter(), false, |_, _| {
                Err(CliError::Usage("callback failed".to_owned()))
            }),
            Err(CliError::Usage(message)) if message == "callback failed"
        ));
        assert!(matches!(
            process_reader(Cursor::new("value\n"), false, |_, _| {
                Err(CliError::Usage("callback failed".to_owned()))
            }),
            Err(CliError::Usage(message)) if message == "callback failed"
        ));
    }

    #[test]
    fn reader_errors_are_reported_as_input_failures() {
        let bytes = [b'v', 0xff, b'\n'];
        assert!(matches!(
            process_reader(Cursor::new(bytes), false, |_, _| Ok(false)),
            Err(CliError::Input(_))
        ));
    }
}
