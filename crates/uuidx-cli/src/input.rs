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
    use std::io::Cursor;

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
    }
}
