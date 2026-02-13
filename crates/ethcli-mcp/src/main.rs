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
        description = "Analyze an Ethereum transaction including decoded events, token transfers, and method calls. Supports partial tx hashes when block number is provided."
    )]
    async fn tx_analyze(&self, Parameters(input): Parameters<TxAnalyzeInput>) -> String {
        tools::tx_analyze(&input.hash, Some(&input.chain), input.block, input.trace)
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

    #[tool(description = "Get the native token (ETH) balance for one or more addresses")]
    async fn account_balance(&self, Parameters(input): Parameters<AccountBalanceInput>) -> String {
        tools::account_balance(&input.addresses, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "List transactions for an address with pagination")]
    async fn account_txs(&self, Parameters(input): Parameters<AccountTxsInput>) -> String {
        tools::account_txs(
            &input.address,
            Some(&input.chain),
            input.page,
            input.limit,
            input.sort.as_deref(),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get internal transactions for an address with pagination")]
    async fn account_internal_txs(
        &self,
        Parameters(input): Parameters<AccountInternalTxsInput>,
    ) -> String {
        tools::account_internal_txs(&input.address, Some(&input.chain), input.page, input.limit)
            .await
            .to_response()
    }

    #[tool(
        description = "Get ERC20 token transfers for an address with optional token filter and pagination"
    )]
    async fn account_erc20(&self, Parameters(input): Parameters<AccountErc20Input>) -> String {
        tools::account_erc20(
            &input.address,
            Some(&input.chain),
            input.token.as_deref(),
            input.page,
            input.limit,
        )
        .await
        .to_response()
    }

    #[tool(
        description = "Get ERC721 NFT transfers for an address with optional token filter and pagination"
    )]
    async fn account_erc721(&self, Parameters(input): Parameters<AccountErc721Input>) -> String {
        tools::account_erc721(
            &input.address,
            Some(&input.chain),
            input.token.as_deref(),
            input.page,
            input.limit,
        )
        .await
        .to_response()
    }

    #[tool(
        description = "Get ERC1155 token transfers for an address with optional token filter and pagination"
    )]
    async fn account_erc1155(&self, Parameters(input): Parameters<AccountErc1155Input>) -> String {
        tools::account_erc1155(
            &input.address,
            Some(&input.chain),
            input.token.as_deref(),
            input.page,
            input.limit,
        )
        .await
        .to_response()
    }

    // =========================================================================
    // CONTRACT
    // =========================================================================

    #[tool(
        description = "Fetch the ABI for a verified contract from Etherscan (optionally save to file)"
    )]
    async fn contract_abi(&self, Parameters(input): Parameters<ContractAbiInput>) -> String {
        tools::contract_abi(&input.address, Some(&input.chain), input.output.as_deref())
            .await
            .to_response()
    }

    #[tool(
        description = "Download verified source code for a contract (optionally save to directory)"
    )]
    async fn contract_source(&self, Parameters(input): Parameters<ContractSourceInput>) -> String {
        tools::contract_source(&input.address, Some(&input.chain), input.output.as_deref())
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

    #[tool(
        description = "Extract function selectors from bytecode using evmole. Returns selectors, arguments, and state mutability without requiring ABI or source code."
    )]
    async fn contract_selectors(
        &self,
        Parameters(input): Parameters<ContractSelectorsInput>,
    ) -> String {
        tools::contract_selectors(&input.address, Some(&input.chain), input.lookup)
            .await
            .to_response()
    }

    #[tool(
        description = "Disassemble contract bytecode into EVM opcodes. Shows offset, opcode name, and operands."
    )]
    async fn contract_disassemble(
        &self,
        Parameters(input): Parameters<ContractDisassembleInput>,
    ) -> String {
        tools::contract_disassemble(&input.address, Some(&input.chain), input.limit)
            .await
            .to_response()
    }

    #[tool(
        description = "Get opcode frequency statistics for contract bytecode. Shows category totals and top opcodes by frequency."
    )]
    async fn contract_opcodes(
        &self,
        Parameters(input): Parameters<ContractOpcodesInput>,
    ) -> String {
        tools::contract_opcodes(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(
        description = "Comprehensive bytecode security analysis. Detects dangerous patterns like SELFDESTRUCT, DELEGATECALL, tx.origin auth (honeypot indicator), and dynamic contract creation. Combines function extraction, security scanning, and opcode statistics. Use follow_proxy to analyze implementation contracts."
    )]
    async fn contract_analyze(
        &self,
        Parameters(input): Parameters<ContractAnalyzeInput>,
    ) -> String {
        tools::contract_analyze(
            &input.address,
            Some(&input.chain),
            input.include_disassembly,
            input.limit,
            input.lookup,
            input.follow_proxy,
        )
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

    #[tool(
        description = "Get token balance for one or more tokens and holders, with optional tag filtering"
    )]
    async fn token_balance(&self, Parameters(input): Parameters<TokenBalanceInput>) -> String {
        tools::token_balance(
            &input.tokens,
            &input.holders,
            input.tag.as_deref(),
            input.show_zero,
            Some(&input.chain),
        )
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

    #[tool(description = "Get block information with optional full transactions")]
    async fn rpc_block(&self, Parameters(input): Parameters<RpcBlockInput>) -> String {
        tools::rpc_block(
            &input.block,
            Some(&input.chain),
            input.full,
            input.rpc_url.as_deref(),
        )
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

    #[tool(
        description = "Get aggregated portfolio/balances from multiple sources. Supports multiple addresses, tag filtering, aggregation, and min-value filtering."
    )]
    async fn portfolio(&self, Parameters(input): Parameters<PortfolioInput>) -> String {
        tools::portfolio(
            &input.addresses,
            input.tag.as_deref(),
            input.aggregate,
            input.chain.as_deref(),
            input.source.as_deref(),
            input.min_value,
            input.show_sources,
            input.show_blacklisted,
        )
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

    #[tool(
        description = "Get best swap quote across DEX aggregators. Supports human-readable amounts with decimals param."
    )]
    async fn quote_best(&self, Parameters(input): Parameters<QuoteBestInput>) -> String {
        tools::quote_best(
            &input.from_token,
            &input.to_token,
            &input.amount,
            Some(&input.chain),
            input.decimals,
            input.sender.as_deref(),
            input.slippage,
            input.show_tx,
        )
        .await
        .to_response()
    }

    #[tool(description = "Compare quotes across DEX aggregators side-by-side")]
    async fn quote_compare(&self, Parameters(input): Parameters<QuoteCompareInput>) -> String {
        tools::quote_compare(
            &input.from_token,
            &input.to_token,
            &input.amount,
            Some(&input.chain),
            input.decimals,
            input.sender.as_deref(),
            input.slippage,
            input.show_tx,
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
        tools::alchemy_transfers(&input.address, Some(&input.chain))
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
        tools::llama_yields(input.chain.as_deref(), None)
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
            Some(&input.gas_price),
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

    #[tool(description = "Simulate a contract call with full state/block override support")]
    async fn simulate_call(&self, Parameters(input): Parameters<SimulateCallInput>) -> String {
        tools::simulate_call(
            &input.contract,
            input.sig.as_deref(),
            input.data.as_deref(),
            input.args,
            Some(&input.chain),
            // Transaction params
            input.from.as_deref(),
            input.value.as_deref(),
            input.block.as_deref(),
            input.gas.as_deref(),
            input.gas_price.as_deref(),
            // State overrides
            &input.balance_overrides,
            &input.storage_overrides,
            &input.code_overrides,
            &input.nonce_overrides,
            // Block overrides
            input.block_timestamp,
            input.block_number_override,
            input.block_gas_limit.as_deref(),
            input.block_coinbase.as_deref(),
            input.block_difficulty.as_deref(),
            input.block_base_fee.as_deref(),
            input.transaction_index,
            // L2 options
            input.l1_block_number,
            input.l1_timestamp,
            input.l1_message_sender.as_deref(),
            input.deposit_tx,
            input.system_tx,
            // Simulation options
            input.via.as_deref(),
            input.rpc_url.as_deref(),
            input.trace,
            input.estimate_gas,
            input.generate_access_list,
            input.access_list.as_deref(),
            input.simulation_type.as_deref(),
            input.network_id,
            input.save,
            // Tenderly options
            input.tenderly_key.as_deref(),
            input.tenderly_account.as_deref(),
            input.tenderly_project.as_deref(),
            // Alchemy options
            input.alchemy_key.as_deref(),
            input.alchemy_network.as_deref(),
            // Debug options
            input.dry_run.as_deref(),
            input.show_secrets,
        )
        .await
        .to_response()
    }

    #[tool(description = "Simulate/trace a historical transaction")]
    async fn simulate_tx(&self, Parameters(input): Parameters<SimulateTxInput>) -> String {
        tools::simulate_tx(
            &input.hash,
            Some(&input.chain),
            input.via.as_deref(),
            input.trace,
        )
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

    #[tool(
        description = "Add an address to the local address book with optional description, tags, and chain"
    )]
    async fn address_add(&self, Parameters(input): Parameters<AddressAddInput>) -> String {
        tools::address_add(
            &input.name,
            &input.address,
            input.description.as_deref(),
            &input.tags,
            input.for_chain.as_deref(),
        )
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

    #[tool(description = "Import addresses from a JSON file with optional overwrite")]
    async fn address_import(&self, Parameters(input): Parameters<AddressImportInput>) -> String {
        tools::address_import(&input.file, input.overwrite)
            .await
            .to_response()
    }

    #[tool(description = "Export addresses to JSON (file or stdout)")]
    async fn address_export(&self, Parameters(input): Parameters<AddressExportInput>) -> String {
        tools::address_export(input.output.as_deref())
            .await
            .to_response()
    }

    // =========================================================================
    // BLACKLIST
    // =========================================================================

    #[tool(
        description = "Add a token to the local blacklist with optional symbol, reason, and chain"
    )]
    async fn blacklist_add(&self, Parameters(input): Parameters<BlacklistAddInput>) -> String {
        tools::blacklist_add(
            &input.address,
            input.symbol.as_deref(),
            input.reason.as_deref(),
            input.chain.as_deref(),
        )
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

    #[tool(
        description = "Call a contract function (read-only) with optional block, RPC URL, and human-readable output"
    )]
    async fn contract_call(&self, Parameters(input): Parameters<ContractCallInput>) -> String {
        tools::contract_call(
            &input.address,
            &input.sig,
            input.args,
            Some(&input.chain),
            input.block.as_deref(),
            input.rpc_url.as_deref(),
            input.human,
        )
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
    // TENDERLY VNETS - Full CRUD + Simulate + Admin
    // =========================================================================

    #[tool(description = "Create a new Tenderly Virtual TestNet forked from a network")]
    async fn tenderly_vnets_create(
        &self,
        Parameters(input): Parameters<TenderlyVnetsCreateInput>,
    ) -> String {
        tools::tenderly_vnets_create(
            &input.slug,
            &input.name,
            input.network_id,
            input.block_number,
            input.chain_id,
            input.sync_state,
        )
        .await
        .to_response()
    }

    #[tool(description = "Get details of a Tenderly Virtual TestNet by ID")]
    async fn tenderly_vnets_get(
        &self,
        Parameters(input): Parameters<TenderlyVnetsIdInput>,
    ) -> String {
        tools::tenderly_vnets_get(&input.id).await.to_response()
    }

    #[tool(description = "Delete one or more Tenderly Virtual TestNets")]
    async fn tenderly_vnets_delete(
        &self,
        Parameters(input): Parameters<TenderlyVnetsDeleteInput>,
    ) -> String {
        tools::tenderly_vnets_delete(&input.ids, input.all)
            .await
            .to_response()
    }

    #[tool(description = "Update a Tenderly Virtual TestNet (name, slug, sync state)")]
    async fn tenderly_vnets_update(
        &self,
        Parameters(input): Parameters<TenderlyVnetsUpdateInput>,
    ) -> String {
        tools::tenderly_vnets_update(
            &input.id,
            input.name.as_deref(),
            input.slug.as_deref(),
            input.sync_state,
        )
        .await
        .to_response()
    }

    #[tool(description = "Fork an existing Tenderly Virtual TestNet")]
    async fn tenderly_vnets_fork(
        &self,
        Parameters(input): Parameters<TenderlyVnetsForkInput>,
    ) -> String {
        tools::tenderly_vnets_fork(&input.source, &input.slug, &input.name, input.block_number)
            .await
            .to_response()
    }

    #[tool(description = "Get RPC URLs (public and admin) for a Virtual TestNet")]
    async fn tenderly_vnets_rpc(
        &self,
        Parameters(input): Parameters<TenderlyVnetsIdInput>,
    ) -> String {
        tools::tenderly_vnets_rpc(&input.id).await.to_response()
    }

    #[tool(description = "List transactions on a Virtual TestNet")]
    async fn tenderly_vnets_transactions(
        &self,
        Parameters(input): Parameters<TenderlyVnetsTransactionsInput>,
    ) -> String {
        tools::tenderly_vnets_transactions(&input.id, input.page, input.per_page)
            .await
            .to_response()
    }

    #[tool(description = "Get a specific transaction from a Virtual TestNet")]
    async fn tenderly_vnets_get_transaction(
        &self,
        Parameters(input): Parameters<TenderlyVnetsGetTransactionInput>,
    ) -> String {
        tools::tenderly_vnets_get_transaction(&input.vnet, &input.hash)
            .await
            .to_response()
    }

    #[tool(description = "Send a transaction to a Virtual TestNet (persists state)")]
    async fn tenderly_vnets_send(
        &self,
        Parameters(input): Parameters<TenderlyVnetsSendInput>,
    ) -> String {
        tools::tenderly_vnets_send(
            &input.vnet,
            &input.from,
            &input.to,
            input.data.as_deref(),
            input.value.as_deref(),
        )
        .await
        .to_response()
    }

    #[tool(description = "Simulate a transaction against a VNet's state without persisting")]
    async fn tenderly_vnets_simulate(
        &self,
        Parameters(input): Parameters<TenderlyVnetsSimulateInput>,
    ) -> String {
        tools::tenderly_vnets_simulate(
            &input.vnet,
            &input.from,
            &input.to,
            input.data.as_deref(),
            input.value.as_deref(),
            input.gas,
            input.gas_price.as_deref(),
            input.max_fee_per_gas.as_deref(),
            input.max_priority_fee_per_gas.as_deref(),
        )
        .await
        .to_response()
    }

    // --- VNet Admin tools ---

    #[tool(description = "Set ETH balance for an address on a Virtual TestNet")]
    async fn tenderly_vnets_admin_set_balance(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminBalanceInput>,
    ) -> String {
        tools::tenderly_vnets_admin_set_balance(&input.vnet, &input.address, &input.amount)
            .await
            .to_response()
    }

    #[tool(description = "Add ETH to an address balance on a Virtual TestNet")]
    async fn tenderly_vnets_admin_add_balance(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminBalanceInput>,
    ) -> String {
        tools::tenderly_vnets_admin_add_balance(&input.vnet, &input.address, &input.amount)
            .await
            .to_response()
    }

    #[tool(description = "Set ERC20 token balance for a wallet on a Virtual TestNet")]
    async fn tenderly_vnets_admin_set_erc20_balance(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminErc20Input>,
    ) -> String {
        let Some(amount) = input.amount.as_deref() else {
            return "Error: 'amount' is required for set_erc20_balance".to_string();
        };
        tools::tenderly_vnets_admin_set_erc20_balance(
            &input.vnet,
            &input.token,
            &input.wallet,
            amount,
        )
        .await
        .to_response()
    }

    #[tool(description = "Set maximum possible ERC20 token balance on a Virtual TestNet")]
    async fn tenderly_vnets_admin_set_max_erc20_balance(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminErc20Input>,
    ) -> String {
        tools::tenderly_vnets_admin_set_max_erc20_balance(&input.vnet, &input.token, &input.wallet)
            .await
            .to_response()
    }

    #[tool(description = "Advance blockchain time by seconds on a Virtual TestNet")]
    async fn tenderly_vnets_admin_increase_time(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminTimeInput>,
    ) -> String {
        tools::tenderly_vnets_admin_increase_time(&input.vnet, input.value)
            .await
            .to_response()
    }

    #[tool(description = "Set timestamp for the next block on a Virtual TestNet")]
    async fn tenderly_vnets_admin_set_timestamp(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminTimeInput>,
    ) -> String {
        tools::tenderly_vnets_admin_set_timestamp(&input.vnet, input.value)
            .await
            .to_response()
    }

    #[tool(description = "Set next block timestamp without mining on a Virtual TestNet")]
    async fn tenderly_vnets_admin_set_timestamp_no_mine(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminTimeInput>,
    ) -> String {
        tools::tenderly_vnets_admin_set_timestamp_no_mine(&input.vnet, input.value)
            .await
            .to_response()
    }

    #[tool(description = "Skip a number of blocks on a Virtual TestNet")]
    async fn tenderly_vnets_admin_increase_blocks(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminBlocksInput>,
    ) -> String {
        tools::tenderly_vnets_admin_increase_blocks(&input.vnet, input.blocks)
            .await
            .to_response()
    }

    #[tool(description = "Create a state snapshot on a Virtual TestNet")]
    async fn tenderly_vnets_admin_snapshot(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminVnetInput>,
    ) -> String {
        tools::tenderly_vnets_admin_snapshot(&input.vnet)
            .await
            .to_response()
    }

    #[tool(description = "Revert to a previous snapshot on a Virtual TestNet")]
    async fn tenderly_vnets_admin_revert(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminRevertInput>,
    ) -> String {
        tools::tenderly_vnets_admin_revert(&input.vnet, &input.snapshot_id)
            .await
            .to_response()
    }

    #[tool(description = "Set storage at a specific slot on a Virtual TestNet")]
    async fn tenderly_vnets_admin_set_storage(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminStorageInput>,
    ) -> String {
        tools::tenderly_vnets_admin_set_storage(
            &input.vnet,
            &input.address,
            &input.slot,
            &input.value,
        )
        .await
        .to_response()
    }

    #[tool(description = "Set bytecode at an address on a Virtual TestNet")]
    async fn tenderly_vnets_admin_set_code(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminCodeInput>,
    ) -> String {
        tools::tenderly_vnets_admin_set_code(&input.vnet, &input.address, &input.code)
            .await
            .to_response()
    }

    #[tool(description = "Send an unsigned transaction via Admin RPC on a Virtual TestNet")]
    async fn tenderly_vnets_admin_send_tx(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminSendTxInput>,
    ) -> String {
        tools::tenderly_vnets_admin_send_tx(
            &input.vnet,
            &input.from,
            input.to.as_deref(),
            input.data.as_deref(),
            input.value.as_deref(),
            input.gas.as_deref(),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get the latest block/transaction info on a Virtual TestNet")]
    async fn tenderly_vnets_admin_get_latest(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminVnetInput>,
    ) -> String {
        tools::tenderly_vnets_admin_get_latest(&input.vnet)
            .await
            .to_response()
    }

    #[tool(
        description = "Simulate a transaction via tenderly_simulateTransaction RPC. Supports state overrides (balance, nonce, code, storage) and block overrides. Returns rich decoded results including logs, traces, asset changes, and state changes."
    )]
    async fn tenderly_vnets_admin_simulate_tx(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminSimulateTxInput>,
    ) -> String {
        tools::tenderly_vnets_admin_simulate_tx(
            &input.vnet,
            &input.from,
            input.to.as_deref(),
            input.data.as_deref(),
            input.value.as_deref(),
            input.gas.as_deref(),
            &input.block,
            input.state_overrides.as_deref(),
            input.block_overrides.as_deref(),
        )
        .await
        .to_response()
    }

    #[tool(
        description = "Simulate a bundle of transactions via tenderly_simulateBundle RPC. Executes multiple transactions sequentially, each seeing state changes from previous ones. Supports state and block overrides."
    )]
    async fn tenderly_vnets_admin_simulate_bundle(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminSimulateBundleInput>,
    ) -> String {
        tools::tenderly_vnets_admin_simulate_bundle(
            &input.vnet,
            &input.txs,
            input.state_overrides.as_deref(),
            input.block_overrides.as_deref(),
        )
        .await
        .to_response()
    }

    // --- Tenderly Actions Batch ---

    #[tool(description = "Stop multiple Tenderly web3 actions by ID")]
    async fn tenderly_actions_stop_many(
        &self,
        Parameters(input): Parameters<TenderlyActionsIdsInput>,
    ) -> String {
        tools::tenderly_actions_stop_many(&input.ids)
            .await
            .to_response()
    }

    #[tool(description = "Resume multiple Tenderly web3 actions by ID")]
    async fn tenderly_actions_resume_many(
        &self,
        Parameters(input): Parameters<TenderlyActionsIdsInput>,
    ) -> String {
        tools::tenderly_actions_resume_many(&input.ids)
            .await
            .to_response()
    }

    #[tool(description = "Get execution details for a Tenderly web3 action call")]
    async fn tenderly_actions_get_call(
        &self,
        Parameters(input): Parameters<TenderlyActionsGetCallInput>,
    ) -> String {
        tools::tenderly_actions_get_call(&input.id, &input.execution_id)
            .await
            .to_response()
    }

    // --- Tenderly Alerts Batch ---

    #[tool(description = "Update a Tenderly alert configuration (name, type, network, addresses)")]
    async fn tenderly_alerts_update(
        &self,
        Parameters(input): Parameters<TenderlyAlertsUpdateInput>,
    ) -> String {
        tools::tenderly_alerts_update(
            &input.id,
            input.name.as_deref(),
            input.alert_type.as_deref(),
            input.network.as_deref(),
            input.addresses.as_deref(),
        )
        .await
        .to_response()
    }

    #[tool(description = "Add a notification destination to a Tenderly alert")]
    async fn tenderly_alerts_add_destination(
        &self,
        Parameters(input): Parameters<TenderlyAlertsAddDestinationInput>,
    ) -> String {
        tools::tenderly_alerts_add_destination(
            &input.id,
            &input.destination_type,
            &input.destination_id,
        )
        .await
        .to_response()
    }

    #[tool(description = "Remove a notification destination from a Tenderly alert")]
    async fn tenderly_alerts_remove_destination(
        &self,
        Parameters(input): Parameters<TenderlyAlertsRemoveDestinationInput>,
    ) -> String {
        tools::tenderly_alerts_remove_destination(&input.id, &input.destination_id)
            .await
            .to_response()
    }

    // --- Tenderly Contracts Batch ---

    #[tool(description = "Update a Tenderly contract (name, network, tags)")]
    async fn tenderly_contracts_update(
        &self,
        Parameters(input): Parameters<TenderlyContractsUpdateInput>,
    ) -> String {
        tools::tenderly_contracts_update(
            &input.address,
            input.network.as_deref(),
            input.name.as_deref(),
            &input.tags,
        )
        .await
        .to_response()
    }

    #[tool(description = "Remove a tag from a Tenderly contract")]
    async fn tenderly_contracts_remove_tag(
        &self,
        Parameters(input): Parameters<TenderlyContractsRemoveTagInput>,
    ) -> String {
        tools::tenderly_contracts_remove_tag(&input.address, input.network.as_deref(), &input.tag)
            .await
            .to_response()
    }

    #[tool(description = "Apply a tag to multiple Tenderly contracts at once")]
    async fn tenderly_contracts_bulk_tag(
        &self,
        Parameters(input): Parameters<TenderlyContractsBulkTagInput>,
    ) -> String {
        tools::tenderly_contracts_bulk_tag(&input.tag, &input.contract_ids)
            .await
            .to_response()
    }

    #[tool(description = "Encode state overrides for Tenderly contract simulation")]
    async fn tenderly_contracts_encode_state(
        &self,
        Parameters(input): Parameters<TenderlyContractsEncodeStateInput>,
    ) -> String {
        tools::tenderly_contracts_encode_state(input.network.as_deref(), &input.state_json)
            .await
            .to_response()
    }

    // --- Tenderly VNets Admin Batch ---

    #[tool(description = "Set ETH balance for multiple addresses on a Virtual TestNet")]
    async fn tenderly_vnets_admin_set_balances(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminBatchBalanceInput>,
    ) -> String {
        tools::tenderly_vnets_admin_set_balances(&input.vnet, &input.addresses, &input.amount)
            .await
            .to_response()
    }

    #[tool(description = "Add ETH to balances for multiple addresses on a Virtual TestNet")]
    async fn tenderly_vnets_admin_add_balances(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminBatchBalanceInput>,
    ) -> String {
        tools::tenderly_vnets_admin_add_balances(&input.vnet, &input.addresses, &input.amount)
            .await
            .to_response()
    }

    #[tool(description = "Create an EIP-2930 access list for a transaction on a Virtual TestNet")]
    async fn tenderly_vnets_admin_create_access_list(
        &self,
        Parameters(input): Parameters<TenderlyVnetsAdminCreateAccessListInput>,
    ) -> String {
        tools::tenderly_vnets_admin_create_access_list(
            &input.vnet,
            &input.from,
            &input.to,
            input.data.as_deref(),
            input.value.as_deref(),
            input.gas.as_deref(),
            input.block.as_deref(),
        )
        .await
        .to_response()
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

    #[tool(description = "Get NFT metadata by contract and token ID via Alchemy")]
    async fn alchemy_nft_metadata(
        &self,
        Parameters(input): Parameters<AlchemyNftMetadataInput>,
    ) -> String {
        tools::alchemy_nft_metadata(&input.contract, &input.token_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get floor price for an NFT collection via Alchemy")]
    async fn alchemy_nft_floor_price(
        &self,
        Parameters(input): Parameters<AlchemyNftContractInput>,
    ) -> String {
        tools::alchemy_nft_floor_price(&input.contract, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get owners of a specific NFT via Alchemy")]
    async fn alchemy_nft_owners(
        &self,
        Parameters(input): Parameters<AlchemyNftMetadataInput>,
    ) -> String {
        tools::alchemy_nft_owners(&input.contract, &input.token_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Check if an address holds an NFT from a contract via Alchemy")]
    async fn alchemy_nft_is_holder(
        &self,
        Parameters(input): Parameters<AlchemyNftIsHolderInput>,
    ) -> String {
        tools::alchemy_nft_is_holder(&input.address, &input.contract, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get token metadata by contract address via Alchemy")]
    async fn alchemy_token_metadata(
        &self,
        Parameters(input): Parameters<AlchemyTokenMetadataInput>,
    ) -> String {
        tools::alchemy_token_metadata(&input.contract, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get token allowance for owner/spender pair via Alchemy")]
    async fn alchemy_token_allowances(
        &self,
        Parameters(input): Parameters<AlchemyTokenAllowancesInput>,
    ) -> String {
        tools::alchemy_token_allowances(&input.owner, &input.spender, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get token transfers TO an address via Alchemy")]
    async fn alchemy_transfers_to(
        &self,
        Parameters(input): Parameters<AlchemyTransfersToInput>,
    ) -> String {
        tools::alchemy_transfers_to(
            &input.address,
            input.from_block.as_deref(),
            input.to_block.as_deref(),
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get token prices by contract address via Alchemy")]
    async fn alchemy_prices_by_address(
        &self,
        Parameters(input): Parameters<AlchemyPricesByAddressInput>,
    ) -> String {
        tools::alchemy_prices_by_address(&input.addresses, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // ALCHEMY NFT (additional)
    // =========================================================================

    #[tool(description = "Get all owners of an NFT contract via Alchemy")]
    async fn alchemy_nft_owners_for_contract(
        &self,
        Parameters(input): Parameters<AlchemyNftContractOwnerInput>,
    ) -> String {
        tools::alchemy_nft_owners_for_contract(&input.contract, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get NFT contracts owned by an address via Alchemy")]
    async fn alchemy_nft_contracts_for_owner(
        &self,
        Parameters(input): Parameters<AlchemyNftAddressInput>,
    ) -> String {
        tools::alchemy_nft_contracts_for_owner(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get all NFTs for a contract with pagination via Alchemy")]
    async fn alchemy_nft_nfts_for_contract(
        &self,
        Parameters(input): Parameters<AlchemyNftForContractInput>,
    ) -> String {
        tools::alchemy_nft_nfts_for_contract(
            &input.contract,
            input.start_token.as_deref(),
            input.limit,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get contract metadata for an NFT contract via Alchemy")]
    async fn alchemy_nft_contract_metadata(
        &self,
        Parameters(input): Parameters<AlchemyNftContractOwnerInput>,
    ) -> String {
        tools::alchemy_nft_contract_metadata(&input.contract, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get collection metadata by OpenSea slug via Alchemy")]
    async fn alchemy_nft_collection_metadata(
        &self,
        Parameters(input): Parameters<AlchemyNftSlugInput>,
    ) -> String {
        tools::alchemy_nft_collection_metadata(&input.slug, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Search NFT contract metadata by keyword via Alchemy")]
    async fn alchemy_nft_search_contract_metadata(
        &self,
        Parameters(input): Parameters<AlchemyNftSearchInput>,
    ) -> String {
        tools::alchemy_nft_search_contract_metadata(&input.query, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Compute rarity for an NFT via Alchemy")]
    async fn alchemy_nft_compute_rarity(
        &self,
        Parameters(input): Parameters<AlchemyNftMetadataInput>,
    ) -> String {
        tools::alchemy_nft_compute_rarity(&input.contract, &input.token_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Summarize NFT attributes for a contract via Alchemy")]
    async fn alchemy_nft_summarize_attributes(
        &self,
        Parameters(input): Parameters<AlchemyNftContractOwnerInput>,
    ) -> String {
        tools::alchemy_nft_summarize_attributes(&input.contract, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Refresh metadata for a specific NFT via Alchemy")]
    async fn alchemy_nft_refresh_metadata(
        &self,
        Parameters(input): Parameters<AlchemyNftMetadataInput>,
    ) -> String {
        tools::alchemy_nft_refresh_metadata(&input.contract, &input.token_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(
        description = "Get NFT sales for a contract with optional token ID and block range via Alchemy"
    )]
    async fn alchemy_nft_sales(
        &self,
        Parameters(input): Parameters<AlchemyNftSalesInput>,
    ) -> String {
        tools::alchemy_nft_sales(
            &input.contract,
            input.token_id.as_deref(),
            input.from_block,
            input.to_block,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get list of known spam NFT contracts via Alchemy")]
    async fn alchemy_nft_spam_contracts(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_nft_spam_contracts(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Check if an NFT contract is spam via Alchemy")]
    async fn alchemy_nft_is_spam(
        &self,
        Parameters(input): Parameters<AlchemyNftContractOwnerInput>,
    ) -> String {
        tools::alchemy_nft_is_spam(&input.contract, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Check if an NFT is an airdrop via Alchemy")]
    async fn alchemy_nft_is_airdrop(
        &self,
        Parameters(input): Parameters<AlchemyNftMetadataInput>,
    ) -> String {
        tools::alchemy_nft_is_airdrop(&input.contract, &input.token_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Report an NFT contract as spam via Alchemy")]
    async fn alchemy_nft_report_spam(
        &self,
        Parameters(input): Parameters<AlchemyNftContractOwnerInput>,
    ) -> String {
        tools::alchemy_nft_report_spam(&input.contract, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get all NFTs for a collection by slug via Alchemy")]
    async fn alchemy_nft_nfts_for_collection(
        &self,
        Parameters(input): Parameters<AlchemyNftForCollectionInput>,
    ) -> String {
        tools::alchemy_nft_nfts_for_collection(
            &input.slug,
            input.start_token.as_deref(),
            input.limit,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get collections owned by an address via Alchemy")]
    async fn alchemy_nft_collections_for_owner(
        &self,
        Parameters(input): Parameters<AlchemyNftAddressInput>,
    ) -> String {
        tools::alchemy_nft_collections_for_owner(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Invalidate cached metadata for an NFT contract via Alchemy")]
    async fn alchemy_nft_invalidate_contract(
        &self,
        Parameters(input): Parameters<AlchemyNftContractOwnerInput>,
    ) -> String {
        tools::alchemy_nft_invalidate_contract(&input.contract, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // ALCHEMY TOKEN (additional)
    // =========================================================================

    #[tool(description = "Get token balances for specific tokens at an address via Alchemy")]
    async fn alchemy_token_balances_for_tokens(
        &self,
        Parameters(input): Parameters<AlchemyTokenBalancesForTokensInput>,
    ) -> String {
        tools::alchemy_token_balances_for_tokens(&input.address, &input.tokens, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // ALCHEMY TRANSFERS (additional)
    // =========================================================================

    #[tool(description = "Get all transfers (sent and received) for an address via Alchemy")]
    async fn alchemy_transfers_all(
        &self,
        Parameters(input): Parameters<AlchemyTransfersAllInput>,
    ) -> String {
        tools::alchemy_transfers_all(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // ALCHEMY PORTFOLIO (additional)
    // =========================================================================

    #[tool(description = "Get token info for multiple tokens via Alchemy Portfolio API")]
    async fn alchemy_portfolio_token_info(
        &self,
        Parameters(input): Parameters<AlchemyPortfolioTokenInfoInput>,
    ) -> String {
        tools::alchemy_portfolio_token_info(&input.tokens, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get NFTs owned by an address via Alchemy Portfolio API")]
    async fn alchemy_portfolio_nfts(
        &self,
        Parameters(input): Parameters<AlchemyPortfolioNftsInput>,
    ) -> String {
        tools::alchemy_portfolio_nfts(&input.address, input.with_metadata, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get NFT contracts owned by an address via Alchemy Portfolio API")]
    async fn alchemy_portfolio_nft_contracts(
        &self,
        Parameters(input): Parameters<AlchemyPortfolioNftContractsInput>,
    ) -> String {
        tools::alchemy_portfolio_nft_contracts(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // ALCHEMY PRICES (additional)
    // =========================================================================

    #[tool(description = "Get token prices by symbol via Alchemy")]
    async fn alchemy_prices_by_symbol(
        &self,
        Parameters(input): Parameters<AlchemyPricesBySymbolInput>,
    ) -> String {
        tools::alchemy_prices_by_symbol(&input.symbols, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get historical token prices by symbol via Alchemy")]
    async fn alchemy_prices_historical_by_symbol(
        &self,
        Parameters(input): Parameters<AlchemyPricesHistoricalBySymbolInput>,
    ) -> String {
        tools::alchemy_prices_historical_by_symbol(
            &input.symbol,
            &input.start_time,
            &input.end_time,
            input.interval.as_deref(),
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get historical token prices by contract address via Alchemy")]
    async fn alchemy_prices_historical_by_address(
        &self,
        Parameters(input): Parameters<AlchemyPricesHistoricalByAddressInput>,
    ) -> String {
        tools::alchemy_prices_historical_by_address(
            &input.address,
            &input.start_time,
            &input.end_time,
            input.interval.as_deref(),
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    // =========================================================================
    // ALCHEMY DEBUG (additional)
    // =========================================================================

    #[tool(description = "Trace a call without executing it on-chain via Alchemy debug API")]
    async fn alchemy_debug_trace_call(
        &self,
        Parameters(input): Parameters<AlchemyDebugTraceCallInput>,
    ) -> String {
        tools::alchemy_debug_trace_call(
            &input.to,
            input.block.as_deref(),
            input.from.as_deref(),
            input.data.as_deref(),
            input.value.as_deref(),
            input.gas.as_deref(),
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Trace all transactions in a block by hash via Alchemy debug API")]
    async fn alchemy_debug_trace_block_by_hash(
        &self,
        Parameters(input): Parameters<AlchemyDebugBlockHashInput>,
    ) -> String {
        tools::alchemy_debug_trace_block_by_hash(&input.hash, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Trace all transactions in a block by number via Alchemy debug API")]
    async fn alchemy_debug_trace_block_by_number(
        &self,
        Parameters(input): Parameters<AlchemyDebugBlockInput>,
    ) -> String {
        tools::alchemy_debug_trace_block_by_number(&input.block, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get RLP-encoded block via Alchemy debug API")]
    async fn alchemy_debug_get_raw_block(
        &self,
        Parameters(input): Parameters<AlchemyDebugBlockInput>,
    ) -> String {
        tools::alchemy_debug_get_raw_block(&input.block, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get RLP-encoded block header via Alchemy debug API")]
    async fn alchemy_debug_get_raw_header(
        &self,
        Parameters(input): Parameters<AlchemyDebugBlockInput>,
    ) -> String {
        tools::alchemy_debug_get_raw_header(&input.block, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get EIP-2718 binary-encoded receipts for a block via Alchemy debug API")]
    async fn alchemy_debug_get_raw_receipts(
        &self,
        Parameters(input): Parameters<AlchemyDebugBlockInput>,
    ) -> String {
        tools::alchemy_debug_get_raw_receipts(&input.block, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // ALCHEMY TRACE (Parity-style)
    // =========================================================================

    #[tool(description = "Get Parity-style traces for a block via Alchemy")]
    async fn alchemy_trace_block(
        &self,
        Parameters(input): Parameters<AlchemyTraceBlockInput>,
    ) -> String {
        tools::alchemy_trace_block(&input.block, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Execute a call and return Parity-style traces via Alchemy")]
    async fn alchemy_trace_call(
        &self,
        Parameters(input): Parameters<AlchemyTraceCallInput>,
    ) -> String {
        tools::alchemy_trace_call(
            &input.to,
            input.block.as_deref(),
            input.from.as_deref(),
            input.data.as_deref(),
            input.value.as_deref(),
            input.gas.as_deref(),
            input.trace_types.as_deref(),
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get a specific trace by position in a transaction via Alchemy")]
    async fn alchemy_trace_get(
        &self,
        Parameters(input): Parameters<AlchemyTraceGetInput>,
    ) -> String {
        tools::alchemy_trace_get(&input.hash, &input.indices, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Trace a raw transaction without executing via Alchemy")]
    async fn alchemy_trace_raw_transaction(
        &self,
        Parameters(input): Parameters<AlchemyTraceRawTxInput>,
    ) -> String {
        tools::alchemy_trace_raw_transaction(
            &input.raw_tx,
            input.trace_types.as_deref(),
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Replay all transactions in a block with traces via Alchemy")]
    async fn alchemy_trace_replay_block_transactions(
        &self,
        Parameters(input): Parameters<AlchemyTraceReplayBlockInput>,
    ) -> String {
        tools::alchemy_trace_replay_block_transactions(
            &input.block,
            input.trace_types.as_deref(),
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Replay a transaction with traces via Alchemy")]
    async fn alchemy_trace_replay_transaction(
        &self,
        Parameters(input): Parameters<AlchemyTraceReplayTxInput>,
    ) -> String {
        tools::alchemy_trace_replay_transaction(
            &input.hash,
            input.trace_types.as_deref(),
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get all traces for a transaction via Alchemy")]
    async fn alchemy_trace_transaction(
        &self,
        Parameters(input): Parameters<AlchemyTraceTxInput>,
    ) -> String {
        tools::alchemy_trace_transaction(&input.hash, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Filter traces by criteria (block range, addresses) via Alchemy")]
    async fn alchemy_trace_filter(
        &self,
        Parameters(input): Parameters<AlchemyTraceFilterInput>,
    ) -> String {
        tools::alchemy_trace_filter(
            input.from_block.as_deref(),
            input.to_block.as_deref(),
            input.from_address.as_deref(),
            input.to_address.as_deref(),
            input.after,
            input.count,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    // =========================================================================
    // ALCHEMY SIMULATION
    // =========================================================================

    #[tool(description = "Simulate a transaction and return asset changes via Alchemy")]
    async fn alchemy_sim_asset_changes(
        &self,
        Parameters(input): Parameters<AlchemySimAssetChangesInput>,
    ) -> String {
        tools::alchemy_sim_asset_changes(
            &input.to,
            input.from.as_deref(),
            input.data.as_deref(),
            input.value.as_deref(),
            input.gas.as_deref(),
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Simulate execution with decoded traces and logs via Alchemy")]
    async fn alchemy_sim_execution(
        &self,
        Parameters(input): Parameters<AlchemySimExecutionInput>,
    ) -> String {
        tools::alchemy_sim_execution(
            &input.to,
            input.from.as_deref(),
            input.data.as_deref(),
            input.value.as_deref(),
            input.gas.as_deref(),
            input.block.as_deref(),
            input.trace_format.as_deref(),
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    // =========================================================================
    // ALCHEMY BUNDLER (ERC-4337)
    // =========================================================================

    #[tool(description = "Get supported ERC-4337 entry points via Alchemy")]
    async fn alchemy_bundler_supported_entry_points(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_bundler_supported_entry_points(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Estimate gas for an ERC-4337 UserOperation via Alchemy")]
    async fn alchemy_bundler_estimate_gas(
        &self,
        Parameters(input): Parameters<AlchemyBundlerEstimateGasInput>,
    ) -> String {
        tools::alchemy_bundler_estimate_gas(
            &input.user_op_json,
            input.entry_point.as_deref(),
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get a UserOperation by hash via Alchemy")]
    async fn alchemy_bundler_get_by_hash(
        &self,
        Parameters(input): Parameters<AlchemyBundlerHashInput>,
    ) -> String {
        tools::alchemy_bundler_get_by_hash(&input.hash, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get a UserOperation receipt via Alchemy")]
    async fn alchemy_bundler_get_receipt(
        &self,
        Parameters(input): Parameters<AlchemyBundlerHashInput>,
    ) -> String {
        tools::alchemy_bundler_get_receipt(&input.hash, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get recommended max priority fee per gas via Alchemy Bundler")]
    async fn alchemy_bundler_max_priority_fee(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_bundler_max_priority_fee(Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // ALCHEMY GAS MANAGER
    // =========================================================================

    #[tool(description = "List all gas manager policies via Alchemy")]
    async fn alchemy_gas_manager_list_policies(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_gas_manager_list_policies(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get a gas manager policy by ID via Alchemy")]
    async fn alchemy_gas_manager_get_policy(
        &self,
        Parameters(input): Parameters<AlchemyGasManagerPolicyInput>,
    ) -> String {
        tools::alchemy_gas_manager_get_policy(&input.policy_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get gas manager policy statistics via Alchemy")]
    async fn alchemy_gas_manager_policy_stats(
        &self,
        Parameters(input): Parameters<AlchemyGasManagerPolicyInput>,
    ) -> String {
        tools::alchemy_gas_manager_policy_stats(&input.policy_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "List sponsorships for a gas manager policy via Alchemy")]
    async fn alchemy_gas_manager_list_sponsorships(
        &self,
        Parameters(input): Parameters<AlchemyGasManagerPolicyInput>,
    ) -> String {
        tools::alchemy_gas_manager_list_sponsorships(&input.policy_id, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // ALCHEMY NOTIFY
    // =========================================================================

    #[tool(description = "List all webhooks via Alchemy Notify")]
    async fn alchemy_notify_list_webhooks(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_notify_list_webhooks(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "List addresses tracked by a webhook via Alchemy Notify")]
    async fn alchemy_notify_list_addresses(
        &self,
        Parameters(input): Parameters<AlchemyNotifyWebhookInput>,
    ) -> String {
        tools::alchemy_notify_list_addresses(&input.webhook_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "List NFT filters for a webhook via Alchemy Notify")]
    async fn alchemy_notify_list_nft_filters(
        &self,
        Parameters(input): Parameters<AlchemyNotifyWebhookInput>,
    ) -> String {
        tools::alchemy_notify_list_nft_filters(&input.webhook_id, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // ALCHEMY BEACON
    // =========================================================================

    #[tool(description = "Get Ethereum beacon chain genesis info via Alchemy")]
    async fn alchemy_beacon_genesis(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_beacon_genesis(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain fork schedule via Alchemy")]
    async fn alchemy_beacon_fork_schedule(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_beacon_fork_schedule(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain deposit contract info via Alchemy")]
    async fn alchemy_beacon_deposit_contract(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_beacon_deposit_contract(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain spec/config values via Alchemy")]
    async fn alchemy_beacon_spec(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_beacon_spec(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain block headers via Alchemy")]
    async fn alchemy_beacon_headers(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_beacon_headers(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get a beacon chain block header by ID via Alchemy")]
    async fn alchemy_beacon_header(
        &self,
        Parameters(input): Parameters<AlchemyBeaconBlockIdInput>,
    ) -> String {
        tools::alchemy_beacon_header(&input.block_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get a beacon chain block by ID via Alchemy")]
    async fn alchemy_beacon_block(
        &self,
        Parameters(input): Parameters<AlchemyBeaconBlockIdInput>,
    ) -> String {
        tools::alchemy_beacon_block(&input.block_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain block root via Alchemy")]
    async fn alchemy_beacon_block_root(
        &self,
        Parameters(input): Parameters<AlchemyBeaconBlockIdInput>,
    ) -> String {
        tools::alchemy_beacon_block_root(&input.block_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain block attestations via Alchemy")]
    async fn alchemy_beacon_block_attestations(
        &self,
        Parameters(input): Parameters<AlchemyBeaconBlockIdInput>,
    ) -> String {
        tools::alchemy_beacon_block_attestations(&input.block_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get blob sidecars for a beacon chain block via Alchemy")]
    async fn alchemy_beacon_blob_sidecars(
        &self,
        Parameters(input): Parameters<AlchemyBeaconBlockIdInput>,
    ) -> String {
        tools::alchemy_beacon_blob_sidecars(&input.block_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain state root via Alchemy")]
    async fn alchemy_beacon_state_root(
        &self,
        Parameters(input): Parameters<AlchemyBeaconStateIdInput>,
    ) -> String {
        tools::alchemy_beacon_state_root(&input.state_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain fork info for a state via Alchemy")]
    async fn alchemy_beacon_state_fork(
        &self,
        Parameters(input): Parameters<AlchemyBeaconStateIdInput>,
    ) -> String {
        tools::alchemy_beacon_state_fork(&input.state_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain finality checkpoints via Alchemy")]
    async fn alchemy_beacon_finality_checkpoints(
        &self,
        Parameters(input): Parameters<AlchemyBeaconStateIdInput>,
    ) -> String {
        tools::alchemy_beacon_finality_checkpoints(&input.state_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain validators for a state via Alchemy")]
    async fn alchemy_beacon_validators(
        &self,
        Parameters(input): Parameters<AlchemyBeaconStateIdInput>,
    ) -> String {
        tools::alchemy_beacon_validators(&input.state_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get a specific beacon chain validator via Alchemy")]
    async fn alchemy_beacon_validator(
        &self,
        Parameters(input): Parameters<AlchemyBeaconValidatorInput>,
    ) -> String {
        tools::alchemy_beacon_validator(&input.state_id, &input.validator_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain validator balances for a state via Alchemy")]
    async fn alchemy_beacon_validator_balances(
        &self,
        Parameters(input): Parameters<AlchemyBeaconStateIdInput>,
    ) -> String {
        tools::alchemy_beacon_validator_balances(&input.state_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain sync committees for a state via Alchemy")]
    async fn alchemy_beacon_sync_committees(
        &self,
        Parameters(input): Parameters<AlchemyBeaconStateIdInput>,
    ) -> String {
        tools::alchemy_beacon_sync_committees(&input.state_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get RANDAO value for a beacon chain state via Alchemy")]
    async fn alchemy_beacon_randao(
        &self,
        Parameters(input): Parameters<AlchemyBeaconStateIdInput>,
    ) -> String {
        tools::alchemy_beacon_randao(&input.state_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain pool attestations via Alchemy")]
    async fn alchemy_beacon_pool_attestations(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_beacon_pool_attestations(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get voluntary exits in the beacon chain pool via Alchemy")]
    async fn alchemy_beacon_voluntary_exits(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_beacon_voluntary_exits(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain block rewards via Alchemy")]
    async fn alchemy_beacon_block_rewards(
        &self,
        Parameters(input): Parameters<AlchemyBeaconBlockIdInput>,
    ) -> String {
        tools::alchemy_beacon_block_rewards(&input.block_id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain node sync status via Alchemy")]
    async fn alchemy_beacon_syncing(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_beacon_syncing(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain node version via Alchemy")]
    async fn alchemy_beacon_version(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_beacon_version(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain peers via Alchemy")]
    async fn alchemy_beacon_peers(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_beacon_peers(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get beacon chain peer count via Alchemy")]
    async fn alchemy_beacon_peer_count(
        &self,
        Parameters(input): Parameters<AlchemyChainOnlyInput>,
    ) -> String {
        tools::alchemy_beacon_peer_count(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get attester duties for an epoch via Alchemy beacon API")]
    async fn alchemy_beacon_attester_duties(
        &self,
        Parameters(input): Parameters<AlchemyBeaconDutiesInput>,
    ) -> String {
        tools::alchemy_beacon_attester_duties(&input.epoch, &input.validators, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get proposer duties for an epoch via Alchemy beacon API")]
    async fn alchemy_beacon_proposer_duties(
        &self,
        Parameters(input): Parameters<AlchemyBeaconEpochInput>,
    ) -> String {
        tools::alchemy_beacon_proposer_duties(&input.epoch, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get sync duties for an epoch via Alchemy beacon API")]
    async fn alchemy_beacon_sync_duties(
        &self,
        Parameters(input): Parameters<AlchemyBeaconDutiesInput>,
    ) -> String {
        tools::alchemy_beacon_sync_duties(&input.epoch, &input.validators, Some(&input.chain))
            .await
            .to_response()
    }

    // =========================================================================
    // ALCHEMY SOLANA
    // =========================================================================

    #[tool(description = "Get a single Solana asset by ID via Alchemy")]
    async fn alchemy_solana_get_asset(
        &self,
        Parameters(input): Parameters<AlchemySolanaAssetInput>,
    ) -> String {
        tools::alchemy_solana_get_asset(&input.id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get multiple Solana assets by IDs via Alchemy")]
    async fn alchemy_solana_get_assets(
        &self,
        Parameters(input): Parameters<AlchemySolanaAssetsInput>,
    ) -> String {
        tools::alchemy_solana_get_assets(&input.ids, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get proof for a compressed Solana asset via Alchemy")]
    async fn alchemy_solana_get_asset_proof(
        &self,
        Parameters(input): Parameters<AlchemySolanaAssetInput>,
    ) -> String {
        tools::alchemy_solana_get_asset_proof(&input.id, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get proofs for multiple compressed Solana assets via Alchemy")]
    async fn alchemy_solana_get_asset_proofs(
        &self,
        Parameters(input): Parameters<AlchemySolanaAssetsInput>,
    ) -> String {
        tools::alchemy_solana_get_asset_proofs(&input.ids, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get Solana assets owned by an address via Alchemy")]
    async fn alchemy_solana_get_assets_by_owner(
        &self,
        Parameters(input): Parameters<AlchemySolanaOwnerInput>,
    ) -> String {
        tools::alchemy_solana_get_assets_by_owner(
            &input.owner,
            input.page,
            input.limit,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get Solana assets by creator address via Alchemy")]
    async fn alchemy_solana_get_assets_by_creator(
        &self,
        Parameters(input): Parameters<AlchemySolanaAddressInput>,
    ) -> String {
        tools::alchemy_solana_get_assets_by_creator(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get Solana assets by authority address via Alchemy")]
    async fn alchemy_solana_get_assets_by_authority(
        &self,
        Parameters(input): Parameters<AlchemySolanaAddressInput>,
    ) -> String {
        tools::alchemy_solana_get_assets_by_authority(&input.address, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get Solana assets by group (e.g., collection) via Alchemy")]
    async fn alchemy_solana_get_assets_by_group(
        &self,
        Parameters(input): Parameters<AlchemySolanaGroupInput>,
    ) -> String {
        tools::alchemy_solana_get_assets_by_group(
            &input.group_key,
            &input.group_value,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get Solana token accounts via Alchemy")]
    async fn alchemy_solana_get_token_accounts(
        &self,
        Parameters(input): Parameters<AlchemySolanaTokenAccountsInput>,
    ) -> String {
        tools::alchemy_solana_get_token_accounts(
            input.owner.as_deref(),
            input.mint.as_deref(),
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get Solana NFT editions via Alchemy")]
    async fn alchemy_solana_get_nft_editions(
        &self,
        Parameters(input): Parameters<AlchemySolanaMintInput>,
    ) -> String {
        tools::alchemy_solana_get_nft_editions(&input.mint, Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get Solana asset signatures (transactions) via Alchemy")]
    async fn alchemy_solana_get_asset_signatures(
        &self,
        Parameters(input): Parameters<AlchemySolanaAssetInput>,
    ) -> String {
        tools::alchemy_solana_get_asset_signatures(&input.id, Some(&input.chain))
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

    // --- Gecko Simple (additional) ---

    #[tool(description = "Get token price by contract address from CoinGecko")]
    async fn gecko_simple_token_price(
        &self,
        Parameters(input): Parameters<GeckoTokenPriceInput>,
    ) -> String {
        tools::gecko_simple_token_price(&input.platform, &input.addresses, input.vs.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "List supported vs currencies from CoinGecko")]
    async fn gecko_simple_currencies(&self) -> String {
        tools::gecko_simple_currencies().await.to_response()
    }

    // --- Gecko Coins (additional) ---

    #[tool(description = "List all coins from CoinGecko")]
    async fn gecko_coins_list(&self, Parameters(input): Parameters<GeckoCoinsListInput>) -> String {
        tools::gecko_coins_list(input.with_platforms)
            .await
            .to_response()
    }

    #[tool(description = "Get coin market data from CoinGecko")]
    async fn gecko_coins_markets(
        &self,
        Parameters(input): Parameters<GeckoCoinsMarketsInput>,
    ) -> String {
        tools::gecko_coins_markets(input.vs_currency.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get coin exchange tickers from CoinGecko")]
    async fn gecko_coins_tickers(&self, Parameters(input): Parameters<GeckoCoinInput>) -> String {
        tools::gecko_coins_tickers(&input.id).await.to_response()
    }

    #[tool(description = "Get coin market chart data from CoinGecko")]
    async fn gecko_coins_chart(
        &self,
        Parameters(input): Parameters<GeckoCoinsChartInput>,
    ) -> String {
        tools::gecko_coins_chart(&input.id, input.vs.as_deref(), input.days.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get coin OHLC candlestick data from CoinGecko")]
    async fn gecko_coins_ohlc(&self, Parameters(input): Parameters<GeckoCoinsOhlcInput>) -> String {
        tools::gecko_coins_ohlc(&input.id, input.vs.as_deref(), input.days.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get coin historical data for a specific date from CoinGecko")]
    async fn gecko_coins_history(
        &self,
        Parameters(input): Parameters<GeckoCoinsHistoryInput>,
    ) -> String {
        tools::gecko_coins_history(&input.id, &input.date)
            .await
            .to_response()
    }

    #[tool(description = "Get top gainers and losers from CoinGecko")]
    async fn gecko_coins_top_movers(
        &self,
        Parameters(input): Parameters<GeckoCoinsTopMoversInput>,
    ) -> String {
        tools::gecko_coins_top_movers(input.vs.as_deref(), input.duration.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get recently added coins from CoinGecko")]
    async fn gecko_coins_new(&self) -> String {
        tools::gecko_coins_new().await.to_response()
    }

    #[tool(description = "Get coin data by contract address from CoinGecko")]
    async fn gecko_coins_by_contract(
        &self,
        Parameters(input): Parameters<GeckoContractInput>,
    ) -> String {
        tools::gecko_coins_by_contract(&input.platform, &input.address)
            .await
            .to_response()
    }

    // --- Gecko Global (additional) ---

    #[tool(description = "Ping CoinGecko API status")]
    async fn gecko_global_ping(&self) -> String {
        tools::gecko_global_ping().await.to_response()
    }

    #[tool(description = "Get global DeFi market data from CoinGecko")]
    async fn gecko_global_defi(&self) -> String {
        tools::gecko_global_defi().await.to_response()
    }

    #[tool(description = "Get trending coins, NFTs, and categories from CoinGecko")]
    async fn gecko_global_trending(&self) -> String {
        tools::gecko_global_trending().await.to_response()
    }

    #[tool(description = "Search coins, exchanges, categories, and NFTs on CoinGecko")]
    async fn gecko_global_search(&self, Parameters(input): Parameters<GeckoSearchInput>) -> String {
        tools::gecko_global_search(&input.query).await.to_response()
    }

    #[tool(description = "Get BTC exchange rates from CoinGecko")]
    async fn gecko_global_exchange_rates(&self) -> String {
        tools::gecko_global_exchange_rates().await.to_response()
    }

    #[tool(description = "List asset platforms (blockchains) from CoinGecko")]
    async fn gecko_global_platforms(&self) -> String {
        tools::gecko_global_platforms().await.to_response()
    }

    // --- Gecko NFTs (additional) ---

    #[tool(description = "List NFT collections from CoinGecko")]
    async fn gecko_nfts_list(&self) -> String {
        tools::gecko_nfts_list().await.to_response()
    }

    #[tool(description = "Get NFT collection by contract address from CoinGecko")]
    async fn gecko_nfts_by_contract(
        &self,
        Parameters(input): Parameters<GeckoContractInput>,
    ) -> String {
        tools::gecko_nfts_by_contract(&input.platform, &input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get NFT markets data from CoinGecko")]
    async fn gecko_nfts_markets(&self) -> String {
        tools::gecko_nfts_markets().await.to_response()
    }

    #[tool(description = "Get NFT collection tickers from CoinGecko")]
    async fn gecko_nfts_tickers(&self, Parameters(input): Parameters<GeckoNftInput>) -> String {
        tools::gecko_nfts_tickers(&input.id).await.to_response()
    }

    // --- Gecko Onchain (additional) ---

    #[tool(description = "List supported onchain networks from CoinGecko")]
    async fn gecko_onchain_networks(&self) -> String {
        tools::gecko_onchain_networks().await.to_response()
    }

    #[tool(description = "List DEXes on a network from CoinGecko")]
    async fn gecko_onchain_dexes(
        &self,
        Parameters(input): Parameters<GeckoOnchainNetworkInput>,
    ) -> String {
        tools::gecko_onchain_dexes(&input.network)
            .await
            .to_response()
    }

    #[tool(description = "Get trending pools from CoinGecko onchain data")]
    async fn gecko_onchain_trending_pools(
        &self,
        Parameters(input): Parameters<GeckoOnchainOptionalNetworkInput>,
    ) -> String {
        tools::gecko_onchain_trending_pools(input.network.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get top pools from CoinGecko onchain data")]
    async fn gecko_onchain_top_pools(
        &self,
        Parameters(input): Parameters<GeckoOnchainOptionalNetworkInput>,
    ) -> String {
        tools::gecko_onchain_top_pools(input.network.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get new pools from CoinGecko onchain data")]
    async fn gecko_onchain_new_pools(
        &self,
        Parameters(input): Parameters<GeckoOnchainOptionalNetworkInput>,
    ) -> String {
        tools::gecko_onchain_new_pools(input.network.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get onchain token data from CoinGecko")]
    async fn gecko_onchain_token(
        &self,
        Parameters(input): Parameters<GeckoOnchainInput>,
    ) -> String {
        tools::gecko_onchain_token_info(&input.network, &input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get onchain token prices from CoinGecko")]
    async fn gecko_onchain_token_price(
        &self,
        Parameters(input): Parameters<GeckoOnchainTokenPriceInput>,
    ) -> String {
        tools::gecko_onchain_token_price(&input.network, &input.addresses)
            .await
            .to_response()
    }

    #[tool(description = "Get pools for a token from CoinGecko onchain data")]
    async fn gecko_onchain_token_pools(
        &self,
        Parameters(input): Parameters<GeckoOnchainInput>,
    ) -> String {
        tools::gecko_onchain_token_pools(&input.network, &input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get pool OHLCV candlestick data from CoinGecko onchain")]
    async fn gecko_onchain_pool_ohlcv(
        &self,
        Parameters(input): Parameters<GeckoOnchainPoolOhlcvInput>,
    ) -> String {
        tools::gecko_onchain_pool_ohlcv(&input.network, &input.address, input.timeframe.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Search pools on CoinGecko onchain")]
    async fn gecko_onchain_search_pools(
        &self,
        Parameters(input): Parameters<GeckoSearchInput>,
    ) -> String {
        tools::gecko_onchain_search_pools(&input.query)
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
    // MORALIS TOKEN
    // =========================================================================

    #[tool(description = "Get token metadata via Moralis (name, symbol, decimals, supply)")]
    async fn moralis_token_metadata(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_metadata(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get token price via Moralis")]
    async fn moralis_token_price(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_price(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get top token holders via Moralis")]
    async fn moralis_token_holders(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_holders(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get token DEX pairs via Moralis")]
    async fn moralis_token_pairs(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_pairs(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get token transfers via Moralis")]
    async fn moralis_token_transfers(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_transfers(&input.address)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS WALLET
    // =========================================================================

    #[tool(description = "Get native token balance (ETH, MATIC, etc.) via Moralis")]
    async fn moralis_wallet_balance(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_wallet_balance(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get ERC20 token balances for a wallet via Moralis")]
    async fn moralis_wallet_tokens(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_wallet_tokens(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get decoded wallet transaction history via Moralis")]
    async fn moralis_wallet_history(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_wallet_history(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get wallet net worth across all chains via Moralis")]
    async fn moralis_wallet_net_worth(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_wallet_net_worth(&input.address)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS RESOLVE
    // =========================================================================

    #[tool(description = "Resolve ENS/domain name to address via Moralis")]
    async fn moralis_resolve_domain(
        &self,
        Parameters(input): Parameters<MoralisDomainInput>,
    ) -> String {
        tools::moralis_resolve_domain(&input.domain)
            .await
            .to_response()
    }

    #[tool(description = "Reverse lookup address to ENS/domain via Moralis")]
    async fn moralis_resolve_address(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_resolve_address(&input.address)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS MARKET
    // =========================================================================

    #[tool(description = "Get top ERC20 tokens by market cap via Moralis")]
    async fn moralis_market_top_tokens(&self) -> String {
        tools::moralis_market_top_tokens().await.to_response()
    }

    #[tool(description = "Get top price movers (gainers/losers) via Moralis")]
    async fn moralis_market_top_movers(&self) -> String {
        tools::moralis_market_top_movers().await.to_response()
    }

    #[tool(description = "Get top NFT collections by market cap via Moralis")]
    async fn moralis_market_top_nfts(&self) -> String {
        tools::moralis_market_top_nfts().await.to_response()
    }

    // =========================================================================
    // MORALIS WALLET (additional)
    // =========================================================================

    #[tool(description = "Get wallet transactions via Moralis")]
    async fn moralis_wallet_transactions(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_wallet_transactions(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get chains where a wallet has been active via Moralis")]
    async fn moralis_wallet_active_chains(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_wallet_active_chains(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get token approvals for a wallet via Moralis")]
    async fn moralis_wallet_approvals(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_wallet_approvals(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get wallet stats (transaction counts, volumes) via Moralis")]
    async fn moralis_wallet_stats(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_wallet_stats(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get wallet profitability summary via Moralis")]
    async fn moralis_wallet_profitability(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_wallet_profitability(&input.address)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS TOKEN (additional)
    // =========================================================================

    #[tool(description = "Get token swaps/trades via Moralis")]
    async fn moralis_token_swaps(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_swaps(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get token stats (holders, transactions, liquidity) via Moralis")]
    async fn moralis_token_stats(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_stats(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Search for tokens by name or symbol via Moralis")]
    async fn moralis_token_search(
        &self,
        Parameters(input): Parameters<MoralisSearchInput>,
    ) -> String {
        tools::moralis_token_search(&input.query)
            .await
            .to_response()
    }

    #[tool(description = "Get trending tokens via Moralis")]
    async fn moralis_token_trending(&self) -> String {
        tools::moralis_token_trending().await.to_response()
    }

    #[tool(description = "Get OHLCV candlestick data for a token pair via Moralis")]
    async fn moralis_token_pair_ohlcv(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_pair_ohlcv(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get pair stats (volume, liquidity, price change) via Moralis")]
    async fn moralis_token_pair_stats(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_pair_stats(&input.address)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS NFT
    // =========================================================================

    #[tool(description = "Get NFTs owned by a wallet via Moralis")]
    async fn moralis_nft_list(&self, Parameters(input): Parameters<MoralisAddressInput>) -> String {
        tools::moralis_nft_list(&input.address).await.to_response()
    }

    #[tool(description = "Get NFT metadata (name, image, attributes) via Moralis")]
    async fn moralis_nft_metadata(
        &self,
        Parameters(input): Parameters<MoralisNftMetadataInput>,
    ) -> String {
        tools::moralis_nft_metadata(&input.contract, &input.token_id)
            .await
            .to_response()
    }

    #[tool(description = "Get NFT transfers for a wallet or contract via Moralis")]
    async fn moralis_nft_transfers(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_transfers(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get NFT collection metadata via Moralis")]
    async fn moralis_nft_collection(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_collection(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get NFT collection stats (volume, sales, floor price) via Moralis")]
    async fn moralis_nft_collection_stats(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_collection_stats(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get owners of an NFT collection via Moralis")]
    async fn moralis_nft_owners(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_owners(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get NFT trades/sales history via Moralis")]
    async fn moralis_nft_trades(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_trades(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get NFT collection floor price via Moralis")]
    async fn moralis_nft_floor_price(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_floor_price(&input.address)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS WALLET (new)
    // =========================================================================

    #[tool(description = "Get profitability breakdown by token for a wallet via Moralis")]
    async fn moralis_wallet_profitability_tokens(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_wallet_profitability_tokens(&input.address)
            .await
            .to_response()
    }

    #[tool(
        description = "Get native balances for multiple wallets in batch (comma-separated addresses) via Moralis"
    )]
    async fn moralis_wallet_multiple_balances(
        &self,
        Parameters(input): Parameters<MoralisAddressesInput>,
    ) -> String {
        tools::moralis_wallet_multiple_balances(&input.addresses)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS TOKEN (new)
    // =========================================================================

    #[tool(description = "Get token swaps for a wallet address via Moralis")]
    async fn moralis_token_wallet_swaps(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_wallet_swaps(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get swaps for a specific token pair via Moralis")]
    async fn moralis_token_pair_swaps(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_pair_swaps(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get all token categories via Moralis")]
    async fn moralis_token_categories(&self) -> String {
        tools::moralis_token_categories().await.to_response()
    }

    #[tool(
        description = "Get newly listed tokens on a DEX exchange (e.g., uniswapv2, uniswapv3, pancakeswap) via Moralis"
    )]
    async fn moralis_token_exchange_new_tokens(
        &self,
        Parameters(input): Parameters<MoralisExchangeInput>,
    ) -> String {
        tools::moralis_token_exchange_new_tokens(&input.exchange)
            .await
            .to_response()
    }

    #[tool(description = "Get tokens in bonding phase on an exchange (e.g., pump.fun) via Moralis")]
    async fn moralis_token_exchange_bonding_tokens(
        &self,
        Parameters(input): Parameters<MoralisExchangeInput>,
    ) -> String {
        tools::moralis_token_exchange_bonding_tokens(&input.exchange)
            .await
            .to_response()
    }

    #[tool(description = "Get graduated tokens on an exchange via Moralis")]
    async fn moralis_token_exchange_graduated_tokens(
        &self,
        Parameters(input): Parameters<MoralisExchangeInput>,
    ) -> String {
        tools::moralis_token_exchange_graduated_tokens(&input.exchange)
            .await
            .to_response()
    }

    #[tool(
        description = "Get prices for multiple tokens in batch (comma-separated addresses) via Moralis"
    )]
    async fn moralis_token_multiple_prices(
        &self,
        Parameters(input): Parameters<MoralisAddressesInput>,
    ) -> String {
        tools::moralis_token_multiple_prices(&input.addresses)
            .await
            .to_response()
    }

    #[tool(
        description = "Get tokens by their symbols (comma-separated, e.g., USDC,WETH,DAI) via Moralis"
    )]
    async fn moralis_token_by_symbols(
        &self,
        Parameters(input): Parameters<MoralisSymbolsInput>,
    ) -> String {
        tools::moralis_token_by_symbols(&input.symbols)
            .await
            .to_response()
    }

    #[tool(description = "Get transfers for a token contract (not wallet transfers) via Moralis")]
    async fn moralis_token_contract_transfers(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_contract_transfers(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get holders summary for a token via Moralis")]
    async fn moralis_token_holders_summary(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_holders_summary(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get historical holders data for a token via Moralis")]
    async fn moralis_token_holders_historical(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_holders_historical(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get aggregated pair stats for a token via Moralis")]
    async fn moralis_token_pairs_stats(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_pairs_stats(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get top gainers/traders for a token via Moralis")]
    async fn moralis_token_top_gainers(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_top_gainers(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get pair snipers (early buyers) via Moralis")]
    async fn moralis_token_pair_snipers(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_pair_snipers(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get token bonding status (pump.fun graduation status, etc.) via Moralis")]
    async fn moralis_token_bonding_status(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_token_bonding_status(&input.address)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS NFT (new)
    // =========================================================================

    #[tool(description = "Get NFT collections owned by a wallet via Moralis")]
    async fn moralis_nft_wallet_collections(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_wallet_collections(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get NFT transfers by contract address via Moralis")]
    async fn moralis_nft_contract_transfers(
        &self,
        Parameters(input): Parameters<MoralisNftContractInput>,
    ) -> String {
        tools::moralis_nft_contract_transfers(&input.contract)
            .await
            .to_response()
    }

    #[tool(description = "Get transfers for a specific NFT token via Moralis")]
    async fn moralis_nft_token_transfers(
        &self,
        Parameters(input): Parameters<MoralisNftTokenInput>,
    ) -> String {
        tools::moralis_nft_token_transfers(&input.contract, &input.token_id)
            .await
            .to_response()
    }

    #[tool(description = "Get owners of a specific NFT token via Moralis")]
    async fn moralis_nft_token_owners(
        &self,
        Parameters(input): Parameters<MoralisNftTokenInput>,
    ) -> String {
        tools::moralis_nft_token_owners(&input.contract, &input.token_id)
            .await
            .to_response()
    }

    #[tool(description = "Get floor price for a specific NFT token via Moralis")]
    async fn moralis_nft_token_floor_price(
        &self,
        Parameters(input): Parameters<MoralisNftTokenInput>,
    ) -> String {
        tools::moralis_nft_token_floor_price(&input.contract, &input.token_id)
            .await
            .to_response()
    }

    #[tool(description = "Get trades for a specific NFT token via Moralis")]
    async fn moralis_nft_token_trades(
        &self,
        Parameters(input): Parameters<MoralisNftTokenInput>,
    ) -> String {
        tools::moralis_nft_token_trades(&input.contract, &input.token_id)
            .await
            .to_response()
    }

    #[tool(description = "Get NFT trades for a wallet via Moralis")]
    async fn moralis_nft_wallet_trades(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_wallet_trades(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get NFT collection traits via Moralis")]
    async fn moralis_nft_collection_traits(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_collection_traits(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get NFT collection traits with pagination via Moralis")]
    async fn moralis_nft_collection_traits_paginated(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_collection_traits_paginated(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get unique owners count for an NFT collection via Moralis")]
    async fn moralis_nft_unique_owners(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_unique_owners(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Resync metadata for a specific NFT token via Moralis")]
    async fn moralis_nft_resync_metadata(
        &self,
        Parameters(input): Parameters<MoralisNftTokenInput>,
    ) -> String {
        tools::moralis_nft_resync_metadata(&input.contract, &input.token_id)
            .await
            .to_response()
    }

    #[tool(
        description = "Get multiple NFTs by token addresses and IDs in batch (format: addr:id,addr:id) via Moralis"
    )]
    async fn moralis_nft_multiple_nfts(
        &self,
        Parameters(input): Parameters<MoralisMultipleNftsInput>,
    ) -> String {
        tools::moralis_nft_multiple_nfts(&input.tokens)
            .await
            .to_response()
    }

    #[tool(description = "Get historical floor price data for an NFT collection via Moralis")]
    async fn moralis_nft_floor_price_historical(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_floor_price_historical(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Sync NFT collection metadata via Moralis")]
    async fn moralis_nft_sync_collection(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_sync_collection(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get all NFTs by contract address via Moralis")]
    async fn moralis_nft_contract_nfts(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_contract_nfts(&input.address)
            .await
            .to_response()
    }

    #[tool(
        description = "Get metadata for multiple NFT contracts in batch (comma-separated addresses) via Moralis"
    )]
    async fn moralis_nft_multiple_collections(
        &self,
        Parameters(input): Parameters<MoralisAddressesInput>,
    ) -> String {
        tools::moralis_nft_multiple_collections(&input.addresses)
            .await
            .to_response()
    }

    #[tool(description = "Resync NFT collection traits via Moralis")]
    async fn moralis_nft_resync_traits(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_resync_traits(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get NFT sale prices for a collection via Moralis")]
    async fn moralis_nft_collection_prices(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_nft_collection_prices(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get NFT sale prices for a specific token via Moralis")]
    async fn moralis_nft_token_prices(
        &self,
        Parameters(input): Parameters<MoralisNftTokenInput>,
    ) -> String {
        tools::moralis_nft_token_prices(&input.contract, &input.token_id)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS RESOLVE (new)
    // =========================================================================

    #[tool(description = "Get all domains for an address via Moralis")]
    async fn moralis_resolve_address_domains(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_resolve_address_domains(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get ENS domain details via Moralis")]
    async fn moralis_resolve_ens_domain(
        &self,
        Parameters(input): Parameters<MoralisDomainInput>,
    ) -> String {
        tools::moralis_resolve_ens_domain(&input.domain)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS MARKET (new)
    // =========================================================================

    #[tool(description = "Get hottest NFT collections via Moralis")]
    async fn moralis_market_hottest_nfts(&self) -> String {
        tools::moralis_market_hottest_nfts().await.to_response()
    }

    #[tool(description = "Get global crypto market capitalization via Moralis")]
    async fn moralis_market_global_market_cap(&self) -> String {
        tools::moralis_market_global_market_cap()
            .await
            .to_response()
    }

    #[tool(description = "Get global crypto trading volume via Moralis")]
    async fn moralis_market_global_volume(&self) -> String {
        tools::moralis_market_global_volume().await.to_response()
    }

    // =========================================================================
    // MORALIS TRANSACTION
    // =========================================================================

    #[tool(description = "Get transaction details by hash via Moralis")]
    async fn moralis_transaction_get(
        &self,
        Parameters(input): Parameters<MoralisTxHashInput>,
    ) -> String {
        tools::moralis_transaction_get(&input.tx_hash)
            .await
            .to_response()
    }

    #[tool(
        description = "Get verbose transaction with decoded data and optional internal transactions via Moralis"
    )]
    async fn moralis_transaction_verbose(
        &self,
        Parameters(input): Parameters<MoralisTxVerboseInput>,
    ) -> String {
        tools::moralis_transaction_verbose(&input.tx_hash, input.include_internal)
            .await
            .to_response()
    }

    #[tool(description = "Get transactions for a wallet address via Moralis transaction API")]
    async fn moralis_transaction_wallet(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_transaction_wallet(&input.address)
            .await
            .to_response()
    }

    #[tool(
        description = "Get verbose transactions for a wallet with decoded data and optional internal transactions via Moralis"
    )]
    async fn moralis_transaction_wallet_verbose(
        &self,
        Parameters(input): Parameters<MoralisWalletVerboseInput>,
    ) -> String {
        tools::moralis_transaction_wallet_verbose(&input.address, input.include_internal)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS BLOCK
    // =========================================================================

    #[tool(
        description = "Get block by number or hash with optional transaction details via Moralis"
    )]
    async fn moralis_block_get(&self, Parameters(input): Parameters<MoralisBlockInput>) -> String {
        tools::moralis_block_get(&input.block_number_or_hash, input.include_transactions)
            .await
            .to_response()
    }

    #[tool(description = "Get the latest block number for a chain via Moralis")]
    async fn moralis_block_latest(&self) -> String {
        tools::moralis_block_latest().await.to_response()
    }

    #[tool(
        description = "Get block number by date (ISO 8601 format, e.g. 2023-01-01T00:00:00Z) via Moralis"
    )]
    async fn moralis_block_date_to_block(
        &self,
        Parameters(input): Parameters<MoralisDateInput>,
    ) -> String {
        tools::moralis_block_date_to_block(&input.date)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS DEFI
    // =========================================================================

    #[tool(description = "Get price between two tokens in a DEX pair via Moralis")]
    async fn moralis_defi_pair_price(
        &self,
        Parameters(input): Parameters<MoralisDefiPairPriceInput>,
    ) -> String {
        tools::moralis_defi_pair_price(&input.token0, &input.token1, input.exchange.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get reserves for a DEX pair via Moralis")]
    async fn moralis_defi_pair_reserves(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_defi_pair_reserves(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get pair address for two tokens on a DEX via Moralis")]
    async fn moralis_defi_pair_address(
        &self,
        Parameters(input): Parameters<MoralisDefiPairAddressInput>,
    ) -> String {
        tools::moralis_defi_pair_address(&input.token0, &input.token1, input.exchange.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get DeFi summary for a wallet via Moralis")]
    async fn moralis_defi_wallet_summary(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_defi_wallet_summary(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get all DeFi positions for a wallet via Moralis")]
    async fn moralis_defi_wallet_positions(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_defi_wallet_positions(&input.address)
            .await
            .to_response()
    }

    #[tool(description = "Get DeFi positions for a specific protocol via Moralis")]
    async fn moralis_defi_protocol_positions(
        &self,
        Parameters(input): Parameters<MoralisProtocolPositionsInput>,
    ) -> String {
        tools::moralis_defi_protocol_positions(&input.address, &input.protocol)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS DISCOVERY
    // =========================================================================

    #[tool(description = "Get tokens with rising liquidity via Moralis discovery")]
    async fn moralis_discovery_rising_liquidity(&self) -> String {
        tools::moralis_discovery_rising_liquidity()
            .await
            .to_response()
    }

    #[tool(description = "Get tokens with buying pressure via Moralis discovery")]
    async fn moralis_discovery_buying_pressure(&self) -> String {
        tools::moralis_discovery_buying_pressure()
            .await
            .to_response()
    }

    #[tool(description = "Get solid performer tokens via Moralis discovery")]
    async fn moralis_discovery_solid_performers(&self) -> String {
        tools::moralis_discovery_solid_performers()
            .await
            .to_response()
    }

    #[tool(description = "Get tokens with experienced buyers via Moralis discovery")]
    async fn moralis_discovery_experienced_buyers(&self) -> String {
        tools::moralis_discovery_experienced_buyers()
            .await
            .to_response()
    }

    #[tool(description = "Get risky bet tokens via Moralis discovery")]
    async fn moralis_discovery_risky_bets(&self) -> String {
        tools::moralis_discovery_risky_bets().await.to_response()
    }

    #[tool(description = "Get blue chip tokens via Moralis discovery")]
    async fn moralis_discovery_blue_chip(&self) -> String {
        tools::moralis_discovery_blue_chip().await.to_response()
    }

    #[tool(description = "Get top gainer tokens via Moralis discovery")]
    async fn moralis_discovery_top_gainers(&self) -> String {
        tools::moralis_discovery_top_gainers().await.to_response()
    }

    #[tool(description = "Get top loser tokens via Moralis discovery")]
    async fn moralis_discovery_top_losers(&self) -> String {
        tools::moralis_discovery_top_losers().await.to_response()
    }

    #[tool(description = "Get trending tokens via Moralis discovery")]
    async fn moralis_discovery_trending(&self) -> String {
        tools::moralis_discovery_trending().await.to_response()
    }

    #[tool(
        description = "Get token analytics (buyers, sellers, volume) for a specific token via Moralis discovery"
    )]
    async fn moralis_discovery_token_analytics(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_discovery_token_analytics(&input.address)
            .await
            .to_response()
    }

    #[tool(
        description = "Get token score (security, verification, spam status) via Moralis discovery"
    )]
    async fn moralis_discovery_token_score(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_discovery_token_score(&input.address)
            .await
            .to_response()
    }

    #[tool(
        description = "Filter tokens by custom criteria (market cap, liquidity, volume, holders, security) via Moralis discovery"
    )]
    async fn moralis_discovery_filter(
        &self,
        Parameters(input): Parameters<MoralisDiscoveryFilterInput>,
    ) -> String {
        tools::moralis_discovery_filter(
            input.min_market_cap,
            input.max_market_cap,
            input.min_liquidity,
            input.max_liquidity,
            input.min_volume_24h,
            input.max_volume_24h,
            input.min_holders,
            input.min_security_score,
        )
        .await
        .to_response()
    }

    #[tool(description = "Get single token details from Moralis discovery")]
    async fn moralis_discovery_token(
        &self,
        Parameters(input): Parameters<MoralisAddressInput>,
    ) -> String {
        tools::moralis_discovery_token(&input.address)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS ANALYTICS
    // =========================================================================

    #[tool(
        description = "Get token analytics timeseries data (comma-separated addresses, optional timeframe and date range) via Moralis"
    )]
    async fn moralis_analytics_timeseries(
        &self,
        Parameters(input): Parameters<MoralisAnalyticsTimeseriesInput>,
    ) -> String {
        tools::moralis_analytics_timeseries(
            &input.addresses,
            input.timeframe.as_deref(),
            input.from_date.as_deref(),
            input.to_date.as_deref(),
        )
        .await
        .to_response()
    }

    #[tool(
        description = "Get batch token analytics for multiple tokens (comma-separated addresses) via Moralis"
    )]
    async fn moralis_analytics_batch(
        &self,
        Parameters(input): Parameters<MoralisAddressesInput>,
    ) -> String {
        tools::moralis_analytics_batch(&input.addresses)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS ENTITIES
    // =========================================================================

    #[tool(description = "Search for entities (wallets, protocols, exchanges) via Moralis")]
    async fn moralis_entities_search(
        &self,
        Parameters(input): Parameters<MoralisSearchInput>,
    ) -> String {
        tools::moralis_entities_search(&input.query)
            .await
            .to_response()
    }

    #[tool(description = "Get entity details by ID via Moralis")]
    async fn moralis_entities_get(
        &self,
        Parameters(input): Parameters<MoralisEntityIdInput>,
    ) -> String {
        tools::moralis_entities_get(&input.entity_id)
            .await
            .to_response()
    }

    #[tool(description = "Get all entity categories via Moralis")]
    async fn moralis_entities_categories(&self) -> String {
        tools::moralis_entities_categories().await.to_response()
    }

    #[tool(description = "Get entities in a specific category via Moralis")]
    async fn moralis_entities_category_entities(
        &self,
        Parameters(input): Parameters<MoralisCategoryIdInput>,
    ) -> String {
        tools::moralis_entities_category_entities(&input.category_id)
            .await
            .to_response()
    }

    // =========================================================================
    // MORALIS VOLUME
    // =========================================================================

    #[tool(description = "Get trading volume by chain via Moralis")]
    async fn moralis_volume_chains(&self) -> String {
        tools::moralis_volume_chains().await.to_response()
    }

    #[tool(description = "Get trading volume by category via Moralis")]
    async fn moralis_volume_categories(&self) -> String {
        tools::moralis_volume_categories().await.to_response()
    }

    #[tool(
        description = "Get overall trading volume timeseries (optional timeframe and date range) via Moralis"
    )]
    async fn moralis_volume_timeseries(
        &self,
        Parameters(input): Parameters<MoralisVolumeTimeseriesInput>,
    ) -> String {
        tools::moralis_volume_timeseries(
            input.timeframe.as_deref(),
            input.from_date.as_deref(),
            input.to_date.as_deref(),
        )
        .await
        .to_response()
    }

    #[tool(
        description = "Get trading volume timeseries for a specific category (optional timeframe and date range) via Moralis"
    )]
    async fn moralis_volume_category_timeseries(
        &self,
        Parameters(input): Parameters<MoralisVolumeCategoryTimeseriesInput>,
    ) -> String {
        tools::moralis_volume_category_timeseries(
            &input.category_id,
            input.timeframe.as_deref(),
            input.from_date.as_deref(),
            input.to_date.as_deref(),
        )
        .await
        .to_response()
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

    // --- Dune Queries CRUD ---

    #[tool(description = "Create a saved Dune Analytics query")]
    async fn dune_queries_create(
        &self,
        Parameters(input): Parameters<DuneQueriesCreateInput>,
    ) -> String {
        tools::dune_queries_create(
            &input.name,
            &input.sql,
            input.description.as_deref(),
            input.private,
            input.tags.as_deref(),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get details of a saved Dune query")]
    async fn dune_queries_get(&self, Parameters(input): Parameters<DuneQueriesGetInput>) -> String {
        tools::dune_queries_get(&input.query_id).await.to_response()
    }

    #[tool(description = "Update an existing Dune query")]
    async fn dune_queries_update(
        &self,
        Parameters(input): Parameters<DuneQueriesUpdateInput>,
    ) -> String {
        tools::dune_queries_update(
            &input.query_id,
            input.name.as_deref(),
            input.sql.as_deref(),
            input.description.as_deref(),
            input.tags.as_deref(),
        )
        .await
        .to_response()
    }

    #[tool(description = "List all saved Dune queries")]
    async fn dune_queries_list(
        &self,
        Parameters(input): Parameters<DuneQueriesListInput>,
    ) -> String {
        tools::dune_queries_list(input.limit, input.offset)
            .await
            .to_response()
    }

    #[tool(description = "Archive a Dune query")]
    async fn dune_queries_archive(
        &self,
        Parameters(input): Parameters<DuneQueriesIdInput>,
    ) -> String {
        tools::dune_queries_archive(&input.query_id)
            .await
            .to_response()
    }

    #[tool(description = "Unarchive a Dune query")]
    async fn dune_queries_unarchive(
        &self,
        Parameters(input): Parameters<DuneQueriesIdInput>,
    ) -> String {
        tools::dune_queries_unarchive(&input.query_id)
            .await
            .to_response()
    }

    #[tool(description = "Make a Dune query private")]
    async fn dune_queries_make_private(
        &self,
        Parameters(input): Parameters<DuneQueriesIdInput>,
    ) -> String {
        tools::dune_queries_make_private(&input.query_id)
            .await
            .to_response()
    }

    #[tool(description = "Make a Dune query public")]
    async fn dune_queries_make_public(
        &self,
        Parameters(input): Parameters<DuneQueriesIdInput>,
    ) -> String {
        tools::dune_queries_make_public(&input.query_id)
            .await
            .to_response()
    }

    // --- Dune Tables CRUD ---

    #[tool(description = "Create a Dune table with a JSON schema definition")]
    async fn dune_tables_create(
        &self,
        Parameters(input): Parameters<DuneTablesCreateInput>,
    ) -> String {
        tools::dune_tables_create(
            &input.namespace,
            &input.table_name,
            &input.schema_json,
            input.description.as_deref(),
            input.private,
        )
        .await
        .to_response()
    }

    #[tool(description = "Upload CSV data to a Dune table")]
    async fn dune_tables_upload_csv(
        &self,
        Parameters(input): Parameters<DuneTablesUploadCsvInput>,
    ) -> String {
        tools::dune_tables_upload_csv(
            &input.table_name,
            &input.data,
            input.description.as_deref(),
            input.private,
        )
        .await
        .to_response()
    }

    #[tool(description = "List Dune tables")]
    async fn dune_tables_list(&self, Parameters(input): Parameters<DuneTablesListInput>) -> String {
        tools::dune_tables_list(input.limit, input.offset)
            .await
            .to_response()
    }

    #[tool(description = "Get details of a Dune table")]
    async fn dune_tables_get(&self, Parameters(input): Parameters<DuneTablesGetInput>) -> String {
        tools::dune_tables_get(&input.namespace, &input.table_name)
            .await
            .to_response()
    }

    #[tool(description = "Insert rows into a Dune table")]
    async fn dune_tables_insert(
        &self,
        Parameters(input): Parameters<DuneTablesInsertInput>,
    ) -> String {
        tools::dune_tables_insert(&input.namespace, &input.table_name, &input.data_json)
            .await
            .to_response()
    }

    #[tool(description = "Clear all data from a Dune table")]
    async fn dune_tables_clear(
        &self,
        Parameters(input): Parameters<DuneTablesNamespaceTableInput>,
    ) -> String {
        tools::dune_tables_clear(&input.namespace, &input.table_name)
            .await
            .to_response()
    }

    #[tool(description = "Delete a Dune table")]
    async fn dune_tables_delete(
        &self,
        Parameters(input): Parameters<DuneTablesNamespaceTableInput>,
    ) -> String {
        tools::dune_tables_delete(&input.namespace, &input.table_name)
            .await
            .to_response()
    }

    // --- Dune Materialized Views ---

    #[tool(description = "Create or update a Dune materialized view")]
    async fn dune_matviews_upsert(
        &self,
        Parameters(input): Parameters<DuneMatviewsUpsertInput>,
    ) -> String {
        tools::dune_matviews_upsert(
            &input.name,
            &input.query_id,
            input.cron.as_deref(),
            input.expires_at.as_deref(),
            input.private,
            input.performance.as_deref(),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get details of a Dune materialized view")]
    async fn dune_matviews_get(
        &self,
        Parameters(input): Parameters<DuneMatviewsNameInput>,
    ) -> String {
        tools::dune_matviews_get(&input.name).await.to_response()
    }

    #[tool(description = "List Dune materialized views")]
    async fn dune_matviews_list(
        &self,
        Parameters(input): Parameters<DuneMatviewsListInput>,
    ) -> String {
        tools::dune_matviews_list(input.limit, input.offset)
            .await
            .to_response()
    }

    #[tool(description = "Refresh a Dune materialized view")]
    async fn dune_matviews_refresh(
        &self,
        Parameters(input): Parameters<DuneMatviewsRefreshInput>,
    ) -> String {
        tools::dune_matviews_refresh(&input.name, input.performance.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Delete a Dune materialized view")]
    async fn dune_matviews_delete(
        &self,
        Parameters(input): Parameters<DuneMatviewsNameInput>,
    ) -> String {
        tools::dune_matviews_delete(&input.name).await.to_response()
    }

    // --- Dune Pipelines ---

    #[tool(description = "Execute a Dune pipeline")]
    async fn dune_pipelines_execute(
        &self,
        Parameters(input): Parameters<DunePipelinesExecuteInput>,
    ) -> String {
        tools::dune_pipelines_execute(
            &input.pipeline_json,
            input.params.as_deref(),
            input.performance.as_deref(),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get Dune pipeline execution status")]
    async fn dune_pipelines_status(
        &self,
        Parameters(input): Parameters<DunePipelinesStatusInput>,
    ) -> String {
        tools::dune_pipelines_status(&input.execution_id)
            .await
            .to_response()
    }

    // --- Dune Usage ---

    #[tool(description = "Get Dune Analytics usage and billing information")]
    async fn dune_usage(&self) -> String {
        tools::dune_usage().await.to_response()
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

    // --- Router additional ---

    #[tool(description = "Get calldata for a Curve swap (encode route into transaction data)")]
    async fn curve_router_encode(
        &self,
        Parameters(input): Parameters<CurveRouterEncodeInput>,
    ) -> String {
        tools::curve_router_encode(
            &input.from,
            &input.to,
            &input.amount,
            &input.min_out,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Get Curve router graph statistics (token count, edge count)")]
    async fn curve_router_stats(&self, Parameters(input): Parameters<CurvePoolsInput>) -> String {
        tools::curve_router_stats(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get Curve router contract address for a chain")]
    async fn curve_router_address(&self, Parameters(input): Parameters<CurvePoolsInput>) -> String {
        tools::curve_router_address(Some(&input.chain))
            .await
            .to_response()
    }

    // --- Pools additional ---

    #[tool(description = "Get Curve pools from a specific registry on a chain")]
    async fn curve_pools_registry(
        &self,
        Parameters(input): Parameters<CurveChainRegistryInput>,
    ) -> String {
        tools::curve_pools_registry(&input.chain, &input.registry)
            .await
            .to_response()
    }

    #[tool(description = "Get all Curve pools across all chains")]
    async fn curve_pools_all(&self) -> String {
        tools::curve_pools_all().await.to_response()
    }

    #[tool(description = "Get Curve pools with TVL >= $10k (optionally filtered by chain)")]
    async fn curve_pools_big(
        &self,
        Parameters(input): Parameters<CurveOptionalChainInput>,
    ) -> String {
        tools::curve_pools_big(input.chain.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get Curve pools with TVL < $10k (optionally filtered by chain)")]
    async fn curve_pools_small(
        &self,
        Parameters(input): Parameters<CurveOptionalChainInput>,
    ) -> String {
        tools::curve_pools_small(input.chain.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get Curve pools with $0 TVL (optionally filtered by chain)")]
    async fn curve_pools_empty(
        &self,
        Parameters(input): Parameters<CurveOptionalChainInput>,
    ) -> String {
        tools::curve_pools_empty(input.chain.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get Curve pool addresses on a chain")]
    async fn curve_pools_addresses(
        &self,
        Parameters(input): Parameters<CurvePoolsInput>,
    ) -> String {
        tools::curve_pools_addresses(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get hidden/dysfunctional Curve pools")]
    async fn curve_pools_hidden(&self) -> String {
        tools::curve_pools_hidden().await.to_response()
    }

    // --- Volumes additional ---

    #[tool(description = "Get all Curve gauge data")]
    async fn curve_volumes_gauges(&self) -> String {
        tools::curve_volumes_gauges().await.to_response()
    }

    #[tool(description = "Get total 24h Curve volume for a chain")]
    async fn curve_volumes_total(&self, Parameters(input): Parameters<CurvePoolsInput>) -> String {
        tools::curve_volumes_total(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get Curve base APYs for pools on a chain")]
    async fn curve_volumes_apys(&self, Parameters(input): Parameters<CurvePoolsInput>) -> String {
        tools::curve_volumes_apys(Some(&input.chain))
            .await
            .to_response()
    }

    #[tool(description = "Get crvUSD AMM volumes")]
    async fn curve_volumes_crvusd(&self) -> String {
        tools::curve_volumes_crvusd().await.to_response()
    }

    // --- Lending additional ---

    #[tool(description = "Get all Curve lending vaults across all chains")]
    async fn curve_lending_all(&self) -> String {
        tools::curve_lending_all().await.to_response()
    }

    #[tool(description = "Get Curve lending vaults from a specific registry on a chain")]
    async fn curve_lending_registry(
        &self,
        Parameters(input): Parameters<CurveChainRegistryInput>,
    ) -> String {
        tools::curve_lending_registry(&input.chain, &input.registry)
            .await
            .to_response()
    }

    // --- CrvUSD additional ---

    #[tool(description = "Get crvUSD circulating supply")]
    async fn curve_crvusd_circulating_supply(&self) -> String {
        tools::curve_crvusd_circulating_supply().await.to_response()
    }

    #[tool(description = "Get scrvUSD total supply")]
    async fn curve_crvusd_scrvusd_supply(&self) -> String {
        tools::curve_crvusd_scrvusd_supply().await.to_response()
    }

    #[tool(description = "Get crvUSD markets (optionally filtered by chain)")]
    async fn curve_crvusd_markets(
        &self,
        Parameters(input): Parameters<CurveOptionalChainInput>,
    ) -> String {
        tools::curve_crvusd_markets(input.chain.as_deref())
            .await
            .to_response()
    }

    #[tool(description = "Get crvUSD savings stats")]
    async fn curve_crvusd_savings(&self) -> String {
        tools::curve_crvusd_savings().await.to_response()
    }

    // --- Prices additional ---

    #[tool(description = "Get Curve supported chains for prices")]
    async fn curve_prices_chains(&self) -> String {
        tools::curve_prices_chains().await.to_response()
    }

    #[tool(description = "Get Curve chain stats")]
    async fn curve_prices_chain_stats(&self) -> String {
        tools::curve_prices_chain_stats().await.to_response()
    }

    #[tool(description = "Get USD price for a specific token on Curve")]
    async fn curve_prices_token(
        &self,
        Parameters(input): Parameters<CurveChainAddressInput>,
    ) -> String {
        tools::curve_prices_token(&input.chain, &input.address)
            .await
            .to_response()
    }

    #[tool(
        description = "Get price history for a token on Curve. Requires start and end timestamps."
    )]
    async fn curve_prices_history(
        &self,
        Parameters(input): Parameters<CurveChainAddressTimeRangeInput>,
    ) -> String {
        tools::curve_prices_history(&input.chain, &input.address, input.start, input.end)
            .await
            .to_response()
    }

    #[tool(description = "Get top tokens by volume on Curve")]
    async fn curve_prices_top_volume(&self) -> String {
        tools::curve_prices_top_volume().await.to_response()
    }

    // --- OHLC additional ---

    #[tool(description = "Get LP token OHLC data on Curve. Requires start and end timestamps.")]
    async fn curve_ohlc_lp_token(
        &self,
        Parameters(input): Parameters<CurveChainAddressTimeRangeInput>,
    ) -> String {
        tools::curve_ohlc_lp_token(&input.chain, &input.address, input.start, input.end)
            .await
            .to_response()
    }

    // --- DAO additional ---

    #[tool(description = "Get Curve DAO proposals")]
    async fn curve_dao_proposals(&self) -> String {
        tools::curve_dao_proposals().await.to_response()
    }

    #[tool(description = "Get top CRV lockers")]
    async fn curve_dao_lockers(
        &self,
        Parameters(input): Parameters<CurveDaoLockersInput>,
    ) -> String {
        tools::curve_dao_lockers(input.top).await.to_response()
    }

    // =========================================================================
    // QUOTE (additional)
    // =========================================================================

    #[tool(
        description = "Get quote from a specific aggregator (open-ocean, kyber-swap, 0x, 1inch, cow-swap, li-fi, velora, enso)"
    )]
    async fn quote_from(&self, Parameters(input): Parameters<QuoteFromInput>) -> String {
        tools::quote_from(
            &input.source,
            &input.from_token,
            &input.to_token,
            &input.amount,
            Some(&input.chain),
            input.decimals,
            input.sender.as_deref(),
            input.slippage,
            input.show_tx,
        )
        .await
        .to_response()
    }

    // =========================================================================
    // CHAINLINK (additional)
    // =========================================================================

    #[tool(description = "List available Chainlink Data Streams feeds")]
    async fn chainlink_streams_feeds(&self) -> String {
        tools::chainlink_streams_feeds().await.to_response()
    }

    #[tool(description = "Get latest report for a Chainlink Data Streams feed")]
    async fn chainlink_streams_latest(
        &self,
        Parameters(input): Parameters<ChainlinkStreamsFeedInput>,
    ) -> String {
        tools::chainlink_streams_latest(&input.feed_id)
            .await
            .to_response()
    }

    #[tool(description = "Get Chainlink Data Streams report at a specific timestamp")]
    async fn chainlink_streams_report(
        &self,
        Parameters(input): Parameters<ChainlinkStreamsReportInput>,
    ) -> String {
        tools::chainlink_streams_report(&input.feed_id, input.timestamp)
            .await
            .to_response()
    }

    #[tool(
        description = "Get bulk Chainlink Data Streams reports for multiple feeds at a timestamp"
    )]
    async fn chainlink_streams_bulk(
        &self,
        Parameters(input): Parameters<ChainlinkStreamsBulkInput>,
    ) -> String {
        tools::chainlink_streams_bulk(&input.feed_ids, input.timestamp)
            .await
            .to_response()
    }

    #[tool(description = "Get paginated Chainlink Data Streams report history for a feed")]
    async fn chainlink_streams_history(
        &self,
        Parameters(input): Parameters<ChainlinkStreamsHistoryInput>,
    ) -> String {
        tools::chainlink_streams_history(&input.feed_id, input.timestamp, input.limit)
            .await
            .to_response()
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

    #[tool(description = "Create a new order on CoW Swap (MEV-protected)")]
    async fn cowswap_create_order(
        &self,
        Parameters(input): Parameters<CowswapCreateOrderInput>,
    ) -> String {
        tools::cowswap_create_order(
            &input.sell_token,
            &input.buy_token,
            &input.sell_amount,
            &input.buy_amount,
            input.valid_to,
            &input.from,
            &input.receiver,
            &input.signature,
            &input.kind,
            &input.signing_scheme,
            input.fee_amount.as_deref(),
            input.app_data.as_deref(),
            input.partially_fillable,
            input.quote_id,
            Some(&input.chain),
        )
        .await
        .to_response()
    }

    #[tool(description = "Cancel an existing CoW Swap order")]
    async fn cowswap_cancel_order(
        &self,
        Parameters(input): Parameters<CowswapCancelOrderInput>,
    ) -> String {
        tools::cowswap_cancel_order(&input.uid, &input.signature, Some(&input.chain))
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

    #[tool(description = "Get just the transaction data from a LI.FI quote (convenience wrapper)")]
    async fn lifi_get_transaction(&self, Parameters(input): Parameters<LifiQuoteInput>) -> String {
        tools::lifi_get_transaction(
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

    #[tool(
        description = "Get updated transaction data for a route step from LI.FI (pass step JSON from a route response)"
    )]
    async fn lifi_step_transaction(
        &self,
        Parameters(input): Parameters<LifiStepTransactionInput>,
    ) -> String {
        tools::lifi_step_transaction(&input.step_json)
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

    #[tool(
        description = "Bundle multiple DeFi actions into one transaction via Enso. Actions format: [{\"protocol\":\"...\",\"action\":\"...\",\"args\":{...}}]"
    )]
    async fn enso_bundle(&self, Parameters(input): Parameters<EnsoBundleInput>) -> String {
        tools::enso_bundle(
            &input.from_address,
            &input.actions_json,
            Some(&input.chain_id),
            input.routing_strategy.as_deref(),
        )
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

    #[tool(description = "Add a new RPC endpoint with optional auto-optimization")]
    async fn endpoints_add(&self, Parameters(input): Parameters<EndpointsAddInput>) -> String {
        tools::endpoints_add(&input.url, input.chain.as_deref(), input.no_optimize)
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
