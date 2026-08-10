pub fn value(value: &str) -> String {
    value.to_owned()
}

pub fn data_error(index: u64, input: &str, error: &str) -> String {
    format!("record {index}: {input}: {error}")
}
