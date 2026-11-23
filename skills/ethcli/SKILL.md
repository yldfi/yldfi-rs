---
name: ethcli
description: Read Ethereum and EVM chain data with the ethcli command line tool or its MCP server (ethcli-mcp) - addresses, contracts, transactions, balances, logs, token transfers, prices, swap quotes and simulations. Use when a task needs on-chain facts, and to get decoded, already-computed answers instead of fetching raw data and working it out by hand.
---

# ethcli

ethcli is a read-only Ethereum and EVM command line tool. `ethcli-mcp` exposes
the same commands as MCP tools (`contract_inspect`, `tx_analyze`,
`rpc_multicall`, ...). Prefer the MCP tools when they are connected; otherwise
run the CLI. If a command below is missing, run `ethcli upgrade`.

ethcli does not sign or send transactions.

## Start here

| Question | Command | MCP tool |
|---|---|---|
| What is this address? | `ethcli contract inspect <addr> --json` | `contract_inspect` |
| What did this transaction do? | `ethcli tx <hash> --full --json` | `tx_analyze` with `full: true` |
| Several contract reads | `ethcli rpc multicall "<addr> <sig>(<returns>) [args]" ... --json` | `rpc_multicall` |
| One contract read | `ethcli contract call <addr> <function> [args] --json` | `contract_call` |
| A storage variable with no getter | `ethcli contract storage <addr> <var> [keys...] -o json` (`--list` for the layout) | `contract_storage` |
| State at a date | add `--block 2024-01-31` (or `@<unix time>`, or `7d`) | `block` input |
| Block for a time | `ethcli rpc block-at <time> --json` | `rpc_block_at` |
| When did a value change? | `ethcli rpc find-change "<addr> <sig>(<returns>) [args]" --from <block\|time> --json` (or `--balance`, `--storage <addr> <slot>`, `--code`, `--nonce`). Finds *a* change; add `--all` when the first change matters (`single_change_assumed`, `warning` in JSON) | `rpc_find_change` (`all: true`) |
| Why did a call revert? | read the error, or `ethcli sig error <data>` | `sig_error` |
| Safe owners, queue, a Safe tx | `ethcli safe info\|queue <safe> --json`, `ethcli safe tx <safeTxHash> --json` | `safe_info`, `safe_queue`, `safe_tx` |
| Whole address history | `ethcli account txs <addr> --all --json` | `account_txs` with `all: true` |
| Who funded an address? | `ethcli account funded-by <addr> --json` | `account_funded_by` |
| Cost of a transaction (incl. L1 data fee) | `ethcli gas cost <to> [data] -s <sig> -a "<args>" --from <addr> --chain base --json` | `gas_cost` |
| Names for addresses | `ethcli account labels <addr> ... --json` | `account_labels` |
| Open approvals and amount at risk | `ethcli account approvals <addr> --json` | `account_approvals` |
| Value a Uniswap LP position | `ethcli uniswap position <id> [--version v4] --json`, or `--v2 <pair> --owner <addr>` | `uniswap_position_value` |
| All v3 LP positions of an address | `ethcli uniswap lp-positions --owner <addr> --json` | `uniswap_lp_positions` |

These replace chains of smaller calls. `contract inspect` covers `rpc code`,
`contract selectors`, `contract abi`, `contract creation` and `token info`.
`tx --full` covers `rpc receipt` and `rpc block`. One `rpc multicall` replaces
a loop of `rpc call` or `contract call`. The smaller commands still exist for
single facts.

## Rules

- **Historical prices:** `price <token> --at <date|block|7d> --json` gives
  the DefiLlama price with `price_time` and `offset_seconds`; don't call
  price APIs by hand.
- **Don't read whole sources.** `contract source <addr> --list`, then
  `--function <name>` or `--grep <regex>`, instead of piping the full source
  through grep or head.
- **Ask for JSON.** Add `--json` to any command. (`--csv` exists on
  `account txs`, `internal-txs`, `erc20` and `rpc multicall` for
  spreadsheets, but it cannot say whether an `--all` history is complete.) Integers are decimal strings,
  because a uint256 does not fit a JSON number. Do not pipe text output
  through `tail`, `sed`, `awk` or `grep`.
- **Ask only for the fields you need.** `--fields hash,value` (dot paths
  such as `balance.wei`; applied to each row of an array) trims JSON output
  and implies `--json`: `ethcli account txs <addr> --fields hash,value`.
  Read-only MCP tools take the same thing as a `fields` input. Commands that
  change state refuse `--fields` before running.
- **Read JSON errors by code.** With JSON output, or when stdout is piped or
  captured (`ETHCLI_ERROR_FORMAT=json|text` overrides), a failure is one stderr line,
  `{"error":{"code","message","hint"}}`, exit code non-zero. Codes:
  `invalid_argument`, `not_verified` (try `contract source` / `contract inspect`),
  `missing_api_key`, `unauthorized`, `rate_limited`, `unsupported_chain`,
  `not_found`, `rpc_error`, `internal`. Follow `hint` when present.
- **Give return types to `rpc multicall` and `rpc call`.** Write
  `balanceOf(address)(uint256)`, or pass `-d uint256`, and the result comes
  back decoded.
- **Do not pick RPC endpoints.** ethcli chooses an archive or debug-capable
  endpoint for past blocks and traces. Pass `--rpc-url` only for a local fork
  or a URL the user gave you.

## Read these fields before reporting a result

- **`tx` `balance_changes`:** a native-token line with `"partial": true` is
  not final. Without a call trace, transfers made inside the call are not
  seen. Run with `--full`, which traces when an endpoint supports it, and
  report partial lines as incomplete.
- **`tx` `revert_from`:** `"trace"` is the real revert reason. `"replay"`
  comes from re-running the call on the parent block's state and can differ
  from what happened.
- **`revert.source`** on decoded errors: `abi` and `builtin` are certain.
  `signature_database` is a guess by 4-byte selector, and `other_candidates`
  lists alternatives. The same applies to function names from
  `interface.source: "bytecode"` in `contract inspect`.
- **`account ... --all`:** the JSON is
  `{"complete", "requests", "stopped", "count", "rows"}`. If `complete` is
  false, the rows are not the whole history; raise `--max-pages` or narrow
  the block range.
- **`account funded-by` `source`:** `explorer` is the explorer's own answer.
  `first_incoming_transfer` is the free fallback: the earliest incoming
  native-token transfer, ignoring token transfers.
- **`uniswap position` fees:** for v3, `fee_cross_check.agree` false means
  the pool fee-growth figure and the `collect` eth_call differ;
  the `collect` total is split into `tokens_owed` (withdrawn principal and
  fees credited earlier) and `uncollected_fees` (fees since), each counted
  once in `value_usd`. v2 has no separate fees
  (`uncollected_fees` is null; they are in the reserves). A `value_usd`
  field of null means a token had no price, not zero.
- **Fees on rollups:** `tx` `fee_wei` is the total fee, L1 data fee
  included; `fee_breakdown` splits it into `execution_fee_wei` and
  `l1_fee_wei` (the receipt's `l1Fee` on any chain, plus `operator_fee_wei`
  when charged; Arbitrum `gasUsedForL1` × price, already inside `gasUsed`). `l1_fee_included: true` means the `l1Fee` is
  already in the execution fee (Mantle, some eras). `model: "unknown"` means
  the L1 fee could not be determined: say so. On L1 chains `l1_fee_wei` is `null`: not applicable,
  not zero. With `--full`, `fee_breakdown.usd` prices it at block time.
- **`notes`:** many commands list what they could not fetch or what an
  answer leaves out. Pass these on to the user.

## Traps

- **A plain date is the start of the day in UTC.** `--block 2024-04-05` is the
  state before anything that happened on 5 April. For an end-of-day figure,
  pass the next day's date or `2024-04-05T23:59:59Z`.
- **In `rpc multicall`, `msg.sender` is the Multicall3 contract.** A function
  that answers for its caller, such as a no-argument `earned()` or an
  access-checked getter, returns the helper's value or reverts. Read those
  with `ethcli simulate call <addr> --sig "earned()" --from <account>`.
  Functions that take the account as an argument are fine in a batch.
- **MCP output is cut at 1 MB,** roughly 2,000 transactions. For full
  histories use the CLI with `-o ndjson`.
- **Some explorer endpoints need a paid plan.** `funded-by` falls back to a
  free method. `labels` then shows only the address book and built-in labels,
  with a note saying why.

## More

- `ethcli <command> --help` for every option, or `ethcli --help-json` for
  the command tree as JSON.
- Full reference: `crates/ethcli/LLMREF.md` and `crates/ethcli/CLAUDE.md` in
  the repository.
