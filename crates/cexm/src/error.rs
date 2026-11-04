//! Error type for the cexm client

use crate::types::Exchange;
use thiserror::Error;

/// Errors returned by [`Client`](crate::Client)
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The HTTP client could not be built
    #[error("Failed to build HTTP client: {0}")]
    Build(String),

    /// The request failed before a response was received
    #[error("{exchange} request failed: {source}")]
    Http {
        /// Exchange that was queried
        exchange: Exchange,
        /// Underlying transport error
        #[source]
        source: reqwest::Error,
    },

    /// The exchange returned an error code (in the body or with a non-2xx status)
    #[error("{exchange} error {code}: {message}")]
    Api {
        /// Exchange that was queried
        exchange: Exchange,
        /// Exchange-specific error code (or the HTTP status if none was given)
        code: String,
        /// Exchange-provided error message
        message: String,
    },

    /// The response did not have the expected shape
    #[error("Unexpected {exchange} response: {message}")]
    Parse {
        /// Exchange that was queried
        exchange: Exchange,
        /// What was wrong with the response
        message: String,
    },

    /// The symbol is not in `BASE/QUOTE` (or `BASE/QUOTE:SETTLE`) form, or the
    /// exchange doesn't list it
    #[error("Invalid symbol {symbol:?} for {exchange}: {reason}")]
    InvalidSymbol {
        /// Exchange the symbol was meant for
        exchange: Exchange,
        /// Symbol as given
        symbol: String,
        /// Why it was rejected
        reason: String,
    },

    /// The exchange has no candles at this timeframe
    #[error("Timeframe {timeframe} is not supported by {exchange}")]
    UnsupportedTimeframe {
        /// Exchange that was queried
        exchange: Exchange,
        /// Requested timeframe
        timeframe: String,
    },
}

/// Result type for cexm operations
pub type Result<T> = std::result::Result<T, Error>;
