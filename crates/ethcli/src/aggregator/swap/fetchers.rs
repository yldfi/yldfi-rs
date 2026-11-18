//! Fetchers for each DEX aggregator source

use super::types::NormalizedQuote;
use crate::aggregator::{get_cached_config, LatencyMeasure, SourceResult};
use secrecy::ExposeSecret;

/// Native token address placeholder used by various aggregators
pub const NATIVE_TOKEN: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

/// WETH address on Ethereum mainnet
pub const WETH_MAINNET: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

/// Convert basis points to a decimal fraction (50 bps -> 0.005)
pub fn bps_to_fraction(bps: u32) -> f64 {
    f64::from(bps) / 10_000.0
}

/// Check if an address is the native token placeholder
fn is_native_token(addr: &str) -> bool {
    addr.to_lowercase() == NATIVE_TOKEN.to_lowercase()
}

/// Wrapped native token for a chain (for protocols that don't support the
/// native token). On Gnosis the native token is xDAI, so it's WXDAI, not WETH.
fn get_weth_address(chain_id: u64) -> Option<&'static str> {
    match chain_id {
        1 => Some("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // Ethereum
        42161 => Some("0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"), // Arbitrum
        100 => Some("0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d"), // Gnosis (WXDAI)
        11155111 => Some("0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14"), // Sepolia
        _ => None,
    }
}

/// Format a source's API error for the quote output.
///
/// A service that blocks the caller's address (bot protection in front of
/// the API) answers with a whole web page. The page says nothing useful and
/// floods the output, so it is replaced by its title and a plain reason.
fn api_error(error: &impl std::fmt::Display) -> String {
    format!("API error: {}", summarize_html_error(&error.to_string()))
}

fn summarize_html_error(message: &str) -> String {
    let lower = message.to_ascii_lowercase();
    let Some(start) = lower.find("<!doctype html").or_else(|| lower.find("<html")) else {
        return message.to_string();
    };
    // to_ascii_lowercase keeps byte offsets, so `lower` indexes `message` too
    let title = lower[start..].find("<title>").and_then(|open| {
        let from = start + open + "<title>".len();
        let len = lower[from..].find("</title>")?;
        let title = message[from..from + len].trim();
        (!title.is_empty()).then(|| title.to_string())
    });
    let prefix = message[..start].trim_end().trim_end_matches('-').trim_end();
    let page = match title {
        Some(title) => format!("a web page (\"{title}\")"),
        None => "a web page".to_string(),
    };
    let reason = format!(
        "the service returned {page}, not an API response; it may be blocking this host's address"
    );
    if prefix.is_empty() {
        reason
    } else {
        format!("{prefix}: {reason}")
    }
}

/// Fetch quote from OpenOcean
///
/// `amount_in` is in smallest units (wei); openoc forwards it as the v4
/// `amountDecimals` parameter unchanged.
pub async fn fetch_openocean_quote(
    chain_id: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    _sender: Option<&str>,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedQuote> {
    let chain = match openoc::Chain::from_chain_id(chain_id) {
        Some(c) => c,
        None => {
            return SourceResult::error(
                "openocean",
                format!("Unsupported chain ID: {}", chain_id),
                measure.elapsed_ms(),
            )
        }
    };

    // Get proxy from config (respects enabled, sources, exclude_sources)
    let config = get_cached_config();
    let proxy = config
        .as_ref()
        .and_then(|c| c.proxy_for_source("openocean"));

    let client = match openoc::Client::with_config(openoc::default_config().optional_proxy(proxy)) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "openocean",
                format!("Client error: {}", e),
                measure.elapsed_ms(),
            )
        }
    };

    // openoc sends `amountDecimals`, which takes smallest units (wei) directly,
    // so no decimals lookup/conversion is needed.
    if amount_in.parse::<u128>().is_err() {
        return SourceResult::error(
            "openocean",
            format!("Invalid amount format '{}'", amount_in),
            measure.elapsed_ms(),
        );
    }

    // OpenOcean requires a gas price (`gasPriceDecimals`, in wei) - use a default value
    let request =
        openoc::QuoteRequest::new(token_in, token_out, amount_in).with_gas_price("30000000000"); // 30 gwei default

    match client.get_quote(chain, &request).await {
        Ok(response) => {
            // Use the original amount_in (wei) for consistency with other sources
            let mut quote = NormalizedQuote::new(
                "openocean",
                &response.in_token.address,
                &response.out_token.address,
                amount_in, // Use original wei amount for normalized output
                &response.out_amount,
            );

            // estimated_gas is a String, parse it directly
            if let Ok(gas) = response.estimated_gas.parse::<u64>() {
                quote = quote.with_gas(gas);
            }

            // price_impact is Option<String>, parse it to f64
            if let Some(impact) = response
                .price_impact
                .as_ref()
                .and_then(|s| s.parse::<f64>().ok())
            {
                quote = quote.with_price_impact(impact);
            }

            SourceResult::success("openocean", quote, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("openocean", api_error(&e), measure.elapsed_ms()),
    }
}

/// Fetch quote from KyberSwap
pub async fn fetch_kyber_quote(
    chain_id: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedQuote> {
    let chain = match kybr::Chain::from_chain_id(chain_id) {
        Some(c) => c,
        None => {
            return SourceResult::error(
                "kyberswap",
                format!("Unsupported chain ID: {}", chain_id),
                measure.elapsed_ms(),
            )
        }
    };

    // Get proxy from config
    let config = get_cached_config();
    let proxy = config
        .as_ref()
        .and_then(|c| c.proxy_for_source("kyberswap"));

    let client =
        match kybr::Client::with_config(crate::cli::kyber::kyber_config().optional_proxy(proxy)) {
            Ok(c) => c,
            Err(e) => {
                return SourceResult::error(
                    "kyberswap",
                    format!("Client error: {}", e),
                    measure.elapsed_ms(),
                )
            }
        };

    let request = kybr::RouteRequest::new(token_in, token_out, amount_in);

    match client.get_routes(chain, &request).await {
        Ok(route_summary) => {
            let mut quote = NormalizedQuote::new(
                "kyberswap",
                &route_summary.token_in,
                &route_summary.token_out,
                &route_summary.amount_in,
                &route_summary.amount_out,
            );

            if let Some(gas) = route_summary.gas.as_ref().and_then(|g| g.parse().ok()) {
                quote = quote.with_gas(gas);
            }

            if let Some(gas_usd) = route_summary.gas_usd.as_ref().and_then(|g| g.parse().ok()) {
                quote = quote.with_gas_usd(gas_usd);
            }

            if let Some(impact) = route_summary.price_impact_percent() {
                quote = quote.with_price_impact(impact);
            }

            // Extract route path
            if !route_summary.route.is_empty() {
                let mut protocols = Vec::new();
                for step_group in &route_summary.route {
                    for step in step_group {
                        if let Some(exchange) = &step.exchange {
                            if !protocols.contains(exchange) {
                                protocols.push(exchange.clone());
                            }
                        }
                    }
                }
                if !protocols.is_empty() {
                    quote = quote.with_protocols(protocols);
                }
            }

            SourceResult::success("kyberswap", quote, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("kyberswap", api_error(&e), measure.elapsed_ms()),
    }
}

/// Fetch quote from 0x
///
/// Uses /price endpoint for indicative prices when no sender is provided,
/// and /quote endpoint (which requires taker) when sender is available.
pub async fn fetch_zerox_quote(
    chain_id: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: Option<&str>,
    slippage_bps: u32,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedQuote> {
    // Get API key from config or env
    let config = get_cached_config();
    let api_key = config
        .as_ref()
        .and_then(|c| c.zerox.as_ref())
        .map(|z| z.api_key.expose_secret().to_string())
        .or_else(|| std::env::var("ZEROX_API_KEY").ok());

    let api_key = match api_key {
        Some(key) => key,
        None => {
            return SourceResult::error("0x", "ZEROX_API_KEY not configured", measure.elapsed_ms())
        }
    };

    let chain = match zrxswap::Chain::from_chain_id(chain_id) {
        Some(c) => c,
        None => {
            return SourceResult::error(
                "0x",
                format!("Unsupported chain ID: {}", chain_id),
                measure.elapsed_ms(),
            )
        }
    };

    // Get proxy from config
    let proxy = config.as_ref().and_then(|c| c.proxy_for_source("0x"));

    let client = match zrxswap::Client::with_config(
        zrxswap::Config::with_api_key(&api_key).optional_proxy(proxy),
    ) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error("0x", format!("Client error: {}", e), measure.elapsed_ms())
        }
    };

    let request =
        zrxswap::QuoteRequest::sell(token_in, token_out, amount_in).with_slippage_bps(slippage_bps);

    // 0x Permit2 /quote endpoint requires taker address
    // Use /price for indicative pricing when no sender provided
    if let Some(addr) = sender {
        // Full quote with transaction data
        let request = request.with_taker(addr);
        match client.get_quote(chain, &request).await {
            Ok(response) => {
                let mut quote = NormalizedQuote::new(
                    "0x",
                    token_in,
                    token_out,
                    amount_in,
                    &response.buy_amount,
                );

                if let Some(gas) = response.estimated_gas.as_ref().and_then(|g| g.parse().ok()) {
                    quote = quote.with_gas(gas);
                }

                if let Some(gas_price) = &response.gas_price {
                    quote = quote.with_gas_price(gas_price);
                }

                if let Some(tx) = &response.transaction {
                    quote = quote.with_router(&tx.to);
                    quote = quote.with_tx_data(&tx.data);
                    quote = quote.with_tx_value(&tx.value);
                }

                SourceResult::success("0x", quote, measure.elapsed_ms())
            }
            Err(e) => SourceResult::error("0x", api_error(&e), measure.elapsed_ms()),
        }
    } else {
        // Indicative price only (no transaction data)
        match client.get_price(chain, &request).await {
            Ok(response) => {
                let mut quote = NormalizedQuote::new(
                    "0x",
                    token_in,
                    token_out,
                    amount_in,
                    &response.buy_amount,
                );

                if let Some(gas) = response.estimated_gas.as_ref().and_then(|g| g.parse().ok()) {
                    quote = quote.with_gas(gas);
                }

                if let Some(gas_price) = &response.gas_price {
                    quote = quote.with_gas_price(gas_price);
                }

                // Extract route info for protocols
                if let Some(route) = &response.route {
                    let protocols: Vec<String> = route
                        .fills
                        .iter()
                        .map(|f| f.source.clone())
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect();
                    if !protocols.is_empty() {
                        quote = quote.with_protocols(protocols);
                    }
                }

                SourceResult::success("0x", quote, measure.elapsed_ms())
            }
            Err(e) => SourceResult::error("0x", api_error(&e), measure.elapsed_ms()),
        }
    }
}

/// Fetch quote from 1inch
pub async fn fetch_oneinch_quote(
    chain_id: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    _sender: Option<&str>,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedQuote> {
    // Get API key from config or env
    let config = get_cached_config();
    let api_key = config
        .as_ref()
        .and_then(|c| c.oneinch.as_ref())
        .map(|o| o.api_key.expose_secret().to_string())
        .or_else(|| std::env::var("ONEINCH_API_KEY").ok());

    let api_key = match api_key {
        Some(key) => key,
        None => {
            return SourceResult::error(
                "1inch",
                "ONEINCH_API_KEY not configured",
                measure.elapsed_ms(),
            )
        }
    };

    let chain = match oinch::Chain::from_chain_id(chain_id) {
        Some(c) => c,
        None => {
            return SourceResult::error(
                "1inch",
                format!("Unsupported chain ID: {}", chain_id),
                measure.elapsed_ms(),
            )
        }
    };

    // Get proxy from config
    let proxy = config.as_ref().and_then(|c| c.proxy_for_source("1inch"));

    let client =
        match oinch::Client::with_config(oinch::Config::new(&api_key).with_optional_proxy(proxy)) {
            Ok(c) => c,
            Err(e) => {
                return SourceResult::error(
                    "1inch",
                    format!("Client error: {}", e),
                    measure.elapsed_ms(),
                )
            }
        };

    // Include token info and protocols for more complete quote
    let request = oinch::QuoteRequest::new(token_in, token_out, amount_in)
        .with_tokens_info()
        .with_protocols_info()
        .with_gas_info();

    match client.get_quote(chain, &request).await {
        Ok(response) => {
            // Get token addresses from token info if available
            let src_addr = response
                .src_token
                .as_ref()
                .map(|t| t.address.as_str())
                .unwrap_or(token_in);
            let dst_addr = response
                .dst_token
                .as_ref()
                .map(|t| t.address.as_str())
                .unwrap_or(token_out);

            let from_amount = response.from_amount.as_deref().unwrap_or(amount_in);

            let mut quote = NormalizedQuote::new(
                "1inch",
                src_addr,
                dst_addr,
                from_amount,
                &response.to_amount,
            );

            if let Some(gas) = response.gas {
                quote = quote.with_gas(gas);
            }

            let protocol_names = response.protocol_names();
            if !protocol_names.is_empty() {
                quote = quote.with_protocols(protocol_names);
            }

            SourceResult::success("1inch", quote, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("1inch", api_error(&e), measure.elapsed_ms()),
    }
}

/// Fetch quote from CowSwap
///
/// NOTE: CowSwap doesn't support native ETH directly, so we automatically
/// substitute WETH when native token is specified.
pub async fn fetch_cowswap_quote(
    chain_id: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: Option<&str>,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedQuote> {
    let chain = match cowp::Chain::from_chain_id(chain_id) {
        Some(c) => c,
        None => {
            return SourceResult::error(
                "cowswap",
                format!("Unsupported chain ID: {}", chain_id),
                measure.elapsed_ms(),
            )
        }
    };

    // CowSwap requires a valid sender address for order creation
    let from = match sender {
        Some(addr) if !addr.is_empty() && addr != "0x0000000000000000000000000000000000000000" => {
            addr
        }
        _ => {
            return SourceResult::error(
                "cowswap",
                "CowSwap requires a valid sender address (use --sender)",
                measure.elapsed_ms(),
            )
        }
    };

    // CowSwap doesn't support native ETH - substitute WETH
    let (actual_token_in, used_weth_in) = if is_native_token(token_in) {
        match get_weth_address(chain_id) {
            Some(weth) => (weth, true),
            None => {
                return SourceResult::error(
                    "cowswap",
                    format!("No WETH address known for chain {}", chain_id),
                    measure.elapsed_ms(),
                )
            }
        }
    } else {
        (token_in, false)
    };

    let (actual_token_out, used_weth_out) = if is_native_token(token_out) {
        match get_weth_address(chain_id) {
            Some(weth) => (weth, true),
            None => {
                return SourceResult::error(
                    "cowswap",
                    format!("No WETH address known for chain {}", chain_id),
                    measure.elapsed_ms(),
                )
            }
        }
    } else {
        (token_out, false)
    };

    // Get proxy from config
    let config = get_cached_config();
    let proxy = config.as_ref().and_then(|c| c.proxy_for_source("cowswap"));

    let client = match cowp::Client::with_config(cowp::Config::new().with_optional_proxy(proxy)) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "cowswap",
                format!("Client error: {}", e),
                measure.elapsed_ms(),
            )
        }
    };

    let request = cowp::QuoteRequest::sell(actual_token_in, actual_token_out, amount_in, from);

    match client.get_quote(Some(chain), &request).await {
        Ok(response) => {
            // Use original token addresses in the quote for consistency
            // but note that CowSwap actually used WETH
            let display_token_in = if used_weth_in {
                token_in
            } else {
                &response.quote.sell_token
            };
            let display_token_out = if used_weth_out {
                token_out
            } else {
                &response.quote.buy_token
            };

            let mut quote = NormalizedQuote::new(
                "cowswap",
                display_token_in,
                display_token_out,
                &response.quote.sell_amount,
                &response.quote.buy_amount,
            );

            // CowSwap fee_amount is in sell token units, NOT gas units
            // Store it properly in fee_amount field, not estimated_gas
            if !response.quote.fee_amount.is_empty() && response.quote.fee_amount != "0" {
                quote = quote.with_fee(&response.quote.fee_amount, &response.quote.sell_token);
            }

            // Build protocol list indicating WETH substitution
            let mut protocols = vec!["CoW Protocol".to_string()];
            if used_weth_in || used_weth_out {
                protocols.push("using WETH".to_string());
            }
            quote = quote.with_protocols(protocols);

            SourceResult::success("cowswap", quote, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("cowswap", api_error(&e), measure.elapsed_ms()),
    }
}

/// Total of the fees LI.FI has already taken out of the quoted amounts, as
/// `(amount, token address)`. Fees in a different token from the first
/// included one are left out rather than added to it.
fn lifi_included_fee(fee_costs: Option<&[lfi::FeeCost]>) -> Option<(String, String)> {
    let mut included = fee_costs?.iter().filter(|fee| fee.included);
    let first = included.next()?;
    let token = first.token.address.clone();
    let mut total: u128 = first.amount.parse().ok()?;
    for fee in included {
        if fee.token.address.eq_ignore_ascii_case(&token) {
            total = total.checked_add(fee.amount.parse().ok()?)?;
        }
    }
    (total > 0).then(|| (total.to_string(), token))
}

/// Fetch quote from LI.FI (cross-chain capable)
pub async fn fetch_lifi_quote(
    chain_id: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: Option<&str>,
    slippage_bps: u32,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedQuote> {
    // LI.FI requires a valid sender address (rejects zero address)
    let from_address = match sender {
        Some(addr) if !addr.is_empty() && addr != "0x0000000000000000000000000000000000000000" => {
            addr
        }
        _ => {
            return SourceResult::error(
                "lifi",
                "LI.FI requires a valid sender address (use --sender)",
                measure.elapsed_ms(),
            )
        }
    };

    // Get proxy from config
    let config = get_cached_config();
    let proxy = config.as_ref().and_then(|c| c.proxy_for_source("lifi"));

    let lifi_config = lfi::Config::new()
        .with_optional_proxy(proxy)
        .with_optional_api_key(crate::cli::lifi::lifi_api_key());
    let client = match lfi::Client::with_config(lifi_config) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "lifi",
                format!("Client error: {}", e),
                measure.elapsed_ms(),
            )
        }
    };

    // LI.FI uses same-chain swaps when from_chain == to_chain

    let request = lfi::QuoteRequest::new(
        chain_id,
        chain_id, // Same chain for regular swaps
        token_in,
        token_out,
        amount_in,
        from_address,
    )
    // LI.FI expects slippage as a decimal fraction (0.005 = 0.5%)
    .with_slippage(bps_to_fraction(slippage_bps));

    match client.get_quote(&request).await {
        Ok(response) => {
            let estimate = &response.estimate;

            let mut quote = NormalizedQuote::new(
                "lifi",
                &response.action.from_token.address,
                &response.action.to_token.address,
                &response.action.from_amount,
                &estimate.to_amount,
            );

            // to_amount_min is a String, not Option
            quote = quote.with_min_out(&estimate.to_amount_min);

            // gas_costs is Option<Vec<GasCost>>
            if let Some(ref gas_costs) = estimate.gas_costs {
                if let Some(gas) = gas_costs.first() {
                    if let Some(ref est) = gas.estimate {
                        if let Ok(gas_val) = est.parse::<u64>() {
                            quote = quote.with_gas(gas_val);
                        }
                    }
                    if let Some(ref amount_usd) = gas.amount_usd {
                        if let Ok(gas_usd) = amount_usd.parse::<f64>() {
                            quote = quote.with_gas_usd(gas_usd);
                        }
                    }
                }
            }

            // LI.FI takes its own fee out of the input before swapping, so
            // `to_amount` is already net of it. Report it so the lower figure
            // is explained.
            if let Some((amount, token)) = lifi_included_fee(estimate.fee_costs.as_deref()) {
                quote = quote.with_fee(amount, token);
            }

            // Extract tool names (protocols)
            let protocols: Vec<String> = response
                .included_steps
                .iter()
                .map(|s| s.tool.clone())
                .collect();
            if !protocols.is_empty() {
                quote = quote.with_protocols(protocols);
            }

            // Transaction data if available (to, data, value are String not Option)
            if let Some(tx) = &response.transaction_request {
                quote = quote.with_router(&tx.to);
                quote = quote.with_tx_data(&tx.data);
                quote = quote.with_tx_value(&tx.value);
            }

            SourceResult::success("lifi", quote, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("lifi", api_error(&e), measure.elapsed_ms()),
    }
}

/// Fetch quote from Velora (ParaSwap)
pub async fn fetch_velora_quote(
    chain_id: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: Option<&str>,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedQuote> {
    let chain = match vlra::Chain::from_chain_id(chain_id) {
        Some(c) => c,
        None => {
            return SourceResult::error(
                "velora",
                format!("Unsupported chain ID: {}", chain_id),
                measure.elapsed_ms(),
            )
        }
    };

    // Get proxy from config
    let config = get_cached_config();
    let proxy = config.as_ref().and_then(|c| c.proxy_for_source("velora"));

    let client = match vlra::Client::with_config(vlra::default_config().optional_proxy(proxy)) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "velora",
                format!("Client error: {}", e),
                measure.elapsed_ms(),
            )
        }
    };

    let mut request = vlra::PriceRequest::sell(token_in, token_out, amount_in);
    // Velora requires srcDecimals/destDecimals for tokens outside its list
    // ("Token not found. Please pass srcDecimals & destDecimals"). That happens
    // on every chain, mainnet included, so always send them when the lookup
    // succeeds; a failed lookup falls back to Velora's own token list.
    let (src, dest) = futures::join!(
        crate::utils::token_meta::fetch_token_decimals(chain_id, token_in),
        crate::utils::token_meta::fetch_token_decimals(chain_id, token_out)
    );
    if let Ok(d) = src {
        request = request.with_src_decimals(d);
    }
    if let Ok(d) = dest {
        request = request.with_dest_decimals(d);
    }
    if let Some(addr) = sender {
        request = request.with_user_address(addr);
    }

    match client.get_price(chain, &request).await {
        Ok(response) => {
            let price_route = &response.price_route;
            let mut quote = NormalizedQuote::new(
                "velora",
                &price_route.src_token,
                &price_route.dest_token,
                &price_route.src_amount,
                &price_route.dest_amount,
            );

            if let Some(gas) = price_route
                .gas_cost
                .as_ref()
                .and_then(|g| g.parse::<u64>().ok())
            {
                quote = quote.with_gas(gas);
            }

            quote = quote.with_router(&price_route.contract_address);

            // Extract protocols from route (nested: route -> swaps -> swap_exchanges -> exchange)
            let protocols: Vec<String> = price_route
                .best_route
                .iter()
                .flat_map(|r| r.swaps.iter())
                .flat_map(|s| s.swap_exchanges.iter())
                .map(|e| e.exchange.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            if !protocols.is_empty() {
                quote = quote.with_protocols(protocols);
            }

            SourceResult::success("velora", quote, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("velora", api_error(&e), measure.elapsed_ms()),
    }
}

/// Label put in `protocols` for a Velora Delta quote, so its figure is not
/// read like the others: it is an order to sign, and gas is already paid.
pub const VELORA_DELTA_LABEL: &str = "Delta intent (gasless order, gas fee already deducted)";

/// Fetch a Velora Delta quote
///
/// Delta is Velora's intent protocol: the user signs an order and solvers
/// fill it, paying the gas and taking it out of the output. The amount
/// returned is therefore net of gas, unlike the Market quote from
/// [`fetch_velora_quote`], and there is no transaction to send.
pub async fn fetch_velora_delta_quote(
    chain_id: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: Option<&str>,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedQuote> {
    const SOURCE: &str = "velora-delta";

    // The endpoint rejects a request without both decimals
    let (src, dest) = futures::join!(
        crate::utils::token_meta::fetch_token_decimals(chain_id, token_in),
        crate::utils::token_meta::fetch_token_decimals(chain_id, token_out)
    );
    let (src_decimals, dest_decimals) = match (src, dest) {
        (Ok(src), Ok(dest)) => (src, dest),
        (Err(e), _) | (_, Err(e)) => {
            return SourceResult::error(
                SOURCE,
                format!("Token decimals lookup failed: {e}"),
                measure.elapsed_ms(),
            )
        }
    };

    let config = get_cached_config();
    let proxy = config.as_ref().and_then(|c| c.proxy_for_source("velora"));
    let client = match vlra::Client::with_config(vlra::default_config().optional_proxy(proxy)) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(SOURCE, format!("Client error: {e}"), measure.elapsed_ms())
        }
    };

    let mut request =
        vlra::QuoteRequest::sell(token_in, token_out, amount_in, src_decimals, dest_decimals)
            .with_mode(vlra::QuoteMode::Delta);
    if let Some(addr) = sender {
        request = request.with_user_address(addr);
    }

    match client.get_quote(chain_id, &request).await {
        Ok(response) => match response.delta {
            Some(delta) => {
                SourceResult::success(SOURCE, velora_delta_to_quote(&delta), measure.elapsed_ms())
            }
            None => SourceResult::error(
                SOURCE,
                response
                    .fallback_reason
                    .unwrap_or_else(|| "No Delta quote returned".to_string()),
                measure.elapsed_ms(),
            ),
        },
        Err(e) => SourceResult::error(SOURCE, api_error(&e), measure.elapsed_ms()),
    }
}

/// Normalize a Delta quote. Gas is not set on the quote: the fee is already
/// out of `amount_out`, and showing it as a gas cost would count it twice.
fn velora_delta_to_quote(delta: &vlra::DeltaQuote) -> NormalizedQuote {
    let route = &delta.route;
    let mut quote = NormalizedQuote::new(
        "velora-delta",
        &route.origin.input.token.address,
        &route.destination.output.token.address,
        &route.origin.input.amount,
        &route.destination.output.amount,
    )
    .with_protocols(vec![VELORA_DELTA_LABEL.to_string()]);
    if let Some(spender) = &delta.spender {
        quote = quote.with_router(spender);
    }
    quote
}

/// Fetch quote from Enso Finance
///
/// NOTE: Enso requires an API key and a valid sender address.
pub async fn fetch_enso_quote(
    chain_id: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: Option<&str>,
    slippage_bps: u32,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedQuote> {
    // Enso requires a valid sender address
    let from_address = match sender {
        Some(addr) if !addr.is_empty() && addr != "0x0000000000000000000000000000000000000000" => {
            addr
        }
        _ => {
            return SourceResult::error(
                "enso",
                "Enso requires a valid sender address (use --sender)",
                measure.elapsed_ms(),
            )
        }
    };

    // Get API key from config or env
    let config = get_cached_config();
    let api_key = config
        .as_ref()
        .and_then(|c| c.enso.as_ref())
        .map(|e| e.api_key.expose_secret().to_string())
        .or_else(|| std::env::var("ENSO_API_KEY").ok());

    let api_key = match api_key {
        Some(key) => key,
        None => {
            return SourceResult::error("enso", "ENSO_API_KEY not configured", measure.elapsed_ms())
        }
    };

    // Get proxy from config
    let proxy = config.as_ref().and_then(|c| c.proxy_for_source("enso"));

    let client = match ensof::Client::with_config(
        ensof::config_with_api_key(&api_key).optional_proxy(proxy),
    ) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "enso",
                format!("Client error: {}", e),
                measure.elapsed_ms(),
            )
        }
    };

    let request = ensof::RouteRequest::new(
        chain_id,
        from_address,
        token_in,
        token_out,
        amount_in,
        u16::try_from(slippage_bps).unwrap_or(u16::MAX), // basis points
    );

    match client.get_route(&request).await {
        Ok(response) => {
            let mut quote =
                NormalizedQuote::new("enso", token_in, token_out, amount_in, &response.amount_out);
            if let Some(min_out) = &response.min_amount_out {
                quote = quote.with_min_out(min_out);
            }

            // gas is Option<String>
            if let Some(ref gas_str) = response.gas {
                if let Ok(gas) = gas_str.parse::<u64>() {
                    quote = quote.with_gas(gas);
                }
            }

            // tx is TransactionData (not Option)
            quote = quote.with_router(&response.tx.to);
            quote = quote.with_tx_data(&response.tx.data);
            quote = quote.with_tx_value(&response.tx.value);

            // Extract protocols from route steps (route is Vec, not Option)
            if !response.route.is_empty() {
                let protocols: Vec<String> =
                    response.route.iter().map(|r| r.protocol.clone()).collect();
                quote = quote.with_protocols(protocols);
            }

            SourceResult::success("enso", quote, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("enso", api_error(&e), measure.elapsed_ms()),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn gnosis_native_wraps_to_wxdai() {
        assert_eq!(
            get_weth_address(100),
            Some("0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d")
        );
    }

    use super::*;

    #[test]
    fn bps_convert_to_fraction() {
        assert!((bps_to_fraction(50) - 0.005).abs() < 1e-12);
        assert!((bps_to_fraction(100) - 0.01).abs() < 1e-12);
    }
}

#[cfg(test)]
mod velora_delta_tests {
    use super::{velora_delta_to_quote, VELORA_DELTA_LABEL};

    #[test]
    fn delta_quote_reports_net_output_and_no_gas_or_transaction() {
        let delta: vlra::DeltaQuote = serde_json::from_value(serde_json::json!({
            "route": {
                "origin": {
                    "input": {"token": {"chainId": 1, "address": "0xin"}, "amount": "1000"},
                    "output": {"token": {"chainId": 1, "address": "0xout"}, "amount": "990"}
                },
                "destination": {
                    "input": {"token": {"chainId": 1, "address": "0xout"}, "amount": "990"},
                    "output": {"token": {"chainId": 1, "address": "0xout"}, "amount": "990"}
                },
                "fees": {"gas": {"token": {"chainId": 1, "address": "0xeeee"}, "amount": "716170",
                                 "amountUSD": "0.13"}}
            },
            "spender": "0xspender"
        }))
        .expect("valid delta quote");

        let quote = velora_delta_to_quote(&delta);
        assert_eq!(quote.source, "velora-delta");
        assert_eq!(quote.token_in, "0xin");
        assert_eq!(quote.token_out, "0xout");
        assert_eq!(quote.amount_in, "1000");
        assert_eq!(quote.amount_out, "990");
        assert_eq!(
            quote.protocols.as_deref(),
            Some(&[VELORA_DELTA_LABEL.to_string()][..])
        );
        assert_eq!(quote.router_address.as_deref(), Some("0xspender"));
        assert!(quote.estimated_gas.is_none());
        assert!(quote.gas_usd.is_none());
        assert!(quote.tx_data.is_none());
    }
}

#[cfg(test)]
mod lifi_fee_tests {
    use super::lifi_included_fee;

    fn fee(amount: &str, token: &str, included: bool) -> lfi::FeeCost {
        serde_json::from_value(serde_json::json!({
            "name": "LIFI Fixed Fee",
            "token": {
                "address": token, "chainId": 1, "symbol": "TKN", "decimals": 18,
                "name": "Token", "priceUSD": "1"
            },
            "amount": amount,
            "percentage": "0.0025",
            "included": included
        }))
        .expect("valid fee cost")
    }

    #[test]
    fn included_fees_in_one_token_are_added_up() {
        let fees = [fee("5", "0xAAA", true), fee("2", "0xaaa", true)];
        assert_eq!(
            lifi_included_fee(Some(&fees)),
            Some(("7".to_string(), "0xAAA".to_string()))
        );
    }

    #[test]
    fn fees_not_included_or_in_another_token_are_left_out() {
        let fees = [
            fee("9", "0xBBB", false),
            fee("5", "0xAAA", true),
            fee("3", "0xCCC", true),
        ];
        assert_eq!(
            lifi_included_fee(Some(&fees)),
            Some(("5".to_string(), "0xAAA".to_string()))
        );
    }

    #[test]
    fn no_included_fee_gives_none() {
        assert_eq!(lifi_included_fee(None), None);
        assert_eq!(lifi_included_fee(Some(&[])), None);
        assert_eq!(lifi_included_fee(Some(&[fee("9", "0xBBB", false)])), None);
        assert_eq!(lifi_included_fee(Some(&[fee("0", "0xAAA", true)])), None);
    }
}

#[cfg(test)]
mod api_error_tests {
    use super::{api_error, summarize_html_error};

    #[test]
    fn html_page_is_replaced_by_its_title_and_a_reason() {
        let raw = "API error: 403 - <!DOCTYPE html>\n<!--[if lt IE 7]> <html class=\"no-js\"> <![endif]-->\n<head>\n<title>Attention Required! | Cloudflare</title>\n<meta charset=\"UTF-8\" />... (truncated)";
        assert_eq!(
            summarize_html_error(raw),
            "API error: 403: the service returned a web page (\"Attention Required! | Cloudflare\"), \
             not an API response; it may be blocking this host's address"
        );
    }

    #[test]
    fn html_page_without_a_title_still_gets_a_reason() {
        assert_eq!(
            summarize_html_error("<html><body>blocked</body></html>"),
            "the service returned a web page, not an API response; it may be blocking this \
             host's address"
        );
        // A page cut off before its title closes has no usable title
        assert!(summarize_html_error("502 - <HTML><title>Bad gate").contains("a web page,"));
    }

    #[test]
    fn ordinary_errors_are_left_alone() {
        for message in [
            "Rate limited (retry after 60s)",
            "API error: 404 - {\"error\":\"Token not found\"}",
            "value < 5 and <b>bold</b>",
        ] {
            assert_eq!(summarize_html_error(message), message);
        }
        assert_eq!(api_error(&"boom"), "API error: boom");
    }
}
