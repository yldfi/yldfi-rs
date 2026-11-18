//! Account-related commands
//!
//! Query balances, transactions, and token transfers for addresses

use super::OutputFormat;
use crate::config::Chain;
use crate::etherscan::Client;
use crate::rpc::get_rpc_endpoint;
use crate::rpc::multicall::{selectors, MulticallBuilder, MULTICALL3_ADDRESS};
use crate::utils::address::{is_ens_name, resolve_from_book};
use crate::utils::format::format_wei_to_eth;
use alloy::primitives::Address;
use alloy::providers::Provider;
use clap::Subcommand;

/// Etherscan `endblock` meaning "up to the latest block". A fixed 99_999_999 cap
/// silently dropped history on chains past that height (Arbitrum, Optimism).
const LATEST_BLOCK: u64 = u64::MAX;
use std::io::Write;
use std::str::FromStr;

/// Resolve an address from label, ENS name, or raw address
/// Checks address book first, then tries ENS resolution
async fn resolve_address(
    input: &str,
    chain: Chain,
    quiet: bool,
) -> anyhow::Result<(Address, Option<String>)> {
    // Try sync resolution first (hex address or address book)
    match resolve_from_book(input) {
        Ok(result) => return Ok(result),
        Err(_) if is_ens_name(input) => {
            // Fall through to ENS resolution
        }
        Err(e) => return Err(e),
    }

    // Try ENS resolution if it looks like an ENS name
    // Only works on Ethereum mainnet
    if chain != Chain::Ethereum {
        return Err(anyhow::anyhow!("ENS is only available on Ethereum mainnet"));
    }

    if !quiet {
        eprintln!("Resolving ENS name {}...", input);
        let _ = std::io::stderr().flush();
    }

    // Get RPC endpoint for ENS resolution (smart selection)
    let endpoint = get_rpc_endpoint(chain)?;
    let provider = endpoint.provider();

    let address = crate::cli::ens::resolve_name(&provider, input).await?;
    Ok((address, Some(input.to_string())))
}

#[derive(Subcommand)]
pub enum AccountCommands {
    /// Get comprehensive account information (balance, recent txs, tokens)
    Info {
        /// Address to query (hex, ENS name, or address book label)
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short,
            visible_alias = "format",
            value_enum,
            default_value = "table"
        )]
        output: OutputFormat,
    },

    /// Get native token balance for address(es)
    #[command(visible_alias = "bal")]
    Balance {
        /// Address(es) to query (hex, ENS name, or address book label)
        #[arg(value_name = "ADDRESS")]
        addresses: Vec<String>,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short,
            visible_alias = "format",
            value_enum,
            default_value = "table"
        )]
        output: OutputFormat,
    },

    /// Get transaction history for an address
    #[command(visible_alias = "transactions", alias = "txlist")]
    Txs {
        /// Address to query (hex, ENS name, or address book label)
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// First block to include (inclusive)
        #[arg(long, default_value = "0")]
        start_block: u64,

        /// Last block to include (inclusive; default: latest)
        #[arg(long)]
        end_block: Option<u64>,

        /// Page number (1-indexed)
        #[arg(long, default_value = "1")]
        page: u64,

        /// Number of results per page
        #[arg(long, default_value = "50")]
        limit: u64,

        /// Sort order (asc, desc)
        #[arg(long, default_value = "desc")]
        sort: String,

        /// Fetch every page instead of one (1,000 rows per request)
        ///
        /// Walks past the explorer's 10,000-row window by moving the block
        /// range, so the full history can be read. Ignores --page and --limit.
        /// The JSON output becomes {"complete", "requests", "stopped",
        /// "count", "rows"}; check "complete" before treating it as the
        /// whole history.
        #[arg(long)]
        all: bool,

        /// With --all, stop after this many requests
        #[arg(long, default_value = "10", requires = "all")]
        max_pages: u64,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short,
            visible_alias = "format",
            value_enum,
            default_value = "table"
        )]
        output: OutputFormat,
    },

    /// Get internal transactions for an address
    InternalTxs {
        /// Address to query
        address: String,

        /// Page number
        #[arg(long, default_value = "1")]
        page: u64,

        /// Number of results per page
        #[arg(long, default_value = "50")]
        limit: u64,

        /// Fetch every page instead of one (1,000 rows per request)
        ///
        /// Walks past the explorer's 10,000-row window by moving the block
        /// range, so the full history can be read. Ignores --page and --limit.
        /// The JSON output becomes {"complete", "requests", "stopped",
        /// "count", "rows"}; check "complete" before treating it as the
        /// whole history.
        #[arg(long)]
        all: bool,

        /// With --all, stop after this many requests
        #[arg(long, default_value = "10", requires = "all")]
        max_pages: u64,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short,
            visible_alias = "format",
            value_enum,
            default_value = "table"
        )]
        output: OutputFormat,
    },

    /// Get ERC20 token transfers for an address
    Erc20 {
        /// Address to query
        address: String,

        /// Filter by token contract address
        #[arg(long)]
        token: Option<String>,

        /// Page number
        #[arg(long, default_value = "1")]
        page: u64,

        /// Number of results per page
        #[arg(long, default_value = "50")]
        limit: u64,

        /// Fetch every page instead of one (1,000 rows per request)
        ///
        /// Walks past the explorer's 10,000-row window by moving the block
        /// range, so the full history can be read. Ignores --page and --limit.
        /// The JSON output becomes {"complete", "requests", "stopped",
        /// "count", "rows"}; check "complete" before treating it as the
        /// whole history.
        #[arg(long)]
        all: bool,

        /// With --all, stop after this many requests
        #[arg(long, default_value = "10", requires = "all")]
        max_pages: u64,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short,
            visible_alias = "format",
            value_enum,
            default_value = "table"
        )]
        output: OutputFormat,
    },

    /// Who first funded an address, and with what
    ///
    /// Uses the explorer's funded-by endpoint, which needs a paid Etherscan
    /// plan. Without one, or when it fails, the earliest incoming native-token
    /// transfer is found from the address's first 100 normal and 100 internal
    /// transactions. Token transfers are not considered.
    #[command(visible_alias = "funder")]
    FundedBy {
        /// Address to query (hex, ENS name, or address book label)
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short,
            visible_alias = "format",
            value_enum,
            default_value = "table"
        )]
        output: OutputFormat,
    },

    /// Names and labels for addresses
    ///
    /// Combines the explorer's name tags and labels (an Etherscan Pro Plus
    /// endpoint) with the local address book and ethcli's built-in labels.
    /// Without that plan only the local and built-in labels are shown, with a
    /// note saying why.
    #[command(visible_alias = "nametag")]
    Labels {
        /// Addresses to look up (up to 100)
        #[arg(value_name = "ADDRESS", required = true, num_args = 1..=100)]
        addresses: Vec<String>,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short,
            visible_alias = "format",
            value_enum,
            default_value = "table"
        )]
        output: OutputFormat,
    },

    /// Get ERC721 (NFT) transfers for an address
    Erc721 {
        /// Address to query
        address: String,

        /// Filter by token contract address
        #[arg(long)]
        token: Option<String>,

        /// Page number
        #[arg(long, default_value = "1")]
        page: u64,

        /// Number of results per page
        #[arg(long, default_value = "50")]
        limit: u64,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short,
            visible_alias = "format",
            value_enum,
            default_value = "table"
        )]
        output: OutputFormat,
    },

    /// Get ERC1155 transfers for an address
    Erc1155 {
        /// Address to query
        address: String,

        /// Filter by token contract address
        #[arg(long)]
        token: Option<String>,

        /// Page number
        #[arg(long, default_value = "1")]
        page: u64,

        /// Number of results per page
        #[arg(long, default_value = "50")]
        limit: u64,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short,
            visible_alias = "format",
            value_enum,
            default_value = "table"
        )]
        output: OutputFormat,
    },

    /// Get blocks mined/validated by an address
    MinedBlocks {
        /// Address to query
        address: String,

        /// Page number
        #[arg(long, default_value = "1")]
        page: u64,

        /// Number of results per page
        #[arg(long, default_value = "50")]
        limit: u64,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short,
            visible_alias = "format",
            value_enum,
            default_value = "table"
        )]
        output: OutputFormat,
    },
}

/// Rows requested per call by `--all`
const ALL_PAGE_SIZE: u64 = 1_000;
/// Most rows the explorer serves for one query (page × offset)
const EXPLORER_WINDOW: u64 = 10_000;
/// Pause between `--all` requests, to stay under the explorer's per-second limit
const PAGE_DELAY: std::time::Duration = std::time::Duration::from_millis(350);

/// Rows read by [`fetch_all_pages`]
pub(crate) struct AllPages<T> {
    /// Rows in the order the explorer returned them
    pub rows: Vec<T>,
    /// false when the walk stopped before the end of the history
    pub complete: bool,
    /// Requests made
    pub requests: u64,
    /// Why the walk stopped early, when it did
    pub stopped: Option<String>,
}

/// Read a whole explorer history, one page of [`ALL_PAGE_SIZE`] rows at a time.
///
/// The explorer serves at most [`EXPLORER_WINDOW`] rows for a block range.
/// When a range is used up, the walk moves the range to start (ascending) or
/// end (descending) at the last block read, drops that block's rows and reads
/// it again whole, so no row is missed or repeated at the seam.
///
/// `fetch(page, start_block, end_block)` reads one page. The walk stops at the
/// first short page, after `max_pages` requests, or when a single block holds
/// more rows than one window.
pub(crate) async fn fetch_all_pages<T, F, Fut>(
    start_block: u64,
    end_block: u64,
    ascending: bool,
    max_pages: u64,
    delay: std::time::Duration,
    mut fetch: F,
    block_of: impl Fn(&T) -> u64,
) -> anyhow::Result<AllPages<T>>
where
    F: FnMut(u64, u64, u64) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<Vec<T>>>,
{
    let (mut start, mut end) = (start_block, end_block);
    let mut page = 1u64;
    let mut requests = 0u64;
    let mut rows: Vec<T> = Vec::new();
    let finish = |rows, complete, requests, stopped: Option<&str>| AllPages {
        rows,
        complete,
        requests,
        stopped: stopped.map(str::to_string),
    };

    loop {
        if page * ALL_PAGE_SIZE > EXPLORER_WINDOW {
            let Some(boundary) = rows.last().map(&block_of) else {
                return Ok(finish(rows, true, requests, None));
            };
            let stuck = if ascending {
                boundary <= start
            } else {
                boundary >= end
            };
            if stuck {
                return Ok(finish(
                    rows,
                    false,
                    requests,
                    Some("one block holds more rows than the explorer serves for one query"),
                ));
            }
            // The next range re-reads the boundary block whole
            while rows.last().is_some_and(|row| block_of(row) == boundary) {
                rows.pop();
            }
            if ascending {
                start = boundary;
            } else {
                end = boundary;
            }
            page = 1;
        }
        if requests >= max_pages {
            return Ok(finish(rows, false, requests, Some("reached --max-pages")));
        }
        if requests > 0 && !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
        let batch = fetch(page, start, end).await.or_else(|e| {
            let text = format!("{e:#}").to_ascii_lowercase();
            if text.contains("no transactions found") || text.contains("no records found") {
                Ok(Vec::new())
            } else {
                Err(e)
            }
        })?;
        requests += 1;
        let full = batch.len() as u64 >= ALL_PAGE_SIZE;
        rows.extend(batch);
        if !full {
            return Ok(finish(rows, true, requests, None));
        }
        page += 1;
    }
}

/// How an `--all` walk ended, for the JSON output
#[derive(serde::Serialize)]
struct AllPagesStatus {
    complete: bool,
    requests: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    stopped: Option<String>,
}

impl<T> AllPages<T> {
    fn status(&self) -> AllPagesStatus {
        AllPagesStatus {
            complete: self.complete,
            requests: self.requests,
            stopped: self.stopped.clone(),
        }
    }
}

/// Print a listing. After `--all` the JSON is an object that says whether
/// the history is complete (`{"complete", "requests", "stopped", "rows"}`);
/// NDJSON gets the rows and then one `{"summary": true, ...}` line. Without
/// `--all` the output is the plain array as before.
fn print_listing<T: serde::Serialize>(
    rows: &[T],
    output: OutputFormat,
    status: Option<&AllPagesStatus>,
) -> anyhow::Result<()> {
    let Some(status) = status else {
        return print_rows(rows, output);
    };
    if output.is_ndjson() {
        for row in rows {
            println!("{}", serde_json::to_string(row)?);
        }
        let mut summary = serde_json::to_value(status)?;
        summary["summary"] = serde_json::json!(true);
        summary["count"] = serde_json::json!(rows.len());
        println!("{}", serde_json::to_string(&summary)?);
    } else {
        let mut out = serde_json::to_value(status)?;
        out["count"] = serde_json::json!(rows.len());
        out["rows"] = serde_json::to_value(rows)?;
        println!("{}", serde_json::to_string_pretty(&out)?);
    }
    Ok(())
}

/// Tell the user how an `--all` walk went
fn report_all_pages<T>(fetched: &AllPages<T>, max_pages: u64, quiet: bool) {
    match (fetched.complete, &fetched.stopped) {
        (false, stopped) => eprintln!(
            "Warning: stopped after {} requests ({}); {} rows read, more exist. \
             Raise --max-pages (now {max_pages}) or narrow the block range",
            fetched.requests,
            stopped.as_deref().unwrap_or("incomplete"),
            fetched.rows.len()
        ),
        (true, _) if !quiet => eprintln!(
            "Read all {} rows in {} requests",
            fetched.rows.len(),
            fetched.requests
        ),
        (true, _) => {}
    }
}

/// A field of an explorer row as text (the API sends numbers as strings)
fn field_text(row: &serde_json::Value, key: &str) -> String {
    match &row[key] {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// The earliest successful incoming native-token transfer among explorer rows
fn earliest_incoming(rows: &[serde_json::Value], to: Address) -> Option<&serde_json::Value> {
    let wanted = format!("{to:#x}");
    rows.iter()
        .filter(|row| {
            field_text(row, "to").eq_ignore_ascii_case(&wanted)
                && field_text(row, "isError") != "1"
                && field_text(row, "value")
                    .parse::<alloy::primitives::U256>()
                    .is_ok_and(|v| !v.is_zero())
        })
        .min_by_key(|row| {
            field_text(row, "blockNumber")
                .parse::<u64>()
                .unwrap_or(u64::MAX)
        })
}

/// One funding record in the shape both sources are reported in
fn funding_record(
    funder: &str,
    tx_hash: &str,
    block: u64,
    timestamp: Option<u64>,
    value: &str,
) -> serde_json::Value {
    let mut record = serde_json::json!({
        "funder": funder,
        "tx_hash": tx_hash,
        "block": block,
        "value_wei": value,
    });
    if let Ok(wei) = value.parse::<alloy::primitives::U256>() {
        if let Ok(formatted) = alloy::primitives::utils::format_units(wei, 18) {
            record["value"] = serde_json::json!(formatted);
        }
    }
    if let Some(timestamp) = timestamp {
        record["timestamp"] = serde_json::json!(timestamp);
        record["time"] = serde_json::json!(crate::rpc::block_time::format_timestamp(timestamp));
    }
    record
}

/// Who first funded `address`: the explorer's paid endpoint, or the earliest
/// incoming native-token transfer when that is not available
async fn funded_by(client: &Client, address: Address) -> anyhow::Result<serde_json::Value> {
    let mut notes: Vec<String> = Vec::new();
    let address_text = format!("{address:#x}");

    let paid = client
        .explorer_get(&[
            ("module", "account".to_string()),
            ("action", "fundedby".to_string()),
            ("address", address_text.clone()),
        ])
        .await;
    match paid {
        Ok(result) if result.is_object() => {
            let number = |key: &str| field_text(&result, key).parse::<u64>().ok();
            return Ok(serde_json::json!({
                "address": address.to_checksum(None),
                "funded_by": funding_record(
                    &field_text(&result, "fundingAddress"),
                    &field_text(&result, "fundingTxn"),
                    number("block").unwrap_or(0),
                    number("timeStamp"),
                    &field_text(&result, "value"),
                ),
                "source": "explorer",
                "notes": notes,
            }));
        }
        Ok(_) => notes.push("The explorer's funded-by endpoint returned no record".to_string()),
        Err(e) => notes.push(format!(
            "The explorer's funded-by endpoint was not used ({e}); it needs a paid Etherscan plan \
             and works only for EOAs"
        )),
    }

    // Free fallback: the earliest incoming native-token transfer
    let history = |action: &'static str| {
        let params = vec![
            ("module", "account".to_string()),
            ("action", action.to_string()),
            ("address", address_text.clone()),
            ("startblock", "0".to_string()),
            ("endblock", "99999999999".to_string()),
            ("page", "1".to_string()),
            ("offset", "100".to_string()),
            ("sort", "asc".to_string()),
        ];
        async move { client.explorer_get(&params).await }
    };
    let normal = history("txlist")
        .await
        .map_err(|e| anyhow::anyhow!("Could not read the transaction history: {e}"))?;
    let internal = match history("txlistinternal").await {
        Ok(rows) => rows,
        Err(e) => {
            notes.push(format!("Internal transactions were not read: {e}"));
            serde_json::Value::Array(Vec::new())
        }
    };
    let normal_rows = normal.as_array().cloned().unwrap_or_default();
    let internal_rows = internal.as_array().cloned().unwrap_or_default();
    let block = |row: &serde_json::Value| field_text(row, "blockNumber").parse::<u64>().ok();
    let candidate = match (
        earliest_incoming(&normal_rows, address),
        earliest_incoming(&internal_rows, address),
    ) {
        (Some(n), Some(i)) if block(i) < block(n) => Some(i),
        (Some(n), _) => Some(n),
        (None, i) => i,
    };
    notes.push(
        "Found as the earliest incoming native-token transfer in the address's first 100 \
         normal and 100 internal transactions; token transfers are not considered"
            .to_string(),
    );
    let funded = candidate.map(|row| {
        funding_record(
            &field_text(row, "from"),
            &field_text(row, "hash"),
            block(row).unwrap_or(0),
            field_text(row, "timeStamp").parse::<u64>().ok(),
            &field_text(row, "value"),
        )
    });
    if funded.is_none() {
        notes.push("No incoming native-token transfer found".to_string());
    }
    Ok(serde_json::json!({
        "address": address.to_checksum(None),
        "funded_by": funded,
        "source": "first_incoming_transfer",
        "notes": notes,
    }))
}

fn format_funded_by(report: &serde_json::Value, symbol: &str) -> String {
    use crate::abi::revert::safe_text;
    let text = |v: &serde_json::Value| v.as_str().map(safe_text).unwrap_or_default();
    let mut out = format!("Address:     {}\n", text(&report["address"]));
    let funded = &report["funded_by"];
    if funded.is_null() {
        out.push_str("Funded by:   (not found)\n");
    } else {
        out.push_str(&format!("Funded by:   {}\n", text(&funded["funder"])));
        out.push_str(&format!(
            "Amount:      {} {symbol}\n",
            funded["value"].as_str().unwrap_or("?")
        ));
        let time = funded["time"]
            .as_str()
            .map(|t| format!(" ({t})"))
            .unwrap_or_default();
        out.push_str(&format!("Block:       {}{time}\n", funded["block"]));
        out.push_str(&format!("Transaction: {}\n", text(&funded["tx_hash"])));
    }
    let source = match report["source"].as_str() {
        Some("explorer") => "explorer funded-by endpoint",
        _ => "earliest incoming transfer (free fallback)",
    };
    out.push_str(&format!("Source:      {source}\n"));
    for note in report["notes"].as_array().into_iter().flatten() {
        out.push_str(&format!("Note: {}\n", text(note)));
    }
    out
}

/// Labels for addresses from the explorer, the address book and ethcli's
/// built-in list
async fn address_labels(client: &Client, addresses: &[Address]) -> serde_json::Value {
    let mut notes: Vec<String> = Vec::new();
    let joined = addresses
        .iter()
        .map(|a| format!("{a:#x}"))
        .collect::<Vec<_>>()
        .join(",");
    let explorer: std::collections::HashMap<String, serde_json::Value> = match client
        .explorer_get(&[
            ("module", "nametag".to_string()),
            ("action", "getaddresstag".to_string()),
            ("address", joined),
        ])
        .await
    {
        Ok(result) => result
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                let address = field_text(entry, "address").to_ascii_lowercase();
                (!address.is_empty()).then(|| (address, entry.clone()))
            })
            .collect(),
        Err(e) => {
            notes.push(format!(
                "Explorer name tags were not fetched ({e}); they need an Etherscan Pro Plus plan"
            ));
            std::collections::HashMap::new()
        }
    };

    let book = crate::config::AddressBook::load_default();
    let book_entries = book.list(None);
    let labels: Vec<serde_json::Value> = addresses
        .iter()
        .map(|address| {
            let lower = format!("{address:#x}");
            let mut entry = serde_json::json!({ "address": address.to_checksum(None) });
            if let Some(found) = explorer.get(&lower) {
                let mut tag = serde_json::Map::new();
                for key in ["nametag", "url", "shortdescription", "reputation", "labels"] {
                    let value = &found[key];
                    let empty = value.is_null()
                        || value.as_str().is_some_and(str::is_empty)
                        || value.as_array().is_some_and(Vec::is_empty);
                    if !empty {
                        tag.insert(key.to_string(), value.clone());
                    }
                }
                entry["explorer"] = serde_json::Value::Object(tag);
            }
            if let Some((label, _)) = book_entries
                .iter()
                .find(|(_, e)| e.address.eq_ignore_ascii_case(&lower))
            {
                entry["address_book"] = serde_json::json!(label);
            }
            if let Some(known) = crate::tx::addresses::get_label(address) {
                entry["known"] = serde_json::json!(known);
            }
            entry
        })
        .collect();
    serde_json::json!({ "labels": labels, "notes": notes })
}

fn format_labels(report: &serde_json::Value) -> String {
    use crate::abi::revert::safe_text;
    let text = |v: &serde_json::Value| v.as_str().map(safe_text);
    let mut out = String::new();
    for entry in report["labels"].as_array().into_iter().flatten() {
        let mut parts: Vec<String> = Vec::new();
        if let Some(tag) = text(&entry["explorer"]["nametag"]) {
            let labels = entry["explorer"]["labels"]
                .as_array()
                .map(|l| l.iter().filter_map(text).collect::<Vec<_>>().join(", "))
                .filter(|l| !l.is_empty())
                .map(|l| format!(" [{l}]"))
                .unwrap_or_default();
            parts.push(format!("{tag}{labels} (explorer)"));
        }
        if let Some(label) = text(&entry["address_book"]) {
            parts.push(format!("{label} (address book)"));
        }
        if let Some(label) = text(&entry["known"]) {
            parts.push(format!("{label} (built-in)"));
        }
        let names = if parts.is_empty() {
            "(no label)".to_string()
        } else {
            parts.join(" | ")
        };
        out.push_str(&format!(
            "{}  {names}\n",
            text(&entry["address"]).unwrap_or_default()
        ));
    }
    for note in report["notes"].as_array().into_iter().flatten() {
        out.push_str(&format!("Note: {}\n", text(note).unwrap_or_default()));
    }
    out
}

#[cfg(test)]
mod all_pages_tests {
    use super::*;

    /// A sorted history of (block, id) with several rows per block, served
    /// like the explorer: inclusive block range, 1-indexed pages, and an
    /// error past the 10,000-row window
    fn serve(
        history: &[(u64, u64)],
        ascending: bool,
        page: u64,
        start: u64,
        end: u64,
    ) -> anyhow::Result<Vec<(u64, u64)>> {
        if page * ALL_PAGE_SIZE > EXPLORER_WINDOW {
            anyhow::bail!("Result window is too large");
        }
        let mut rows: Vec<(u64, u64)> = history
            .iter()
            .copied()
            .filter(|(block, _)| *block >= start && *block <= end)
            .collect();
        if !ascending {
            rows.reverse();
        }
        let from = ((page - 1) * ALL_PAGE_SIZE) as usize;
        Ok(rows
            .into_iter()
            .skip(from)
            .take(ALL_PAGE_SIZE as usize)
            .collect())
    }

    fn history(rows_per_block: u64, blocks: u64) -> Vec<(u64, u64)> {
        (0..blocks)
            .flat_map(|b| (0..rows_per_block).map(move |i| (100 + b, b * 100_000 + i)))
            .collect()
    }

    async fn walk(history: &[(u64, u64)], ascending: bool, max_pages: u64) -> AllPages<(u64, u64)> {
        fetch_all_pages(
            0,
            u64::MAX,
            ascending,
            max_pages,
            std::time::Duration::ZERO,
            |page, start, end| {
                let result = serve(history, ascending, page, start, end);
                async move { result }
            },
            |row: &(u64, u64)| row.0,
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn reads_past_the_window_in_both_directions_without_gaps_or_repeats() {
        // 25,004 rows, 7 per block, so window seams fall inside blocks
        let all = history(7, 3572);
        assert!(all.len() > 2 * EXPLORER_WINDOW as usize);

        let up = walk(&all, true, 100).await;
        assert!(up.complete);
        assert_eq!(up.rows, all);

        let down = walk(&all, false, 100).await;
        assert!(down.complete);
        let mut expected = all.clone();
        expected.reverse();
        assert_eq!(down.rows, expected);
    }

    #[tokio::test]
    async fn stops_at_max_pages_and_on_a_block_bigger_than_the_window() {
        let all = history(7, 3572);
        let capped = walk(&all, true, 3).await;
        assert!(!capped.complete);
        assert_eq!(capped.requests, 3);
        assert_eq!(capped.rows, all[..3000].to_vec());
        assert_eq!(capped.stopped.as_deref(), Some("reached --max-pages"));

        let crowded = history(12_000, 1);
        let stuck = walk(&crowded, true, 100).await;
        assert!(!stuck.complete);
        assert!(stuck.stopped.unwrap().contains("one block"));

        let short = history(3, 10);
        let done = walk(&short, false, 100).await;
        assert!(done.complete);
        assert_eq!(done.requests, 1);
    }

    #[test]
    fn status_reports_whether_the_history_is_complete() {
        let fetched = AllPages {
            rows: vec![1u8, 2],
            complete: false,
            requests: 10,
            stopped: Some("reached --max-pages".to_string()),
        };
        let status = serde_json::to_value(fetched.status()).unwrap();
        assert_eq!(status["complete"], false);
        assert_eq!(status["requests"], 10);
        assert_eq!(status["stopped"], "reached --max-pages");
        let done = AllPages {
            rows: vec![1u8],
            complete: true,
            requests: 1,
            stopped: None,
        };
        let status = serde_json::to_value(done.status()).unwrap();
        assert_eq!(status["complete"], true);
        assert!(status.get("stopped").is_none());
    }

    #[test]
    fn earliest_incoming_skips_outgoing_failed_and_zero_value_rows() {
        let me: Address = "0x00000000000000000000000000000000000000aa"
            .parse()
            .unwrap();
        let row = |block: &str, to: &str, value: &str, error: &str| {
            serde_json::json!({"blockNumber": block, "to": to, "value": value, "isError": error,
                               "from": "0x01", "hash": format!("0x{block}")})
        };
        let rows = vec![
            row("5", "0x00000000000000000000000000000000000000bb", "9", "0"),
            row("6", "0x00000000000000000000000000000000000000AA", "0", "0"),
            row("7", "0x00000000000000000000000000000000000000aa", "9", "1"),
            row("9", "0x00000000000000000000000000000000000000aa", "3", "0"),
            row("8", "0x00000000000000000000000000000000000000aa", "4", "0"),
        ];
        assert_eq!(
            field_text(earliest_incoming(&rows, me).unwrap(), "blockNumber"),
            "8"
        );
        assert!(earliest_incoming(&rows[..3], me).is_none());
    }

    #[test]
    fn label_and_funder_text_output() {
        let labels = serde_json::json!({
            "labels": [
                {"address": "0xA", "explorer": {"nametag": "Coinbase 10", "labels": ["Coinbase", "Exchange"]},
                 "known": "Coinbase"},
                {"address": "0xB"}
            ],
            "notes": ["a note"]
        });
        let out = format_labels(&labels);
        assert!(
            out.contains("0xA  Coinbase 10 [Coinbase, Exchange] (explorer) | Coinbase (built-in)"),
            "{out}"
        );
        assert!(out.contains("0xB  (no label)"), "{out}");
        assert!(out.contains("Note: a note"), "{out}");

        let funded = serde_json::json!({
            "address": "0xA",
            "funded_by": funding_record("0xF", "0xH", 12, Some(1_704_067_200), "1000000000000000000"),
            "source": "first_incoming_transfer",
            "notes": []
        });
        let out = format_funded_by(&funded, "ETH");
        assert!(out.contains("Funded by:   0xF"), "{out}");
        assert!(
            out.contains("Amount:      1.000000000000000000 ETH"),
            "{out}"
        );
        assert!(
            out.contains("Block:       12 (2024-01-01T00:00:00Z)"),
            "{out}"
        );
        assert!(out.contains("free fallback"), "{out}");
    }
}

/// Print rows as a JSON array, or one object per line for NDJSON
fn print_rows<T: serde::Serialize>(rows: &[T], output: OutputFormat) -> anyhow::Result<()> {
    if output.is_ndjson() {
        for row in rows {
            println!("{}", serde_json::to_string(row)?);
        }
    } else {
        println!("{}", serde_json::to_string_pretty(rows)?);
    }
    Ok(())
}

/// Print one value as pretty JSON, or on a single line for NDJSON
fn print_value<T: serde::Serialize>(value: &T, output: OutputFormat) -> anyhow::Result<()> {
    if output.is_ndjson() {
        println!("{}", serde_json::to_string(value)?);
    } else {
        println!("{}", serde_json::to_string_pretty(value)?);
    }
    Ok(())
}

pub async fn handle(
    action: &AccountCommands,
    chain: Chain,
    api_key: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = Client::new(chain, api_key)?;
    // Everything except `balance` (RPC multicall) goes through the Etherscan API.
    if !matches!(action, AccountCommands::Balance { .. }) {
        client.ensure_api_supported()?;
    }

    match action {
        AccountCommands::Info { address, output } => {
            let (addr, _) = resolve_address(address, chain, quiet).await?;

            if !quiet {
                eprintln!("Fetching account info for {}...", address);
                let _ = std::io::stderr().flush();
            }

            // Fetch balance
            let balance = client.get_ether_balance_single(&addr, None).await?;
            let balance_eth = format_wei_to_eth(&balance.balance.to_string());

            // Fetch recent transactions (last 10)
            let tx_params = foundry_block_explorers::account::TxListParams {
                start_block: 0,
                end_block: LATEST_BLOCK,
                page: 1,
                offset: 10,
                sort: foundry_block_explorers::account::Sort::Desc,
            };
            let txs = client.get_transactions(&addr, Some(tx_params)).await.ok();

            // Fetch ERC20 transfers to identify token holdings
            let token_params = foundry_block_explorers::account::TxListParams {
                start_block: 0,
                end_block: LATEST_BLOCK,
                page: 1,
                offset: 20,
                sort: foundry_block_explorers::account::Sort::Desc,
            };
            let token_query = foundry_block_explorers::account::TokenQueryOption::ByAddress(addr);
            let token_transfers = client
                .get_erc20_token_transfer_events(token_query, Some(token_params))
                .await
                .ok();

            if !output.is_table() {
                let json = serde_json::json!({
                    "address": format!("{:#x}", addr),
                    "balance": balance_eth,
                    "balanceWei": balance.balance.to_string(),
                    "symbol": chain.native_symbol(),
                    "recentTransactions": txs.as_ref().map(|txs| txs.iter().take(10).map(|tx| {
                        serde_json::json!({
                            "hash": format!("{:#x}", tx.hash.value().unwrap_or_default()),
                            "blockNumber": tx.block_number.as_number().map(|n| n.to::<u64>()),
                            "from": tx.from.value().map(|a| format!("{:#x}", a)),
                            "to": tx.to.map(|a| format!("{:#x}", a)),
                            "value": tx.value.to_string(),
                            "valueEth": format_wei_to_eth(&tx.value.to_string()),
                        })
                    }).collect::<Vec<_>>()),
                    "recentTokenTransfers": token_transfers.as_ref().map(|transfers| transfers.iter().take(10).map(|tx| {
                        serde_json::json!({
                            "token": &tx.token_symbol,
                            "tokenAddress": format!("{:#x}", tx.contract_address),
                            "from": format!("{:#x}", tx.from),
                            "to": tx.to.as_ref().map(|a| format!("{:#x}", a)),
                            "value": &tx.value,
                        })
                    }).collect::<Vec<_>>()),
                });
                print_value(&json, *output)?;
            } else {
                println!("Account: {}", address);
                println!("{}", "═".repeat(60));
                println!("Balance: {} {}", balance_eth, chain.native_symbol());

                if let Some(ref txs) = txs {
                    println!("\nRecent Transactions ({}):", txs.len().min(10));
                    println!("{}", "─".repeat(60));
                    for tx in txs.iter().take(5) {
                        let hash = format!("{:#x}", tx.hash.value().unwrap_or_default());
                        let value_eth = format_wei_to_eth(&tx.value.to_string());
                        let to_addr = tx
                            .to
                            .map(|a| format!("{:#x}", a))
                            .unwrap_or_else(|| "contract".to_string());
                        let status = if tx.is_error == "0" { "✓" } else { "✗" };
                        println!(
                            "  {} {} {} {} → {}",
                            status,
                            &hash[..12],
                            value_eth,
                            chain.native_symbol(),
                            truncate_addr(&to_addr)
                        );
                    }
                    if txs.len() > 5 {
                        println!("  ... and {} more", txs.len() - 5);
                    }
                }

                if let Some(ref transfers) = token_transfers {
                    if !transfers.is_empty() {
                        println!("\nRecent Token Activity:");
                        println!("{}", "─".repeat(60));
                        // Show unique tokens
                        let mut seen_tokens: std::collections::HashSet<String> =
                            std::collections::HashSet::new();
                        for tx in transfers.iter() {
                            let key = format!(
                                "{} ({})",
                                tx.token_symbol,
                                truncate_addr(&format!("{:#x}", tx.contract_address))
                            );
                            if seen_tokens.insert(key.clone()) && seen_tokens.len() <= 5 {
                                println!("  • {}", key);
                            }
                        }
                        if seen_tokens.len() > 5 {
                            println!("  ... and {} more tokens", seen_tokens.len() - 5);
                        }
                    }
                }

                // Explorer link
                if let Some(explorer) = chain.explorer_url() {
                    println!("\nExplorer: {}/address/{}", explorer, address);
                }
            }
        }

        AccountCommands::Balance { addresses, output } => {
            if addresses.is_empty() {
                return Err(anyhow::anyhow!("At least one address is required"));
            }

            // Resolve ENS names and parse addresses
            let mut resolved: Vec<(Address, Option<String>)> = Vec::with_capacity(addresses.len());
            for addr_str in addresses {
                let (addr, ens_name) = resolve_address(addr_str, chain, quiet).await?;
                resolved.push((addr, ens_name));
            }

            if !quiet {
                eprintln!(
                    "Fetching balance for {} address(es) on {}...",
                    resolved.len(),
                    chain.display_name()
                );
                let _ = std::io::stderr().flush();
            }

            // Use RPC endpoint instead of Etherscan API
            let endpoint = get_rpc_endpoint(chain)?;
            let provider = endpoint.provider();

            if resolved.len() == 1 {
                // Single address - direct RPC call
                let (addr, label) = &resolved[0];
                let balance = provider
                    .get_balance(*addr)
                    .await
                    .map_err(|e| anyhow::anyhow!("Balance failed: {}", e))?;
                let balance_eth = format_wei_to_eth(&balance.to_string());

                if !output.is_table() {
                    let mut json = serde_json::json!({
                        "address": format!("{:#x}", addr),
                        "balance_wei": balance.to_string(),
                        "balance": balance_eth,
                        "symbol": chain.native_symbol()
                    });
                    if let Some(name) = label {
                        json["label"] = serde_json::json!(name);
                    }
                    println!("{}", json);
                } else {
                    // Show label if resolved
                    if let Some(name) = label {
                        println!("{} ({})", name, truncate_addr(&format!("{:#x}", addr)));
                    }
                    println!("{} {}", balance_eth, chain.native_symbol());
                    // Add explorer link
                    if let Some(explorer) = chain.explorer_url() {
                        println!("\nExplorer: {}/address/{:#x}", explorer, addr);
                    }
                }
            } else {
                // Multiple addresses - use Multicall3 for single RPC call
                let mut multicall = MulticallBuilder::new();
                for (addr, _) in &resolved {
                    multicall = multicall.add_call_allow_failure(
                        MULTICALL3_ADDRESS,
                        selectors::get_eth_balance(*addr),
                    );
                }

                // Execute with retry (up to 3 retries with exponential backoff)
                let multicall_results = multicall.execute_with_retry(provider, 3).await?;

                let mut results = Vec::new();
                for (i, (addr, label)) in resolved.iter().enumerate() {
                    let balance = multicall_results
                        .get(i)
                        .and_then(|r| r.decode_uint256())
                        .ok_or_else(|| anyhow::anyhow!("Balance failed for {}", addr))?;
                    results.push((*addr, label.clone(), balance));
                }

                if !output.is_table() {
                    let json_results: Vec<serde_json::Value> = results
                        .iter()
                        .map(|(addr, label, bal)| {
                            let mut json = serde_json::json!({
                                "address": format!("{:#x}", addr),
                                "balance_wei": bal.to_string(),
                                "balance": format_wei_to_eth(&bal.to_string()),
                                "symbol": chain.native_symbol()
                            });
                            if let Some(name) = label {
                                json["label"] = serde_json::json!(name);
                            }
                            json
                        })
                        .collect();
                    print_rows(&json_results, *output)?;
                } else {
                    for (addr, label, bal) in &results {
                        let eth = format_wei_to_eth(&bal.to_string());
                        if let Some(name) = label {
                            println!(
                                "{} ({}): {} {}",
                                name,
                                truncate_addr(&format!("{:#x}", addr)),
                                eth,
                                chain.native_symbol()
                            );
                        } else {
                            println!("{:#x}: {} {}", addr, eth, chain.native_symbol());
                        }
                    }
                }
            }
        }

        AccountCommands::Txs {
            address,
            start_block,
            end_block,
            page,
            limit,
            sort,
            all,
            max_pages,
            output,
        } => {
            let end_block = end_block.unwrap_or(LATEST_BLOCK);
            anyhow::ensure!(
                *start_block <= end_block,
                "start-block must not exceed end-block"
            );
            let (addr, _) = resolve_address(address, chain, quiet).await?;

            if !quiet {
                eprintln!("Fetching transactions for {}...", address);
                let _ = std::io::stderr().flush();
            }

            let params = foundry_block_explorers::account::TxListParams {
                start_block: *start_block,
                end_block,
                page: *page,
                offset: *limit,
                sort: if sort == "asc" {
                    foundry_block_explorers::account::Sort::Asc
                } else {
                    foundry_block_explorers::account::Sort::Desc
                },
            };

            let mut all_status = None;
            let txs = if *all {
                let ascending = sort == "asc";
                let client = &client;
                let fetched = fetch_all_pages(
                    *start_block,
                    end_block,
                    ascending,
                    *max_pages,
                    PAGE_DELAY,
                    move |page, start, end| async move {
                        Ok(client
                            .get_transactions(
                                &addr,
                                Some(foundry_block_explorers::account::TxListParams {
                                    start_block: start,
                                    end_block: end,
                                    page,
                                    offset: ALL_PAGE_SIZE,
                                    sort: if ascending {
                                        foundry_block_explorers::account::Sort::Asc
                                    } else {
                                        foundry_block_explorers::account::Sort::Desc
                                    },
                                }),
                            )
                            .await?)
                    },
                    |tx| {
                        tx.block_number
                            .as_number()
                            .map(|n| n.to::<u64>())
                            .unwrap_or(0)
                    },
                )
                .await?;
                report_all_pages(&fetched, *max_pages, quiet);
                all_status = Some(fetched.status());
                fetched.rows
            } else {
                client.get_transactions(&addr, Some(params)).await?
            };

            if !output.is_table() {
                // Manually construct JSON to avoid GenesisOption serialization issues
                let json_txs: Vec<serde_json::Value> = txs
                    .iter()
                    .map(|tx| {
                        serde_json::json!({
                            "hash": format!("{:#x}", tx.hash.value().unwrap_or_default()),
                            "blockNumber": tx.block_number.as_number().map(|n| n.to::<u64>()),
                            "timeStamp": tx.time_stamp,
                            "dateTime": format_timestamp(&tx.time_stamp),
                            "from": tx.from.value().map(|a| format!("{:#x}", a)),
                            "to": tx.to.map(|a| format!("{:#x}", a)),
                            "value": tx.value.to_string(),
                            "valueEth": format_wei_to_eth(&tx.value.to_string()),
                            "gas": tx.gas.to_string(),
                            "gasPrice": tx.gas_price.map(|p| p.to_string()),
                            "gasPriceGwei": tx.gas_price.map(|p| format_wei_to_gwei(&p.to_string())),
                            "gasUsed": tx.gas_used.to_string(),
                            "input": &tx.input,
                            "methodId": tx.method_id.as_ref(),
                            "functionName": tx.function_name.as_ref(),
                            "contractAddress": tx.contract_address.map(|a| format!("{:#x}", a)),
                            "isError": &tx.is_error,
                            "txreceipt_status": &tx.tx_receipt_status,
                            "confirmations": tx.confirmations,
                        })
                    })
                    .collect();
                print_listing(&json_txs, *output, all_status.as_ref())?;
            } else {
                println!("Transactions for {}", address);
                println!("{}", "─".repeat(80));
                for tx in txs.iter().take(20) {
                    let value_eth = format_wei_to_eth(&tx.value.to_string());
                    let status = if tx.is_error == "0" { "✓" } else { "✗" };
                    let to_addr = tx
                        .to
                        .map(|a| format!("{:#x}", a))
                        .unwrap_or_else(|| "contract creation".to_string());
                    let hash_str = format!("{:#x}", tx.hash.value().unwrap_or_default());
                    let block_num = tx
                        .block_number
                        .as_number()
                        .map(|n| n.to::<u64>())
                        .unwrap_or(0);
                    println!(
                        "{} {} | Block {} | {} {} → {}",
                        status,
                        &hash_str[..hash_str.len().min(12)],
                        block_num,
                        value_eth,
                        chain.native_symbol(),
                        truncate_addr(&to_addr)
                    );
                }
                if txs.len() > 20 {
                    println!("... and {} more", txs.len() - 20);
                }
                // Add explorer link
                if let Some(explorer) = chain.explorer_url() {
                    println!("\nExplorer: {}/address/{}", explorer, address);
                }
            }
        }

        AccountCommands::InternalTxs {
            address,
            page,
            limit,
            all,
            max_pages,
            output,
        } => {
            let (addr, _) = resolve_address(address, chain, quiet).await?;

            if !quiet {
                eprintln!("Fetching internal transactions for {}...", address);
                let _ = std::io::stderr().flush();
            }

            let query = foundry_block_explorers::account::InternalTxQueryOption::ByAddress(addr);
            let params = foundry_block_explorers::account::TxListParams {
                start_block: 0,
                end_block: LATEST_BLOCK,
                page: *page,
                offset: *limit,
                sort: foundry_block_explorers::account::Sort::Desc,
            };
            let mut all_status = None;
            let txs = if *all {
                let client = &client;
                let fetched = fetch_all_pages(
                    0,
                    LATEST_BLOCK,
                    false,
                    *max_pages,
                    PAGE_DELAY,
                    move |page, start, end| async move {
                        Ok(client
                            .get_internal_transactions(
                                foundry_block_explorers::account::InternalTxQueryOption::ByAddress(
                                    addr,
                                ),
                                Some(foundry_block_explorers::account::TxListParams {
                                    start_block: start,
                                    end_block: end,
                                    page,
                                    offset: ALL_PAGE_SIZE,
                                    sort: foundry_block_explorers::account::Sort::Desc,
                                }),
                            )
                            .await?)
                    },
                    |tx| {
                        tx.block_number
                            .as_number()
                            .map(|n| n.to::<u64>())
                            .unwrap_or(0)
                    },
                )
                .await?;
                report_all_pages(&fetched, *max_pages, quiet);
                all_status = Some(fetched.status());
                fetched.rows
            } else {
                client
                    .get_internal_transactions(query, Some(params))
                    .await?
            };

            if !output.is_table() {
                print_listing(&txs, *output, all_status.as_ref())?;
            } else {
                println!("Internal Transactions for {}", address);
                println!("{}", "─".repeat(80));
                for tx in txs.iter().take(20) {
                    let value_eth = format_wei_to_eth(&tx.value.to_string());
                    let to_addr = tx
                        .to
                        .value()
                        .map(|a| format!("{:#x}", a))
                        .unwrap_or_else(|| "n/a".to_string());
                    let block_num = tx
                        .block_number
                        .as_number()
                        .map(|n| n.to::<u64>())
                        .unwrap_or(0);
                    println!(
                        "Block {} | {} {} | {} → {}",
                        block_num,
                        value_eth,
                        chain.native_symbol(),
                        truncate_addr(&format!("{:#x}", tx.from)),
                        truncate_addr(&to_addr)
                    );
                }
                if txs.len() > 20 {
                    println!("... and {} more", txs.len() - 20);
                }
            }
        }

        AccountCommands::Erc20 {
            address,
            token,
            page,
            limit,
            all,
            max_pages,
            output,
        } => {
            let (addr, _) = resolve_address(address, chain, quiet).await?;

            if !quiet {
                eprintln!("Fetching ERC20 transfers for {}...", address);
                let _ = std::io::stderr().flush();
            }

            let token_filter = token
                .as_deref()
                .map(Address::from_str)
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid token address: {}", e))?;
            let token_query = move || match token_filter {
                Some(tok_addr) => {
                    foundry_block_explorers::account::TokenQueryOption::ByAddressAndContract(
                        addr, tok_addr,
                    )
                }
                None => foundry_block_explorers::account::TokenQueryOption::ByAddress(addr),
            };
            let query = token_query();

            let params = foundry_block_explorers::account::TxListParams {
                start_block: 0,
                end_block: LATEST_BLOCK,
                page: *page,
                offset: *limit,
                sort: foundry_block_explorers::account::Sort::Desc,
            };

            let mut all_status = None;
            let transfers = if *all {
                let client = &client;
                let fetched = fetch_all_pages(
                    0,
                    LATEST_BLOCK,
                    false,
                    *max_pages,
                    PAGE_DELAY,
                    move |page, start, end| async move {
                        Ok(client
                            .get_erc20_token_transfer_events(
                                token_query(),
                                Some(foundry_block_explorers::account::TxListParams {
                                    start_block: start,
                                    end_block: end,
                                    page,
                                    offset: ALL_PAGE_SIZE,
                                    sort: foundry_block_explorers::account::Sort::Desc,
                                }),
                            )
                            .await?)
                    },
                    |tx| {
                        tx.block_number
                            .as_number()
                            .map(|n| n.to::<u64>())
                            .unwrap_or(0)
                    },
                )
                .await?;
                report_all_pages(&fetched, *max_pages, quiet);
                all_status = Some(fetched.status());
                fetched.rows
            } else {
                client
                    .get_erc20_token_transfer_events(query, Some(params))
                    .await?
            };

            if !output.is_table() {
                print_listing(&transfers, *output, all_status.as_ref())?;
            } else {
                println!("ERC20 Transfers for {}", address);
                println!("{}", "─".repeat(80));
                for tx in transfers.iter().take(20) {
                    let symbol = &tx.token_symbol;
                    let to_addr = tx
                        .to
                        .as_ref()
                        .map(|a| format!("{:#x}", a))
                        .unwrap_or_else(|| "n/a".to_string());
                    let block_num = tx
                        .block_number
                        .as_number()
                        .map(|n| n.to::<u64>())
                        .unwrap_or(0);
                    // Raw units scaled by the token's decimals
                    let amount = match tx.token_decimal.parse::<u8>() {
                        Ok(decimals) => crate::utils::format::format_token_amount(
                            &tx.value.to_string(),
                            decimals,
                        ),
                        Err(_) => tx.value.to_string(),
                    };
                    println!(
                        "Block {} | {} {} | {} → {}",
                        block_num,
                        amount,
                        symbol,
                        truncate_addr(&format!("{:#x}", tx.from)),
                        truncate_addr(&to_addr)
                    );
                }
                if transfers.len() > 20 {
                    println!("... and {} more", transfers.len() - 20);
                }
            }
        }

        AccountCommands::FundedBy { address, output } => {
            let (addr, _) = resolve_address(address, chain, quiet).await?;
            if !quiet {
                eprintln!("Looking up who funded {}...", address);
            }
            let report = funded_by(&client, addr).await?;
            if output.is_table() {
                print!("{}", format_funded_by(&report, chain.native_symbol()));
            } else {
                print_value(&report, *output)?;
            }
        }

        AccountCommands::Labels { addresses, output } => {
            let mut resolved = Vec::with_capacity(addresses.len());
            for address in addresses {
                resolved.push(resolve_address(address, chain, quiet).await?.0);
            }
            let report = address_labels(&client, &resolved).await;
            if output.is_table() {
                print!("{}", format_labels(&report));
            } else {
                print_value(&report, *output)?;
            }
        }

        AccountCommands::Erc721 {
            address,
            token,
            page,
            limit,
            output,
        } => {
            let (addr, _) = resolve_address(address, chain, quiet).await?;

            if !quiet {
                eprintln!("Fetching ERC721 transfers for {}...", address);
                let _ = std::io::stderr().flush();
            }

            let query = if let Some(tok) = token {
                let tok_addr = Address::from_str(tok)
                    .map_err(|e| anyhow::anyhow!("Invalid token address: {}", e))?;
                foundry_block_explorers::account::TokenQueryOption::ByAddressAndContract(
                    addr, tok_addr,
                )
            } else {
                foundry_block_explorers::account::TokenQueryOption::ByAddress(addr)
            };

            let params = foundry_block_explorers::account::TxListParams {
                start_block: 0,
                end_block: LATEST_BLOCK,
                page: *page,
                offset: *limit,
                sort: foundry_block_explorers::account::Sort::Desc,
            };

            let transfers = client
                .get_erc721_token_transfer_events(query, Some(params))
                .await?;

            if !output.is_table() {
                print_rows(&transfers, *output)?;
            } else {
                println!("ERC721 Transfers for {}", address);
                println!("{}", "─".repeat(80));
                for tx in transfers.iter().take(20) {
                    let name = &tx.token_name;
                    let to_addr = tx
                        .to
                        .as_ref()
                        .map(|a| format!("{:#x}", a))
                        .unwrap_or_else(|| "n/a".to_string());
                    let block_num = tx
                        .block_number
                        .as_number()
                        .map(|n| n.to::<u64>())
                        .unwrap_or(0);
                    println!(
                        "Block {} | Token ID {} | {} | {} → {}",
                        block_num,
                        tx.token_id,
                        name,
                        truncate_addr(&format!("{:#x}", tx.from)),
                        truncate_addr(&to_addr)
                    );
                }
                if transfers.len() > 20 {
                    println!("... and {} more", transfers.len() - 20);
                }
            }
        }

        AccountCommands::Erc1155 {
            address,
            token,
            page,
            limit,
            output,
        } => {
            let (addr, _) = resolve_address(address, chain, quiet).await?;

            if !quiet {
                eprintln!("Fetching ERC1155 transfers for {}...", address);
                let _ = std::io::stderr().flush();
            }

            let query = if let Some(tok) = token {
                let tok_addr = Address::from_str(tok)
                    .map_err(|e| anyhow::anyhow!("Invalid token address: {}", e))?;
                foundry_block_explorers::account::TokenQueryOption::ByAddressAndContract(
                    addr, tok_addr,
                )
            } else {
                foundry_block_explorers::account::TokenQueryOption::ByAddress(addr)
            };

            let params = foundry_block_explorers::account::TxListParams {
                start_block: 0,
                end_block: LATEST_BLOCK,
                page: *page,
                offset: *limit,
                sort: foundry_block_explorers::account::Sort::Desc,
            };

            let transfers = client
                .get_erc1155_token_transfer_events(query, Some(params))
                .await?;

            if !output.is_table() {
                print_rows(&transfers, *output)?;
            } else {
                println!("ERC1155 Transfers for {}", address);
                println!("{}", "─".repeat(80));
                for tx in transfers.iter().take(20) {
                    let to_addr = tx
                        .to
                        .as_ref()
                        .map(|a| format!("{:#x}", a))
                        .unwrap_or_else(|| "n/a".to_string());
                    let block_num = tx
                        .block_number
                        .as_number()
                        .map(|n| n.to::<u64>())
                        .unwrap_or(0);
                    println!(
                        "Block {} | Token ID {} | Qty {} | {} → {}",
                        block_num,
                        tx.token_id,
                        tx.token_value,
                        truncate_addr(&format!("{:#x}", tx.from)),
                        truncate_addr(&to_addr)
                    );
                }
                if transfers.len() > 20 {
                    println!("... and {} more", transfers.len() - 20);
                }
            }
        }

        AccountCommands::MinedBlocks {
            address,
            page,
            limit,
            output,
        } => {
            let (addr, _) = resolve_address(address, chain, quiet).await?;

            if !quiet {
                eprintln!("Fetching mined blocks for {}...", address);
                let _ = std::io::stderr().flush();
            }

            let blocks = client
                .get_mined_blocks(&addr, None, Some((*page, *limit)))
                .await?;

            if !output.is_table() {
                print_rows(&blocks, *output)?;
            } else {
                println!("Blocks Mined by {}", address);
                println!("{}", "─".repeat(60));
                for block in &blocks {
                    let reward_eth = format_wei_to_eth(&block.block_reward.to_string());
                    let block_num = block
                        .block_number
                        .as_number()
                        .map(|n| n.to::<u64>())
                        .unwrap_or(0);
                    println!(
                        "Block {} | Reward: {} {}",
                        block_num,
                        reward_eth,
                        chain.native_symbol()
                    );
                }
                if blocks.len() as u64 >= *limit {
                    println!("(page {page}; use --page {} for more)", page + 1);
                }
            }
        }
    }

    Ok(())
}

/// Truncate address for display
fn truncate_addr(addr: &str) -> String {
    if addr.len() > 12 {
        format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}

/// Format wei to gwei string (shows up to 2 decimal places)
fn format_wei_to_gwei(wei: &str) -> String {
    // Parse wei and divide by 1e9 to get gwei
    wei.parse::<u128>()
        .map(|w| {
            let gwei = w / 1_000_000_000;
            let remainder = (w % 1_000_000_000) / 10_000_000; // 2 decimal places
            if remainder == 0 {
                format!("{}", gwei)
            } else {
                format!("{}.{:02}", gwei, remainder)
            }
        })
        .unwrap_or_else(|_| wei.to_string())
}

/// Format unix timestamp to human-readable date (simple implementation)
fn format_timestamp(ts: &str) -> String {
    ts.parse::<i64>()
        .ok()
        .map(|secs| {
            // Simple date calculation from unix timestamp
            let days_since_epoch = secs / 86400;
            let time_of_day = secs % 86400;
            let hours = time_of_day / 3600;
            let minutes = (time_of_day % 3600) / 60;
            let seconds = time_of_day % 60;

            // Calculate year, month, day from days since 1970-01-01
            let (year, month, day) = days_to_ymd(days_since_epoch);
            format!(
                "{:02}/{:02}/{} {:02}:{:02}:{:02} UTC",
                day, month, year, hours, minutes, seconds
            )
        })
        .unwrap_or_else(|| ts.to_string())
}

/// Convert days since epoch to year, month, day
fn days_to_ymd(days: i64) -> (i64, u32, u32) {
    // Simplified algorithm for dates after 1970
    let mut remaining = days;
    let mut year = 1970i64;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    let leap = is_leap_year(year);
    let days_in_months: [i64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for &days_in_month in &days_in_months {
        if remaining < days_in_month {
            break;
        }
        remaining -= days_in_month;
        month += 1;
    }

    (year, month, (remaining + 1) as u32)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod block_range_tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Harness {
        #[command(subcommand)]
        command: AccountCommands,
    }

    #[test]
    fn history_bounds_preserve_defaults_and_accept_single_block() {
        for (args, expected) in [
            (
                vec!["test", "txs", "0x0000000000000000000000000000000000000000"],
                (0, None),
            ),
            (
                vec![
                    "test",
                    "txs",
                    "0x0000000000000000000000000000000000000000",
                    "--start-block",
                    "26047833",
                    "--end-block",
                    "26047833",
                ],
                (26_047_833, Some(26_047_833)),
            ),
        ] {
            let AccountCommands::Txs {
                start_block,
                end_block,
                ..
            } = Harness::try_parse_from(args).unwrap().command
            else {
                panic!("expected txs")
            };
            assert_eq!((start_block, end_block), expected);
        }
    }

    #[test]
    fn history_bounds_reject_negative_numbers() {
        assert!(Harness::try_parse_from([
            "test",
            "txs",
            "0x0000000000000000000000000000000000000000",
            "--start-block",
            "-1"
        ])
        .is_err());
    }
}
