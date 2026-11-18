//! Types for prop AMM quotes, venues and price levels

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::{invalid_param, Result};

/// Region of a builder's data endpoint. Pick the one closest to the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Region {
    /// Europe
    #[default]
    Eu,
    /// United States
    Us,
    /// Asia-Pacific
    Ap,
}

impl Region {
    /// Base URL of Titan Builder's data endpoint in this region
    #[must_use]
    pub const fn titan_base_url(&self) -> &'static str {
        match self {
            Self::Eu => "https://eu.data.titanbuilder.xyz",
            Self::Us => "https://us.data.titanbuilder.xyz",
            Self::Ap => "https://ap.data.titanbuilder.xyz",
        }
    }
}

/// A quote for swapping through one prop AMM.
///
/// The amount is the maker's price in the builder's latest snapshot for the
/// block being built. It is only achievable when the trade is included by a
/// builder that carries the maker's price update, so treat it as conditional,
/// not firm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    /// Token sold
    pub token_in: String,
    /// Token bought
    pub token_out: String,
    /// Amount sold, in the token's smallest units (decimal string)
    #[serde(deserialize_with = "hex_or_decimal")]
    pub amount_in: String,
    /// Amount bought, in the token's smallest units (decimal string)
    #[serde(deserialize_with = "hex_or_decimal")]
    pub amount_out: String,
    /// Address of the prop AMM that produced the quote
    pub pamm: String,
    /// Router contract that executes swaps against the prop AMMs
    pub router: String,
    /// Execution block the snapshot targets
    pub block_number: u64,
    /// Beacon slot of the block being built
    pub slot: u64,
    /// Snapshot time in nanoseconds (not the block timestamp)
    pub timestamp: u64,
}

/// Every prop AMM venue the caller has access to, with its current state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VenueSnapshot {
    /// The venues
    #[serde(default)]
    pub venues: Vec<Venue>,
}

impl VenueSnapshot {
    /// Name of the maker behind `address`, matched against each venue's
    /// quote address and the pools that price from it
    #[must_use]
    pub fn maker_of(&self, address: &str) -> Option<&str> {
        self.venues
            .iter()
            .find(|venue| venue.prices(address))
            .map(|venue| venue.maker.as_str())
    }
}

/// One maker's prop AMM venue
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Venue {
    /// Maker publishing the venue
    pub maker: String,
    /// Address the maker publishes quotes under
    pub quote_address: String,
    /// `quote_address` plus any pool addresses that price from this quote
    #[serde(default)]
    pub priced_addresses: Vec<String>,
    /// Pairs the quote covers
    #[serde(default)]
    pub asset_pairs: Vec<AssetPair>,
    /// Quote sequence for each entry in `asset_pairs`. These exceed 64 bits,
    /// so they are kept as JSON numbers and may lose precision.
    #[serde(default)]
    pub pair_sequences: Vec<serde_json::Value>,
    /// Beacon slot of the block being built
    #[serde(default)]
    pub slot: Option<u64>,
    /// Execution block the state targets
    #[serde(default)]
    pub block_number: Option<u64>,
    /// Snapshot time in nanoseconds
    #[serde(default)]
    pub timestamp: Option<u64>,
    /// State override set for the venue, in the shape `eth_call` accepts as
    /// its third parameter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_override: Option<serde_json::Value>,
}

impl Venue {
    /// Whether `address` is this venue's quote address or one of its pools
    #[must_use]
    pub fn prices(&self, address: &str) -> bool {
        self.quote_address.eq_ignore_ascii_case(address)
            || self
                .priced_addresses
                .iter()
                .any(|priced| priced.eq_ignore_ascii_case(address))
    }

    /// Whether the venue quotes the pair, in either direction
    #[must_use]
    pub fn covers(&self, token_a: &str, token_b: &str) -> bool {
        self.asset_pairs.iter().any(|pair| {
            (pair.token_1.eq_ignore_ascii_case(token_a)
                && pair.token_2.eq_ignore_ascii_case(token_b))
                || (pair.token_1.eq_ignore_ascii_case(token_b)
                    && pair.token_2.eq_ignore_ascii_case(token_a))
        })
    }
}

/// A token pair a venue quotes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetPair {
    /// First token address
    pub token_1: String,
    /// Second token address
    pub token_2: String,
}

/// Price levels for every maker and market, already quoted by the builder
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookSnapshot {
    /// Beacon slot of the block being built
    #[serde(default)]
    pub slot: Option<u64>,
    /// Execution block the books target
    #[serde(default)]
    pub block: Option<u64>,
    /// Snapshot time in nanoseconds
    #[serde(default)]
    pub ts: Option<u64>,
    /// One book per maker and market
    #[serde(default)]
    pub books: Vec<Book>,
}

/// One maker's book for one market
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    /// Maker publishing the book
    pub maker: String,
    /// Address the maker publishes quotes under
    pub quote_address: String,
    /// Symbol of the base token
    pub token1: String,
    /// Symbol of the quote token
    pub token2: String,
    /// Address of the base token
    pub token1_address: String,
    /// Address of the quote token
    pub token2_address: String,
    /// Quote sequence the book was calculated from (may lose precision)
    #[serde(
        default,
        rename = "calculated_from",
        skip_serializing_if = "Option::is_none"
    )]
    pub calculated_from: Option<serde_json::Value>,
    /// Bid levels, best first
    #[serde(default)]
    pub bids: Vec<Level>,
    /// Ask levels, best first
    #[serde(default)]
    pub asks: Vec<Level>,
}

/// One price level of a book
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    /// Price of the base token in the quote token
    pub price: f64,
    /// Cumulative base-token volume available down to this level
    pub cum_volume: f64,
}

/// Validate an address argument: `0x` followed by 40 hex digits
pub(crate) fn check_address(name: &str, address: &str) -> Result<()> {
    let hex = address
        .strip_prefix("0x")
        .or_else(|| address.strip_prefix("0X"));
    match hex {
        Some(hex) if hex.len() == 40 && hex.bytes().all(|b| b.is_ascii_hexdigit()) => Ok(()),
        _ => Err(invalid_param(format!(
            "{name} must be a 0x-prefixed 20-byte address, got '{address}'"
        ))),
    }
}

/// Encode an amount for the endpoint, which takes `0x`-prefixed hex.
///
/// Accepts a decimal string in the token's smallest units, or hex that is
/// already `0x`-prefixed.
pub(crate) fn amount_to_hex(amount: &str) -> Result<String> {
    let amount = amount.trim();
    if let Some(hex) = amount
        .strip_prefix("0x")
        .or_else(|| amount.strip_prefix("0X"))
    {
        return match u128::from_str_radix(hex, 16) {
            Ok(value) if value > 0 => Ok(format!("{value:#x}")),
            _ => Err(invalid_param(format!(
                "amount must be a positive number, got '{amount}'"
            ))),
        };
    }
    match amount.parse::<u128>() {
        Ok(value) if value > 0 => Ok(format!("{value:#x}")),
        _ => Err(invalid_param(format!(
            "amount must be a positive whole number in the token's smallest units, got '{amount}'"
        ))),
    }
}

/// Parse a `0x`-prefixed hex or decimal amount into a decimal string
fn amount_to_decimal(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let value = match raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
        Some(hex) => u128::from_str_radix(hex, 16).ok()?,
        None => raw.parse::<u128>().ok()?,
    };
    Some(value.to_string())
}

fn hex_or_decimal<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    amount_to_decimal(&raw)
        .ok_or_else(|| serde::de::Error::custom(format!("not a hex or decimal amount: {raw}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_to_hex() {
        assert_eq!(
            amount_to_hex("1000000000000000000").unwrap(),
            "0xde0b6b3a7640000"
        );
        assert_eq!(
            amount_to_hex("0xDE0B6B3A7640000").unwrap(),
            "0xde0b6b3a7640000"
        );
        assert_eq!(amount_to_hex(" 10 ").unwrap(), "0xa");
        for bad in ["", "0", "0x0", "1.5", "-1", "1e18", "0xzz", "ten"] {
            assert!(amount_to_hex(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn test_check_address() {
        assert!(check_address("token", "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").is_ok());
        for bad in [
            "",
            "WETH",
            "0x1234",
            "C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        ] {
            let err = check_address("token_in", bad).unwrap_err().to_string();
            assert!(err.contains("token_in"), "{err}");
        }
    }

    #[test]
    fn test_quote_parses_hex_amounts_as_decimal() {
        let json = r#"{
            "tokenIn": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
            "tokenOut": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
            "amountIn": "0xde0b6b3a7640000",
            "amountOut": "0x6be1c920",
            "pamm": "0x00000003f1ec2379e79f58e12ec6c4f51ee92149",
            "router": "0x4ddf368080cd7946db5b459ad591c350158175e1",
            "blockNumber": 26441191,
            "slot": 15680986,
            "timestamp": 1794995849854286566
        }"#;
        let quote: Quote = serde_json::from_str(json).unwrap();
        assert_eq!(quote.amount_in, "1000000000000000000");
        assert_eq!(quote.amount_out, "1809959200");
        assert_eq!(quote.pamm, "0x00000003f1ec2379e79f58e12ec6c4f51ee92149");
        assert_eq!(quote.block_number, 26_441_191);
        assert_eq!(quote.timestamp, 1_794_995_849_854_286_566);
    }

    #[test]
    fn test_quote_rejects_a_malformed_amount() {
        let json = r#"{"tokenIn":"0xa","tokenOut":"0xb","amountIn":"0x1","amountOut":"lots",
            "pamm":"0xc","router":"0xd","blockNumber":1,"slot":1,"timestamp":1}"#;
        assert!(serde_json::from_str::<Quote>(json).is_err());
    }

    #[test]
    fn test_venue_snapshot_parses_and_finds_makers() {
        let json = r#"{"venues": [
            {"maker": "Kipseli",
             "quoteAddress": "0xc9a956b5196dfe110debe8781857bc8b97d32091",
             "pricedAddresses": ["0xc9a956b5196dfe110debe8781857bc8b97d32091",
                                 "0x71e790dd841c8a9061487cb3e78c288e75ce0b3d"],
             "assetPairs": [{"token_1": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                             "token_2": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"}],
             "pairSequences": [288472305200238444769444082],
             "slot": 15638115, "blockNumber": 26398437, "timestamp": 1794481407839599818,
             "stateOverride": {"0x48165e588ec68e724315d3e0fe7b0f3b9b6c7a6d": {"nonce": "0xd00"}}},
            {"maker": "Goliath", "quoteAddress": "0x4367985a63063fc970b009c6e599e9225181f551"}
        ]}"#;
        let snapshot: VenueSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.venues.len(), 2);
        // Matched by a pool that prices from the quote, in any letter case
        assert_eq!(
            snapshot.maker_of("0x71E790DD841C8A9061487CB3E78C288E75CE0B3D"),
            Some("Kipseli")
        );
        assert_eq!(
            snapshot.maker_of("0x4367985a63063fc970b009c6e599e9225181f551"),
            Some("Goliath")
        );
        assert_eq!(
            snapshot.maker_of("0x0000000000000000000000000000000000000001"),
            None
        );

        let kipseli = &snapshot.venues[0];
        assert!(kipseli.covers(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
        ));
        assert!(!kipseli.covers(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "0xdAC17F958D2ee523a2206206994597C13D831ec7"
        ));
        assert!(kipseli.state_override.is_some());
        // A venue with only the required fields still parses
        assert!(snapshot.venues[1].asset_pairs.is_empty());
        assert!(snapshot.venues[1].state_override.is_none());
    }

    #[test]
    fn test_book_snapshot_parses() {
        let json = r#"{"slot": 15436468, "block": 26197444, "ts": 1792061637434840374, "books": [
            {"maker": "Tempest", "quoteAddress": "0x00000003f1ec2379e79f58e12ec6c4f51ee92149",
             "token1": "WETH", "token2": "USDT",
             "token1Address": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
             "token2Address": "0xdac17f958d2ee523a2206206994597c13d831ec7",
             "calculated_from": 284752574598007134814732441,
             "bids": [{"price": 2365.0436, "cumVolume": 0.005}],
             "asks": []}
        ]}"#;
        let snapshot: BookSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.block, Some(26_197_444));
        let book = &snapshot.books[0];
        assert_eq!(book.maker, "Tempest");
        assert_eq!(
            book.token1_address,
            "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
        );
        assert_eq!(
            book.bids,
            vec![Level {
                price: 2365.0436,
                cum_volume: 0.005
            }]
        );
        assert!(book.asks.is_empty());
        assert!(book.calculated_from.is_some());
    }

    #[test]
    fn test_region_urls() {
        assert_eq!(Region::default(), Region::Eu);
        assert_eq!(
            Region::Us.titan_base_url(),
            "https://us.data.titanbuilder.xyz"
        );
        assert_eq!(
            Region::Ap.titan_base_url(),
            "https://ap.data.titanbuilder.xyz"
        );
    }
}
