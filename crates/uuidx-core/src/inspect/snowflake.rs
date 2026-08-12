use thiserror::Error;

use super::BitField;

pub const TWITTER_SNOWFLAKE_EPOCH_MS: u64 = 1_288_834_974_657;

const SEQUENCE_BITS: u8 = 12;
const WORKER_BITS: u8 = 5;
const DATACENTER_BITS: u8 = 5;
const TIMESTAMP_SHIFT: u8 = SEQUENCE_BITS + WORKER_BITS + DATACENTER_BITS;
const WORKER_SHIFT: u8 = SEQUENCE_BITS;
const DATACENTER_SHIFT: u8 = SEQUENCE_BITS + WORKER_BITS;
const SEQUENCE_MASK: u64 = (1 << SEQUENCE_BITS) - 1;
const WORKER_MASK: u64 = (1 << WORKER_BITS) - 1;
const DATACENTER_MASK: u64 = (1 << DATACENTER_BITS) - 1;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SnowflakeParseError {
    #[error("Snowflake input is empty")]
    Empty,

    #[error("invalid Snowflake: expected a non-negative signed 64-bit decimal integer")]
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnowflakeInspection {
    pub normalized: String,
    pub value: u64,
    pub epoch_ms: u64,
    pub timestamp_ms: u64,
    pub datacenter_id: u8,
    pub worker_id: u8,
    pub sequence: u16,
    pub fields: Vec<BitField>,
}

pub fn inspect_snowflake(input: &str) -> Result<SnowflakeInspection, SnowflakeParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(SnowflakeParseError::Empty);
    }
    if !trimmed.bytes().all(|byte| byte.is_ascii_digit())
        || (trimmed.len() > 1 && trimmed.starts_with('0'))
    {
        return Err(SnowflakeParseError::Invalid);
    }

    let value = trimmed
        .parse::<u64>()
        .ok()
        .filter(|value| *value <= i64::MAX as u64)
        .ok_or(SnowflakeParseError::Invalid)?;
    let timestamp_delta_ms = value >> TIMESTAMP_SHIFT;

    let datacenter_id = ((value >> DATACENTER_SHIFT) & DATACENTER_MASK) as u8;
    let worker_id = ((value >> WORKER_SHIFT) & WORKER_MASK) as u8;
    let sequence = (value & SEQUENCE_MASK) as u16;

    Ok(SnowflakeInspection {
        normalized: value.to_string(),
        value,
        epoch_ms: TWITTER_SNOWFLAKE_EPOCH_MS,
        timestamp_ms: TWITTER_SNOWFLAKE_EPOCH_MS + timestamp_delta_ms,
        datacenter_id,
        worker_id,
        sequence,
        fields: vec![
            BitField {
                name: "sign".to_owned(),
                offset: 0,
                width: 1,
                value: (value >> 63).into(),
            },
            BitField {
                name: "timestamp_delta_ms".to_owned(),
                offset: 1,
                width: 41,
                value: timestamp_delta_ms.into(),
            },
            BitField {
                name: "datacenter_id".to_owned(),
                offset: 42,
                width: 5,
                value: datacenter_id.into(),
            },
            BitField {
                name: "worker_id".to_owned(),
                offset: 47,
                width: 5,
                value: worker_id.into(),
            },
            BitField {
                name: "sequence".to_owned(),
                offset: 52,
                width: 12,
                value: sequence.into(),
            },
        ],
    })
}
