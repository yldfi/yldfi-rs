//! ethcli-mcp - MCP Server for ethcli
//!
//! Exposes core ethcli functionality as MCP tools for use with AI assistants.
//!
//! ## IMPORTANT: STDIO Protocol
//!
//! MCP uses STDIO for communication. All logging MUST go to stderr.
//! NEVER write non-MCP output to stdout.

// Prevent accidental stdout writes which would break MCP protocol
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]
#![deny(clippy::dbg_macro)]

mod executor;
mod tools;
mod types;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
    ServerHandler, ServiceExt,
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

// Re-export all types for use in tool implementations
use tools::ToResponse;
use types::*;

// =============================================================================
// MCP SERVER
// =============================================================================

/// Main MCP server handler
#[derive(Clone)]
pub struct EthcliMcpServer {
    tool_router: ToolRouter<Self>,
}

impl EthcliMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for EthcliMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

// Tool implementations using the tool_router macro
#[tool_router(router = tool_router)]
impl EthcliMcpServer {
    // =========================================================================
    // LOGS
    // =========================================================================

    #[tool(description = "Query event logs from a contract with optional filters")]
    async fn logs(&self, Parameters(input): Parameters<LogsInput>) -> String {
        tools::logs(
            &input.contract,
            input.event.as_deref(),
            input.from_block.as_deref(),
            input.to_block.as_deref(),
            input.topics,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    // =========================================================================
    // TRANSACTION
    // =========================================================================

    #[tool(
        description = "Analyze an Ethereum transaction including decoded events, token transfers, and method calls"
    )]
    async fn tx_analyze(&self, Parameters(input): Parameters<TxAnalyzeInput>) -> String {
        tools::tx_analyze(&input.hash, Some(&input.chain), input.trace)
            .await
            .to_response()
    }

    // =========================================================================
    // ACCOUNT
    // =========================================================================

    #[tool(
        description = "Get comprehensive account information including balance and transaction count"
    )]
    async fn account_info(&self, Parameters(input): Parameters<AccountAddressInput>) -> String {
        tools::account_info(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get the native token (ETH) balance for an address")]
    async fn account_balance(&self, Parameters(input): Parameters<AccountAddressInput>) -> String {
        tools::account_balance(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "List transactions for an address")]
    async fn account_txs(&self, Parameters(input): Parameters<AccountTxsInput>) -> String {
        tools::account_txs(&input.address, Some(&input.chain), input.limit)
            .await
            .to_response()
    }

    #[tool(description = "Get internal transactions for an address")]
    async fn account_internal_txs(
        &self,
        Parameters(input): Parameters<AccountAddressInput>,
    ) -> String {
        tools::account_internal_txs(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get ERC20 token transfers for an address")]
    async fn account_erc20(&self, Parameters(input): Parameters<AccountAddressInput>) -> String {
        tools::account_erc20(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get ERC721 NFT transfers for an address")]
    async fn account_erc721(&self, Parameters(input): Parameters<AccountAddressInput>) -> String {
        tools::account_erc721(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get ERC1155 token transfers for an address")]
    async fn account_erc1155(&self, Parameters(input): Parameters<AccountAddressInput>) -> String {
        tools::account_erc1155(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // CONTRACT
    // =========================================================================

    #[tool(description = "Fetch the ABI for a verified contract from Etherscan")]
    async fn contract_abi(&self, Parameters(input): Parameters<ContractAddressInput>) -> String {
        tools::contract_abi(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Download verified source code for a contract")]
    async fn contract_source(&self, Parameters(input): Parameters<ContractAddressInput>) -> String {
        tools::contract_source(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get contract creation transaction and creator address")]
    async fn contract_creation(
        &self,
        Parameters(input): Parameters<ContractAddressInput>,
    ) -> String {
        tools::contract_creation(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // TOKEN
    // =========================================================================

    #[tool(description = "Get token metadata (name, symbol, decimals, total supply)")]
    async fn token_info(&self, Parameters(input): Parameters<TokenInfoInput>) -> String {
        tools::token_info(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get top token holders")]
    async fn token_holders(&self, Parameters(input): Parameters<TokenHoldersInput>) -> String {
        tools::token_holders(&input.address, Some(&input.chain), input.limit)
            .await
            .to_response()
    }

    #[tool(description = "Get token balance for a specific address")]
    async fn token_balance(&self, Parameters(input): Parameters<TokenBalanceInput>) -> String {
        tools::token_balance(&input.token, &input.address, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // GAS
    // =========================================================================

    #[tool(description = "Get current gas prices from multiple sources")]
    async fn gas_oracle(&self, Parameters(input): Parameters<GasOracleInput>) -> String {
        tools::gas_oracle(Some(&input.chain)).await.to_response()
    }

    #[tool(description = "Estimate gas for a transaction")]
    async fn gas_estimate(&self, Parameters(input): Parameters<GasEstimateInput>) -> String {
        tools::gas_estimate(
            &input.to,
            input.value.as_deref(),
            input.data.as_deref(),
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    // =========================================================================
    // SIGNATURE LOOKUP
    // =========================================================================

    #[tool(description = "Lookup function signature from 4-byte selector")]
    async fn sig_fn(&self, Parameters(input): Parameters<SigLookupInput>) -> String {
        tools::sig_fn(&input.selector).await.to_response()
    }

    #[tool(description = "Lookup event signature from topic hash")]
    async fn sig_event(&self, Parameters(input): Parameters<SigLookupInput>) -> String {
        tools::sig_event(&input.selector).await.to_response()
    }

    // =========================================================================
    // CAST (Conversions)
    // =========================================================================

    #[tool(description = "Convert amount to wei")]
    async fn cast_to_wei(&self, Parameters(input): Parameters<CastToWeiInput>) -> String {
        tools::cast_to_wei(&input.amount, Some(&input.unit))
            .await
            .to_response()
    }

    #[tool(description = "Convert wei to ether or gwei")]
    async fn cast_from_wei(&self, Parameters(input): Parameters<CastFromWeiInput>) -> String {
        tools::cast_from_wei(&input.wei, Some(&input.unit))
            .await
            .to_response()
    }

    #[tool(description = "Convert decimal to hex")]
    async fn cast_to_hex(&self, Parameters(input): Parameters<CastValueInput>) -> String {
        tools::cast_to_hex(&input.value).await.to_response()
    }

    #[tool(description = "Convert hex to decimal")]
    async fn cast_to_dec(&self, Parameters(input): Parameters<CastValueInput>) -> String {
        tools::cast_to_dec(&input.value).await.to_response()
    }

    #[tool(description = "Compute keccak256 hash")]
    async fn cast_keccak(&self, Parameters(input): Parameters<CastValueInput>) -> String {
        tools::cast_keccak(&input.value).await.to_response()
    }

    #[tool(description = "Get 4-byte function selector from signature")]
    async fn cast_sig(&self, Parameters(input): Parameters<CastSigInput>) -> String {
        tools::cast_sig(&input.signature).await.to_response()
    }

    #[tool(description = "Get event topic from signature")]
    async fn cast_topic(&self, Parameters(input): Parameters<CastSigInput>) -> String {
        tools::cast_topic(&input.signature).await.to_response()
    }

    #[tool(description = "Checksum an Ethereum address (EIP-55)")]
    async fn cast_checksum(&self, Parameters(input): Parameters<CastValueInput>) -> String {
        tools::cast_checksum(&input.value).await.to_response()
    }

    #[tool(description = "ABI encode function arguments")]
    async fn cast_abi_encode(&self, Parameters(input): Parameters<CastAbiEncodeInput>) -> String {
        tools::cast_abi_encode(&input.sig, input.args)
            .await
            .to_response()
    }

    #[tool(description = "ABI decode function return data")]
    async fn cast_abi_decode(&self, Parameters(input): Parameters<CastAbiDecodeInput>) -> String {
        tools::cast_abi_decode(&input.sig, &input.data)
            .await
            .to_response()
    }

    // =========================================================================
    // RPC
    // =========================================================================

    #[tool(description = "Make an eth_call to a contract (read-only)")]
    async fn rpc_call(&self, Parameters(input): Parameters<RpcCallInput>) -> String {
        tools::rpc_call(
            &input.to,
            &input.data,
            Some(&input.chain),
            input.block.as_deref(),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get block information")]
    async fn rpc_block(&self, Parameters(input): Parameters<RpcBlockInput>) -> String {
        tools::rpc_block(&input.block, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get storage value at a slot")]
    async fn rpc_storage(&self, Parameters(input): Parameters<RpcStorageInput>) -> String {
        tools::rpc_storage(&input.address, &input.slot, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get contract bytecode")]
    async fn rpc_code(&self, Parameters(input): Parameters<RpcAddressInput>) -> String {
        tools::rpc_code(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get account nonce (transaction count)")]
    async fn rpc_nonce(&self, Parameters(input): Parameters<RpcAddressInput>) -> String {
        tools::rpc_nonce(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get transaction receipt")]
    async fn rpc_receipt(&self, Parameters(input): Parameters<RpcHashInput>) -> String {
        tools::rpc_receipt(&input.hash, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get latest block number")]
    async fn rpc_block_number(&self, Parameters(input): Parameters<RpcChainInput>) -> String {
        tools::rpc_block_number(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get chain ID")]
    async fn rpc_chain_id(&self, Parameters(input): Parameters<RpcChainInput>) -> String {
        tools::rpc_chain_id(Some(&input.chain)).await.to_response()
    }

    #[tool(description = "Get current gas price")]
    async fn rpc_gas_price(&self, Parameters(input): Parameters<RpcChainInput>) -> String {
        tools::rpc_gas_price(Some(&input.chain)).await.to_response()
    }

    // =========================================================================
    // ENS
    // =========================================================================

    #[tool(description = "Resolve ENS name to Ethereum address")]
    async fn ens_resolve(&self, Parameters(input): Parameters<EnsResolveInput>) -> String {
        tools::ens_resolve(&input.name).await.to_response()
    }

    #[tool(description = "Reverse lookup address to ENS name")]
    async fn ens_lookup(&self, Parameters(input): Parameters<EnsLookupInput>) -> String {
        tools::ens_lookup(&input.address).await.to_response()
    }

    #[tool(description = "Compute ENS namehash for a domain")]
    async fn ens_namehash(&self, Parameters(input): Parameters<EnsNamehashInput>) -> String {
        tools::ens_namehash(&input.name).await.to_response()
    }

    // =========================================================================
    // PRICE (Aggregated)
    // =========================================================================

    #[tool(description = "Get token price from multiple aggregated sources")]
    async fn price(&self, Parameters(input): Parameters<PriceInput>) -> String {
        tools::price(&input.token, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // PORTFOLIO
    // =========================================================================

    #[tool(description = "Get portfolio holdings for an address across tokens")]
    async fn portfolio(&self, Parameters(input): Parameters<PortfolioInput>) -> String {
        tools::portfolio(&input.address, input.chain.as_deref())
            .await
            .to_response()
    }

    // =========================================================================
    // NFTS
    // =========================================================================

    #[tool(description = "Get NFTs owned by an address")]
    async fn nfts(&self, Parameters(input): Parameters<NftsInput>) -> String {
        tools::nfts(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // YIELDS
    // =========================================================================

    #[tool(description = "Get DeFi yield opportunities")]
    async fn yields(&self, Parameters(input): Parameters<YieldsInput>) -> String {
        tools::yields(input.protocol.as_deref(), input.chain.as_deref())
            .await
            .to_response()
    }

    // =========================================================================
    // QUOTE (DEX Aggregated)
    // =========================================================================

    #[tool(description = "Get best swap quote across DEX aggregators")]
    async fn quote_best(&self, Parameters(input): Parameters<QuoteBestInput>) -> String {
        tools::quote_best(
            &input.from_token,
            &input.to_token,
            &input.amount,
            Some(&input.chain),
            input.slippage,
        )
        .await
        .to_response()
    }

    #[tool(description = "Compare quotes across DEX aggregators")]
    async fn quote_compare(&self, Parameters(input): Parameters<QuoteCompareInput>) -> String {
        tools::quote_compare(
            &input.from_token,
            &input.to_token,
            &input.amount,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    // =========================================================================
    // CHAINLINK
    // =========================================================================

    #[tool(description = "Get Chainlink oracle price")]
    async fn chainlink_price(&self, Parameters(input): Parameters<ChainlinkPriceInput>) -> String {
        tools::chainlink_price(&input.token, Some(&input.chain), input.block.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get Chainlink feed address for a token")]
    async fn chainlink_feed(&self, Parameters(input): Parameters<ChainlinkFeedInput>) -> String {
        tools::chainlink_feed(&input.token, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "List available Chainlink oracles")]
    async fn chainlink_oracles(
        &self,
        Parameters(input): Parameters<ChainlinkOraclesInput>,
    ) -> String {
        tools::chainlink_oracles(Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // GOPLUS (Security)
    // =========================================================================

    #[tool(description = "Check token security info via GoPlus")]
    async fn goplus_token(&self, Parameters(input): Parameters<GoplusInput>) -> String {
        tools::goplus_token(&input.address, input.chain_id)
            .await
            .to_response()
    }

    #[tool(description = "Check address security info via GoPlus")]
    async fn goplus_address(&self, Parameters(input): Parameters<GoplusInput>) -> String {
        tools::goplus_address(&input.address, input.chain_id)
            .await
            .to_response()
    }

    #[tool(description = "Check NFT security info via GoPlus")]
    async fn goplus_nft(&self, Parameters(input): Parameters<GoplusInput>) -> String {
        tools::goplus_nft(&input.address, input.chain_id)
            .await
            .to_response()
    }

    #[tool(description = "Check token approval security via GoPlus")]
    async fn goplus_approval(&self, Parameters(input): Parameters<GoplusInput>) -> String {
        tools::goplus_approval(&input.address, input.chain_id)
            .await
            .to_response()
    }

    // =========================================================================
    // SOLODIT (Security Database)
    // =========================================================================

    #[tool(description = "Search security findings in Solodit database")]
    async fn solodit_search(&self, Parameters(input): Parameters<SoloditSearchInput>) -> String {
        tools::solodit_search(&input.query, input.impact.as_deref(), input.limit)
            .await
            .to_response()
    }

    #[tool(description = "Get details of a specific Solodit finding")]
    async fn solodit_get(&self, Parameters(input): Parameters<SoloditGetInput>) -> String {
        tools::solodit_get(&input.slug).await.to_response()
    }

    // =========================================================================
    // UNISWAP
    // =========================================================================

    #[tool(description = "Get Uniswap pool information")]
    async fn uniswap_pool(&self, Parameters(input): Parameters<UniswapPoolInput>) -> String {
        tools::uniswap_pool(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get ETH price from Uniswap")]
    async fn uniswap_eth_price(
        &self,
        Parameters(input): Parameters<UniswapEthPriceInput>,
    ) -> String {
        tools::uniswap_eth_price(Some(&input.chain), input.version.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get top Uniswap pools by TVL")]
    async fn uniswap_top_pools(
        &self,
        Parameters(input): Parameters<UniswapTopPoolsInput>,
    ) -> String {
        tools::uniswap_top_pools(input.limit, Some(&input.chain), input.version.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get Uniswap LP positions for an address")]
    async fn uniswap_positions(
        &self,
        Parameters(input): Parameters<UniswapPositionsInput>,
    ) -> String {
        tools::uniswap_positions(&input.address, Some(&input.chain), input.version.as_deref())
            .await
            .to_response()
    }

    // =========================================================================
    // CURVE
    // =========================================================================

    #[tool(description = "List Curve pools")]
    async fn curve_pools(&self, Parameters(input): Parameters<CurvePoolsInput>) -> String {
        tools::curve_pools(Some(&input.chain)).await.to_response()
    }

    #[tool(description = "Get Curve router route for a swap")]
    async fn curve_router_route(&self, Parameters(input): Parameters<CurveRouteInput>) -> String {
        tools::curve_router_route(&input.from_token, &input.to_token, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // ALCHEMY
    // =========================================================================

    #[tool(description = "Get portfolio via Alchemy API")]
    async fn alchemy_portfolio(
        &self,
        Parameters(input): Parameters<AlchemyPortfolioInput>,
    ) -> String {
        tools::alchemy_portfolio(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get token transfers via Alchemy API")]
    async fn alchemy_transfers(
        &self,
        Parameters(input): Parameters<AlchemyTransfersInput>,
    ) -> String {
        tools::alchemy_transfers(
            &input.address,
            Some(&input.chain),
            input.category.as_deref(),
        )
        .await
        .to_response()
    }

    // =========================================================================
    // COINGECKO
    // =========================================================================

    #[tool(description = "Get coin info from CoinGecko")]
    async fn gecko_coins_info(&self, Parameters(input): Parameters<GeckoCoinInput>) -> String {
        tools::gecko_coins_info(&input.id).await.to_response()
    }

    #[tool(description = "Get simple price from CoinGecko")]
    async fn gecko_simple_price(&self, Parameters(input): Parameters<GeckoPriceInput>) -> String {
        tools::gecko_simple_price(&input.ids, input.vs_currencies.as_deref())
            .await
            .to_response()
    }

    // =========================================================================
    // DEFILLAMA
    // =========================================================================

    #[tool(description = "Get protocol TVL from DefiLlama")]
    async fn llama_tvl(&self, Parameters(input): Parameters<LlamaTvlInput>) -> String {
        tools::llama_tvl(&input.protocol).await.to_response()
    }

    #[tool(description = "Get yield pools from DefiLlama")]
    async fn llama_yields(&self, Parameters(input): Parameters<LlamaYieldsInput>) -> String {
        tools::llama_yields(input.chain.as_deref(), input.protocol.as_deref())
            .await
            .to_response()
    }

    // =========================================================================
    // DEX AGGREGATORS
    // =========================================================================

    #[tool(description = "Get quote from 1inch DEX aggregator")]
    async fn oneinch_quote(&self, Parameters(input): Parameters<OneinchQuoteInput>) -> String {
        tools::oneinch_quote(&input.src, &input.dst, &input.amount, input.chain_id)
            .await
            .to_response()
    }

    #[tool(description = "Get quote from 0x DEX aggregator")]
    async fn zerox_quote(&self, Parameters(input): Parameters<ZeroxQuoteInput>) -> String {
        tools::zerox_quote(
            &input.sell_token,
            &input.buy_token,
            &input.sell_amount,
            &input.taker,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get quote from OpenOcean DEX aggregator")]
    async fn openocean_quote(&self, Parameters(input): Parameters<OpenoceanQuoteInput>) -> String {
        tools::openocean_quote(
            &input.in_token,
            &input.out_token,
            &input.amount,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get quote from CoW Swap DEX aggregator")]
    async fn cowswap_quote(&self, Parameters(input): Parameters<CowswapQuoteInput>) -> String {
        tools::cowswap_quote(
            &input.sell_token,
            &input.buy_token,
            &input.amount,
            &input.from,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get cross-chain quote from LI.FI")]
    async fn lifi_quote(&self, Parameters(input): Parameters<LifiQuoteInput>) -> String {
        tools::lifi_quote(
            &input.from_chain,
            &input.from_token,
            &input.to_chain,
            &input.to_token,
            &input.amount,
            &input.from_address,
        )
        .await
        .to_response()
    }

    // =========================================================================
    // PYTH
    // =========================================================================

    #[tool(description = "Get price from Pyth Network oracle")]
    async fn pyth_price(&self, Parameters(input): Parameters<PythPriceInput>) -> String {
        tools::pyth_price(&input.symbols).await.to_response()
    }

    #[tool(description = "Search Pyth price feeds")]
    async fn pyth_search(&self, Parameters(input): Parameters<PythSearchInput>) -> String {
        tools::pyth_search(&input.query).await.to_response()
    }

    // =========================================================================
    // CCXT (Exchange Data)
    // =========================================================================

    #[tool(description = "Get ticker from centralized exchange via CCXT")]
    async fn ccxt_ticker(&self, Parameters(input): Parameters<CcxtTickerInput>) -> String {
        tools::ccxt_ticker(&input.exchange, &input.symbol)
            .await
            .to_response()
    }

    #[tool(description = "Get order book from centralized exchange via CCXT")]
    async fn ccxt_orderbook(&self, Parameters(input): Parameters<CcxtOrderbookInput>) -> String {
        tools::ccxt_orderbook(&input.exchange, &input.symbol, input.limit)
            .await
            .to_response()
    }

    // =========================================================================
    // SIMULATE
    // =========================================================================

    #[tool(description = "Simulate a contract call")]
    async fn simulate_call(&self, Parameters(input): Parameters<SimulateCallInput>) -> String {
        tools::simulate_call(
            &input.contract,
            &input.sig,
            input.args,
            Some(&input.chain),
            input.via.as_deref(),
        )
        .await
        .to_response()
    }

    #[tool(description = "Simulate a historical transaction")]
    async fn simulate_tx(&self, Parameters(input): Parameters<SimulateTxInput>) -> String {
        tools::simulate_tx(&input.hash, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // DOCTOR (Diagnostics)
    // =========================================================================

    #[tool(description = "Run ethcli diagnostics to check configuration and connectivity")]
    async fn doctor(&self) -> String {
        tools::doctor().await.to_response()
    }

    // =========================================================================
    // ACCOUNT (additional)
    // =========================================================================

    #[tool(description = "Get mined blocks for a validator/miner address")]
    async fn account_mined_blocks(
        &self,
        Parameters(input): Parameters<AccountAddressInput>,
    ) -> String {
        tools::account_mined_blocks(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // ADDRESS BOOK
    // =========================================================================

    #[tool(description = "Add an address to the local address book")]
    async fn address_add(&self, Parameters(input): Parameters<AddressAddInput>) -> String {
        tools::address_add(&input.name, &input.address)
            .await
            .to_response()
    }

    #[tool(description = "Remove an address from the local address book")]
    async fn address_remove(&self, Parameters(input): Parameters<AddressNameInput>) -> String {
        tools::address_remove(&input.name).await.to_response()
    }

    #[tool(description = "List all addresses in the local address book")]
    async fn address_list(&self) -> String {
        tools::address_list().await.to_response()
    }

    #[tool(description = "Get an address from the local address book by name")]
    async fn address_get(&self, Parameters(input): Parameters<AddressNameInput>) -> String {
        tools::address_get(&input.name).await.to_response()
    }

    #[tool(description = "Search the local address book")]
    async fn address_search(&self, Parameters(input): Parameters<AddressSearchInput>) -> String {
        tools::address_search(&input.query).await.to_response()
    }

    #[tool(description = "Import addresses from a file")]
    async fn address_import(&self, Parameters(input): Parameters<AddressFileInput>) -> String {
        tools::address_import(&input.file).await.to_response()
    }

    #[tool(description = "Export addresses to a file")]
    async fn address_export(&self, Parameters(input): Parameters<AddressFileInput>) -> String {
        tools::address_export(&input.file).await.to_response()
    }

    // =========================================================================
    // BLACKLIST
    // =========================================================================

    #[tool(description = "Add an address to the local blacklist")]
    async fn blacklist_add(&self, Parameters(input): Parameters<BlacklistAddInput>) -> String {
        tools::blacklist_add(&input.address, input.reason.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Remove an address from the local blacklist")]
    async fn blacklist_remove(
        &self,
        Parameters(input): Parameters<BlacklistAddressInput>,
    ) -> String {
        tools::blacklist_remove(&input.address).await.to_response()
    }

    #[tool(description = "List all addresses in the local blacklist")]
    async fn blacklist_list(&self) -> String {
        tools::blacklist_list().await.to_response()
    }

    #[tool(description = "Check if an address is in the local blacklist")]
    async fn blacklist_check(
        &self,
        Parameters(input): Parameters<BlacklistAddressInput>,
    ) -> String {
        tools::blacklist_check(&input.address).await.to_response()
    }

    #[tool(description = "Scan an address against OFAC sanctions and known blacklists")]
    async fn blacklist_scan(&self, Parameters(input): Parameters<BlacklistScanInput>) -> String {
        tools::blacklist_scan(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Scan a portfolio for sanctioned/blacklisted addresses")]
    async fn blacklist_scan_portfolio(
        &self,
        Parameters(input): Parameters<BlacklistScanInput>,
    ) -> String {
        tools::blacklist_scan_portfolio(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Find transaction path between two addresses for compliance")]
    async fn blacklist_path(&self, Parameters(input): Parameters<BlacklistPathInput>) -> String {
        tools::blacklist_path(&input.from, &input.to, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // CONTRACT (additional)
    // =========================================================================

    #[tool(description = "Call a contract function (read-only)")]
    async fn contract_call(&self, Parameters(input): Parameters<ContractCallInput>) -> String {
        tools::contract_call(&input.address, &input.sig, input.args, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // SIG (additional)
    // =========================================================================

    #[tool(description = "Get signature cache statistics")]
    async fn sig_cache_stats(&self) -> String {
        tools::sig_cache_stats().await.to_response()
    }

    #[tool(description = "Clear the signature cache")]
    async fn sig_cache_clear(&self) -> String {
        tools::sig_cache_clear().await.to_response()
    }

    // =========================================================================
    // CAST (additional)
    // =========================================================================

    #[tool(description = "Compute the CREATE address for a deployer and nonce")]
    async fn cast_compute_address(
        &self,
        Parameters(input): Parameters<CastComputeAddressInput>,
    ) -> String {
        tools::cast_compute_address(&input.deployer, &input.nonce)
            .await
            .to_response()
    }

    #[tool(description = "Compute the CREATE2 address")]
    async fn cast_create2(&self, Parameters(input): Parameters<CastCreate2Input>) -> String {
        tools::cast_create2(&input.deployer, &input.salt, &input.init_code_hash)
            .await
            .to_response()
    }

    #[tool(description = "Concatenate hex values")]
    async fn cast_concat(&self, Parameters(input): Parameters<CastConcatInput>) -> String {
        tools::cast_concat(input.values).await.to_response()
    }

    #[tool(description = "Convert value to bytes32")]
    async fn cast_to_bytes32(&self, Parameters(input): Parameters<CastValueInput>) -> String {
        tools::cast_to_bytes32(&input.value).await.to_response()
    }

    // =========================================================================
    // ENS (additional)
    // =========================================================================

    #[tool(description = "Get the resolver address for an ENS name")]
    async fn ens_resolver(&self, Parameters(input): Parameters<EnsNamehashInput>) -> String {
        tools::ens_resolver(&input.name).await.to_response()
    }

    // =========================================================================
    // SIMULATE (additional)
    // =========================================================================

    #[tool(description = "Simulate a bundle of transactions")]
    async fn simulate_bundle(&self, Parameters(input): Parameters<SimulateBundleInput>) -> String {
        tools::simulate_bundle(input.txs, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "List saved simulations")]
    async fn simulate_list(&self) -> String {
        tools::simulate_list().await.to_response()
    }

    #[tool(description = "Get a saved simulation by ID")]
    async fn simulate_get(&self, Parameters(input): Parameters<SimulateIdInput>) -> String {
        tools::simulate_get(&input.id).await.to_response()
    }

    #[tool(description = "Get simulation info by ID")]
    async fn simulate_info(&self, Parameters(input): Parameters<SimulateIdInput>) -> String {
        tools::simulate_info(&input.id).await.to_response()
    }

    #[tool(description = "Share a simulation publicly")]
    async fn simulate_share(&self, Parameters(input): Parameters<SimulateIdInput>) -> String {
        tools::simulate_share(&input.id).await.to_response()
    }

    #[tool(description = "Unshare a simulation")]
    async fn simulate_unshare(&self, Parameters(input): Parameters<SimulateIdInput>) -> String {
        tools::simulate_unshare(&input.id).await.to_response()
    }

    // =========================================================================
    // TENDERLY
    // =========================================================================

    #[tool(description = "Simulate a transaction via Tenderly")]
    async fn tenderly_simulate(
        &self,
        Parameters(input): Parameters<TenderlySimulateInput>,
    ) -> String {
        tools::tenderly_simulate(&input.contract, &input.data, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "List Tenderly virtual networks")]
    async fn tenderly_vnets(&self) -> String {
        tools::tenderly_vnets().await.to_response()
    }

    #[tool(description = "List Tenderly wallets")]
    async fn tenderly_wallets(&self) -> String {
        tools::tenderly_wallets().await.to_response()
    }

    #[tool(description = "List Tenderly contracts")]
    async fn tenderly_contracts(&self) -> String {
        tools::tenderly_contracts().await.to_response()
    }

    #[tool(description = "List Tenderly alerts")]
    async fn tenderly_alerts(&self) -> String {
        tools::tenderly_alerts().await.to_response()
    }

    #[tool(description = "List Tenderly web3 actions")]
    async fn tenderly_actions(&self) -> String {
        tools::tenderly_actions().await.to_response()
    }

    #[tool(description = "List supported Tenderly networks")]
    async fn tenderly_networks(&self) -> String {
        tools::tenderly_networks().await.to_response()
    }

    #[tool(description = "List Tenderly notification channels")]
    async fn tenderly_channels(&self) -> String {
        tools::tenderly_channels().await.to_response()
    }

    // =========================================================================
    // ALCHEMY (additional)
    // =========================================================================

    #[tool(description = "Get NFTs for an address via Alchemy")]
    async fn alchemy_nft(&self, Parameters(input): Parameters<AlchemyPortfolioInput>) -> String {
        tools::alchemy_nft(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get token info via Alchemy")]
    async fn alchemy_token(&self, Parameters(input): Parameters<AlchemyPortfolioInput>) -> String {
        tools::alchemy_token(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get token prices via Alchemy")]
    async fn alchemy_prices(&self, Parameters(input): Parameters<AlchemyPricesInput>) -> String {
        tools::alchemy_prices(&input.tokens, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Debug a transaction via Alchemy")]
    async fn alchemy_debug(&self, Parameters(input): Parameters<AlchemyDebugInput>) -> String {
        tools::alchemy_debug(&input.hash, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // COINGECKO (additional)
    // =========================================================================

    #[tool(description = "Get global crypto market data from CoinGecko")]
    async fn gecko_global(&self) -> String {
        tools::gecko_global().await.to_response()
    }

    #[tool(description = "Get NFT collection info from CoinGecko")]
    async fn gecko_nfts(&self, Parameters(input): Parameters<GeckoNftInput>) -> String {
        tools::gecko_nfts(&input.id).await.to_response()
    }

    #[tool(description = "Get on-chain token info from CoinGecko")]
    async fn gecko_onchain(&self, Parameters(input): Parameters<GeckoOnchainInput>) -> String {
        tools::gecko_onchain(&input.network, &input.address)
            .await
            .to_response()
    }

    // =========================================================================
    // GOPLUS (additional)
    // =========================================================================

    #[tool(description = "Batch check multiple token addresses via GoPlus")]
    async fn goplus_token_batch(&self, Parameters(input): Parameters<GoplusBatchInput>) -> String {
        tools::goplus_token_batch(&input.addresses, input.chain_id)
            .await
            .to_response()
    }

    #[tool(description = "List supported chains for GoPlus")]
    async fn goplus_chains(&self) -> String {
        tools::goplus_chains().await.to_response()
    }

    // =========================================================================
    // SOLODIT (additional)
    // =========================================================================

    #[tool(description = "Get Solodit API rate limit status")]
    async fn solodit_rate_limit(&self) -> String {
        tools::solodit_rate_limit().await.to_response()
    }

    #[tool(description = "List available Solodit tags")]
    async fn solodit_tags(&self) -> String {
        tools::solodit_tags().await.to_response()
    }

    #[tool(description = "List audit firms on Solodit")]
    async fn solodit_firms(&self) -> String {
        tools::solodit_firms().await.to_response()
    }

    // =========================================================================
    // DEFILLAMA (additional)
    // =========================================================================

    #[tool(description = "Get token prices from DefiLlama")]
    async fn llama_coins(&self, Parameters(input): Parameters<LlamaCoinsInput>) -> String {
        tools::llama_coins(&input.addresses).await.to_response()
    }

    #[tool(description = "Get DEX volumes from DefiLlama")]
    async fn llama_volumes(&self, Parameters(input): Parameters<LlamaProtocolInput>) -> String {
        tools::llama_volumes(input.protocol.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get protocol fees from DefiLlama")]
    async fn llama_fees(&self, Parameters(input): Parameters<LlamaProtocolInput>) -> String {
        tools::llama_fees(input.protocol.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get stablecoin data from DefiLlama")]
    async fn llama_stablecoins(&self) -> String {
        tools::llama_stablecoins().await.to_response()
    }

    // =========================================================================
    // MORALIS
    // =========================================================================

    #[tool(description = "Get wallet data via Moralis")]
    async fn moralis_wallet(&self, Parameters(input): Parameters<MoralisAddressInput>) -> String {
        tools::moralis_wallet(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get token data via Moralis")]
    async fn moralis_token(&self, Parameters(input): Parameters<MoralisAddressInput>) -> String {
        tools::moralis_token(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get NFT data via Moralis")]
    async fn moralis_nft(&self, Parameters(input): Parameters<MoralisAddressInput>) -> String {
        tools::moralis_nft(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Resolve domain name via Moralis")]
    async fn moralis_resolve(&self, Parameters(input): Parameters<MoralisDomainInput>) -> String {
        tools::moralis_resolve(&input.domain).await.to_response()
    }

    #[tool(description = "Get market data via Moralis")]
    async fn moralis_market(&self) -> String {
        tools::moralis_market().await.to_response()
    }

    // =========================================================================
    // DSIM (Dune Simulator)
    // =========================================================================

    #[tool(description = "List supported chains for Dune Simulator")]
    async fn dsim_chains(&self) -> String {
        tools::dsim_chains().await.to_response()
    }

    #[tool(description = "Get token balances via Dune Simulator")]
    async fn dsim_balances(&self, Parameters(input): Parameters<DsimAddressInput>) -> String {
        tools::dsim_balances(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get NFT collectibles via Dune Simulator")]
    async fn dsim_collectibles(&self, Parameters(input): Parameters<DsimAddressInput>) -> String {
        tools::dsim_collectibles(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get wallet activity via Dune Simulator")]
    async fn dsim_activity(&self, Parameters(input): Parameters<DsimAddressInput>) -> String {
        tools::dsim_activity(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get token info via Dune Simulator")]
    async fn dsim_token(&self, Parameters(input): Parameters<DsimAddressInput>) -> String {
        tools::dsim_token(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get token holders via Dune Simulator")]
    async fn dsim_holders(&self, Parameters(input): Parameters<DsimTokenInput>) -> String {
        tools::dsim_holders(&input.token, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get DeFi positions via Dune Simulator")]
    async fn dsim_defi(&self, Parameters(input): Parameters<DsimAddressInput>) -> String {
        tools::dsim_defi(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // DUNE
    // =========================================================================

    #[tool(description = "Execute a Dune Analytics query")]
    async fn dune_query(&self, Parameters(input): Parameters<DuneQueryInput>) -> String {
        tools::dune_query(&input.query_id).await.to_response()
    }

    #[tool(description = "Execute SQL directly on Dune")]
    async fn dune_sql(&self, Parameters(input): Parameters<DuneSqlInput>) -> String {
        tools::dune_sql(&input.sql).await.to_response()
    }

    #[tool(description = "Get Dune query execution results")]
    async fn dune_execution(&self, Parameters(input): Parameters<DuneExecutionInput>) -> String {
        tools::dune_execution(&input.execution_id)
            .await
            .to_response()
    }

    // =========================================================================
    // CURVE (additional)
    // =========================================================================

    #[tool(description = "Get Curve pool volumes")]
    async fn curve_volumes(&self, Parameters(input): Parameters<CurvePoolsInput>) -> String {
        tools::curve_volumes(Some(&input.chain)).await.to_response()
    }

    #[tool(description = "Get Curve lending markets")]
    async fn curve_lending(&self, Parameters(input): Parameters<CurvePoolsInput>) -> String {
        tools::curve_lending(Some(&input.chain)).await.to_response()
    }

    #[tool(description = "Get Curve token list")]
    async fn curve_tokens(&self, Parameters(input): Parameters<CurvePoolsInput>) -> String {
        tools::curve_tokens(Some(&input.chain)).await.to_response()
    }

    #[tool(description = "Get crvUSD market data")]
    async fn curve_crvusd(&self) -> String {
        tools::curve_crvusd().await.to_response()
    }

    #[tool(description = "Get Curve token prices")]
    async fn curve_prices(&self, Parameters(input): Parameters<CurvePoolsInput>) -> String {
        tools::curve_prices(Some(&input.chain)).await.to_response()
    }

    #[tool(description = "Get Curve pool OHLC data")]
    async fn curve_ohlc(&self, Parameters(input): Parameters<CurvePoolInput>) -> String {
        tools::curve_ohlc(&input.pool, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get Curve pool trades")]
    async fn curve_trades(&self, Parameters(input): Parameters<CurvePoolInput>) -> String {
        tools::curve_trades(&input.pool, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get Curve DAO data")]
    async fn curve_dao(&self) -> String {
        tools::curve_dao().await.to_response()
    }

    // =========================================================================
    // QUOTE (additional)
    // =========================================================================

    #[tool(description = "Get quote from a specific aggregator")]
    async fn quote_from(&self, Parameters(input): Parameters<QuoteCompareInput>) -> String {
        // Uses same input as compare but with from_token as aggregator name
        tools::quote_from(
            &input.from_token,
            &input.to_token,
            &input.amount,
            "", // empty for this signature
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    // =========================================================================
    // CHAINLINK (additional)
    // =========================================================================

    #[tool(description = "Get Chainlink data streams info")]
    async fn chainlink_streams(&self) -> String {
        tools::chainlink_streams().await.to_response()
    }

    // =========================================================================
    // CCXT (additional)
    // =========================================================================

    #[tool(description = "Get all tickers from an exchange")]
    async fn ccxt_tickers(&self, Parameters(input): Parameters<CcxtExchangeInput>) -> String {
        tools::ccxt_tickers(&input.exchange, None)
            .await
            .to_response()
    }

    #[tool(description = "Get OHLCV candlestick data from an exchange")]
    async fn ccxt_ohlcv(&self, Parameters(input): Parameters<CcxtOhlcvInput>) -> String {
        tools::ccxt_ohlcv(&input.exchange, &input.symbol, input.timeframe.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get recent trades from an exchange")]
    async fn ccxt_trades(&self, Parameters(input): Parameters<CcxtTradesInput>) -> String {
        tools::ccxt_trades(&input.exchange, &input.symbol)
            .await
            .to_response()
    }

    #[tool(description = "Get market info from an exchange")]
    async fn ccxt_markets(&self, Parameters(input): Parameters<CcxtExchangeInput>) -> String {
        tools::ccxt_markets(&input.exchange).await.to_response()
    }

    #[tool(description = "Compare prices across exchanges")]
    async fn ccxt_compare(&self, Parameters(input): Parameters<CcxtCompareInput>) -> String {
        tools::ccxt_compare(&input.symbol, input.exchanges.as_deref())
            .await
            .to_response()
    }

    // =========================================================================
    // UNISWAP (additional)
    // =========================================================================

    #[tool(description = "Get Uniswap pool liquidity data")]
    async fn uniswap_liquidity(
        &self,
        Parameters(input): Parameters<UniswapPoolAddressInput>,
    ) -> String {
        tools::uniswap_liquidity(&input.pool, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get recent swaps for a Uniswap pool")]
    async fn uniswap_swaps(
        &self,
        Parameters(input): Parameters<UniswapPoolAddressInput>,
    ) -> String {
        tools::uniswap_swaps(&input.pool, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get Uniswap pool daily data")]
    async fn uniswap_day_data(
        &self,
        Parameters(input): Parameters<UniswapPoolAddressInput>,
    ) -> String {
        tools::uniswap_day_data(&input.pool, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get token balance from Uniswap")]
    async fn uniswap_balance(&self, Parameters(input): Parameters<UniswapBalanceInput>) -> String {
        tools::uniswap_balance(&input.address, &input.token, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get Uniswap contract addresses for a chain")]
    async fn uniswap_addresses(
        &self,
        Parameters(input): Parameters<UniswapEthPriceInput>,
    ) -> String {
        tools::uniswap_addresses(Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // KONG (Yearn)
    // =========================================================================

    #[tool(description = "Get Yearn vaults via Kong")]
    async fn kong_vaults(&self, Parameters(input): Parameters<KongChainInput>) -> String {
        tools::kong_vaults(Some(&input.chain)).await.to_response()
    }

    #[tool(description = "List all strategies via Kong")]
    async fn kong_strategies(&self, Parameters(input): Parameters<KongChainInput>) -> String {
        tools::kong_strategies(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get token prices via Kong")]
    async fn kong_prices(&self, Parameters(input): Parameters<KongPricesInput>) -> String {
        tools::kong_prices(&input.tokens, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get Yearn TVL via Kong")]
    async fn kong_tvl(&self, Parameters(input): Parameters<KongChainInput>) -> String {
        tools::kong_tvl(Some(&input.chain)).await.to_response()
    }

    #[tool(description = "Get vault reports via Kong")]
    async fn kong_reports(&self, Parameters(input): Parameters<KongVaultInput>) -> String {
        tools::kong_reports(&input.vault, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // 1INCH (additional)
    // =========================================================================

    #[tool(description = "Get swap calldata from 1inch")]
    async fn oneinch_swap(&self, Parameters(input): Parameters<OneinchSwapInput>) -> String {
        tools::oneinch_swap(
            &input.src,
            &input.dst,
            &input.amount,
            &input.from,
            input.chain_id,
        )
        .await
        .to_response()
    }

    #[tool(description = "Get supported tokens from 1inch")]
    async fn oneinch_tokens(&self, Parameters(input): Parameters<OneinchChainInput>) -> String {
        tools::oneinch_tokens(input.chain_id).await.to_response()
    }

    #[tool(description = "Get liquidity sources from 1inch")]
    async fn oneinch_sources(&self, Parameters(input): Parameters<OneinchChainInput>) -> String {
        tools::oneinch_sources(input.chain_id).await.to_response()
    }

    #[tool(description = "Get spender address from 1inch")]
    async fn oneinch_spender(&self, Parameters(input): Parameters<OneinchChainInput>) -> String {
        tools::oneinch_spender(input.chain_id).await.to_response()
    }

    #[tool(description = "Check token allowance for 1inch")]
    async fn oneinch_allowance(
        &self,
        Parameters(input): Parameters<OneinchAllowanceInput>,
    ) -> String {
        tools::oneinch_allowance(&input.token, &input.owner, input.chain_id)
            .await
            .to_response()
    }

    #[tool(description = "Get approve calldata for 1inch")]
    async fn oneinch_approve(&self, Parameters(input): Parameters<OneinchApproveInput>) -> String {
        tools::oneinch_approve(&input.token, input.amount.as_deref(), input.chain_id)
            .await
            .to_response()
    }

    // =========================================================================
    // OPENOCEAN (additional)
    // =========================================================================

    #[tool(description = "Get swap calldata from OpenOcean")]
    async fn openocean_swap(&self, Parameters(input): Parameters<OpenoceanSwapInput>) -> String {
        tools::openocean_swap(
            &input.in_token,
            &input.out_token,
            &input.amount,
            &input.account,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get reverse quote from OpenOcean")]
    async fn openocean_reverse_quote(
        &self,
        Parameters(input): Parameters<OpenoceanQuoteInput>,
    ) -> String {
        tools::openocean_reverse_quote(
            &input.in_token,
            &input.out_token,
            &input.amount,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get supported tokens from OpenOcean")]
    async fn openocean_tokens(&self, Parameters(input): Parameters<OpenoceanChainInput>) -> String {
        tools::openocean_tokens(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get supported DEXes from OpenOcean")]
    async fn openocean_dexes(&self, Parameters(input): Parameters<OpenoceanChainInput>) -> String {
        tools::openocean_dexes(Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // KYBERSWAP
    // =========================================================================

    #[tool(description = "Get swap routes from KyberSwap")]
    async fn kyberswap_routes(&self, Parameters(input): Parameters<KyberRoutesInput>) -> String {
        tools::kyberswap_routes(
            &input.token_in,
            &input.token_out,
            &input.amount_in,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get route data from KyberSwap")]
    async fn kyberswap_route_data(
        &self,
        Parameters(input): Parameters<KyberRoutesInput>,
    ) -> String {
        tools::kyberswap_route_data(
            &input.token_in,
            &input.token_out,
            &input.amount_in,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Build swap transaction from KyberSwap")]
    async fn kyberswap_build(&self, Parameters(input): Parameters<KyberBuildInput>) -> String {
        tools::kyberswap_build(
            &input.route_summary,
            &input.sender,
            &input.recipient,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    // =========================================================================
    // 0X (additional)
    // =========================================================================

    #[tool(description = "Get price indicative quote from 0x")]
    async fn zerox_price(&self, Parameters(input): Parameters<ZeroxPriceInput>) -> String {
        tools::zerox_price(
            &input.sell_token,
            &input.buy_token,
            &input.sell_amount,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get liquidity sources from 0x")]
    async fn zerox_sources(&self, Parameters(input): Parameters<ZeroxChainInput>) -> String {
        tools::zerox_sources(Some(&input.chain)).await.to_response()
    }

    // =========================================================================
    // COWSWAP (additional)
    // =========================================================================

    #[tool(description = "Get order details from CoW Swap")]
    async fn cowswap_order(&self, Parameters(input): Parameters<CowswapOrderInput>) -> String {
        tools::cowswap_order(&input.order_uid, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get orders for an address from CoW Swap")]
    async fn cowswap_orders(&self, Parameters(input): Parameters<CowswapOwnerInput>) -> String {
        tools::cowswap_orders(&input.owner, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get trades for an address from CoW Swap")]
    async fn cowswap_trades(&self, Parameters(input): Parameters<CowswapOwnerInput>) -> String {
        tools::cowswap_trades(&input.owner, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get trades for an order from CoW Swap")]
    async fn cowswap_order_trades(
        &self,
        Parameters(input): Parameters<CowswapOrderInput>,
    ) -> String {
        tools::cowswap_order_trades(&input.order_uid, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get current auction from CoW Swap")]
    async fn cowswap_auction(&self, Parameters(input): Parameters<CowswapChainInput>) -> String {
        tools::cowswap_auction(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get solver competition data from CoW Swap")]
    async fn cowswap_competition(
        &self,
        Parameters(input): Parameters<CowswapAuctionInput>,
    ) -> String {
        tools::cowswap_competition(&input.auction_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get native token price from CoW Swap")]
    async fn cowswap_native_price(
        &self,
        Parameters(input): Parameters<CowswapTokenInput>,
    ) -> String {
        tools::cowswap_native_price(&input.token, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // LIFI (additional)
    // =========================================================================

    #[tool(description = "Get multiple routes from LI.FI")]
    async fn lifi_routes(&self, Parameters(input): Parameters<LifiQuoteInput>) -> String {
        tools::lifi_routes(
            &input.from_chain,
            &input.from_token,
            &input.to_chain,
            &input.to_token,
            &input.amount,
            &input.from_address,
        )
        .await
        .to_response()
    }

    #[tool(description = "Get best route from LI.FI")]
    async fn lifi_best_route(&self, Parameters(input): Parameters<LifiQuoteInput>) -> String {
        tools::lifi_best_route(
            &input.from_chain,
            &input.from_token,
            &input.to_chain,
            &input.to_token,
            &input.amount,
            &input.from_address,
        )
        .await
        .to_response()
    }

    #[tool(description = "Get transaction status from LI.FI")]
    async fn lifi_status(&self, Parameters(input): Parameters<LifiStatusInput>) -> String {
        tools::lifi_status(&input.tx_hash, input.bridge.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get supported chains from LI.FI")]
    async fn lifi_chains(&self) -> String {
        tools::lifi_chains().await.to_response()
    }

    #[tool(description = "Get chain details from LI.FI")]
    async fn lifi_chain(&self, Parameters(input): Parameters<LifiChainInput>) -> String {
        tools::lifi_chain(&input.chain).await.to_response()
    }

    #[tool(description = "Get supported tokens from LI.FI")]
    async fn lifi_tokens(&self, Parameters(input): Parameters<CurvePoolsInput>) -> String {
        tools::lifi_tokens(Some(&input.chain)).await.to_response()
    }

    #[tool(description = "Get available tools from LI.FI")]
    async fn lifi_tools(&self) -> String {
        tools::lifi_tools().await.to_response()
    }

    #[tool(description = "Get supported bridges from LI.FI")]
    async fn lifi_bridges(&self) -> String {
        tools::lifi_bridges().await.to_response()
    }

    #[tool(description = "Get supported exchanges from LI.FI")]
    async fn lifi_exchanges(&self) -> String {
        tools::lifi_exchanges().await.to_response()
    }

    #[tool(description = "Get gas prices for a chain from LI.FI")]
    async fn lifi_gas(&self, Parameters(input): Parameters<LifiGasInput>) -> String {
        tools::lifi_gas(&input.chain_id).await.to_response()
    }

    #[tool(description = "Get cross-chain connections from LI.FI")]
    async fn lifi_connections(
        &self,
        Parameters(input): Parameters<LifiConnectionsInput>,
    ) -> String {
        tools::lifi_connections(&input.from_chain, &input.to_chain)
            .await
            .to_response()
    }

    // =========================================================================
    // VELORA
    // =========================================================================

    #[tool(description = "Get token price from Velora")]
    async fn velora_price(&self, Parameters(input): Parameters<VeloraTokenInput>) -> String {
        tools::velora_price(&input.token, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get transaction details from Velora")]
    async fn velora_transaction(&self, Parameters(input): Parameters<VeloraTxInput>) -> String {
        tools::velora_transaction(&input.hash, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get supported tokens from Velora")]
    async fn velora_tokens(&self, Parameters(input): Parameters<VeloraChainInput>) -> String {
        tools::velora_tokens(Some(&input.chain)).await.to_response()
    }

    // =========================================================================
    // ENSO
    // =========================================================================

    #[tool(description = "Get swap route from Enso")]
    async fn enso_route(&self, Parameters(input): Parameters<EnsoRouteInput>) -> String {
        tools::enso_route(
            &input.from_token,
            &input.to_token,
            &input.amount,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get token price from Enso")]
    async fn enso_price(&self, Parameters(input): Parameters<EnsoPriceInput>) -> String {
        tools::enso_price(&input.token, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get token balances from Enso")]
    async fn enso_balances(&self, Parameters(input): Parameters<EnsoBalancesInput>) -> String {
        tools::enso_balances(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // PYTH (additional)
    // =========================================================================

    #[tool(description = "List Pyth price feeds")]
    async fn pyth_feeds(&self, Parameters(input): Parameters<PythFeedsInput>) -> String {
        tools::pyth_feeds(input.asset_type.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get known Pyth price feeds")]
    async fn pyth_known_feeds(&self) -> String {
        tools::pyth_known_feeds().await.to_response()
    }

    // =========================================================================
    // CONFIG
    // =========================================================================

    #[tool(description = "Initialize ethcli config file")]
    async fn config_init(&self) -> String {
        tools::config_init().await.to_response()
    }

    #[tool(description = "Get the path to ethcli config file")]
    async fn config_path(&self) -> String {
        tools::config_path().await.to_response()
    }

    #[tool(description = "Show current ethcli config")]
    async fn config_show(&self) -> String {
        tools::config_show().await.to_response()
    }

    #[tool(description = "Validate ethcli config file")]
    async fn config_validate(&self) -> String {
        tools::config_validate().await.to_response()
    }

    #[tool(description = "Set Etherscan API key in config")]
    async fn config_set_etherscan_key(
        &self,
        Parameters(input): Parameters<ConfigKeyInput>,
    ) -> String {
        tools::config_set_etherscan_key(&input.key)
            .await
            .to_response()
    }

    #[tool(description = "Set Tenderly credentials in config")]
    async fn config_set_tenderly(
        &self,
        Parameters(input): Parameters<ConfigTenderlyInput>,
    ) -> String {
        tools::config_set_tenderly(&input.account, &input.project, &input.key)
            .await
            .to_response()
    }

    #[tool(description = "Set Moralis API key in config")]
    async fn config_set_moralis(&self, Parameters(input): Parameters<ConfigKeyInput>) -> String {
        tools::config_set_moralis(&input.key).await.to_response()
    }

    #[tool(description = "Set Alchemy API key in config")]
    async fn config_set_alchemy(&self, Parameters(input): Parameters<ConfigKeyInput>) -> String {
        tools::config_set_alchemy(&input.key).await.to_response()
    }

    #[tool(description = "Set Dune API key in config")]
    async fn config_set_dune(&self, Parameters(input): Parameters<ConfigKeyInput>) -> String {
        tools::config_set_dune(&input.key).await.to_response()
    }

    #[tool(description = "Set Dune simulation API key in config")]
    async fn config_set_dune_sim(&self, Parameters(input): Parameters<ConfigKeyInput>) -> String {
        tools::config_set_dune_sim(&input.key).await.to_response()
    }

    #[tool(description = "Set Solodit API key in config")]
    async fn config_set_solodit(&self, Parameters(input): Parameters<ConfigKeyInput>) -> String {
        tools::config_set_solodit(&input.key).await.to_response()
    }

    #[tool(description = "Set Chainlink Data Streams credentials")]
    async fn config_set_chainlink(
        &self,
        Parameters(input): Parameters<ConfigChainlinkInput>,
    ) -> String {
        tools::config_set_chainlink(&input.key, &input.secret)
            .await
            .to_response()
    }

    #[tool(description = "Add debug RPC endpoint")]
    async fn config_add_debug_rpc(
        &self,
        Parameters(input): Parameters<ConfigDebugRpcInput>,
    ) -> String {
        tools::config_add_debug_rpc(&input.url, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Remove debug RPC endpoint")]
    async fn config_remove_debug_rpc(
        &self,
        Parameters(input): Parameters<ConfigDebugRpcInput>,
    ) -> String {
        tools::config_remove_debug_rpc(&input.url, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // ENDPOINTS
    // =========================================================================

    #[tool(description = "List configured RPC endpoints")]
    async fn endpoints_list(&self, Parameters(input): Parameters<EndpointsChainInput>) -> String {
        tools::endpoints_list(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Add a new RPC endpoint")]
    async fn endpoints_add(&self, Parameters(input): Parameters<EndpointsUrlInput>) -> String {
        tools::endpoints_add(&input.url, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Remove an RPC endpoint")]
    async fn endpoints_remove(&self, Parameters(input): Parameters<EndpointsUrlInput>) -> String {
        tools::endpoints_remove(&input.url, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Check health of RPC endpoints")]
    async fn endpoints_health(&self, Parameters(input): Parameters<EndpointsChainInput>) -> String {
        tools::endpoints_health(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Enable a disabled RPC endpoint")]
    async fn endpoints_enable(&self, Parameters(input): Parameters<EndpointsUrlInput>) -> String {
        tools::endpoints_enable(&input.url, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Disable an RPC endpoint")]
    async fn endpoints_disable(&self, Parameters(input): Parameters<EndpointsUrlInput>) -> String {
        tools::endpoints_disable(&input.url, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Optimize/rank RPC endpoints by performance")]
    async fn endpoints_optimize(
        &self,
        Parameters(input): Parameters<EndpointsOptimizeInput>,
    ) -> String {
        tools::endpoints_optimize(input.url.as_deref(), Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Test RPC endpoint for archive support")]
    async fn endpoints_test(&self, Parameters(input): Parameters<EndpointsUrlInput>) -> String {
        tools::endpoints_test(&input.url, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // HEALTH CHECK
    // =========================================================================

    #[tool(description = "Health check - verify ethcli-mcp and ethcli are working")]
    async fn health(&self) -> String {
        tools::health().await
    }
}

// Server handler implementation
#[tool_handler(router = self.tool_router)]
impl ServerHandler for EthcliMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Comprehensive Ethereum CLI tools: transaction analysis, account queries, \
                 contract interactions, ENS, gas prices, DEX quotes, oracles (Chainlink/Pyth), \
                 security checks (GoPlus/Solodit), portfolio tracking, and more. \
                 Supports multiple chains: ethereum, polygon, arbitrum, optimism, base, etc."
                    .to_string(),
            ),
        }
    }
}

/// Verify ethcli is available and get version
async fn verify_ethcli() -> anyhow::Result<String> {
    use tokio::process::Command;

    let output = Command::new("ethcli")
        .arg("--version")
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute ethcli: {}. Is ethcli installed?", e))?;

    if !output.status.success() {
        anyhow::bail!("ethcli --version failed. Is ethcli installed correctly?");
    }

    let version = String::from_utf8_lossy(&output.stdout);
    Ok(version.trim().to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to stderr (NEVER stdout!)
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(false)
                .with_target(false),
        )
        .with(EnvFilter::from_default_env().add_directive("ethcli_mcp=info".parse()?))
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting ethcli-mcp server"
    );

    // Verify ethcli is available
    match verify_ethcli().await {
        Ok(ethcli_version) => {
            tracing::info!(ethcli_version = %ethcli_version, "ethcli verified");
        }
        Err(e) => {
            tracing::warn!(error = %e, "ethcli verification failed - some tools may not work");
        }
    }

    // Create server and run with STDIO transport
    let server = EthcliMcpServer::new();
    let service = server.serve(stdio()).await?;

    tracing::info!(tools = 236, "ethcli-mcp server ready");

    // Wait for completion
    service.waiting().await?;

    tracing::info!("ethcli-mcp server shutting down");

    Ok(())
}
