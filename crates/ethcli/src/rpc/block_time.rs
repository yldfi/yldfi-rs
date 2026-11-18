//! Resolve a point in time to a block number
//!
//! Lets a `--block` argument be written as a date, a Unix timestamp or an
//! age ("7d") instead of a block number. The block returned is the latest
//! one whose timestamp is at or before the requested time, found by binary
//! search over block headers, so the answer is exact rather than estimated
//! from an average block time.

use alloy::eips::BlockNumberOrTag;
use alloy::providers::Provider;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, OffsetDateTime, PrimitiveDateTime};

/// A block chosen for a requested time
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedBlock {
    /// Block number
    pub number: u64,
    /// That block's timestamp (seconds since the Unix epoch)
    pub timestamp: u64,
}

impl ResolvedBlock {
    /// The block's time as an RFC 3339 string in UTC
    pub fn iso_time(&self) -> String {
        format_timestamp(self.timestamp)
    }
}

/// A Unix timestamp as an RFC 3339 string in UTC
pub fn format_timestamp(timestamp: u64) -> String {
    i64::try_from(timestamp)
        .ok()
        .and_then(|secs| OffsetDateTime::from_unix_timestamp(secs).ok())
        .and_then(|t| t.format(&Rfc3339).ok())
        .unwrap_or_else(|| timestamp.to_string())
}

/// Parse a time written in one of the accepted forms, as seconds since the
/// Unix epoch. `now` is the reference for ages.
///
/// Accepted:
/// - `@1700000000`: a Unix timestamp
/// - `2026-01-31`: midnight UTC on that date
/// - `2026-01-31T12:00:00Z`, `2026-01-31T12:00:00+01:00`: RFC 3339
/// - `2026-01-31 12:00` or `2026-01-31 12:00:00`: UTC
/// - `90s`, `30m`, `12h`, `7d`, `2w`: that long before `now`
///
/// Returns `None` for anything else, including block numbers, hashes and
/// tags, so a caller can fall through to its normal block parsing.
pub fn parse_time_spec(spec: &str, now: u64) -> Option<u64> {
    let spec = spec.trim();
    if let Some(unix) = spec.strip_prefix('@') {
        return unix.parse::<u64>().ok();
    }

    // An age: digits followed by one unit letter
    if let Some(unit) = spec.chars().last().filter(|c| "smhdw".contains(*c)) {
        let digits = &spec[..spec.len() - 1];
        if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
            let amount = digits.parse::<u64>().ok()?;
            let seconds = amount.checked_mul(match unit {
                's' => 1,
                'm' => 60,
                'h' => 3_600,
                'd' => 86_400,
                _ => 604_800,
            })?;
            return now.checked_sub(seconds);
        }
    }

    // Dates all start with a four-digit year and a dash, which no block
    // number, hash or tag does
    let bytes = spec.as_bytes();
    if bytes.len() < 10 || !bytes[..4].iter().all(u8::is_ascii_digit) || bytes[4] != b'-' {
        return None;
    }
    let to_unix = |t: OffsetDateTime| u64::try_from(t.unix_timestamp()).ok();
    if let Ok(t) = OffsetDateTime::parse(spec, &Rfc3339) {
        return to_unix(t);
    }
    if let Ok(date) = Date::parse(spec, format_description!("[year]-[month]-[day]")) {
        return to_unix(date.midnight().assume_utc());
    }
    for format in [
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
        format_description!("[year]-[month]-[day] [hour]:[minute]"),
        format_description!("[year]-[month]-[day]T[hour]:[minute]"),
    ] {
        if let Ok(t) = PrimitiveDateTime::parse(spec, format) {
            return to_unix(t.assume_utc());
        }
    }
    None
}

/// Timestamp of one block
async fn block_timestamp<P: Provider>(provider: &P, number: u64) -> anyhow::Result<u64> {
    let block = provider
        .get_block(BlockNumberOrTag::Number(number).into())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get block {number}: {e}"))?
        .ok_or_else(|| anyhow::anyhow!("Block {number} not found"))?;
    Ok(block.header.timestamp)
}

/// The latest block whose timestamp is at or before `timestamp`
///
/// # Errors
///
/// Fails if `timestamp` is before the first block or a header cannot be read.
pub async fn block_at_or_before<P: Provider>(
    provider: &P,
    timestamp: u64,
) -> anyhow::Result<ResolvedBlock> {
    let latest = provider
        .get_block(BlockNumberOrTag::Latest.into())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get the latest block: {e}"))?
        .ok_or_else(|| anyhow::anyhow!("Latest block not found"))?;
    let (latest_number, latest_time) = (latest.header.number, latest.header.timestamp);
    if timestamp >= latest_time {
        return Ok(ResolvedBlock {
            number: latest_number,
            timestamp: latest_time,
        });
    }

    // Some chains (Ethereum mainnet) give the genesis block a timestamp of
    // zero; the chain's real start is then block 1
    let mut first_time = block_timestamp(provider, 0).await?;
    if first_time == 0 && latest_number >= 1 {
        first_time = block_timestamp(provider, 1).await?;
    }
    if timestamp < first_time {
        anyhow::bail!(
            "{} is before the first block ({})",
            format_timestamp(timestamp),
            format_timestamp(first_time)
        );
    }

    // Invariant: time(low) <= timestamp < time(high). Block 0 satisfies it
    // even when its own timestamp is zero.
    let (mut low, mut low_time) = (0u64, 0u64);
    let mut high = latest_number;
    while high - low > 1 {
        let mid = low + (high - low) / 2;
        let mid_time = block_timestamp(provider, mid).await?;
        if mid_time <= timestamp {
            low = mid;
            low_time = mid_time;
        } else {
            high = mid;
        }
    }
    Ok(ResolvedBlock {
        number: low,
        timestamp: low_time,
    })
}

/// If `spec` is a time rather than a block, find its block
///
/// Returns `Ok(None)` when `spec` is not a time, so the caller parses it as
/// a block number, hash or tag as usual.
///
/// # Errors
///
/// Fails if the time is in the future relative to the chain or before its
/// first block, or if headers cannot be read.
pub async fn resolve_time_spec<P: Provider>(
    provider: &P,
    spec: &str,
) -> anyhow::Result<Option<ResolvedBlock>> {
    let Some(timestamp) = parse_time_spec(spec, crate::utils::unix_timestamp_secs()) else {
        return Ok(None);
    };
    block_at_or_before(provider, timestamp).await.map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = 1_800_000_000;

    #[test]
    fn parses_timestamps_dates_and_ages() {
        assert_eq!(parse_time_spec("@1700000000", NOW), Some(1_700_000_000));
        assert_eq!(parse_time_spec("2024-01-01", NOW), Some(1_704_067_200));
        assert_eq!(
            parse_time_spec("2024-01-01T12:00:00Z", NOW),
            Some(1_704_110_400)
        );
        assert_eq!(
            parse_time_spec("2024-01-01T13:00:00+01:00", NOW),
            Some(1_704_110_400)
        );
        assert_eq!(
            parse_time_spec("2024-01-01 12:00", NOW),
            Some(1_704_110_400)
        );
        assert_eq!(
            parse_time_spec("2024-01-01 12:00:30", NOW),
            Some(1_704_110_430)
        );
        assert_eq!(parse_time_spec("7d", NOW), Some(NOW - 604_800));
        assert_eq!(parse_time_spec("2w", NOW), Some(NOW - 1_209_600));
        assert_eq!(parse_time_spec("90s", NOW), Some(NOW - 90));
    }

    #[test]
    fn leaves_blocks_hashes_and_tags_alone() {
        for spec in [
            "latest",
            "safe",
            "finalized",
            "pending",
            "earliest",
            "21000000",
            "0x1406f40",
            "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3",
            "",
            "d",
            "7x",
            "2024",
            "2024-13-40",
            "1700000000",
        ] {
            assert_eq!(parse_time_spec(spec, NOW), None, "{spec}");
        }
        // An age longer than the reference time cannot be resolved
        assert_eq!(parse_time_spec("999999999w", NOW), None);
    }

    #[test]
    fn formats_timestamps_as_utc() {
        assert_eq!(format_timestamp(1_704_110_400), "2024-01-01T12:00:00Z");
    }
}
