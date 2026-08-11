use clap::Args;

use super::UuidFormatArg;

#[derive(Debug, Args)]
#[command(next_help_heading = "Generate options")]
pub struct GenerateArgs {
    /// Generation target. Only v3, v4, v5, v6, v7, and v8 are supported.
    #[arg(value_name = "VERSION", default_value = "v7")]
    pub target: String,

    /// Number of UUIDs to generate.
    #[arg(short = 'n', long, default_value_t = 1, value_parser = positive_count)]
    pub count: u64,

    /// [v3/v5] Namespace UUID or dns, url, oid, or x500.
    #[arg(short = 's', long)]
    pub namespace: Option<String>,

    /// [v3/v5] Name bytes.
    #[arg(short = 'N', long)]
    pub name: Option<String>,

    /// [v6] Six-byte node ID as hexadecimal, or 'random' for the default.
    #[arg(short = 'd', long)]
    pub node: Option<String>,

    /// [v6/v7] RFC3339 timestamp or Unix milliseconds.
    #[arg(short = 't', long)]
    pub timestamp: Option<String>,

    /// [v8] Sixteen custom bytes as hexadecimal.
    #[arg(short = 'C', long)]
    pub custom: Option<String>,

    /// UUID output format.
    #[arg(
        short = 'F',
        long,
        value_name = "FORMAT",
        value_enum,
        default_value_t = UuidFormatArg::Canonical
    )]
    pub format: UuidFormatArg,
}

fn positive_count(value: &str) -> Result<u64, String> {
    let count = value
        .parse::<u64>()
        .map_err(|_| "count must be a positive integer".to_owned())?;
    if count == 0 {
        return Err("count must be greater than zero".to_owned());
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_count_accepts_positive_integers_and_rejects_invalid_values() {
        assert_eq!(positive_count("1").unwrap(), 1);
        assert_eq!(positive_count("42").unwrap(), 42);
        assert!(positive_count("0").is_err());
        assert!(positive_count("not-a-number").is_err());
    }
}
