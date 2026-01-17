//! Tests for dsim types against OpenAPI example responses

#[cfg(test)]
mod deserialization_tests {
    use crate::activity::ActivityResponse;
    use crate::balances::{BalancesResponse, SingleBalanceResponse};
    use crate::chains::ChainsResponse;
    use crate::collectibles::CollectiblesResponse;
    use crate::defi::DefiPositionsResponse;
    use crate::holders::TokenHoldersResponse;
    use crate::tokens::TokensResponse;
    use crate::transactions::TransactionsResponse;
    use crate::webhooks::{AddressesListResponse, Webhook, WebhooksListResponse};

    #[test]
    fn test_activity_response_deserialization() {
        let json = r#"{
            "activity": [
                {
                    "chain_id": 8453,
                    "block_number": 26635101,
                    "block_time": "2025-02-20T13:52:29+00:00",
                    "tx_hash": "0x184544c8d67a0cbed0a3f04abe5f958b96635e8c743c070f70e24b1c06cd1aa6",
                    "type": "receive",
                    "asset_type": "erc20",
                    "token_address": "0xf92e740ad181b13a484a886ed16aa6d32d71b19a",
                    "from": "0xd152f549545093347a162dce210e7293f1452150",
                    "value": "123069652500000000000",
                    "value_usd": 0.14017463965013963,
                    "token_metadata": {
                        "symbol": "ENT",
                        "decimals": 18,
                        "price_usd": 0.001138986230989314,
                        "pool_size": 5.2274054439382835
                    }
                }
            ],
            "next_offset": "KgAAAAAAAAAweDQ4ZDAwNGE2YzE3NWRiMzMxZTk5YmVhZjY0NDIzYjMwOTgzNTdhZTdAVxVC"
        }"#;

        let response: ActivityResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.activity.len(), 1);
        assert_eq!(response.activity[0].chain_id, 8453);
        assert_eq!(
            response.activity[0].activity_type.as_deref(),
            Some("receive")
        );
        assert_eq!(response.activity[0].asset_type.as_deref(), Some("erc20"));
        assert!(response.next_offset.is_some());
    }

    #[test]
    fn test_activity_with_function_call() {
        let json = r#"{
            "activity": [
                {
                    "chain_id": 1,
                    "block_number": 18500000,
                    "block_time": "2025-02-20T13:52:29+00:00",
                    "tx_hash": "0xabc123",
                    "type": "call",
                    "asset_type": "native",
                    "to": "0x1234567890123456789012345678901234567890",
                    "value": "1000000000000000000",
                    "function": {
                        "signature": "transfer(address,uint256)",
                        "name": "transfer",
                        "inputs": [
                            {"name": "to", "type": "address", "value": "0xabc"},
                            {"name": "amount", "type": "uint256", "value": "1000"}
                        ]
                    },
                    "contract_metadata": {
                        "name": "MyContract"
                    }
                }
            ]
        }"#;

        let response: ActivityResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.activity.len(), 1);
        let func = response.activity[0].function.as_ref().unwrap();
        assert_eq!(func.name.as_deref(), Some("transfer"));
        assert_eq!(func.inputs.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_activity_with_warnings() {
        let json = r#"{
            "activity": [],
            "warnings": [
                {
                    "code": "UNSUPPORTED_CHAIN_IDS",
                    "message": "Some requested chain_ids are not supported.",
                    "chain_ids": [9999, 77777777777],
                    "docs_url": "https://docs.sim.dune.com/evm/supported-chains"
                }
            ]
        }"#;

        let response: ActivityResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.warnings.len(), 1);
        assert_eq!(response.warnings[0].code, "UNSUPPORTED_CHAIN_IDS");
        assert_eq!(response.warnings[0].chain_ids.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_balances_response_deserialization() {
        let json = r#"{
            "wallet_address": "0xd8da6bf26964af9d7eed9e03e53415d37aa96045",
            "balances": [
                {
                    "chain": "ethereum",
                    "chain_id": 1,
                    "address": "0x0e36c45d16585d1801e37cfaa577a9ad39f9343a",
                    "amount": "45787557175393926414790300082",
                    "symbol": "Kendu",
                    "name": "Kendu of Bank",
                    "decimals": 18,
                    "price_usd": 1.233346836175539e+21,
                    "value_usd": 5.647193877847869e+31,
                    "pool_size": 1233.34683617554,
                    "low_liquidity": true
                }
            ],
            "next_offset": "opaque-pagination-token",
            "request_time": "2025-08-13T10:31:08Z",
            "response_time": "2025-08-13T10:31:08Z"
        }"#;

        let response: BalancesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            response.wallet_address,
            "0xd8da6bf26964af9d7eed9e03e53415d37aa96045"
        );
        assert_eq!(response.balances.len(), 1);
        assert_eq!(response.balances[0].symbol, "Kendu");
        assert_eq!(response.balances[0].low_liquidity, Some(true));
    }

    #[test]
    fn test_balances_with_historical_prices() {
        let json = r#"{
            "wallet_address": "0xd8da6bf26964af9d7eed9e03e53415d37aa96045",
            "balances": [
                {
                    "chain": "ethereum",
                    "chain_id": 1,
                    "address": "native",
                    "amount": "1000000000000000000",
                    "symbol": "ETH",
                    "name": "Ethereum",
                    "decimals": 18,
                    "price_usd": 3500.0,
                    "value_usd": 3500.0,
                    "historical_prices": [
                        {"offset_hours": 24, "price_usd": 3400.0},
                        {"offset_hours": 168, "price_usd": 3200.0}
                    ]
                }
            ],
            "request_time": "2025-08-13T10:31:08Z",
            "response_time": "2025-08-13T10:31:08Z"
        }"#;

        let response: BalancesResponse = serde_json::from_str(json).unwrap();
        let hp = response.balances[0].historical_prices.as_ref().unwrap();
        assert_eq!(hp.len(), 2);
        assert_eq!(hp[0].offset_hours, 24);
        assert_eq!(hp[0].price_usd, 3400.0);
    }

    #[test]
    fn test_balances_with_pool_metadata() {
        let json = r#"{
            "wallet_address": "0xtest",
            "balances": [
                {
                    "chain": "ethereum",
                    "chain_id": 1,
                    "address": "0xtoken",
                    "amount": "1000",
                    "symbol": "TKN",
                    "name": "Token",
                    "decimals": 18,
                    "pool": {
                        "pool_type": "uniswap_v3",
                        "address": "0xpool",
                        "token0": "0xtoken0",
                        "token1": "0xtoken1"
                    }
                }
            ],
            "request_time": "2025-08-13T10:31:08Z",
            "response_time": "2025-08-13T10:31:08Z"
        }"#;

        let response: BalancesResponse = serde_json::from_str(json).unwrap();
        let pool = response.balances[0].pool.as_ref().unwrap();
        assert_eq!(pool.pool_type.as_deref(), Some("uniswap_v3"));
    }

    #[test]
    fn test_single_balance_response() {
        let json = r#"{
            "request_time": "2025-10-07T13:18:14.789152386+00:00",
            "response_time": "2025-10-07T13:18:14.850098525+00:00",
            "wallet_address": "0xd8da6bf26964af9d7eed9e03e53415d37aa96045",
            "balances": [
                {
                    "chain": "ethereum",
                    "chain_id": 1,
                    "address": "0x146523e8db6337291243a63a5555f446fa6c279f",
                    "amount": "7156868995423049840501842481",
                    "symbol": "AiMeme",
                    "name": "Ai Meme",
                    "decimals": 18,
                    "price_usd": 129086.448055109,
                    "value_usd": 923854797814899,
                    "pool_size": 9.09741149400001,
                    "low_liquidity": true
                }
            ]
        }"#;

        let response: SingleBalanceResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.balances.len(), 1);
        assert_eq!(response.balances[0].symbol, "AiMeme");
    }

    #[test]
    fn test_tokens_response_deserialization() {
        let json = r#"{
            "contract_address": "native",
            "tokens": [
                {
                    "chain": "ethereum",
                    "chain_id": 1,
                    "price_usd": 3500.50,
                    "symbol": "ETH",
                    "name": "Ethereum",
                    "decimals": 18,
                    "total_supply": "120000000000000000000000000",
                    "market_cap": 420000000000.0,
                    "logo": "https://example.com/eth.png"
                }
            ]
        }"#;

        let response: TokensResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.contract_address, "native");
        assert_eq!(response.tokens.len(), 1);
        assert_eq!(response.tokens[0].symbol, "ETH");
        assert_eq!(response.tokens[0].price_usd, Some(3500.50));
    }

    #[test]
    fn test_token_holders_response_deserialization() {
        let json = r#"{
            "token_address": "0x63706e401c06ac8513145b7687a14804d17f814b",
            "chain_id": 8453,
            "holders": [
                {
                    "wallet_address": "0x4a79b0168296c0ef7b8f314973b82ad406a29f1b",
                    "balance": "13794442047246482254818",
                    "first_acquired": "2025-02-06T15:11:07+00:00",
                    "has_initiated_transfer": false
                },
                {
                    "wallet_address": "0xAb5801a7D398351b8bE11C439e05C5B3259aeC9B",
                    "balance": "25000000000000000000",
                    "first_acquired": "2024-01-15T10:30:00+00:00",
                    "has_initiated_transfer": true
                }
            ],
            "next_offset": "eyJwYWdlIjoyLCJsaW1pdCI6Mn0="
        }"#;

        let response: TokenHoldersResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.chain_id, 8453);
        assert_eq!(response.holders.len(), 2);
        assert!(!response.holders[0].has_initiated_transfer);
        assert!(response.holders[1].has_initiated_transfer);
    }

    #[test]
    fn test_transactions_response_deserialization() {
        let json = r#"{
            "transactions": [
                {
                    "address": "0x7532cd0651030d3dc80b28199a125fc9f5ac80fa",
                    "block_hash": "0x745bdf699cee1fef27b9304be43a2435c744500ef455cc6b7dfb4c34417601a2",
                    "block_number": 30819446,
                    "block_time": "2025-05-28T10:30:39Z",
                    "chain": "base",
                    "chain_id": 8453,
                    "from": "0x7532cd0651030d3dc80b28199a125fc9f5ac80fa",
                    "to": "0x6ff5693b99212da76ad316178a184ab56d299b43",
                    "data": "0x3593564c",
                    "gas_price": "0x5aaa88",
                    "hash": "0x1081ebf623669338e9de6865d08d7ff3a3d1b0ef6f6486b350c3caf5b2e9257d",
                    "index": 35,
                    "nonce": "0x8",
                    "transaction_type": "0x2",
                    "value": "0x0"
                }
            ],
            "next_offset": "QBHrUqbdBQDkBwAAAAAAACHjyAAAAAAAAAAAAAAAAAACAAAAAAAAAA"
        }"#;

        let response: TransactionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.transactions.len(), 1);
        assert_eq!(response.transactions[0].chain, "base");
        assert_eq!(response.transactions[0].block_number, 30819446);
    }

    #[test]
    fn test_transactions_with_decoded() {
        let json = r#"{
            "transactions": [
                {
                    "address": "0x7532cd0651030d3dc80b28199a125fc9f5ac80fa",
                    "block_hash": "0x745bdf699cee1fef27b9304be43a2435c744500ef455cc6b7dfb4c34417601a2",
                    "block_number": 30819446,
                    "block_time": "2025-05-28T10:30:39Z",
                    "chain": "base",
                    "chain_id": 8453,
                    "from": "0x7532cd0651030d3dc80b28199a125fc9f5ac80fa",
                    "hash": "0x1081ebf623669338e9de6865d08d7ff3a3d1b0ef6f6486b350c3caf5b2e9257d",
                    "decoded": {
                        "name": "execute",
                        "inputs": [
                            {"name": "_commands", "type": "bytes", "value": "0x08060c"},
                            {"name": "_deadline", "type": "uint256", "value": "1748430030"}
                        ]
                    },
                    "logs": [
                        {
                            "address": "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913",
                            "data": "0x00000000000000000000000000000000000000000000000000000000000186a0",
                            "topics": [
                                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
                            ],
                            "decoded": {
                                "name": "Transfer",
                                "inputs": [
                                    {"name": "sender", "type": "address", "value": "0x7532cd0651030d3dc80b28199a125fc9f5ac80fa"},
                                    {"name": "value", "type": "uint256", "value": "100000"}
                                ]
                            }
                        }
                    ]
                }
            ]
        }"#;

        let response: TransactionsResponse = serde_json::from_str(json).unwrap();
        let decoded = response.transactions[0].decoded.as_ref().unwrap();
        assert_eq!(decoded.name.as_deref(), Some("execute"));
        assert_eq!(decoded.inputs.as_ref().unwrap().len(), 2);

        let logs = response.transactions[0].logs.as_ref().unwrap();
        assert_eq!(logs.len(), 1);
        let log_decoded = logs[0].decoded.as_ref().unwrap();
        assert_eq!(log_decoded.name.as_deref(), Some("Transfer"));
    }

    #[test]
    fn test_chains_response_deserialization() {
        let json = r#"{
            "chains": [
                {
                    "chain_id": 42161,
                    "name": "arbitrum",
                    "tags": ["default", "mainnet"],
                    "balances": {"supported": true},
                    "transactions": {"supported": true},
                    "activity": {"supported": true}
                },
                {
                    "chain_id": 1,
                    "name": "ethereum",
                    "tags": ["default", "mainnet"],
                    "balances": {"supported": true},
                    "transactions": {"supported": true},
                    "activity": {"supported": true},
                    "token_info": {"supported": true},
                    "token_holders": {"supported": true},
                    "collectibles": {"supported": true},
                    "defi_positions": {"supported": true}
                }
            ]
        }"#;

        let response: ChainsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.chains.len(), 2);
        assert_eq!(response.chains[0].name, "arbitrum");
        assert_eq!(response.chains[0].chain_id, 42161);
        assert!(response.chains[1].balances.as_ref().unwrap().supported);
    }

    #[test]
    fn test_collectibles_response_deserialization() {
        let json = r#"{
            "address": "0xd8da6bf26964af9d7eed9e03e53415d37aa96045",
            "entries": [
                {
                    "contract_address": "0x5d28dcf2fbbd3738c0ebe9de03eafcb4ec33015d",
                    "token_standard": "ERC1155",
                    "token_id": "1",
                    "chain": "ethereum",
                    "chain_id": 1,
                    "name": "Beeplfg",
                    "description": "Beeplfg",
                    "symbol": "CRAP",
                    "image_url": "https://api.sim.dune.com/v1/evm/collectible/image/1/0x5d28dcf2fbbd3738c0ebe9de03eafcb4ec33015d/1",
                    "last_sale_price": "0",
                    "metadata": {
                        "uri": "ipfs://QmcnkkMnfL7fugsyrZPEZhPGciLMoo9kwWt1cg4QHLLx3w/0",
                        "attributes": [
                            {"key": "Color", "value": "255, 43, 163"},
                            {"key": "Stance", "value": "Greased"}
                        ]
                    },
                    "is_spam": false,
                    "spam_score": 0,
                    "balance": "8",
                    "last_acquired": "2025-08-10T03:58:59Z"
                }
            ],
            "next_offset": "opaque-pagination-token",
            "request_time": "2025-08-13T09:40:53Z",
            "response_time": "2025-08-13T09:40:53Z"
        }"#;

        let response: CollectiblesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.entries.len(), 1);
        assert_eq!(response.entries[0].token_standard, "ERC1155");
        assert!(!response.entries[0].is_spam);

        let metadata = response.entries[0].metadata.as_ref().unwrap();
        let attrs = metadata.attributes.as_ref().unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].key.as_deref(), Some("Color"));
    }

    #[test]
    fn test_collectibles_with_spam_explanations() {
        let json = r#"{
            "address": "0xtest",
            "entries": [
                {
                    "contract_address": "0xspam",
                    "token_standard": "ERC721",
                    "token_id": "1",
                    "chain": "ethereum",
                    "chain_id": 1,
                    "is_spam": true,
                    "spam_score": 85,
                    "explanations": [
                        {"feature": "trade_volume", "value": 0, "feature_score": 100, "feature_weight": 0.25},
                        {"feature": "externally_flagged", "value": true, "feature_score": 100, "feature_weight": 0.25}
                    ],
                    "balance": "1",
                    "last_acquired": "2025-08-10T03:58:59Z"
                }
            ],
            "request_time": "2025-08-13T09:40:53Z",
            "response_time": "2025-08-13T09:40:53Z"
        }"#;

        let response: CollectiblesResponse = serde_json::from_str(json).unwrap();
        assert!(response.entries[0].is_spam);
        assert_eq!(response.entries[0].spam_score, Some(85));

        let explanations = response.entries[0].explanations.as_ref().unwrap();
        assert_eq!(explanations.len(), 2);
    }

    #[test]
    fn test_defi_positions_response_erc4626() {
        let json = r#"{
            "positions": [
                {
                    "type": "Erc4626",
                    "chain_id": 1,
                    "token": "0xdcd0f5ab30856f28385f641580bbd85f88349124",
                    "token_name": "Autonomous Liquidity USD",
                    "token_symbol": "alUSD",
                    "underlying_token": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                    "underlying_token_name": "USD Coin",
                    "underlying_token_symbol": "USDC",
                    "underlying_token_decimals": 6,
                    "calculated_balance": "0.0673736869415349",
                    "price_in_usd": 0,
                    "usd_value": 0,
                    "logo": "https://api.sim.dune.com/beta/token/logo/1/0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                }
            ],
            "aggregations": {
                "total_usd_value": 0,
                "total_by_chain": {"1": 0}
            }
        }"#;

        let response: DefiPositionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.positions.len(), 1);
        assert_eq!(response.positions[0].position_type, "Erc4626");
        assert_eq!(
            response.positions[0].underlying_token_symbol.as_deref(),
            Some("USDC")
        );
    }

    #[test]
    fn test_defi_positions_uniswap_v2() {
        let json = r#"{
            "positions": [
                {
                    "type": "UniswapV2",
                    "chain_id": 1,
                    "protocol": "UniswapV2",
                    "pool": "0x09c29277d081a1b347f41277ff53116a30d4ddff",
                    "token0": "0x4206975c6d7135ad73129476ebe2b06e42f41f50",
                    "token0_name": "FWOG",
                    "token0_symbol": "FWOG",
                    "token0_decimals": 18,
                    "token1": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                    "token1_name": "Wrapped Ether",
                    "token1_symbol": "WETH",
                    "token1_decimals": 18,
                    "lp_balance": "0xca7455529bd53680000",
                    "token0_price": 3.981792910653e-11,
                    "token1_price": 3553.20946160682,
                    "calculated_balance": "59754",
                    "price_in_usd": 0.000802696996635525,
                    "usd_value": 47.9643563369592,
                    "logo": null
                }
            ]
        }"#;

        let response: DefiPositionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.positions[0].position_type, "UniswapV2");
        assert_eq!(response.positions[0].protocol.as_deref(), Some("UniswapV2"));
        assert_eq!(response.positions[0].token0_symbol.as_deref(), Some("FWOG"));
    }

    #[test]
    fn test_defi_positions_nft_v3() {
        let json = r#"{
            "positions": [
                {
                    "type": "Nft",
                    "chain_id": 8453,
                    "protocol": "UniswapV3",
                    "pool": "0x7a0640ca0c8b07789edaaefda82bf5790109aa44",
                    "token0": "0x07449b62ec61f12d6110c36db240436222b02530",
                    "token0_name": "DOGE",
                    "token0_symbol": "DOGE",
                    "token0_decimals": 18,
                    "token1": "0x4200000000000000000000000000000000000006",
                    "token1_name": "Wrapped Ether",
                    "token1_symbol": "WETH",
                    "token1_decimals": 18,
                    "positions": [
                        {
                            "tick_lower": -887272,
                            "tick_upper": 887272,
                            "token_id": "0x132c81",
                            "token0_price": 9.48452516697798e-7,
                            "token0_holdings": "613763253.0586",
                            "token0_rewards": "21676.3984019538",
                            "token1_price": 3570.25944039075,
                            "token1_holdings": "0.162913339628563",
                            "token1_rewards": "0.000010452682406464"
                        }
                    ],
                    "logo": null,
                    "usd_value": 1163.82606861717
                }
            ]
        }"#;

        let response: DefiPositionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.positions[0].position_type, "Nft");
        assert_eq!(response.positions[0].protocol.as_deref(), Some("UniswapV3"));

        let nft_positions = response.positions[0].positions.as_ref().unwrap();
        assert_eq!(nft_positions.len(), 1);
        assert_eq!(nft_positions[0].tick_lower, Some(-887272));
        assert_eq!(nft_positions[0].tick_upper, Some(887272));
    }

    #[test]
    fn test_defi_positions_nft_v4() {
        let json = r#"{
            "positions": [
                {
                    "type": "NftV4",
                    "chain_id": 1,
                    "protocol": "UniswapV4",
                    "pool_id": [33, 251, 41, 59, 157, 197, 59, 66, 250, 110, 99, 250, 36, 225, 33, 45, 231, 108, 136, 235, 122, 21, 185, 76, 210, 32, 252, 102, 39, 72, 81, 191],
                    "pool_manager": "0x000000000004444c5dc75cb358380d2e3de08a90",
                    "salt": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 33, 24],
                    "token0": "0x0000000000000000000000000000000000000000",
                    "token0_name": "Ether",
                    "token0_symbol": "ETH",
                    "token0_decimals": 18,
                    "token1": "0xf9c8631fba291bac14ed549a2dde7c7f2ddff1a8",
                    "token1_name": "Mighty Morphin Power Rangers",
                    "token1_symbol": "GoGo",
                    "token1_decimals": 18,
                    "positions": [
                        {
                            "tick_lower": -184220,
                            "tick_upper": 207220,
                            "token_id": "0x2118",
                            "token0_price": 3553.20946160682,
                            "token0_holdings": "0.000047749999999999",
                            "token0_rewards": "0.000008749999999999",
                            "token1_price": 0.000003581798534713,
                            "token1_holdings": "479952364.515627",
                            "token1_rewards": "8399.79998597393"
                        }
                    ],
                    "logo": "https://api.sim.dune.com/beta/token/logo/1/0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                    "usd_value": 1719.32351868043
                }
            ]
        }"#;

        let response: DefiPositionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.positions[0].position_type, "NftV4");
        assert!(response.positions[0].pool_id.is_some());
        assert!(response.positions[0].pool_manager.is_some());
        assert!(response.positions[0].salt.is_some());
    }

    #[test]
    fn test_webhook_deserialization() {
        let json = r#"{
            "id": "019a81c2-d84b-7141-85e8-86b072a59142",
            "team_id": "01K7RVMRT0BXQ9DRC8BK87MK39",
            "name": "Balance Changes Monitor",
            "type": "balances",
            "url": "https://example.com/webhooks/balances",
            "active": true,
            "created_at": "2025-11-14T09:47:01.580104Z",
            "updated_at": "2025-11-14T09:47:01.580104Z",
            "chain_ids": [1, 8453, 84532],
            "asset_type": "erc20",
            "token_address": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        }"#;

        let webhook: Webhook = serde_json::from_str(json).unwrap();
        assert_eq!(webhook.id, "019a81c2-d84b-7141-85e8-86b072a59142");
        assert_eq!(webhook.webhook_type, "balances");
        assert!(webhook.active);
        assert_eq!(webhook.chain_ids.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_webhooks_list_response() {
        let json = r#"{
            "webhooks": [
                {
                    "id": "019a81c2-d84b-7141-85e8-86b072a59142",
                    "name": "Balance Monitor",
                    "type": "balances",
                    "url": "https://example.com/webhooks",
                    "active": true
                },
                {
                    "id": "019a81c2-e95c-7241-95f9-97c183b6a253",
                    "name": "Activity Tracker",
                    "type": "activities",
                    "url": "https://example.com/activities",
                    "active": false,
                    "activity_type": "swap"
                }
            ],
            "next_offset": "cursor123"
        }"#;

        let response: WebhooksListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.webhooks.len(), 2);
        assert_eq!(response.webhooks[1].activity_type.as_deref(), Some("swap"));
    }

    #[test]
    fn test_addresses_list_response() {
        let json = r#"{
            "addresses": [
                "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
                "0x3f60008Dfd0EfC03F476D9B489D6C5B13B3eBF2C"
            ],
            "next_offset": "cursor456"
        }"#;

        let response: AddressesListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.addresses.len(), 2);
        assert!(response.next_offset.is_some());
    }
}
