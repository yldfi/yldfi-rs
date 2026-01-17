//! Coin prices and historical data
//!
//! Fetch current and historical token prices across multiple chains.
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> llama::error::Result<()> {
//! use llama::coins::Token;
//!
//! let client = llama::Client::new()?;
//!
//! // Get current ETH price
//! let tokens = vec![Token::coingecko("ethereum")];
//! let prices = client.coins().current(&tokens).await?;
//!
//! // Get historical price
//! let historical = client.coins().historical(1609459200, &tokens).await?;
//! # Ok(())
//! # }
//! ```

mod api;
mod types;

pub use api::CoinsApi;
pub use types::*;
