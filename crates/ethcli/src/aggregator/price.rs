//! Price aggregation from multiple API sources
//!
//! This module fetches token prices from multiple sources in parallel
//! and aggregates them into a single result.

use super::{
    chain_map::normalize_chain_for_source,
    get_cached_config,
    normalize::{NormalizedPrice, PriceAggregation},
    AggregatedResult, LatencyMeasure, PriceSource, SourceResult,
};
use futures::future::join_all;
use secrecy::ExposeSecret;

/// Fetch prices from all available sources in parallel
pub async fn fetch_prices_all(
    token: &str,
    chain: &str,
) -> AggregatedResult<NormalizedPrice, PriceAggregation> {
    let sources = vec![
        PriceSource::Gecko,
        PriceSource::Llama,
        PriceSource::Alchemy,
        PriceSource::Moralis,
        PriceSource::Ccxt,
        PriceSource::Chainlink,
        PriceSource::Pyth,
    ];

    fetch_prices_parallel(token, chain, &sources).await
}

/// Fetch prices from specified sources in parallel
pub async fn fetch_prices_parallel(
    token: &str,
    chain: &str,
    sources: &[PriceSource],
) -> AggregatedResult<NormalizedPrice, PriceAggregation> {
    let start = LatencyMeasure::start();

    // Build futures for each source
    let futures: Vec<_> = sources
        .iter()
        .filter(|s| **s != PriceSource::All)
        .map(|source| {
            let token = token.to_string();
            let chain = chain.to_string();
            let source = *source;
            async move { fetch_price_from_source(&token, &chain, source).await }
        })
        .collect();

    // Execute ALL in parallel
    let results: Vec<SourceResult<NormalizedPrice>> = join_all(futures).await;

    // Calculate aggregation
    let prices: Vec<f64> = results
        .iter()
        .filter_map(|r| r.data.as_ref().map(|p| p.usd))
        .collect();

    let aggregation = PriceAggregation::from_prices(&prices).unwrap_or(PriceAggregation {
        median_usd: 0.0,
        mean_usd: 0.0,
        min_usd: 0.0,
        max_usd: 0.0,
        spread_pct: 0.0,
        sources_agreed: false,
        best_source: None,
    });

    AggregatedResult::new(aggregation, results, start.elapsed_ms())
}

/// Fetch price from a single source
pub async fn fetch_price_from_source(
    token: &str,
    chain: &str,
    source: PriceSource,
) -> SourceResult<NormalizedPrice> {
    let measure = LatencyMeasure::start();

    match source {
        PriceSource::Gecko => fetch_gecko_price(token, chain, measure).await,
        PriceSource::Llama => fetch_llama_price(token, chain, measure).await,
        PriceSource::Alchemy => fetch_alchemy_price(token, chain, measure).await,
        PriceSource::Moralis => fetch_moralis_price(token, chain, measure).await,
        PriceSource::Curve => fetch_curve_price(token, chain, measure).await,
        PriceSource::Ccxt => fetch_ccxt_price(token, measure).await,
        PriceSource::Chainlink => fetch_chainlink_price(token, chain, measure).await,
        PriceSource::Pyth => fetch_pyth_price(token, measure).await,
        PriceSource::All => SourceResult::error("all", "Use fetch_prices_all instead", 0),
    }
}

/// Fetch price from CoinGecko
async fn fetch_gecko_price(
    token: &str,
    chain: &str,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedPrice> {
    // Get API key from cached config or env
    let config = get_cached_config();
    let gecko_config = config.as_ref().and_then(|c| c.coingecko.as_ref());

    // Create gecko client with API key if available
    let client = if let Some(cfg) = gecko_config {
        if cfg.use_pro {
            if let Some(ref key) = cfg.api_key {
                match cgko::Client::pro(key.expose_secret()) {
                    Ok(c) => c,
                    Err(e) => {
                        return SourceResult::error(
                            "gecko",
                            format!("Client error: {}", e),
                            measure.elapsed_ms(),
                        )
                    }
                }
            } else {
                return SourceResult::error(
                    "gecko",
                    "Pro API configured but no API key provided",
                    measure.elapsed_ms(),
                );
            }
        } else {
            match cgko::Client::demo(cfg.api_key.as_ref().map(|s| s.expose_secret().to_string())) {
                Ok(c) => c,
                Err(e) => {
                    return SourceResult::error(
                        "gecko",
                        format!("Client error: {}", e),
                        measure.elapsed_ms(),
                    )
                }
            }
        }
    } else {
        // Try env var fallback
        let api_key = std::env::var("COINGECKO_API_KEY").ok();
        let is_pro = std::env::var("COINGECKO_PRO")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        if is_pro {
            match cgko::Client::pro(api_key.unwrap_or_default()) {
                Ok(c) => c,
                Err(e) => {
                    return SourceResult::error(
                        "gecko",
                        format!("Client error: {}", e),
                        measure.elapsed_ms(),
                    )
                }
            }
        } else {
            match cgko::Client::demo(api_key) {
                Ok(c) => c,
                Err(e) => {
                    return SourceResult::error(
                        "gecko",
                        format!("Client error: {}", e),
                        measure.elapsed_ms(),
                    )
                }
            }
        }
    };

    // Determine if this is an address or symbol
    let is_address = token.starts_with("0x");

    if is_address {
        // Use contract address endpoint
        let platform = normalize_chain_for_source("gecko", chain);

        match client
            .simple()
            .token_price(&platform, &[token], &["usd"])
            .await
        {
            Ok(response) => {
                if let Some(price_data) = response.get(&token.to_lowercase()) {
                    if let Some(usd) = price_data.get("usd").and_then(|v| v.as_f64()) {
                        let price = NormalizedPrice::new(usd);
                        SourceResult::success("gecko", price, measure.elapsed_ms())
                    } else {
                        SourceResult::error(
                            "gecko",
                            "No USD price in response",
                            measure.elapsed_ms(),
                        )
                    }
                } else {
                    SourceResult::error(
                        "gecko",
                        "Token not found in response",
                        measure.elapsed_ms(),
                    )
                }
            }
            Err(e) => {
                SourceResult::error("gecko", format!("API error: {}", e), measure.elapsed_ms())
            }
        }
    } else {
        // Use coin ID endpoint (symbol lookup)
        // Map common symbols to CoinGecko IDs (e.g., "ETH" -> "ethereum")
        let coin_id = symbol_to_coingecko_id(token);
        match client.simple().price(&[&coin_id], &["usd"]).await {
            Ok(response) => {
                if let Some(price_data) = response.get(&coin_id.to_lowercase()) {
                    if let Some(usd) = price_data.get("usd").and_then(|v| v.as_f64()) {
                        let price = NormalizedPrice::new(usd);
                        SourceResult::success("gecko", price, measure.elapsed_ms())
                    } else {
                        SourceResult::error(
                            "gecko",
                            "No USD price in response",
                            measure.elapsed_ms(),
                        )
                    }
                } else {
                    SourceResult::error(
                        "gecko",
                        format!("Coin ID '{}' not found (mapped from '{}')", coin_id, token),
                        measure.elapsed_ms(),
                    )
                }
            }
            Err(e) => {
                SourceResult::error("gecko", format!("API error: {}", e), measure.elapsed_ms())
            }
        }
    }
}

/// Fetch price from DefiLlama
async fn fetch_llama_price(
    token: &str,
    chain: &str,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedPrice> {
    let is_address = token.starts_with("0x");

    // Build the token identifier for DefiLlama
    let llama_token = if is_address {
        dllma::coins::Token::new(normalize_chain_for_source("llama", chain), token)
    } else {
        // For symbols, try common mappings via coingecko
        match token.to_lowercase().as_str() {
            "eth" | "ethereum" => dllma::coins::Token::coingecko("ethereum"),
            "btc" | "bitcoin" => dllma::coins::Token::coingecko("bitcoin"),
            _ => dllma::coins::Token::coingecko(token.to_lowercase()),
        }
    };

    // Get API key from cached config or env
    let config = get_cached_config();
    let llama_config = config.as_ref().and_then(|c| c.defillama.as_ref());

    // Create llama client with API key if available
    let client = if let Some(cfg) = llama_config {
        if let Some(ref key) = cfg.api_key {
            match dllma::Client::with_api_key(key.expose_secret()) {
                Ok(c) => c,
                Err(e) => {
                    return SourceResult::error(
                        "llama",
                        format!("Client error: {}", e),
                        measure.elapsed_ms(),
                    )
                }
            }
        } else {
            match dllma::Client::new() {
                Ok(c) => c,
                Err(e) => {
                    return SourceResult::error(
                        "llama",
                        format!("Client error: {}", e),
                        measure.elapsed_ms(),
                    )
                }
            }
        }
    } else {
        // Try env var fallback
        match std::env::var("DEFILLAMA_API_KEY") {
            Ok(key) if !key.is_empty() => match dllma::Client::with_api_key(key) {
                Ok(c) => c,
                Err(e) => {
                    return SourceResult::error(
                        "llama",
                        format!("Client error: {}", e),
                        measure.elapsed_ms(),
                    )
                }
            },
            _ => match dllma::Client::new() {
                Ok(c) => c,
                Err(e) => {
                    return SourceResult::error(
                        "llama",
                        format!("Client error: {}", e),
                        measure.elapsed_ms(),
                    )
                }
            },
        }
    };

    match client
        .coins()
        .current(std::slice::from_ref(&llama_token))
        .await
    {
        Ok(response) => {
            let coin_key = llama_token.format();
            if let Some(coin_data) = response.coins.get(&coin_key) {
                let price = NormalizedPrice::new(coin_data.price)
                    .with_confidence(coin_data.confidence.unwrap_or(1.0));
                SourceResult::success("llama", price, measure.elapsed_ms())
            } else {
                SourceResult::error("llama", "Token not found in response", measure.elapsed_ms())
            }
        }
        Err(e) => SourceResult::error("llama", format!("API error: {}", e), measure.elapsed_ms()),
    }
}

/// Fetch price from Alchemy
async fn fetch_alchemy_price(
    token: &str,
    chain: &str,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedPrice> {
    // Get API key from cached config
    let config = get_cached_config();
    let api_key = match config
        .as_ref()
        .and_then(|c| c.alchemy.as_ref())
        .map(|a| a.api_key.expose_secret().to_string())
    {
        Some(key) => key,
        None => match std::env::var("ALCHEMY_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                return SourceResult::error(
                    "alchemy",
                    "ALCHEMY_API_KEY not configured",
                    measure.elapsed_ms(),
                )
            }
        },
    };

    let network_str = normalize_chain_for_source("alchemy", chain);
    let network = crate::cli::simulate::AlchemyArgs::parse_network(&network_str);
    let client = match alcmy::Client::new(&api_key, network) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "alchemy",
                format!("Client creation error: {}", e),
                measure.elapsed_ms(),
            )
        }
    };

    let is_address = token.starts_with("0x");

    if is_address {
        // Get price by address
        match client
            .prices()
            .get_prices_by_address(&[(&network_str, token)])
            .await
        {
            Ok(response) => {
                if let Some(token_data) = response.data.first() {
                    if let Some(price_entry) =
                        token_data.prices.iter().find(|p| p.currency == "usd")
                    {
                        if let Ok(usd) = price_entry.value.parse::<f64>() {
                            let price = NormalizedPrice::new(usd);
                            return SourceResult::success("alchemy", price, measure.elapsed_ms());
                        }
                    }
                    SourceResult::error("alchemy", "No USD price in response", measure.elapsed_ms())
                } else {
                    SourceResult::error(
                        "alchemy",
                        "No price data in response",
                        measure.elapsed_ms(),
                    )
                }
            }
            Err(e) => {
                SourceResult::error("alchemy", format!("API error: {}", e), measure.elapsed_ms())
            }
        }
    } else {
        // Get price by symbol
        match client.prices().get_prices_by_symbol(&[token]).await {
            Ok(response) => {
                if let Some(token_data) = response.data.first() {
                    if let Some(price_entry) =
                        token_data.prices.iter().find(|p| p.currency == "usd")
                    {
                        if let Ok(usd) = price_entry.value.parse::<f64>() {
                            let price = NormalizedPrice::new(usd);
                            return SourceResult::success("alchemy", price, measure.elapsed_ms());
                        }
                    }
                    SourceResult::error("alchemy", "No USD price in response", measure.elapsed_ms())
                } else {
                    SourceResult::error(
                        "alchemy",
                        "No price data in response",
                        measure.elapsed_ms(),
                    )
                }
            }
            Err(e) => {
                SourceResult::error("alchemy", format!("API error: {}", e), measure.elapsed_ms())
            }
        }
    }
}

/// Fetch price from Moralis
async fn fetch_moralis_price(
    token: &str,
    chain: &str,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedPrice> {
    // Get API key from cached config
    let config = get_cached_config();
    let api_key = match config
        .as_ref()
        .and_then(|c| c.moralis.as_ref())
        .map(|m| m.api_key.expose_secret().to_string())
    {
        Some(key) => key,
        None => match std::env::var("MORALIS_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                return SourceResult::error(
                    "moralis",
                    "MORALIS_API_KEY not configured",
                    measure.elapsed_ms(),
                )
            }
        },
    };

    // Moralis only works with contract addresses, so map symbols to wrapped token addresses
    let token_address = if token.starts_with("0x") {
        token.to_string()
    } else {
        // Map native tokens to their wrapped equivalents for Moralis
        match token.to_uppercase().as_str() {
            "ETH" | "ETHEREUM" => "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(), // WETH
            "BTC" | "BITCOIN" => "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string(),  // WBTC
            "USDC" => "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            "USDT" => "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            "DAI" => "0x6B175474E89094C44Da98b954EecdeCB5BE3d823".to_string(),
            "LINK" => "0x514910771AF9Ca656af840dff83E8264EcF986CA".to_string(),
            _ => {
                // Try to resolve using our symbol_to_eth_address mapping
                match symbol_to_eth_address(token) {
                    Some(addr) => addr.to_string(),
                    None => {
                        return SourceResult::error(
                            "moralis",
                            format!("Cannot map symbol '{}' to address for Moralis", token),
                            measure.elapsed_ms(),
                        )
                    }
                }
            }
        }
    };

    let chain_name = normalize_chain_for_source("moralis", chain);
    let client = match mrls::Client::new(&api_key) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "moralis",
                format!("Client error: {}", e),
                measure.elapsed_ms(),
            )
        }
    };

    match client
        .token()
        .get_price(&token_address, Some(&chain_name))
        .await
    {
        Ok(response) => {
            if let Some(usd_price) = response.usd_price {
                let mut price = NormalizedPrice::new(usd_price);
                if let Some(change_str) = &response.percent_change_24h {
                    if let Ok(change) = change_str.parse::<f64>() {
                        price = price.with_change_24h(change);
                    }
                }
                SourceResult::success("moralis", price, measure.elapsed_ms())
            } else {
                SourceResult::error("moralis", "No USD price in response", measure.elapsed_ms())
            }
        }
        Err(e) => SourceResult::error("moralis", format!("API error: {}", e), measure.elapsed_ms()),
    }
}

/// Fetch price from Curve (for LP tokens and stable pools)
async fn fetch_curve_price(
    token: &str,
    chain: &str,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedPrice> {
    let is_address = token.starts_with("0x");

    if !is_address {
        return SourceResult::error(
            "curve",
            "Curve requires contract address",
            measure.elapsed_ms(),
        );
    }

    let chain_name = normalize_chain_for_source("curve", chain);
    let client = match crv::Client::new() {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "curve",
                format!("Client error: {}", e),
                measure.elapsed_ms(),
            )
        }
    };

    // Try to find the token in pool data
    match client.pools().get_all_on_chain(&chain_name).await {
        Ok(pools) => {
            // Search for the token in pool assets
            for pool in &pools.data.pool_data {
                if let Some(coins) = pool.coins() {
                    for (i, coin) in coins.iter().enumerate() {
                        if let Some(addr) = coin
                            .get("address")
                            .and_then(|v: &serde_json::Value| v.as_str())
                        {
                            if addr.to_lowercase() == token.to_lowercase() {
                                // Try to get the USD rate for this coin
                                if let Some(rates) = pool
                                    .0
                                    .get("usdRate")
                                    .and_then(|v: &serde_json::Value| v.as_array())
                                {
                                    if let Some(rate) =
                                        rates.get(i).and_then(|v: &serde_json::Value| v.as_f64())
                                    {
                                        let price = NormalizedPrice::new(rate);
                                        return SourceResult::success(
                                            "curve",
                                            price,
                                            measure.elapsed_ms(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            SourceResult::error(
                "curve",
                "Token not found in Curve pools",
                measure.elapsed_ms(),
            )
        }
        Err(e) => SourceResult::error("curve", format!("API error: {}", e), measure.elapsed_ms()),
    }
}

/// Fetch LP token price with Curve priority
///
/// For LP tokens, Curve is the authoritative source. Falls back to other
/// sources if Curve doesn't have the data.
pub async fn fetch_lp_price(
    lp_token: &str,
    chain: &str,
) -> AggregatedResult<NormalizedPrice, PriceAggregation> {
    let start = LatencyMeasure::start();

    let is_address = lp_token.starts_with("0x");
    if !is_address {
        // LP tokens must be addresses
        let aggregation = PriceAggregation {
            median_usd: 0.0,
            mean_usd: 0.0,
            min_usd: 0.0,
            max_usd: 0.0,
            spread_pct: 0.0,
            sources_agreed: false,
            best_source: None,
        };
        return AggregatedResult::new(
            aggregation,
            vec![SourceResult::error(
                "input",
                "LP token must be a contract address",
                0,
            )],
            start.elapsed_ms(),
        );
    }

    // Try Curve first (authoritative for Curve LP tokens)
    let curve_result = fetch_curve_lp_price(lp_token, chain).await;

    // If Curve succeeded, use it as the primary source
    if curve_result.is_success() {
        let price = curve_result.data.as_ref().map(|p| p.usd).unwrap_or(0.0);
        let aggregation = PriceAggregation {
            median_usd: price,
            mean_usd: price,
            min_usd: price,
            max_usd: price,
            spread_pct: 0.0,
            sources_agreed: true,
            best_source: Some("curve".to_string()),
        };
        return AggregatedResult::new(aggregation, vec![curve_result], start.elapsed_ms());
    }

    // Fall back to other sources (LP might not be a Curve LP)
    let sources = [
        PriceSource::Llama,
        PriceSource::Gecko,
        PriceSource::Alchemy,
        PriceSource::Moralis,
    ];

    let futures: Vec<_> = sources
        .iter()
        .map(|source| {
            let token = lp_token.to_string();
            let chain = chain.to_string();
            let source = *source;
            async move { fetch_price_from_source(&token, &chain, source).await }
        })
        .collect();

    let mut results: Vec<SourceResult<NormalizedPrice>> = join_all(futures).await;

    // Add the failed Curve result for transparency
    results.insert(0, curve_result);

    // Calculate aggregation from successful results
    let prices: Vec<f64> = results
        .iter()
        .filter_map(|r| r.data.as_ref().map(|p| p.usd))
        .collect();

    let aggregation = PriceAggregation::from_prices(&prices).unwrap_or(PriceAggregation {
        median_usd: 0.0,
        mean_usd: 0.0,
        min_usd: 0.0,
        max_usd: 0.0,
        spread_pct: 0.0,
        sources_agreed: false,
        best_source: None,
    });

    AggregatedResult::new(aggregation, results, start.elapsed_ms())
}

/// Fetch LP token price from Curve PricesClient
async fn fetch_curve_lp_price(lp_token: &str, chain: &str) -> SourceResult<NormalizedPrice> {
    let measure = LatencyMeasure::start();

    let chain_name = normalize_chain_for_source("curve", chain);
    let client = match crv::PricesClient::new() {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "curve-lp",
                format!("Client error: {}", e),
                measure.elapsed_ms(),
            );
        }
    };

    // Try to get LP token price directly from Curve Prices API
    match client.get_usd_price(&chain_name, lp_token).await {
        Ok(response) => {
            // Response is a JSON value with price data
            if let Some(price) = response
                .get("data")
                .and_then(|d| d.get("usd_price"))
                .and_then(|p| p.as_f64())
            {
                let normalized = NormalizedPrice::new(price);
                return SourceResult::success("curve-lp", normalized, measure.elapsed_ms());
            }

            // Try alternative response structure
            if let Some(price) = response.get("usd_price").and_then(|p| p.as_f64()) {
                let normalized = NormalizedPrice::new(price);
                return SourceResult::success("curve-lp", normalized, measure.elapsed_ms());
            }

            SourceResult::error(
                "curve-lp",
                "Could not parse LP price from response",
                measure.elapsed_ms(),
            )
        }
        Err(e) => {
            // If direct price lookup fails, try OHLC data for the LP token
            match client.get_lp_ohlc(&chain_name, lp_token).await {
                Ok(ohlc_response) => {
                    // Extract latest close price from OHLC data
                    if let Some(data) = ohlc_response.get("data").and_then(|d| d.as_array()) {
                        if let Some(latest) = data.last() {
                            // OHLC format: [timestamp, open, high, low, close, ...]
                            if let Some(close) = latest.get(4).and_then(|c| c.as_f64()) {
                                let normalized = NormalizedPrice::new(close);
                                return SourceResult::success(
                                    "curve-lp",
                                    normalized,
                                    measure.elapsed_ms(),
                                );
                            }
                        }
                    }
                    SourceResult::error("curve-lp", "No OHLC data available", measure.elapsed_ms())
                }
                Err(_) => SourceResult::error(
                    "curve-lp",
                    format!("LP token not found: {}", e),
                    measure.elapsed_ms(),
                ),
            }
        }
    }
}

/// Check if a token identifier is an address
pub fn is_token_address(token: &str) -> bool {
    token.starts_with("0x") && token.len() == 42
}

/// Map common token symbols to addresses on Ethereum
pub fn symbol_to_eth_address(symbol: &str) -> Option<&'static str> {
    match symbol.to_uppercase().as_str() {
        "USDC" => Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
        "USDT" => Some("0xdAC17F958D2ee523a2206206994597C13D831ec7"),
        "DAI" => Some("0x6B175474E89094C44Da98b954EecdeCB5BE3d823"),
        "WETH" => Some("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
        "WBTC" => Some("0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"),
        "LINK" => Some("0x514910771AF9Ca656af840dff83E8264EcF986CA"),
        "UNI" => Some("0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984"),
        "AAVE" => Some("0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9"),
        "MKR" => Some("0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2"),
        "COMP" => Some("0xc00e94Cb662C3520282E6f5717214004A7f26888"),
        "CRV" => Some("0xD533a949740bb3306d119CC777fa900bA034cd52"),
        "SNX" => Some("0xC011a73ee8576Fb46F5E1c5751cA3B9Fe0af2a6F"),
        "YFI" => Some("0x0bc529c00C6401aEF6D220BE8C6Ea1667F6Ad93e"),
        "SUSHI" => Some("0x6B3595068778DD592e39A122f4f5a5cF09C90fE2"),
        "LDO" => Some("0x5A98FcBEA516Cf06857215779Fd812CA3beF1B32"),
        "RPL" => Some("0xD33526068D116cE69F19A9ee46F0bd304F21A51f"),
        "SHIB" => Some("0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE"),
        "PEPE" => Some("0x6982508145454Ce325dDbE47a25d4ec3d2311933"),
        _ => None,
    }
}

/// Reverse map: Ethereum address to token symbol (for CCXT compatibility)
pub fn eth_address_to_symbol(address: &str) -> Option<&'static str> {
    match address.to_lowercase().as_str() {
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48" => Some("USDC"),
        "0xdac17f958d2ee523a2206206994597c13d831ec7" => Some("USDT"),
        "0x6b175474e89094c44da98b954eecdecb5be3d823" => Some("DAI"),
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2" => Some("WETH"),
        "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599" => Some("WBTC"),
        "0x514910771af9ca656af840dff83e8264ecf986ca" => Some("LINK"),
        "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984" => Some("UNI"),
        "0x7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9" => Some("AAVE"),
        "0x9f8f72aa9304c8b593d555f12ef6589cc3a579a2" => Some("MKR"),
        "0xc00e94cb662c3520282e6f5717214004a7f26888" => Some("COMP"),
        "0xd533a949740bb3306d119cc777fa900ba034cd52" => Some("CRV"),
        "0xc011a73ee8576fb46f5e1c5751ca3b9fe0af2a6f" => Some("SNX"),
        "0x0bc529c00c6401aef6d220be8c6ea1667f6ad93e" => Some("YFI"),
        "0x6b3595068778dd592e39a122f4f5a5cf09c90fe2" => Some("SUSHI"),
        "0x5a98fcbea516cf06857215779fd812ca3bef1b32" => Some("LDO"),
        "0xd33526068d116ce69f19a9ee46f0bd304f21a51f" => Some("RPL"),
        "0x95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce" => Some("SHIB"),
        "0x6982508145454ce325ddbe47a25d4ec3d2311933" => Some("PEPE"),
        _ => None,
    }
}

/// Map common token symbols to CoinGecko coin IDs
pub fn symbol_to_coingecko_id(symbol: &str) -> String {
    match symbol.to_uppercase().as_str() {
        // Native tokens
        "ETH" | "ETHEREUM" => "ethereum".to_string(),
        "BTC" | "BITCOIN" => "bitcoin".to_string(),
        "BNB" => "binancecoin".to_string(),
        "SOL" | "SOLANA" => "solana".to_string(),
        "AVAX" | "AVALANCHE" => "avalanche-2".to_string(),
        "MATIC" | "POLYGON" => "matic-network".to_string(),
        "DOT" | "POLKADOT" => "polkadot".to_string(),
        "ADA" | "CARDANO" => "cardano".to_string(),
        "XRP" | "RIPPLE" => "ripple".to_string(),
        "DOGE" | "DOGECOIN" => "dogecoin".to_string(),
        "TRX" | "TRON" => "tron".to_string(),
        "ATOM" | "COSMOS" => "cosmos".to_string(),
        "LTC" | "LITECOIN" => "litecoin".to_string(),
        "ARB" | "ARBITRUM" => "arbitrum".to_string(),
        "OP" | "OPTIMISM" => "optimism".to_string(),
        "NEAR" => "near".to_string(),
        "FTM" | "FANTOM" => "fantom".to_string(),

        // Stablecoins
        "USDC" => "usd-coin".to_string(),
        "USDT" | "TETHER" => "tether".to_string(),
        "DAI" => "dai".to_string(),
        "BUSD" => "binance-usd".to_string(),
        "FRAX" => "frax".to_string(),
        "LUSD" => "liquity-usd".to_string(),
        "TUSD" => "true-usd".to_string(),
        "USDP" => "paxos-standard".to_string(),
        "GUSD" => "gemini-dollar".to_string(),

        // Wrapped tokens
        "WETH" => "weth".to_string(),
        "WBTC" => "wrapped-bitcoin".to_string(),
        "STETH" => "staked-ether".to_string(),
        "WSTETH" => "wrapped-steth".to_string(),
        "RETH" => "rocket-pool-eth".to_string(),
        "CBETH" => "coinbase-wrapped-staked-eth".to_string(),
        "FRXETH" => "frax-ether".to_string(),

        // DeFi tokens
        "LINK" | "CHAINLINK" => "chainlink".to_string(),
        "UNI" | "UNISWAP" => "uniswap".to_string(),
        "AAVE" => "aave".to_string(),
        "MKR" | "MAKER" => "maker".to_string(),
        "COMP" | "COMPOUND" => "compound-governance-token".to_string(),
        "CRV" | "CURVE" => "curve-dao-token".to_string(),
        "SNX" | "SYNTHETIX" => "havven".to_string(),
        "YFI" | "YEARN" => "yearn-finance".to_string(),
        "SUSHI" | "SUSHISWAP" => "sushi".to_string(),
        "LDO" | "LIDO" => "lido-dao".to_string(),
        "RPL" | "ROCKETPOOL" => "rocket-pool".to_string(),
        "GRT" | "GRAPH" => "the-graph".to_string(),
        "ENS" => "ethereum-name-service".to_string(),
        "CVX" | "CONVEX" => "convex-finance".to_string(),
        "BAL" | "BALANCER" => "balancer".to_string(),
        "1INCH" => "1inch".to_string(),
        "DYDX" => "dydx".to_string(),
        "GMX" => "gmx".to_string(),

        // Meme tokens
        "SHIB" | "SHIBA" => "shiba-inu".to_string(),
        "PEPE" => "pepe".to_string(),
        "FLOKI" => "floki".to_string(),
        "BONK" => "bonk".to_string(),
        "WIF" => "dogwifcoin".to_string(),

        // Other popular tokens
        "APE" | "APECOIN" => "apecoin".to_string(),
        "IMX" | "IMMUTABLE" => "immutable-x".to_string(),
        "SAND" | "SANDBOX" => "the-sandbox".to_string(),
        "MANA" | "DECENTRALAND" => "decentraland".to_string(),
        "AXS" | "AXIE" => "axie-infinity".to_string(),
        "BLUR" => "blur".to_string(),
        "FXS" => "frax-share".to_string(),
        "PENDLE" => "pendle".to_string(),
        "ENA" | "ETHENA" => "ethena".to_string(),

        // Default: lowercase the symbol as fallback
        _ => symbol.to_lowercase(),
    }
}

/// Fetch price from CCXT exchanges (aggregates from Binance, Bitget, OKX)
///
/// CCXT uses trading pair format like "BTC/USDT", so we need to convert
/// token symbols to trading pairs against USDT.
///
/// Uses tokio::select! to race exchanges in parallel, returning the first successful result.
async fn fetch_ccxt_price(token: &str, measure: LatencyMeasure) -> SourceResult<NormalizedPrice> {
    use ccxt_rust::prelude::{Binance, Bitget, Exchange as ExchangeTrait, Okx};
    use num_traits::ToPrimitive;
    use tokio::select;

    // CCXT requires trading pairs, not contract addresses
    // If we receive an address, try to reverse-map it to a symbol
    let symbol = if token.starts_with("0x") {
        match eth_address_to_symbol(token) {
            Some(sym) => sym.to_string(),
            None => {
                return SourceResult::error(
                    "ccxt",
                    format!("Cannot map address {} to trading symbol", token),
                    measure.elapsed_ms(),
                );
            }
        }
    } else {
        token.to_uppercase()
    };

    // Convert symbol to trading pair (assume USDT quote)
    let trading_pair = format!("{}/USDT", symbol);
    let pair_clone1 = trading_pair.clone();
    let pair_clone2 = trading_pair.clone();
    let pair_clone3 = trading_pair.clone();

    // Helper to fetch from a specific exchange
    async fn fetch_from_binance(
        trading_pair: String,
    ) -> Option<(f64, f64, &'static str)> {
        let exchange = Binance::builder().build().ok()?;
        exchange.load_markets(false).await.ok()?;
        let ticker = ExchangeTrait::fetch_ticker(&exchange, &trading_pair)
            .await
            .ok()?;
        let price = ticker.last?.0.to_f64()?;
        let change = ticker.percentage.and_then(|p| p.to_f64()).unwrap_or(0.0);
        Some((price, change, "ccxt-binance"))
    }

    async fn fetch_from_bitget(
        trading_pair: String,
    ) -> Option<(f64, f64, &'static str)> {
        let exchange = Bitget::builder().build().ok()?;
        exchange.load_markets(false).await.ok()?;
        let ticker = ExchangeTrait::fetch_ticker(&exchange, &trading_pair)
            .await
            .ok()?;
        let price = ticker.last?.0.to_f64()?;
        let change = ticker.percentage.and_then(|p| p.to_f64()).unwrap_or(0.0);
        Some((price, change, "ccxt-bitget"))
    }

    async fn fetch_from_okx(
        trading_pair: String,
    ) -> Option<(f64, f64, &'static str)> {
        let exchange = Okx::builder().build().ok()?;
        exchange.load_markets(false).await.ok()?;
        let ticker = ExchangeTrait::fetch_ticker(&exchange, &trading_pair)
            .await
            .ok()?;
        let price = ticker.last?.0.to_f64()?;
        let change = ticker.percentage.and_then(|p| p.to_f64()).unwrap_or(0.0);
        Some((price, change, "ccxt-okx"))
    }

    // Race all exchanges in parallel, return first successful result
    let binance_fut = fetch_from_binance(pair_clone1);
    let bitget_fut = fetch_from_bitget(pair_clone2);
    let okx_fut = fetch_from_okx(pair_clone3);

    tokio::pin!(binance_fut);
    tokio::pin!(bitget_fut);
    tokio::pin!(okx_fut);

    // Track which futures have completed
    let mut binance_done = false;
    let mut bitget_done = false;
    let mut okx_done = false;

    loop {
        select! {
            result = &mut binance_fut, if !binance_done => {
                binance_done = true;
                if let Some((price_f64, change, source)) = result {
                    let price = NormalizedPrice::new(price_f64).with_change_24h(change);
                    return SourceResult::success(source, price, measure.elapsed_ms());
                }
            }
            result = &mut bitget_fut, if !bitget_done => {
                bitget_done = true;
                if let Some((price_f64, change, source)) = result {
                    let price = NormalizedPrice::new(price_f64).with_change_24h(change);
                    return SourceResult::success(source, price, measure.elapsed_ms());
                }
            }
            result = &mut okx_fut, if !okx_done => {
                okx_done = true;
                if let Some((price_f64, change, source)) = result {
                    let price = NormalizedPrice::new(price_f64).with_change_24h(change);
                    return SourceResult::success(source, price, measure.elapsed_ms());
                }
            }
            else => break,
        }
    }

    SourceResult::error(
        "ccxt",
        format!("Trading pair {} not found on any exchange", trading_pair),
        measure.elapsed_ms(),
    )
}

/// Fetch price from Chainlink
///
/// Tries RPC-based query first (free, no API key), then falls back to
/// Data Streams if credentials are configured.
async fn fetch_chainlink_price(
    token: &str,
    chain: &str,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedPrice> {
    // Try RPC-based query first (free, no API key needed)
    if let Some(result) = fetch_chainlink_rpc(token, chain, &measure).await {
        return result;
    }

    // Fall back to Data Streams if credentials are configured
    fetch_chainlink_streams(token, measure).await
}

/// Fetch price via RPC (Feed Registry or direct oracle)
async fn fetch_chainlink_rpc(
    token: &str,
    chain: &str,
    measure: &LatencyMeasure,
) -> Option<SourceResult<NormalizedPrice>> {
    use crate::chainlink;
    use crate::config::Chain;
    use crate::rpc::Endpoint;

    // Parse the chain parameter
    let target_chain = Chain::from_str_or_default(chain);

    // Get RPC endpoint for the target chain from cached config
    let config = get_cached_config().as_ref()?;
    let chain_endpoints: Vec<_> = config
        .endpoints
        .iter()
        .filter(|e| e.enabled && e.chain == target_chain)
        .cloned()
        .collect();

    if chain_endpoints.is_empty() {
        return None; // No RPC configured, skip to Data Streams
    }

    let endpoint = match Endpoint::new(chain_endpoints[0].clone(), 30, None) {
        Ok(e) => e,
        Err(_) => return None,
    };

    let provider = endpoint.provider().clone();

    // Try to fetch price via RPC
    match chainlink::fetch_price(provider, token, chain).await {
        Ok(price_data) => {
            if let Some(price_f64) = price_data.to_f64() {
                let price = NormalizedPrice::new(price_f64);
                Some(SourceResult::success(
                    "chainlink-rpc",
                    price,
                    measure.elapsed_ms(),
                ))
            } else {
                // Price is stale or invalid
                Some(SourceResult::error(
                    "chainlink-rpc",
                    "Stale or invalid price data",
                    measure.elapsed_ms(),
                ))
            }
        }
        Err(chainlink::ChainlinkError::NoFeed) => {
            // Token not supported by Chainlink, don't report as error
            // Just return None to try Data Streams
            None
        }
        Err(e) => {
            // RPC error, try Data Streams
            Some(SourceResult::error(
                "chainlink-rpc",
                format!("RPC error: {}", e),
                measure.elapsed_ms(),
            ))
        }
    }
}

/// Fetch price from Chainlink Data Streams (requires API key)
async fn fetch_chainlink_streams(
    token: &str,
    measure: LatencyMeasure,
) -> SourceResult<NormalizedPrice> {
    use chainlink_data_streams_report::feed_id::ID;
    use chainlink_data_streams_sdk::{client::Client, config::Config};

    // Get credentials from cached config first, then fall back to environment variables
    let file_config = get_cached_config();
    let chainlink_config = file_config.as_ref().and_then(|c| c.chainlink.as_ref());

    let api_key = match chainlink_config.map(|c| c.api_key.expose_secret().to_string()) {
        Some(key) => key,
        None => match std::env::var("CHAINLINK_API_KEY")
            .or_else(|_| std::env::var("CHAINLINK_CLIENT_ID"))
        {
            Ok(key) => key,
            Err(_) => {
                return SourceResult::error(
                    "chainlink",
                    "No RPC feed, no Data Streams credentials",
                    measure.elapsed_ms(),
                )
            }
        },
    };

    let user_secret = match chainlink_config.map(|c| c.user_secret.expose_secret().to_string()) {
        Some(secret) => secret,
        None => match std::env::var("CHAINLINK_USER_SECRET")
            .or_else(|_| std::env::var("CHAINLINK_CLIENT_SECRET"))
        {
            Ok(secret) => secret,
            Err(_) => {
                return SourceResult::error(
                    "chainlink",
                    "CHAINLINK_USER_SECRET not configured",
                    measure.elapsed_ms(),
                )
            }
        },
    };

    let rest_url = chainlink_config
        .and_then(|c| c.rest_url.clone())
        .or_else(|| std::env::var("CHAINLINK_REST_URL").ok())
        .unwrap_or_else(|| "https://api.testnet-dataengine.chain.link".to_string());

    let ws_url = chainlink_config
        .and_then(|c| c.ws_url.clone())
        .or_else(|| std::env::var("CHAINLINK_WS_URL").ok())
        .unwrap_or_else(|| "wss://ws.testnet-dataengine.chain.link".to_string());

    // Build config and client
    let config = match Config::new(api_key, user_secret, rest_url, ws_url).build() {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "chainlink-streams",
                format!("Config error: {:?}", e),
                measure.elapsed_ms(),
            )
        }
    };

    let client = match Client::new(config) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "chainlink-streams",
                format!("Client error: {:?}", e),
                measure.elapsed_ms(),
            )
        }
    };

    // Map token to Chainlink feed ID
    // These are Chainlink Data Streams feed IDs (different from price feed addresses)
    let feed_id = match token.to_uppercase().as_str() {
        // Common Chainlink Data Streams feed IDs (testnet)
        "ETH" | "ETHEREUM" => "0x000359843a543ee2fe414dc14c7e7920ef10f4372990b79d6361cdc0dd1ba782",
        "BTC" | "BITCOIN" => "0x00037da06d56d083fe599397a4769a042d63aa73dc4ef57709d31e9971a5b439",
        "LINK" | "CHAINLINK" => {
            "0x00036fe43f87884450b4c7e093cd5ed99cac6640d8c2000e6afc02c8838d0265"
        }
        _ => {
            // Check if it's already a feed ID (hex string)
            if token.starts_with("0x") && token.len() == 66 {
                token
            } else {
                return SourceResult::error(
                    "chainlink-streams",
                    format!(
                        "No Chainlink feed ID mapping for '{}'. Use feed ID directly (0x...)",
                        token
                    ),
                    measure.elapsed_ms(),
                );
            }
        }
    };

    // Parse feed ID
    let id = match ID::from_hex_str(feed_id) {
        Ok(id) => id,
        Err(e) => {
            return SourceResult::error(
                "chainlink-streams",
                format!("Invalid feed ID: {:?}", e),
                measure.elapsed_ms(),
            )
        }
    };

    // Fetch latest report
    match client.get_latest_report(id).await {
        Ok(response) => {
            // Decode the V3 report to extract the price
            // The full_report is a hex string containing the encoded report data
            use chainlink_data_streams_report::report::v3::ReportDataV3;

            // The report blob is in response.report.full_report
            let report_hex = &response.report.full_report;

            // Remove "0x" prefix if present and decode hex
            let hex_str = report_hex.strip_prefix("0x").unwrap_or(report_hex);
            let report_bytes = match hex::decode(hex_str) {
                Ok(bytes) => bytes,
                Err(e) => {
                    return SourceResult::error(
                        "chainlink-streams",
                        format!("Failed to decode report hex: {}", e),
                        measure.elapsed_ms(),
                    );
                }
            };

            // Decode the V3 report
            match ReportDataV3::decode(&report_bytes) {
                Ok(report_data) => {
                    // benchmark_price is a BigInt with 18 decimal places
                    // Convert to f64 by dividing by 10^18
                    use num_traits::ToPrimitive;

                    // Convert BigInt to f64 and divide by 10^18
                    let price_f64 = report_data
                        .benchmark_price
                        .to_f64()
                        .map(|p| p / 1e18)
                        .unwrap_or(0.0);

                    if price_f64 > 0.0 {
                        let price = NormalizedPrice::new(price_f64);
                        SourceResult::success("chainlink-streams", price, measure.elapsed_ms())
                    } else {
                        SourceResult::error(
                            "chainlink-streams",
                            "Invalid price value from report",
                            measure.elapsed_ms(),
                        )
                    }
                }
                Err(e) => SourceResult::error(
                    "chainlink-streams",
                    format!("Failed to decode V3 report: {:?}", e),
                    measure.elapsed_ms(),
                ),
            }
        }
        Err(e) => SourceResult::error(
            "chainlink-streams",
            format!("API error: {:?}", e),
            measure.elapsed_ms(),
        ),
    }
}

/// Fetch price from Pyth Network Hermes API
async fn fetch_pyth_price(token: &str, measure: LatencyMeasure) -> SourceResult<NormalizedPrice> {
    // Try to map the token to a Pyth feed ID
    let feed_id = if token.starts_with("0x") && token.len() == 66 {
        // Already a Pyth feed ID
        token
    } else if token.starts_with("0x") && token.len() == 42 {
        // Ethereum address - try to reverse-map to symbol first
        match eth_address_to_symbol(token) {
            Some(symbol) => match pyth::symbol_to_feed_id(symbol) {
                Some(id) => id,
                None => {
                    return SourceResult::error(
                        "pyth",
                        format!(
                            "No Pyth feed for symbol '{}' (from address {})",
                            symbol, token
                        ),
                        measure.elapsed_ms(),
                    );
                }
            },
            None => {
                return SourceResult::error(
                    "pyth",
                    format!("Cannot map address {} to Pyth feed ID", token),
                    measure.elapsed_ms(),
                );
            }
        }
    } else {
        // Try to map symbol to feed ID
        match pyth::symbol_to_feed_id(token) {
            Some(id) => id,
            None => {
                return SourceResult::error(
                    "pyth",
                    format!(
                        "No Pyth feed ID mapping for '{}'. Use feed ID directly (0x...)",
                        token
                    ),
                    measure.elapsed_ms(),
                );
            }
        }
    };

    // Create Pyth client
    let client = match pyth::Client::new() {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "pyth",
                format!("Client error: {}", e),
                measure.elapsed_ms(),
            );
        }
    };

    // Fetch latest price
    match client.get_latest_price(feed_id).await {
        Ok(Some(feed)) => {
            if let Some(price_f64) = feed.price_f64() {
                // Check for stale data (older than 60 seconds)
                if feed.is_stale(60) {
                    return SourceResult::error(
                        "pyth",
                        "Price data is stale (>60s old)",
                        measure.elapsed_ms(),
                    );
                }

                let mut price = NormalizedPrice::new(price_f64);

                // Add confidence interval as metadata if available
                if let Some(confidence) = feed.confidence_f64() {
                    // Confidence as a percentage of price
                    let confidence_pct = (confidence / price_f64) * 100.0;
                    price = price.with_confidence(1.0 - confidence_pct.min(1.0));
                }

                SourceResult::success("pyth", price, measure.elapsed_ms())
            } else {
                SourceResult::error("pyth", "Failed to parse price data", measure.elapsed_ms())
            }
        }
        Ok(None) => SourceResult::error("pyth", "No price data returned", measure.elapsed_ms()),
        Err(e) => SourceResult::error("pyth", format!("API error: {}", e), measure.elapsed_ms()),
    }
}
