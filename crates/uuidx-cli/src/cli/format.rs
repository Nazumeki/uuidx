use clap::ValueEnum;
use uuidx_core::UuidOutputFormat;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum UuidFormatArg {
    #[value(alias = "hyphenated")]
    Canonical,
    #[value(alias = "hex")]
    Simple,
    Urn,
    #[value(alias = "brace")]
    Braced,
}

impl From<UuidFormatArg> for UuidOutputFormat {
    fn from(value: UuidFormatArg) -> Self {
        match value {
            UuidFormatArg::Canonical => Self::Canonical,
            UuidFormatArg::Simple => Self::Simple,
            UuidFormatArg::Urn => Self::Urn,
            UuidFormatArg::Braced => Self::Braced,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_arguments_map_to_core_formats() {
        assert_eq!(
            UuidOutputFormat::from(UuidFormatArg::Canonical),
            UuidOutputFormat::Canonical
        );
        assert_eq!(
            UuidOutputFormat::from(UuidFormatArg::Simple),
            UuidOutputFormat::Simple
        );
        assert_eq!(
            UuidOutputFormat::from(UuidFormatArg::Urn),
            UuidOutputFormat::Urn
        );
        assert_eq!(
            UuidOutputFormat::from(UuidFormatArg::Braced),
            UuidOutputFormat::Braced
        );
    }
}
