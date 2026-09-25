//! Fetchers for each DEX aggregator source

use super::types::NormalizedQuote;
use crate::aggregator::{get_cached_config, LatencyMeasure, SourceResult};
use secrecy::ExposeSecret;

/// Native token address placeholder used by various aggregators
pub const NATIVE_TOKEN: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

/// WETH address on Ethereum mainnet
pub const WETH_MAINNET: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

/// Check if an address is the native token placeholder
fn is_native_token(addr: &str) -> bool {
    addr.to_lowercase() == NATIVE_TOKEN.to_lowercase()
}

/// Get WETH address for a chain (for protocols that don't support native ETH)
fn get_weth_address(chain_id: u64) -> Option<&'static str> {
    match chain_id {
        1 => Some("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // Ethereum
        42161 => Some("0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"), // Arbitrum
        100 => Some("0x6A023CCd1ff6F2045C3309768eAd9E68F978f6e1"), // Gnosis (WETH)
        11155111 => Some("0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14"), // Sepolia
        _ => None,
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
        Err(e) => SourceResult::error(
            "openocean",
            format!("API error: {}", e),
            measure.elapsed_ms(),
        ),
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

    let client = match kybr::Client::with_config(kybr::default_config().optional_proxy(proxy)) {
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

            if let Some(impact) = route_summary.price_impact {
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
        Err(e) => SourceResult::error(
            "kyberswap",
            format!("API error: {}", e),
            measure.elapsed_ms(),
        ),
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

    let request = zrxswap::QuoteRequest::sell(token_in, token_out, amount_in);

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
            Err(e) => SourceResult::error("0x", format!("API error: {}", e), measure.elapsed_ms()),
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
            Err(e) => SourceResult::error("0x", format!("API error: {}", e), measure.elapsed_ms()),
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

            // Extract protocols from route (nested: route -> step -> part)
            if let Some(ref protocols) = response.protocols {
                let mut protocol_names = Vec::new();
                for route in protocols {
                    for step in route {
                        for part in step {
                            if !protocol_names.contains(&part.name) {
                                protocol_names.push(part.name.clone());
                            }
                        }
                    }
                }
                if !protocol_names.is_empty() {
                    quote = quote.with_protocols(protocol_names);
                }
            }

            SourceResult::success("1inch", quote, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("1inch", format!("API error: {}", e), measure.elapsed_ms()),
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
    // CowSwap only supports certain chains
    let chain = match cowp::Chain::from_chain_id(chain_id) {
        Some(c) => c,
        None => {
            return SourceResult::error(
                "cowswap",
                format!(
                    "Unsupported chain ID: {} (CowSwap supports Ethereum, Gnosis, Arbitrum)",
                    chain_id
                ),
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
        Err(e) => SourceResult::error("cowswap", format!("API error: {}", e), measure.elapsed_ms()),
    }
}

/// Fetch quote from LI.FI (cross-chain capable)
pub async fn fetch_lifi_quote(
    chain_id: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: Option<&str>,
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

    let client = match lfi::Client::with_config(lfi::Config::new().with_optional_proxy(proxy)) {
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
    );

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
        Err(e) => SourceResult::error("lifi", format!("API error: {}", e), measure.elapsed_ms()),
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
        Err(e) => SourceResult::error("velora", format!("API error: {}", e), measure.elapsed_ms()),
    }
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
        100, // 1% slippage in basis points
    );

    match client.get_route(&request).await {
        Ok(response) => {
            let mut quote =
                NormalizedQuote::new("enso", token_in, token_out, amount_in, &response.amount_out);

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
        Err(e) => SourceResult::error("enso", format!("API error: {}", e), measure.elapsed_ms()),
    }
}
