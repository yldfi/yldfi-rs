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
| Who can call what on an unverified contract; where its calls go | `ethcli contract calls <addr> --lookup --json` (summary also in `contract analyze`) | `contract_calls` |
| Verified copy of the same code? | `ethcli contract twins <addr> --json` | `contract_twins` |
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
- **Clones with immutable args** (Sudoswap pairs, wighawag, Solady
  LibClone): `immutable_args` (in `contract inspect`'s `proxy` and in
  fingerprint/decompile/match `target`) is raw data. Its `words` and
  `address_like` split is a guess; read the implementation's getters for
  what the bytes mean.
- **Unverified contracts:** for access control and call targets, use the
  facts from `contract calls` (and the `calls` summary in `contract
  analyze`): each call site's target, selector and value with its
  provenance (`constant`, `code_constant`, `storage` slot, `calldata`
  offset, `msg_sender`, `call_return`, `keccak`, `unresolved`), and per
  function the ORIGIN/CALLER pcs and `guards` (msg.sender, tx.origin or
  address(this) compared with a constant, a slot or a mapping). These are
  inferred by a bounded abstract interpretation, not proven: they are more
  reliable than decompiler text (which is heuristic; do not base an
  access-control conclusion on it), but a resolved value can still be
  wrong on code built to mislead, and a function with `guards: []` may
  check access in a form the analysis does not recognise. Cross-check
  `caller_pcs`, and treat a sensitive function with no guard as "not
  shown", not "unprotected". Always read `coverage`: `unresolved` means
  unknown, not safe; `complete: false` means some code was not analysed
  (`states_pruned`, `invalid_opcodes`, `unresolved_jumps`; `opcode_sites`
  with `reached: false` lists sensitive opcodes no path reached, and
  `in_metadata: true` marks bytes in a trailing metadata region).
- **Verification match:** `contract inspect` `verification[]` and the
  `contract source` stderr line say `exact` (code and metadata hash
  match) or `partial` (code only; comments/names may differ from what was
  compiled). Etherscan does not report it (`not_reported`); when Etherscan
  serves the source of another address through a similar match,
  `contract source` says so in its output (a `// Note:` line, or
  `source_verified_for` on each JSON element) and `contract twins` shows
  `verified_for_this_address: false`.
- `contract twins --submit-similarity` publishes a match on Sourcify; it is
  a write (MCP `contract_twins_submit_similarity`, write tools only). Do not
  run it unless asked.
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

## MCP inputs

- `contract_call`: `sig` is a bare function name or a full signature (use
  the full one for overloads); arguments go in `args`, e.g.
  `{"address": "0x…", "sig": "symbol()", "args": [], "chain": "ethereum"}`.
- `cast_abi_decode`: `transfer(address,uint256)` for calldata that starts
  with the selector; bare types (`uint256`, `(address,uint256)`) for
  selector-free data. Not return-type syntax such as `symbol()(string)`.
- `contract_analyze` / `contract_selectors` on proxies: `follow_proxy: true`
  analyses the implementation; `lookup: true` names selectors;
  `dispatcher: true` and `checks: true` add selector-to-handler offsets and
  guard/external-call heuristics. It is bytecode analysis, not source.

## Other reads

- `rpc ...` for raw JSON-RPC (`rpc receipt <hash>`, `rpc code`, `rpc storage`).
- `contract abi|source|creation` and `account txs|internal-txs|erc20|erc721|erc1155`
  use the explorer (Etherscan, falling back to Sourcify/Blockscout for
  sources); set `ETHERSCAN_API_KEY` when rate limits matter. `tx --enrich`
  adds explorer names and symbols.
- `logs -c <addr> -e "Transfer(address,address,uint256)" -f <from> -t <to>`,
  `ens resolve|lookup`, `gas oracle`.
- `ethcli cast` replaces Foundry `cast` for local helpers: `to-wei`,
  `from-wei`, `to-hex`, `to-dec`, `keccak`, `sig`, `topic`, `checksum`,
  `compute-address`, `create2`, `abi-encode`, `abi-decode`, `index`.

## Simulation and keys

- `ethcli simulate call <to> --sig "fn(types)" <args> --from <addr> --via tenderly`
  (or `--data 0x…`). `--via tenderly|debug|trace|alchemy` need no Foundry;
  `--via cast` (default) and `--via anvil` do.
- Saved Tenderly runs: `simulate get|info <uuid>`; a truncated (non
  36-character) id gives "not found".
- Keys are set by `ethcli config set-<service>` (`set-tenderly`,
  `set-etherscan-key`, `set-alchemy`, ...) or environment variables
  (`TENDERLY_ACCESS_KEY` + `TENDERLY_ACCOUNT` + `TENDERLY_PROJECT`,
  `ETHERSCAN_API_KEY`, `ALCHEMY_API_KEY`, ...). Never print the config file or
  copy key values into output; `config show` and `endpoints list` redact them.

## Diagnostics and upgrading

- A key or endpoint failure: `ethcli doctor`; `ethcli endpoints health --probes 1`
  for a quick endpoint check.
- `ethcli upgrade` updates `ethcli` and the matching `ethcli-mcp` in place
  (follows symlinks, verifies, rolls back on failure) and refreshes installed
  skills. It runs the installed version's code. Restart the MCP server
  afterwards. Never put a wrapper script where `ethcli` could resolve to
  `ethcli-mcp`.

## When not to use ethcli

Writes and local Solidity work belong to Foundry: `cast send`, `forge`
build/test/script/deploy, `anvil` forks.

## More

- Unverified contract, want source that compiles to it: follow
  `skills/bytecode-to-source/SKILL.md` (`contract fingerprint`, which also
  hints the optimizer runs, optional `contract decompile`, `contract match
  --diff`).

- `ethcli <command> --help` for every option, or `ethcli --help-json` for
  the command tree as JSON.
- Full reference: `crates/ethcli/LLMREF.md` and `crates/ethcli/CLAUDE.md` in
  the repository.
- This skill ships inside ethcli: `ethcli skills install` writes the copy that
  matches the installed version (`--target claude|codex|dir`), and `ethcli
  skills status` says whether an installed copy is out of date.
