use clap::ValueEnum;
use uuidx_core::IdentifierFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum IdentifierTypeArg {
    Uuid,
    #[cfg(feature = "ulid-inspect")]
    Ulid,
    Nanoid,
    Snowflake,
}

impl From<IdentifierTypeArg> for IdentifierFamily {
    fn from(value: IdentifierTypeArg) -> Self {
        match value {
            IdentifierTypeArg::Uuid => Self::Uuid,
            #[cfg(feature = "ulid-inspect")]
            IdentifierTypeArg::Ulid => Self::Ulid,
            IdentifierTypeArg::Nanoid => Self::Nanoid,
            IdentifierTypeArg::Snowflake => Self::Snowflake,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_type_arguments_map_to_core_families() {
        assert_eq!(
            IdentifierFamily::from(IdentifierTypeArg::Uuid),
            IdentifierFamily::Uuid
        );
        assert_eq!(
            IdentifierFamily::from(IdentifierTypeArg::Nanoid),
            IdentifierFamily::Nanoid
        );
        assert_eq!(
            IdentifierFamily::from(IdentifierTypeArg::Snowflake),
            IdentifierFamily::Snowflake
        );

        #[cfg(feature = "ulid-inspect")]
        assert_eq!(
            IdentifierFamily::from(IdentifierTypeArg::Ulid),
            IdentifierFamily::Ulid
        );
    }
}
