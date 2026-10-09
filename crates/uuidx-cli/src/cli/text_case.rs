use clap::ValueEnum;
use uuidx_core::UuidTextCase;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum TextCaseArg {
    Lower,
    Upper,
}

impl From<TextCaseArg> for UuidTextCase {
    fn from(value: TextCaseArg) -> Self {
        match value {
            TextCaseArg::Lower => Self::Lower,
            TextCaseArg::Upper => Self::Upper,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_case_arguments_map_to_core_cases() {
        assert_eq!(UuidTextCase::from(TextCaseArg::Lower), UuidTextCase::Lower);
        assert_eq!(UuidTextCase::from(TextCaseArg::Upper), UuidTextCase::Upper);
    }
}
