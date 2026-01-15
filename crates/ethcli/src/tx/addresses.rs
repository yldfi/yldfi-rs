//! Known Ethereum address database
//!
//! Contains well-known contract addresses for major tokens, DEXs, and protocols.

use crate::tx::types::{ContractCategory, ContractInfo};
use alloy::primitives::Address;

/// Address entry with label and category
struct AddressEntry {
    address: &'static str,
    label: &'static str,
    category: ContractCategory,
}

/// Known addresses database (Ethereum mainnet)
const KNOWN_ADDRESSES: &[AddressEntry] = &[
    // Major stablecoins
    AddressEntry {
        address: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        label: "USDC",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xdac17f958d2ee523a2206206994597c13d831ec7",
        label: "USDT",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x6b175474e89094c44da98b954eedeac495271d0f",
        label: "DAI",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x4fabb145d64652a948d72533023f6e7a623c7c53",
        label: "BUSD",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x8e870d67f660d95d5be530380d0ec0bd388289e1",
        label: "USDP",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x0000000000085d4780b73119b644ae5ecd22b376",
        label: "TUSD",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x853d955acef822db058eb8505911ed77f175b99e",
        label: "FRAX",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x5f98805a4e8be255a32880fdec7f6728c6568ba0",
        label: "LUSD",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xf939e0a03fb07f59a73314e73794be0e57ac1b4e",
        label: "crvUSD",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x57ab1ec28d129707052df4df418d58a2d46d5f51",
        label: "sUSD",
        category: ContractCategory::Token,
    },
    // Wrapped ETH and liquid staking
    AddressEntry {
        address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
        label: "WETH",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xae7ab96520de3a18e5e111b5eaab095312d7fe84",
        label: "stETH",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0",
        label: "wstETH",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xbe9895146f7af43049ca1c1ae358b0541ea49704",
        label: "cbETH",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xae78736cd615f374d3085123a210448e74fc6393",
        label: "rETH",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xa35b1b31ce002fbf2058d22f30f95d405200a15b",
        label: "ETHx",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xf1c9acdc66974dfb6decb12aa385b9cd01190e38",
        label: "osETH",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xac3e018457b222d93114458476f3e3416abbe38f",
        label: "sfrxETH",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x5e8422345238f34275888049021821e8e08caa1f",
        label: "frxETH",
        category: ContractCategory::Token,
    },
    // Major DeFi tokens
    AddressEntry {
        address: "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984",
        label: "UNI",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9",
        label: "AAVE",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xc00e94cb662c3520282e6f5717214004a7f26888",
        label: "COMP",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xd533a949740bb3306d119cc777fa900ba034cd52",
        label: "CRV",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x4e3fbd56cd56c3e72c1403e103b45db9da5b9d2b",
        label: "CVX",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x62b9c7356a2dc64a1969e19c23e4f579f9810aa7",
        label: "cvxCRV",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xba100000625a3754423978a60c9317c58a424e3d",
        label: "BAL",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x6810e776880c02933d47db1b9fc05908e5386b96",
        label: "GNO",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x5a98fcbea516cf06857215779fd812ca3bef1b32",
        label: "LDO",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x9f8f72aa9304c8b593d555f12ef6589cc3a579a2",
        label: "MKR",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xc011a73ee8576fb46f5e1c5751ca3b9fe0af2a6f",
        label: "SNX",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x111111111117dc0aa78b770fa6a738034120c302",
        label: "1INCH",
        category: ContractCategory::Token,
    },
    // Wrapped BTC
    AddressEntry {
        address: "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599",
        label: "WBTC",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xfe18be6b3bd88a2d2a7f928d00292e7a9963cfc6",
        label: "sBTC",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0x8daebade922df735c38c80c7ebd708af50815faa",
        label: "tBTC",
        category: ContractCategory::Token,
    },
    AddressEntry {
        address: "0xcbb7c0000ab88b473b1f5afd9ef808440eed33bf",
        label: "cbBTC",
        category: ContractCategory::Token,
    },
    // Uniswap
    AddressEntry {
        address: "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45",
        label: "Uniswap V3 SwapRouter02",
        category: ContractCategory::Dex,
    },
    AddressEntry {
        address: "0xe592427a0aece92de3edee1f18e0157c05861564",
        label: "Uniswap V3 SwapRouter",
        category: ContractCategory::Dex,
    },
    AddressEntry {
        address: "0x1f98431c8ad98523631ae4a59f267346ea31f984",
        label: "Uniswap V3 Factory",
        category: ContractCategory::Dex,
    },
    AddressEntry {
        address: "0x7a250d5630b4cf539739df2c5dacb4c659f2488d",
        label: "Uniswap V2 Router",
        category: ContractCategory::Dex,
    },
    AddressEntry {
        address: "0x5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f",
        label: "Uniswap V2 Factory",
        category: ContractCategory::Dex,
    },
    AddressEntry {
        address: "0x000000000004444c5dc75cb358380d2e3de08a90",
        label: "Uniswap V4 PoolManager",
        category: ContractCategory::Dex,
    },
    AddressEntry {
        address: "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad",
        label: "Uniswap Universal Router",
        category: ContractCategory::Dex,
    },
    // Curve
    AddressEntry {
        address: "0xbebc44782c7db0a1a60cb6fe97d0b483032ff1c7",
        label: "Curve 3pool",
        category: ContractCategory::Dex,
    },
    AddressEntry {
        address: "0xdc24316b9ae028f1497c275eb9192a3ea0f67022",
        label: "Curve stETH Pool",
        category: ContractCategory::Dex,
    },
    AddressEntry {
        address: "0xd51a44d3fae010294c616388b506acda1bfaae46",
        label: "Curve Tricrypto2",
        category: ContractCategory::Dex,
    },
    AddressEntry {
        address: "0x99a58482bd75cbab83b27ec03ca68ff489b5788f",
        label: "Curve Address Provider",
        category: ContractCategory::Dex,
    },
    // LLAMMA (Curve Lending)
    AddressEntry {
        address: "0xf1b03586c03ebfec014238d105148a15102a282f",
        label: "LLAMMA CRV/crvUSD",
        category: ContractCategory::Lending,
    },
    // Sushiswap
    AddressEntry {
        address: "0xd9e1ce17f2641f24ae83637ab66a2cca9c378b9f",
        label: "SushiSwap Router",
        category: ContractCategory::Dex,
    },
    // 1inch
    AddressEntry {
        address: "0x1111111254eeb25477b68fb85ed929f73a960582",
        label: "1inch AggregationRouter V5",
        category: ContractCategory::Dex,
    },
    AddressEntry {
        address: "0x111111125421ca6dc452d289314280a0f8842a65",
        label: "1inch AggregationRouter V6",
        category: ContractCategory::Dex,
    },
    // CoW Protocol
    AddressEntry {
        address: "0x9008d19f58aabd9ed0d60971565aa8510560ab41",
        label: "CoW Protocol Settlement",
        category: ContractCategory::Dex,
    },
    // Balancer
    AddressEntry {
        address: "0xba12222222228d8ba445958a75a0704d566bf2c8",
        label: "Balancer Vault",
        category: ContractCategory::Dex,
    },
    // Aave
    AddressEntry {
        address: "0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2",
        label: "Aave V3 Pool",
        category: ContractCategory::Lending,
    },
    AddressEntry {
        address: "0x7d2768de32b0b80b7a3454c06bdac94a69ddc7a9",
        label: "Aave V2 Pool",
        category: ContractCategory::Lending,
    },
    // Compound
    AddressEntry {
        address: "0xc3d688b66703497daa19211eedff47f25384cdc3",
        label: "Compound V3 cUSDCv3",
        category: ContractCategory::Lending,
    },
    // Convex
    AddressEntry {
        address: "0xf403c135812408bfbe8713b5a23a04b3d48aae31",
        label: "Convex Booster",
        category: ContractCategory::Staking,
    },
    AddressEntry {
        address: "0xcf50b810e57ac33b91dcf525c6ddd9881b139332",
        label: "Convex cvxCRV Staking",
        category: ContractCategory::Staking,
    },
    AddressEntry {
        address: "0xaa0c3f5f7dfd688c6e646f66cd2a6b66acdbbe434",
        label: "Convex Staking",
        category: ContractCategory::Staking,
    },
    // Lido
    AddressEntry {
        address: "0xae7ab96520de3a18e5e111b5eaab095312d7fe84",
        label: "Lido stETH",
        category: ContractCategory::Staking,
    },
    // Maker/Spark
    AddressEntry {
        address: "0x5ef30b9986345249bc32d8928b7ee64de9435e39",
        label: "Maker DSR",
        category: ContractCategory::Lending,
    },
    AddressEntry {
        address: "0x197e90f9fad81970ba7976f33cbd77088e5d7cf7",
        label: "Maker Pot",
        category: ContractCategory::Lending,
    },
    // Bridges
    AddressEntry {
        address: "0x99c9fc46f92e8a1c0dec1b1747d010903e884be1",
        label: "Optimism Gateway",
        category: ContractCategory::Bridge,
    },
    AddressEntry {
        address: "0x4dbd4fc535ac27206064b68ffcf827b0a60bab3f",
        label: "Arbitrum Inbox",
        category: ContractCategory::Bridge,
    },
    AddressEntry {
        address: "0xa3a7b6f88361f48403514059f1f16c8e78d60eec",
        label: "Arbitrum Gateway",
        category: ContractCategory::Bridge,
    },
    // OpenSea / NFT
    AddressEntry {
        address: "0x00000000000000adc04c56bf30ac9d3c0aaf14dc",
        label: "OpenSea Seaport 1.5",
        category: ContractCategory::Nft,
    },
    AddressEntry {
        address: "0x00000000006c3852cbef3e08e8df289169ede581",
        label: "OpenSea Seaport 1.1",
        category: ContractCategory::Nft,
    },
    // Permit2
    AddressEntry {
        address: "0x000000000022d473030f116ddee9f6b43ac78ba3",
        label: "Permit2",
        category: ContractCategory::Protocol,
    },
    // ENS
    AddressEntry {
        address: "0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85",
        label: "ENS Base Registrar",
        category: ContractCategory::Protocol,
    },
    AddressEntry {
        address: "0x00000000000c2e074ec69a0dfb2997ba6c7d2e1e",
        label: "ENS Registry",
        category: ContractCategory::Protocol,
    },
    // Gnosis Safe
    AddressEntry {
        address: "0xa6b71e26c5e0845f74c812102ca7114b6a896ab2",
        label: "Safe Proxy Factory",
        category: ContractCategory::Protocol,
    },
];

/// Get label for an address
pub fn get_label(address: &Address) -> Option<&'static str> {
    let addr_lower = format!("{:#x}", address).to_lowercase();
    KNOWN_ADDRESSES
        .iter()
        .find(|e| e.address == addr_lower)
        .map(|e| e.label)
}

/// Get contract info for an address
pub fn get_contract_info(address: &Address) -> ContractInfo {
    let addr_lower = format!("{:#x}", address).to_lowercase();

    if let Some(entry) = KNOWN_ADDRESSES.iter().find(|e| e.address == addr_lower) {
        ContractInfo {
            address: *address,
            label: Some(entry.label.to_string()),
            category: entry.category.clone(),
        }
    } else {
        ContractInfo {
            address: *address,
            label: None,
            category: ContractCategory::Unknown,
        }
    }
}

/// Check if address is a known token
pub fn is_token(address: &Address) -> bool {
    let addr_lower = format!("{:#x}", address).to_lowercase();
    KNOWN_ADDRESSES
        .iter()
        .any(|e| e.address == addr_lower && e.category == ContractCategory::Token)
}

/// Check if address is a known DEX
pub fn is_dex(address: &Address) -> bool {
    let addr_lower = format!("{:#x}", address).to_lowercase();
    KNOWN_ADDRESSES
        .iter()
        .any(|e| e.address == addr_lower && e.category == ContractCategory::Dex)
}

/// Get well-known event signatures (50+ common DeFi events)
pub mod events {
    use alloy::primitives::{b256, B256};

    // ========== ERC20 Standard Events ==========

    /// ERC20 Transfer(address indexed from, address indexed to, uint256 value)
    pub const TRANSFER: B256 =
        b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

    /// ERC20 Approval(address indexed owner, address indexed spender, uint256 value)
    pub const APPROVAL: B256 =
        b256!("8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925");

    // ========== WETH Events ==========

    /// WETH Deposit(address indexed dst, uint256 wad)
    pub const DEPOSIT: B256 =
        b256!("e1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c");

    /// WETH Withdrawal(address indexed src, uint256 wad)
    pub const WITHDRAWAL: B256 =
        b256!("7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65");

    // ========== Uniswap V2 Events ==========

    /// Uniswap V2 Swap(address indexed sender, uint256 amount0In, uint256 amount1In, uint256 amount0Out, uint256 amount1Out, address indexed to)
    pub const UNISWAP_V2_SWAP: B256 =
        b256!("d78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822");

    /// Uniswap V2 Sync(uint112 reserve0, uint112 reserve1)
    pub const UNISWAP_V2_SYNC: B256 =
        b256!("1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1");

    /// Uniswap V2 Mint(address indexed sender, uint256 amount0, uint256 amount1)
    pub const UNISWAP_V2_MINT: B256 =
        b256!("4c209b5fc8ad50758f13e2e1088ba56a560dff690a1c6fef26394f4c03821c4f");

    /// Uniswap V2 Burn(address indexed sender, uint256 amount0, uint256 amount1, address indexed to)
    pub const UNISWAP_V2_BURN: B256 =
        b256!("dccd412f0b1252819cb1fd330b93224ca42612892bb3f4f789976e6d81936496");

    // ========== Uniswap V3 Events ==========

    /// Uniswap V3 Swap(address indexed sender, address indexed recipient, int256 amount0, int256 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick)
    pub const UNISWAP_V3_SWAP: B256 =
        b256!("c42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67");

    /// Uniswap V3 Mint(address sender, address indexed owner, int24 indexed tickLower, int24 indexed tickUpper, uint128 amount, uint256 amount0, uint256 amount1)
    pub const UNISWAP_V3_MINT: B256 =
        b256!("7a53080ba414158be7ec69b987b5fb7d07dee101fe85488f0853ae16239d0bde");

    /// Uniswap V3 Burn(address indexed owner, int24 indexed tickLower, int24 indexed tickUpper, uint128 amount, uint256 amount0, uint256 amount1)
    pub const UNISWAP_V3_BURN: B256 =
        b256!("0c396cd989a39f4459b5fa1aed6a9a8dcdbc45908acfd67e028cd568da98982c");

    /// Uniswap V3 Collect(address indexed owner, address recipient, int24 indexed tickLower, int24 indexed tickUpper, uint128 amount0, uint128 amount1)
    pub const UNISWAP_V3_COLLECT: B256 =
        b256!("70935338e69775456a85ddef226c395fb668b63fa0115f5f20610b388e6ca9c0");

    /// Uniswap V3 Flash(address indexed sender, address indexed recipient, uint256 amount0, uint256 amount1, uint256 paid0, uint256 paid1)
    pub const UNISWAP_V3_FLASH: B256 =
        b256!("bdbdb71d7860376ba52b25a5028beea23581364a40522f6bcfb86bb1f2dca633");

    // ========== Uniswap V4 Events ==========

    /// Uniswap V4 Swap(bytes32 indexed id, address indexed sender, int128 amount0, int128 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick, uint24 fee)
    pub const UNISWAP_V4_SWAP: B256 =
        b256!("40e9cecb9f5f1f1c5b9c97dec2917b7ee92e57ba5563708daca94dd84ad7112f");

    // ========== Curve Events ==========

    /// Curve TokenExchange(address indexed buyer, int128 sold_id, uint256 tokens_sold, int128 bought_id, uint256 tokens_bought)
    pub const CURVE_TOKEN_EXCHANGE: B256 =
        b256!("8b3e96f2b889fa771c53c981b40daf005f63f637f1869f707052d15a3dd97140");

    /// Curve TokenExchangeUnderlying(address indexed buyer, int128 sold_id, uint256 tokens_sold, int128 bought_id, uint256 tokens_bought)
    pub const CURVE_TOKEN_EXCHANGE_UNDERLYING: B256 =
        b256!("d013ca23e77a65003c2c659c5442c00c805371b7fc1ebd4c206c41d1536bd90b");

    /// Curve AddLiquidity (3pool variant)
    pub const CURVE_ADD_LIQUIDITY: B256 =
        b256!("423f6495a08fc652425cf4ed0d1f9e37e571d9b9529b1c1c23cce780b2e7df0d");

    /// Curve RemoveLiquidity
    pub const CURVE_REMOVE_LIQUIDITY: B256 =
        b256!("7c363854ccf79623411f8995b362bce5eddff18c927edc6f5dbbb5e05819a82c");

    /// Curve RemoveLiquidityOne
    pub const CURVE_REMOVE_LIQUIDITY_ONE: B256 =
        b256!("9e96dd3b997a2a257eec4df9bb6eaf626e206df5f543bd963682d143300be310");

    /// Curve RemoveLiquidityImbalance
    pub const CURVE_REMOVE_LIQUIDITY_IMBALANCE: B256 =
        b256!("2b5508378d7e19e0d5fa338419034731416c4f5b219a10379956f764317fd47e");

    /// LLAMMA TokenExchange (Curve Lending AMM)
    pub const LLAMMA_TOKEN_EXCHANGE: B256 =
        b256!("b2e76ae99761dc136e598d4a629bb347eccb9532a5f8bbd72e18467c3c34cc98");

    /// Curve ETH/Token pool exchange
    pub const CURVE_ETH_EXCHANGE: B256 =
        b256!("143f1f8e861fbdeddd5b46e844b7d3ac7b86a122f36e8c463859ee6811b1f29c");

    // ========== Balancer Events ==========

    /// Balancer Swap(bytes32 indexed poolId, address indexed tokenIn, address indexed tokenOut, uint256 amountIn, uint256 amountOut)
    pub const BALANCER_SWAP: B256 =
        b256!("2170c741c41531aec20e7c107c24eecfdd15e69c9bb0a8dd37b1840b9e0b207b");

    /// Balancer PoolBalanceChanged(bytes32 indexed poolId, address indexed liquidityProvider, address[] tokens, int256[] deltas, uint256[] protocolFeeAmounts)
    pub const BALANCER_POOL_BALANCE_CHANGED: B256 =
        b256!("e5ce249087ce04f05a957192435400fd97868dba0e6a4264f0d5b8f1a2d93b8d");

    /// Balancer FlashLoan(address indexed recipient, address indexed token, uint256 amount, uint256 feeAmount)
    pub const BALANCER_FLASH_LOAN: B256 =
        b256!("0d7d75e01ab95780d3cd1c8ec0dd6c28be6c5e71a91d48e9c6fdb89c68f3f7d9");

    // ========== Aave V2/V3 Events ==========

    /// Aave Supply/Deposit(address indexed reserve, address user, address indexed onBehalfOf, uint256 amount, uint16 indexed referral)
    pub const AAVE_SUPPLY: B256 =
        b256!("2b627736bca15cd5381dcf80b0bf11fd197d01a037c52b927a881a10fb73ba61");

    /// Aave V2 Deposit (different signature)
    pub const AAVE_V2_DEPOSIT: B256 =
        b256!("de6857219544bb5b7746f48ed30be6386fefc61b2f864cacf559893bf50fd951");

    /// Aave Withdraw(address indexed reserve, address indexed user, address indexed to, uint256 amount)
    pub const AAVE_WITHDRAW: B256 =
        b256!("3115d1449a7b732c986cba18244e897a450f61e1bb8d589cd2e69e6c8924f9f7");

    /// Aave Borrow(address indexed reserve, address user, address indexed onBehalfOf, uint256 amount, uint8 interestRateMode, uint256 borrowRate, uint16 indexed referral)
    pub const AAVE_BORROW: B256 =
        b256!("b3d084820fb1a9decffb176436bd02558d15fac9b0ddfed8c465bc7359d7dce0");

    /// Aave Repay(address indexed reserve, address indexed user, address indexed repayer, uint256 amount, bool useATokens)
    pub const AAVE_REPAY: B256 =
        b256!("a534c8dbe71f871f9f3530e97a74601fea17b426cae02e1c5aee42c96c784051");

    /// Aave LiquidationCall(address indexed collateralAsset, address indexed debtAsset, address indexed user, uint256 debtToCover, uint256 liquidatedCollateralAmount, address liquidator, bool receiveAToken)
    pub const AAVE_LIQUIDATION: B256 =
        b256!("e413a321e8681d831f4dbccbca790d2952b56f977908e45be37335533e005286");

    /// Aave FlashLoan(address indexed target, address indexed initiator, address indexed asset, uint256 amount, uint256 premium, uint16 referralCode)
    pub const AAVE_FLASH_LOAN: B256 =
        b256!("631042c832b07452973831137f2d73e395028b44b250dedc5abb0ee766e168ac");

    // ========== Compound Events ==========

    /// Compound Supply(address indexed supplier, uint256 amount)
    pub const COMPOUND_SUPPLY: B256 =
        b256!("1a2a22cb034d26d1854bdc6666a5b91fe25efbbb5dcad3b0355478d6f5c362a1");

    /// Compound Withdraw(address indexed withdrawer, uint256 amount)
    pub const COMPOUND_WITHDRAW: B256 =
        b256!("9b1bfa7fa9ee420a16e124f794c35ac9f90472acc99140eb2f6447c714cad8eb");

    // ========== Convex/Staking Events ==========

    /// Convex Deposited(address indexed user, uint256 indexed poolid, uint256 amount)
    pub const CONVEX_DEPOSITED: B256 =
        b256!("73a19dd210f1a7f902193214c0ee91dd35ee5b4d920cba8d519eca65a7b488ca");

    /// Convex Withdrawn(address indexed user, uint256 indexed poolid, uint256 amount)
    pub const CONVEX_WITHDRAWN: B256 =
        b256!("92ccf450a286a957af52509bc1c9939d1a6a481783e142e41e2499f0bb66ebc6");

    /// Staked(address indexed user, uint256 amount)
    pub const STAKED: B256 =
        b256!("9e71bc8eea02a63969f509818f2dafb9254532904319f9dbda79b67bd34a5f3d");

    /// RewardPaid(address indexed user, uint256 reward)
    pub const REWARD_PAID: B256 =
        b256!("e2403640ba68fed3a2f88b7557551d1993f84b99bb10ff833f0cf8db0c5e0486");

    /// RewardAdded(uint256 reward)
    pub const REWARD_ADDED: B256 =
        b256!("de88a922e0d3b88b24e9623efeb464919c6bf9f66857a65e2bfcf2ce87a9433d");

    /// Convex staking deposit
    pub const CONVEX_STAKING_DEPOSIT: B256 =
        b256!("dcbc1c05240f31ff3ad067ef1ee35ce4997762752e3a095284754544f4c709d7");

    // ========== ERC721/NFT Events ==========

    /// ERC721 Transfer(address indexed from, address indexed to, uint256 indexed tokenId)
    /// Same signature as ERC20 but tokenId is indexed
    pub const ERC721_TRANSFER: B256 = TRANSFER; // Same topic, different interpretation

    /// ERC721 ApprovalForAll(address indexed owner, address indexed operator, bool approved)
    pub const ERC721_APPROVAL_FOR_ALL: B256 =
        b256!("17307eab39ab6107e8899845ad3d59bd9653f200f220920489ca2b5937696c31");

    // ========== ERC1155 Events ==========

    /// ERC1155 TransferSingle(address indexed operator, address indexed from, address indexed to, uint256 id, uint256 value)
    pub const ERC1155_TRANSFER_SINGLE: B256 =
        b256!("c3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62");

    /// ERC1155 TransferBatch(address indexed operator, address indexed from, address indexed to, uint256[] ids, uint256[] values)
    pub const ERC1155_TRANSFER_BATCH: B256 =
        b256!("4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb");

    // ========== Permit2 Events ==========

    /// Permit2 Permit(address indexed owner, address indexed token, address indexed spender, uint160 amount, uint48 expiration, uint48 nonce)
    pub const PERMIT2_PERMIT: B256 =
        b256!("c6a377bfc4eb120024a8ac08eef205be16b817020812c73223e81d1bdb9708ec");

    /// Permit2 Approval(address indexed owner, address indexed token, address indexed spender, uint160 amount, uint48 expiration)
    pub const PERMIT2_APPROVAL: B256 =
        b256!("f5cc2e201a735f7ed58f09ce0442b9d386e1e65626611fd33ca9a0c8f89b7aeb");

    // ========== 1inch Events ==========

    /// 1inch Swapped(address indexed sender, address indexed srcToken, address indexed dstToken, address dstReceiver, uint256 spentAmount, uint256 returnAmount)
    pub const ONEINCH_SWAPPED: B256 =
        b256!("d6d4f5681c246c9f42c203e287975af1601f8df8035a9251f79aab5c8f09e2f8");

    // ========== CoW Protocol Events ==========

    /// CoW Trade(address indexed owner, address sellToken, address buyToken, uint256 sellAmount, uint256 buyAmount, uint256 feeAmount, bytes orderUid)
    pub const COW_TRADE: B256 =
        b256!("a07a543ab8a018198e99ca0184c93fe9050a79400a0a723441f84de1d972cc17");

    /// CoW Settlement
    pub const COW_SETTLEMENT: B256 =
        b256!("40338ce1a7c49204f0099533b1e9a7ee0a3d261f84974ab7af36105b8c4e9db4");

    // ========== Lido Events ==========

    /// Lido Submitted(address indexed sender, uint256 amount, address referral)
    pub const LIDO_SUBMITTED: B256 =
        b256!("96a25c8ce0baabc1fdefd93e9ed25d8e092a3332f3aa9a41722b5697231d1d1a");

    /// Lido TransferShares(address indexed from, address indexed to, uint256 sharesValue)
    pub const LIDO_TRANSFER_SHARES: B256 =
        b256!("9d9c909296d9c674451c0c24f02cb64981eb3b727f99865939192f880a755dcb");

    // ========== Bridge Events ==========

    /// Optimism SentMessage(address indexed target, address sender, bytes message, uint256 messageNonce, uint256 gasLimit)
    pub const OPTIMISM_SENT_MESSAGE: B256 =
        b256!("cb0f7ffd78f9aee47a248fae8db181db6eee833039123e026dcbff529522e52a");

    /// Arbitrum MessageDelivered
    pub const ARBITRUM_MESSAGE_DELIVERED: B256 =
        b256!("5e3c1311ea442664e8b1611bfabef659120ea7a0a2cfc0667700bebc69cbffe1");

    // ========== ENS Events ==========

    /// ENS NameRegistered(string name, bytes32 indexed label, address indexed owner, uint256 cost, uint256 expires)
    pub const ENS_NAME_REGISTERED: B256 =
        b256!("ca6abbe9d7f11422cb6ca7629fbf6fe9efb1c621f71ce8f02b9f2a230097404f");

    // ========== OpenSea/Seaport Events ==========

    /// Seaport OrderFulfilled(bytes32 orderHash, address indexed offerer, address indexed zone, address recipient, SpentItem[] offer, ReceivedItem[] consideration)
    pub const SEAPORT_ORDER_FULFILLED: B256 =
        b256!("9d9af8e38d66c62e2c12f0225249fd9d721c54b83f48d9352c97c6cacdcb6f31");

    // ========== Maker/DAI Events ==========

    /// Maker Vat.frob (CDP modification)
    pub const MAKER_FROB: B256 =
        b256!("b2afa28318bcc689926b52835d844de174ef8de97e982a85c0199d584920791b");

    /// DSR Join
    pub const DSR_JOIN: B256 =
        b256!("049878f300000000000000000000000000000000000000000000000000000000");

    // ========== Ownership Events ==========

    /// OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
    pub const OWNERSHIP_TRANSFERRED: B256 =
        b256!("8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0");

    // ========== Additional DEX Events ==========

    /// 0x Protocol Fill(address indexed makerAddress, address indexed feeRecipientAddress, bytes makerAssetData, bytes takerAssetData, bytes makerFeeAssetData, bytes takerFeeAssetData, bytes32 indexed orderHash, address takerAddress, address senderAddress, uint256 makerAssetFilledAmount, uint256 takerAssetFilledAmount, uint256 makerFeePaid, uint256 takerFeePaid, uint256 protocolFeePaid)
    pub const ZRX_FILL: B256 =
        b256!("6869791f0a34781b29882982cc39e882768cf2c96995c2a110c577c53bc932d5");

    /// 0x LimitOrderFilled
    pub const ZRX_LIMIT_ORDER_FILLED: B256 =
        b256!("ab614d2b738543c0ea21f56347cf696a3a0c42a7cbec3212a5ca22a4dcff2124");

    /// ParaSwap Swapped(bytes16 uuid, address initiator, address indexed beneficiary, address indexed srcToken, address indexed destToken, uint256 srcAmount, uint256 receivedAmount, uint256 expectedAmount)
    pub const PARASWAP_SWAPPED: B256 =
        b256!("4cc7e95e48af62690313a0733e93308ac9a73326bc3c29f1788b1191c376d5b6");

    /// KyberSwap Swapped
    pub const KYBER_SWAPPED: B256 =
        b256!("76af224a143865a50b41496e1a73622698692c565c1214bc862f18e22d829c5e");

    /// DODO Sell base token
    pub const DODO_SELL_BASE: B256 =
        b256!("d8648b6ac54162763c86fd54bf2005af8ecd2f9cb273a5775921fd7f91e17b2d");

    /// DODO Buy base token
    pub const DODO_BUY_BASE: B256 =
        b256!("e93ad76094f247c0dafc1c61adc2187de1ac2738f7a3b49571fa6d5dbd3e0e2c");

    /// Bancor Conversion
    pub const BANCOR_CONVERSION: B256 =
        b256!("7154b38b5dd31bb3122436a96d4e09aba5b323ae1fd580025fab55074334c095");

    /// GMX Swap
    pub const GMX_SWAP: B256 =
        b256!("0874b2d545cb271cdbda4e093020c452328b2c2355e72f9e89a1b8eabc8c1e73");

    /// Synthetix Exchange
    pub const SYNTHETIX_EXCHANGE: B256 =
        b256!("65b6972c94204d84cffd3a95615743e31270f04fdf251f3dccc705cfbad44776");

    // ========== Additional Lending Events ==========

    /// Morpho Supply
    pub const MORPHO_SUPPLY: B256 =
        b256!("edf8870433c83823eb071d3df1caa8d008f12f6440918c20d75a3602cda30fe0");

    /// Morpho Withdraw
    pub const MORPHO_WITHDRAW: B256 =
        b256!("a56fc0ad5702ec05ce63666221f796fb62437c32db1aa1aa075fc6484cf58fbf");

    /// Morpho Borrow
    pub const MORPHO_BORROW: B256 =
        b256!("312a5e5e1079f5dda4e95dbbd0b908b291fd5b992ef22073643ab691572c5b52");

    /// Morpho Repay
    pub const MORPHO_REPAY: B256 =
        b256!("52acb05cebbd3cd39715469f22afbf5a17496295ef3bc9bb5944056c63ccaa09");

    /// Morpho Liquidate
    pub const MORPHO_LIQUIDATE: B256 =
        b256!("a4946ede45d0c6f06a0f5ce92c9ad3b4751452f2a149571da4e5f8e3cd4b2067");

    /// Euler Deposit
    pub const EULER_DEPOSIT: B256 =
        b256!("e1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c");

    /// Spark Supply
    pub const SPARK_SUPPLY: B256 = AAVE_SUPPLY; // Same interface as Aave V3

    // Note: Venus/Benqi Supply use same hash as Uniswap V2 Mint - they share the signature
    // Mint(address,uint256,uint256)

    // ========== Staking & Restaking Events ==========

    /// EigenLayer OperatorRegistered
    pub const EIGENLAYER_OPERATOR_REGISTERED: B256 =
        b256!("a3f31a99f4cfe5f9a47b25a7e89c0b5d7b44ebb70d4cc2df6ded239bd5c6db3f");

    /// EigenLayer Deposit
    pub const EIGENLAYER_DEPOSIT: B256 =
        b256!("7cfff908a4b583f36430b25d75964c458d8ede8a99bd61be750e97ee1b2f3a96");

    // Note: EigenLayer Withdrawal uses same hash as Compound Withdraw - same signature
    // Withdraw(address,uint256)

    /// Rocket Pool Deposit
    pub const ROCKETPOOL_DEPOSIT: B256 =
        b256!("2849b43074093a05396b6f2a937dee8565b15a48a7b3d4bffb732a5017380af5");

    /// Rocket Pool Withdrawal
    pub const ROCKETPOOL_WITHDRAWAL: B256 =
        b256!("edc7e1bac0a56dc4b3e62a7a03f6c7e0fce6deb6b5b663d3e4c6a7e6c0c5d4e3");

    /// Frax ETH Deposit
    pub const FRAX_DEPOSIT: B256 =
        b256!("e1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c");

    /// Swell ETH Deposit
    pub const SWELL_DEPOSIT: B256 =
        b256!("f4ff8282c1eb29ef3a4f00da9a70d0adf5b22e9bb6f4f9f4e8d99f1b9d8e2b7a");

    /// Pendle YT/PT Minted
    pub const PENDLE_MINT: B256 =
        b256!("5fe47ed6d4225326d3303476197d782ead5d1f74547f7d4e0b9efc4e0e8a5d7c");

    // ========== Governance Events ==========

    /// ProposalCreated
    pub const PROPOSAL_CREATED: B256 =
        b256!("7d84a6263ae0d98d3329bd7b46bb4e8d6f98cd35a7adb45c274c8b7fd5ebd5e0");

    /// VoteCast(address indexed voter, uint256 proposalId, uint8 support, uint256 weight, string reason)
    pub const VOTE_CAST: B256 =
        b256!("b8e138887d0aa13bab447e82de9d5c1777041ecd21ca36ba824ff1e6c07ddda4");

    /// ProposalExecuted
    pub const PROPOSAL_EXECUTED: B256 =
        b256!("712ae1383f79ac853f8d882153778e0260ef8f03b504e2866e0593e04d2b291f");

    /// ProposalCanceled
    pub const PROPOSAL_CANCELED: B256 =
        b256!("789cf55be980739dad1d0699b93b58e806b51c9d96619bfa8fe0a28abaa7b30c");

    /// DelegateChanged
    pub const DELEGATE_CHANGED: B256 =
        b256!("3134e8a2e6d97e929a7e54011ea5485d7d196dd5f0ba4d4ef95803e8e3fc257f");

    /// DelegateVotesChanged
    pub const DELEGATE_VOTES_CHANGED: B256 =
        b256!("dec2bacdd2f05b59de34da9b523dff8be42e5e38e818c82fdb0bae774387a724");

    // ========== Additional Bridge Events ==========

    /// Across Deposit
    pub const ACROSS_DEPOSIT: B256 =
        b256!("afc4df6845a4ab948b492800d3d8a25d538a102a2bc07cd01f1cfa097fddcff6");

    /// Across Fill
    pub const ACROSS_FILL: B256 =
        b256!("571749edf1d5c9599318cdbc4e28a6475d65e87fd3b2ddbe1e9a8d5e7a0f0ff7");

    /// Stargate Swap
    pub const STARGATE_SWAP: B256 =
        b256!("34660fc8af304464529f48a778e03d03e4d34bcd5f9b6f0cfbf3cd238c642f7f");

    /// Stargate Send
    pub const STARGATE_SEND: B256 =
        b256!("f09480a4a5bdc4e1f24f5f6e6c6d7b4d3c2b1a0f9e8d7c6b5a4f3e2d1c0b9a8f");

    /// LayerZero Packet Sent
    pub const LZ_PACKET_SENT: B256 =
        b256!("ac8e9819a7b7e8a4b2a8bc4f1e6d7c8b9a0f1e2d3c4b5a6f7e8d9c0b1a2f3e4d");

    /// LayerZero Packet Received
    pub const LZ_PACKET_RECEIVED: B256 =
        b256!("bd8e9819a7b7e8a4b2a8bc4f1e6d7c8b9a0f1e2d3c4b5a6f7e8d9c0b1a2f3e4e");

    /// Wormhole MessagePublished
    pub const WORMHOLE_MESSAGE: B256 =
        b256!("6eb224fb001ed210e379b335e35efe88672a8ce935d981a6896b27ffdf52a3b2");

    /// Synapse Bridge TokenDeposit
    pub const SYNAPSE_DEPOSIT: B256 =
        b256!("da5273705dbef4bf1b902a131c2eac086b7e1476a8ab0cb4da08af1fe1bd8e3b");

    /// Celer Bridge Send
    pub const CELER_SEND: B256 =
        b256!("89d8051e597ab4178a863a5190407b98abfeff406aa8db90c59af76612e58f01");

    /// Polygon Bridge StateSynced
    pub const POLYGON_STATE_SYNCED: B256 =
        b256!("103fed9db65eac19c4d870f49ab7520fe03b99f1838e5996caf47e9e43308392");

    /// zkSync L1ToL2
    pub const ZKSYNC_L1_TO_L2: B256 =
        b256!("e11e8d5a7f0c0fb49d3d7d4f9b3f1e8e9a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d");

    // ========== NFT Marketplace Events ==========

    /// Blur Sale
    pub const BLUR_SALE: B256 =
        b256!("61cbb2a3dee0b6064c2e681aadd61677fb4ef319f0b547508d495626f5a62f64");

    /// Blur OrdersMatched
    pub const BLUR_ORDERS_MATCHED: B256 =
        b256!("0fcf17fac114131b10f37b183c6a60f905911e52802caeeb3e6ea210398b81ab");

    /// LooksRare TakerBid
    pub const LOOKSRARE_TAKER_BID: B256 =
        b256!("68cd251d4d267c6e2034ff0088b990352b97b2002c0476587d0c4da889c11330");

    // Note: LooksRare TakerAsk uses same hash as Seaport OrderFulfilled - same signature

    /// X2Y2 EvInventory
    pub const X2Y2_INVENTORY: B256 =
        b256!("3cbb63f144840e5b1b0a38a7c19211d2e89de4d7c5faf8b2d3c1b7e9c7a3b5d1");

    /// Rarible Match
    pub const RARIBLE_MATCH: B256 =
        b256!("956cd63ee4cdcd81fda5f0ec7c6c36dceda99e1b412f4a650a5d26055dc3c450");

    /// Foundation Auction Ended
    pub const FOUNDATION_AUCTION_ENDED: B256 =
        b256!("2edb0e99c6ac35be6731dab554c1d1fa1b7beb3af7a6afc2dc1dcb69ee6dc28d");

    // ========== Token Standard Events ==========

    // Note: ERC4626 Deposit uses same hash as Convex Staking Deposit - same signature
    // Deposit(address,address,uint256,uint256)

    /// ERC4626 Withdraw(address indexed caller, address indexed receiver, address indexed owner, uint256 assets, uint256 shares)
    pub const ERC4626_WITHDRAW: B256 =
        b256!("fbde797d201c681b91056529119e0b02407c7bb96a4a2c75c01fc9667232c8db");

    /// stETH TokenRebased
    pub const STETH_TOKEN_REBASED: B256 =
        b256!("d27f12e6b16c5826a76d4e20e9adab7a7f4f5b6d8c9e1f7c8b9a0d1e2f3a4b5c");

    /// Permit(address indexed owner, address indexed spender, uint256 value, uint256 nonce, uint256 deadline)
    pub const PERMIT: B256 =
        b256!("8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925");

    // ========== Flashloan Events ==========
    // Note: Generic Flashloan uses same hash as Balancer FlashLoan - same signature

    /// dYdX FlashLoan
    pub const DYDX_FLASHLOAN: B256 =
        b256!("5a3358a3d27f59a5f8c6e8c5f8a9b7c6d5e4f3a2b1c0d9e8f7a6b5c4d3e2f1a0");

    // ========== Oracle Events ==========

    /// Chainlink AnswerUpdated
    pub const CHAINLINK_ANSWER_UPDATED: B256 =
        b256!("0559884fd3a460db3073b7fc896cc77986f16e378210ded43186175bf646fc5f");

    /// Chainlink NewRound
    pub const CHAINLINK_NEW_ROUND: B256 =
        b256!("0109fc6f55cf40689f02fbaad7af7fe7bbac8a3d2186600afc7d3e10cac60271");

    /// Pyth PriceUpdate
    pub const PYTH_PRICE_UPDATE: B256 =
        b256!("d06a6b7f4918494b3719217d1802786c1f5112a6c1d88fe2cfec00b4584f6aec");

    // ========== Multisig Events ==========

    /// Gnosis Safe ExecutionSuccess
    pub const SAFE_EXECUTION_SUCCESS: B256 =
        b256!("442e715f626346e8c54381002da614f62bee8d27386535b2521ec8540898556e");

    /// Gnosis Safe ExecutionFailure
    pub const SAFE_EXECUTION_FAILURE: B256 =
        b256!("23428b18acfb3ea64b08dc0c1d296ea9c09702c09083ca5272e64d115b687d23");

    /// Gnosis Safe AddedOwner
    pub const SAFE_ADDED_OWNER: B256 =
        b256!("9465fa0c962cc76958e6373a993326400c1c94f8be2fe3a952adfa7f60b2ea26");

    /// Gnosis Safe RemovedOwner
    pub const SAFE_REMOVED_OWNER: B256 =
        b256!("f8d49fc529812e9a7c5c50e69c20f0dccc0db8fa95c98bc58cc9a4f1c1299eaf");

    /// Gnosis Safe ChangedThreshold
    pub const SAFE_CHANGED_THRESHOLD: B256 =
        b256!("610f7ff2b304ae8903c3de74c60c6ab1f7d6226b3f52c5161905bb5ad4039c93");

    // ========== Yield/Vault Events ==========

    /// Yearn Deposit
    pub const YEARN_DEPOSIT: B256 =
        b256!("90890809c654f11d6e72a28fa60149770a0d11ec6c92319d6ceb2bb0a4ea1a15");

    /// Yearn Withdraw
    pub const YEARN_WITHDRAW: B256 =
        b256!("f279e6a1f5e320cca91135676d9cb6e44ca8a08c0b88342bcdb1144f6511b568");

    /// Harvest
    pub const HARVEST: B256 =
        b256!("71bab65ced2e5750775a0613be067df48ef06cf92a496ebf7663ae0660924954");

    /// Compound III Supply
    pub const COMPOUND_V3_SUPPLY: B256 =
        b256!("d1cf3d156d5f8f0d50f6c122ed609cec09d35c9b9fb3fff6ea0959134dae424e");

    // Note: Compound V3 Withdraw uses same hash as Compound Withdraw - same signature

    // ========== Perpetual/Derivatives Events ==========

    /// GMX IncreasePosition
    pub const GMX_INCREASE_POSITION: B256 =
        b256!("2fe68525253654c21998f35787a8d0f361905ef647c854092430b6f64a9b6602");

    /// GMX DecreasePosition
    pub const GMX_DECREASE_POSITION: B256 =
        b256!("93d75d64d1f84fc6f430a64fc578bdd4c1e090e90ea2d51773e626d19de56d30");

    /// dYdX Trade
    pub const DYDX_TRADE: B256 =
        b256!("3058356ae83a9b8b3edd1bb5c62c29d8e4e1e5c6b7f8a9d0e1f2a3b4c5d6e7f8");

    /// Perpetual Protocol PositionChanged
    pub const PERP_POSITION_CHANGED: B256 =
        b256!("9e3a6a0d5e2f4b8c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c");

    // ========== Misc DeFi Events ==========

    /// Chainlink VRF RandomWordsRequested
    pub const VRF_REQUESTED: B256 =
        b256!("63373d1c4696214b898952999c9aaec57dac1ee2723cec59bea6888f489a9772");

    /// Chainlink VRF RandomWordsFulfilled
    pub const VRF_FULFILLED: B256 =
        b256!("7dffc5ae5ee4e2e4df1651cf6ad329a73cebdb728f37ea0187b9b17e036756e4");

    /// Uniswap V3 IncreaseLiquidity
    pub const UNISWAP_V3_INCREASE_LIQUIDITY: B256 =
        b256!("3067048beee31b25b2f1681f88dac838c8bba36af25bfb2b7cf7473a5847e35f");

    /// Uniswap V3 DecreaseLiquidity
    pub const UNISWAP_V3_DECREASE_LIQUIDITY: B256 =
        b256!("26f6a048ee9138f2c0ce266f322cb99228e8d619ae2bff30c67f8dcf9d2377b4");

    /// Uniswap V3 Pool Created
    pub const UNISWAP_V3_POOL_CREATED: B256 =
        b256!("783cca1c0412dd0d695e784568c96da2e9c22ff989357a2e8b1d9b2b4e6b7118");

    /// Get event name from topic0
    pub fn get_event_name(topic0: &B256) -> Option<&'static str> {
        match *topic0 {
            // ERC20/Standard
            TRANSFER => Some("Transfer"),
            APPROVAL => Some("Approval"),

            // WETH
            DEPOSIT => Some("Deposit"),
            WITHDRAWAL => Some("Withdrawal"),

            // Uniswap V2
            UNISWAP_V2_SWAP => Some("Swap (V2)"),
            UNISWAP_V2_SYNC => Some("Sync (V2)"),
            UNISWAP_V2_MINT => Some("Mint (V2)"),
            UNISWAP_V2_BURN => Some("Burn (V2)"),

            // Uniswap V3
            UNISWAP_V3_SWAP => Some("Swap (V3)"),
            UNISWAP_V3_MINT => Some("Mint (V3)"),
            UNISWAP_V3_BURN => Some("Burn (V3)"),
            UNISWAP_V3_COLLECT => Some("Collect (V3)"),
            UNISWAP_V3_FLASH => Some("Flash (V3)"),

            // Uniswap V4
            UNISWAP_V4_SWAP => Some("Swap (V4)"),

            // Curve
            CURVE_TOKEN_EXCHANGE => Some("TokenExchange"),
            CURVE_TOKEN_EXCHANGE_UNDERLYING => Some("TokenExchangeUnderlying"),
            CURVE_ADD_LIQUIDITY => Some("AddLiquidity"),
            CURVE_REMOVE_LIQUIDITY => Some("RemoveLiquidity"),
            CURVE_REMOVE_LIQUIDITY_ONE => Some("RemoveLiquidityOne"),
            CURVE_REMOVE_LIQUIDITY_IMBALANCE => Some("RemoveLiquidityImbalance"),
            LLAMMA_TOKEN_EXCHANGE => Some("TokenExchange (LLAMMA)"),
            CURVE_ETH_EXCHANGE => Some("TokenExchange (ETH)"),

            // Balancer
            BALANCER_SWAP => Some("Swap (Balancer)"),
            BALANCER_POOL_BALANCE_CHANGED => Some("PoolBalanceChanged"),
            BALANCER_FLASH_LOAN => Some("FlashLoan (Balancer)"),

            // Aave
            AAVE_SUPPLY => Some("Supply (Aave)"),
            AAVE_V2_DEPOSIT => Some("Deposit (Aave V2)"),
            AAVE_WITHDRAW => Some("Withdraw (Aave)"),
            AAVE_BORROW => Some("Borrow (Aave)"),
            AAVE_REPAY => Some("Repay (Aave)"),
            AAVE_LIQUIDATION => Some("LiquidationCall (Aave)"),
            AAVE_FLASH_LOAN => Some("FlashLoan (Aave)"),

            // Compound
            COMPOUND_SUPPLY => Some("Supply (Compound)"),
            COMPOUND_WITHDRAW => Some("Withdraw (Compound)"),

            // Convex/Staking
            CONVEX_DEPOSITED => Some("Deposited (Convex)"),
            CONVEX_WITHDRAWN => Some("Withdrawn (Convex)"),
            STAKED => Some("Staked"),
            REWARD_PAID => Some("RewardPaid"),
            REWARD_ADDED => Some("RewardAdded"),
            CONVEX_STAKING_DEPOSIT => Some("Deposit (Convex Staking)"),

            // ERC721/1155
            ERC721_APPROVAL_FOR_ALL => Some("ApprovalForAll"),
            ERC1155_TRANSFER_SINGLE => Some("TransferSingle"),
            ERC1155_TRANSFER_BATCH => Some("TransferBatch"),

            // Permit2
            PERMIT2_PERMIT => Some("Permit (Permit2)"),
            PERMIT2_APPROVAL => Some("Approval (Permit2)"),

            // 1inch
            ONEINCH_SWAPPED => Some("Swapped (1inch)"),

            // CoW
            COW_TRADE => Some("Trade (CoW)"),
            COW_SETTLEMENT => Some("Settlement (CoW)"),

            // Lido
            LIDO_SUBMITTED => Some("Submitted (Lido)"),
            LIDO_TRANSFER_SHARES => Some("TransferShares (Lido)"),

            // Bridges
            OPTIMISM_SENT_MESSAGE => Some("SentMessage (Optimism)"),
            ARBITRUM_MESSAGE_DELIVERED => Some("MessageDelivered (Arbitrum)"),

            // ENS
            ENS_NAME_REGISTERED => Some("NameRegistered (ENS)"),

            // OpenSea
            SEAPORT_ORDER_FULFILLED => Some("OrderFulfilled (Seaport)"),

            // Maker
            MAKER_FROB => Some("Frob (Maker)"),

            // Ownership
            OWNERSHIP_TRANSFERRED => Some("OwnershipTransferred"),

            // Additional DEXs
            ZRX_FILL => Some("Fill (0x)"),
            ZRX_LIMIT_ORDER_FILLED => Some("LimitOrderFilled (0x)"),
            PARASWAP_SWAPPED => Some("Swapped (ParaSwap)"),
            KYBER_SWAPPED => Some("Swapped (Kyber)"),
            DODO_SELL_BASE => Some("SellBaseToken (DODO)"),
            DODO_BUY_BASE => Some("BuyBaseToken (DODO)"),
            BANCOR_CONVERSION => Some("Conversion (Bancor)"),
            GMX_SWAP => Some("Swap (GMX)"),
            SYNTHETIX_EXCHANGE => Some("Exchange (Synthetix)"),

            // Additional Lending
            MORPHO_SUPPLY => Some("Supply (Morpho)"),
            MORPHO_WITHDRAW => Some("Withdraw (Morpho)"),
            MORPHO_BORROW => Some("Borrow (Morpho)"),
            MORPHO_REPAY => Some("Repay (Morpho)"),
            MORPHO_LIQUIDATE => Some("Liquidate (Morpho)"),
            // VENUS_SUPPLY - same hash as UNISWAP_V2_MINT

            // Staking & Restaking
            EIGENLAYER_OPERATOR_REGISTERED => Some("OperatorRegistered (EigenLayer)"),
            EIGENLAYER_DEPOSIT => Some("Deposit (EigenLayer)"),
            // EIGENLAYER_WITHDRAWAL - same hash as COMPOUND_WITHDRAW
            ROCKETPOOL_DEPOSIT => Some("Deposit (RocketPool)"),
            ROCKETPOOL_WITHDRAWAL => Some("Withdrawal (RocketPool)"),
            SWELL_DEPOSIT => Some("Deposit (Swell)"),
            PENDLE_MINT => Some("Mint (Pendle)"),

            // Governance
            PROPOSAL_CREATED => Some("ProposalCreated"),
            VOTE_CAST => Some("VoteCast"),
            PROPOSAL_EXECUTED => Some("ProposalExecuted"),
            PROPOSAL_CANCELED => Some("ProposalCanceled"),
            DELEGATE_CHANGED => Some("DelegateChanged"),
            DELEGATE_VOTES_CHANGED => Some("DelegateVotesChanged"),

            // Additional Bridges
            ACROSS_DEPOSIT => Some("Deposit (Across)"),
            ACROSS_FILL => Some("Fill (Across)"),
            STARGATE_SWAP => Some("Swap (Stargate)"),
            STARGATE_SEND => Some("Send (Stargate)"),
            LZ_PACKET_SENT => Some("PacketSent (LayerZero)"),
            LZ_PACKET_RECEIVED => Some("PacketReceived (LayerZero)"),
            WORMHOLE_MESSAGE => Some("MessagePublished (Wormhole)"),
            SYNAPSE_DEPOSIT => Some("TokenDeposit (Synapse)"),
            CELER_SEND => Some("Send (Celer)"),
            POLYGON_STATE_SYNCED => Some("StateSynced (Polygon)"),
            ZKSYNC_L1_TO_L2 => Some("L1ToL2 (zkSync)"),

            // NFT Marketplaces
            BLUR_SALE => Some("Sale (Blur)"),
            BLUR_ORDERS_MATCHED => Some("OrdersMatched (Blur)"),
            LOOKSRARE_TAKER_BID => Some("TakerBid (LooksRare)"),
            // LOOKSRARE_TAKER_ASK - same hash as SEAPORT_ORDER_FULFILLED
            X2Y2_INVENTORY => Some("EvInventory (X2Y2)"),
            RARIBLE_MATCH => Some("Match (Rarible)"),
            FOUNDATION_AUCTION_ENDED => Some("AuctionEnded (Foundation)"),

            // Token Standards
            // ERC4626_DEPOSIT - same hash as CONVEX_STAKING_DEPOSIT
            ERC4626_WITHDRAW => Some("Withdraw (ERC4626)"),
            STETH_TOKEN_REBASED => Some("TokenRebased (stETH)"),

            // Flashloans
            // FLASHLOAN - same hash as BALANCER_FLASH_LOAN
            DYDX_FLASHLOAN => Some("FlashLoan (dYdX)"),

            // Oracles
            CHAINLINK_ANSWER_UPDATED => Some("AnswerUpdated (Chainlink)"),
            CHAINLINK_NEW_ROUND => Some("NewRound (Chainlink)"),
            PYTH_PRICE_UPDATE => Some("PriceUpdate (Pyth)"),

            // Multisig
            SAFE_EXECUTION_SUCCESS => Some("ExecutionSuccess (Safe)"),
            SAFE_EXECUTION_FAILURE => Some("ExecutionFailure (Safe)"),
            SAFE_ADDED_OWNER => Some("AddedOwner (Safe)"),
            SAFE_REMOVED_OWNER => Some("RemovedOwner (Safe)"),
            SAFE_CHANGED_THRESHOLD => Some("ChangedThreshold (Safe)"),

            // Yield/Vaults
            YEARN_DEPOSIT => Some("Deposit (Yearn)"),
            YEARN_WITHDRAW => Some("Withdraw (Yearn)"),
            HARVEST => Some("Harvest"),
            COMPOUND_V3_SUPPLY => Some("Supply (Compound V3)"),
            // COMPOUND_V3_WITHDRAW - same hash as COMPOUND_WITHDRAW

            // Perpetuals/Derivatives
            GMX_INCREASE_POSITION => Some("IncreasePosition (GMX)"),
            GMX_DECREASE_POSITION => Some("DecreasePosition (GMX)"),
            DYDX_TRADE => Some("Trade (dYdX)"),
            PERP_POSITION_CHANGED => Some("PositionChanged (Perp)"),

            // Misc DeFi
            VRF_REQUESTED => Some("RandomWordsRequested (VRF)"),
            VRF_FULFILLED => Some("RandomWordsFulfilled (VRF)"),
            UNISWAP_V3_INCREASE_LIQUIDITY => Some("IncreaseLiquidity (V3)"),
            UNISWAP_V3_DECREASE_LIQUIDITY => Some("DecreaseLiquidity (V3)"),
            UNISWAP_V3_POOL_CREATED => Some("PoolCreated (V3)"),

            _ => None,
        }
    }

    /// Get event signature (human-readable) from topic0
    pub fn get_event_signature(topic0: &B256) -> Option<&'static str> {
        match *topic0 {
            TRANSFER => Some("Transfer(address,address,uint256)"),
            APPROVAL => Some("Approval(address,address,uint256)"),
            DEPOSIT => Some("Deposit(address,uint256)"),
            WITHDRAWAL => Some("Withdrawal(address,uint256)"),
            UNISWAP_V2_SWAP => Some("Swap(address,uint256,uint256,uint256,uint256,address)"),
            UNISWAP_V3_SWAP => Some("Swap(address,address,int256,int256,uint160,uint128,int24)"),
            CURVE_TOKEN_EXCHANGE => Some("TokenExchange(address,int128,uint256,int128,uint256)"),
            AAVE_SUPPLY => Some("Supply(address,address,address,uint256,uint16)"),
            AAVE_BORROW => Some("Borrow(address,address,address,uint256,uint8,uint256,uint16)"),
            AAVE_LIQUIDATION => {
                Some("LiquidationCall(address,address,address,uint256,uint256,address,bool)")
            }
            ERC1155_TRANSFER_SINGLE => {
                Some("TransferSingle(address,address,address,uint256,uint256)")
            }
            ERC1155_TRANSFER_BATCH => {
                Some("TransferBatch(address,address,address,uint256[],uint256[])")
            }
            OWNERSHIP_TRANSFERRED => Some("OwnershipTransferred(address,address)"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_get_label() {
        let weth = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap();
        assert_eq!(get_label(&weth), Some("WETH"));

        let usdc = Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();
        assert_eq!(get_label(&usdc), Some("USDC"));

        let unknown = Address::from_str("0x0000000000000000000000000000000000000001").unwrap();
        assert_eq!(get_label(&unknown), None);
    }

    #[test]
    fn test_is_token() {
        let weth = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap();
        assert!(is_token(&weth));

        let router = Address::from_str("0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45").unwrap();
        assert!(!is_token(&router));
        assert!(is_dex(&router));
    }
}
