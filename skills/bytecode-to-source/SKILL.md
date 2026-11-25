---
name: bytecode-to-source
description: Rebuild Solidity source for an unverified contract until it compiles to the deployed bytecode, using ethcli (contract fingerprint, contract decompile, contract match) and forge. Use when asked to recover, reconstruct or rebuild source for a contract that has no verified source, or to check that a rebuild matches the chain.
---

# Bytecode to source

Goal: Solidity that, compiled with the right settings, gives the deployed
runtime code. Only `ethcli contract match` decides whether you got there.
Everything a decompiler prints is a guess.

Tools: `ethcli` (or the MCP tools `contract_fingerprint`, `contract_inspect`,
`contract_decompile`, `contract_match`) and Foundry's `forge`.

## Procedure

1. **Fingerprint.** `ethcli contract fingerprint <addr> --json`.
   Note the compiler and version (`metadata.version`), the metadata hash type,
   `min_evm_version` / `max_evm_version` (with `evm_version_reasons`) and the
   pipeline guess. Check the EVM version first: an executed PUSH0
   (`push0_executed`) means shanghai or later; none with solc 0.8.20 or
   later gives `max_evm_version: paris` with confidence `consistent`
   (that solc pushes zeros with PUSH0 from shanghai on; a shanghai build
   that never pushes zero would look the same, so try paris first, not
   only); MCOPY, TLOAD, TSTORE or BLOBHASH mean cancun. A wrong
   `evm_version` changes every function, so settle it before anything else.
   Set `solc_version`, `evm_version` and the hash type (`bytecode_hash = "ipfs"`, `"bzzr1"` or
   `"none"`) in `foundry.toml` to match. Via-IR is not in the code; start
   with the legacy pipeline. Runs: read `optimizer_runs_hint` before
   sweeping. Solc writes wide constants out when runs are high and computes
   them when runs are low: `PUSH32 0x08c379a0..` (the `Error(string)`
   selector) means about 5000 runs or more, `PUSH3 0x461bcd PUSH1 0xe5 SHL`
   means fewer, and other selectors or masks written out (`PUSH32
   0x70a08231..`, `PUSH20 0xff..ff`) next to the shifted selector mean about
   1000-4000; `optimizer_runs_bucket` sums this up as low, medium or high,
   or `unknown` when the evidence is weak. A computed `Error(string)`
   selector with every other constant written out is only "≥ ~1000" (solc
   0.8.17 keeps the selector computed even at 20000 runs), so the bucket is
   `unknown` there. EVM lower bounds come from executed opcodes; REVERT
   does not count (solc emitted it before Byzantium too). Sweep coarse values, keyed on
   the executable size: try the bucket's value (200, 1000 or 10000) and
   its neighbours, keep the run count whose executable size equals the
   deployed one, then refine. Optimizer-off code also writes constants out,
   so check `optimizer` first. `likely off` means two shapes solc's
   optimizer removes are both frequent (`SWAP2 SWAP1 POP` cleanups and the
   free memory pointer re-read at block starts; counts in
   `optimizer_evidence`): try a build with the optimizer disabled first,
   and confirm by size. It is a heuristic calibrated on 80 contracts, not
   proof. The runs bucket is then `unknown (optimizer likely off)`, with
   the hint as read kept in its evidence. `not detected as off` is not
   proof that it was on (small off contracts can miss it).
   If the proxy was followed, the output names the implementation: rebuild
   that. A set EIP-1967 slot is not proof of a proxy: some contracts keep an
   address there and run their own dispatcher, delegating only the
   selectors it does not match from the fallback (Yearn V3 tokenized
   strategies do this). Fingerprint, match and decompile follow the value
   only when the call analysis confirms every unmatched call runs that
   code with no dispatcher of its own; otherwise `target.proxy_slot`
   shows the value, its source and why it was not followed, and
   `--follow-proxy` forces it. A slot proxy whose forwarding DELEGATECALL
   can also take its target from somewhere else is held as uncertain. A
   proxy whose DELEGATECALL target depends on who calls (Arbitrum's
   RollupProxy: the admin runs the EIP-1967 implementation, everyone else
   a secondary slot) is never followed by default: `proxy_slot.kind` is
   `msg.sender-dependent proxy`, `reason` says why (one caller check with a
   different target on each side; one target only the checked caller
   reaches; a target computed from msg.sender/tx.origin; or incomplete
   analysis with several targets behind checks), and `candidates` lists
   each target (slot or constant), its value, `caller` (`admin`, `user`,
   `any`) and the `caller_checks` on the way (msg.sender or tx.origin
   against a value, one of several values, or a role mapping). Rebuild the
   one you need: `--follow-proxy` follows the user-side target only when
   there is exactly one; otherwise pick with `--follow-proxy-target
   admin|user|<address>` (an impossible choice is an `invalid_argument`
   error, never a silent fallback). The same decision is used by inspect,
   storage and the `--follow-proxy` paths of selectors, calls and analyze.
   Transparent proxies (the admin is refused or handled in the proxy
   itself) are not affected. The recognition covers branches on these
   checks; a check written another way may still be missed. Proxies with no standard slot (Compound
   delegators such as cDAI, Compound's Unitroller, aragonOS app proxies
   such as stETH, Decentraland LAND) are followed when all of these hold:
   every DELEGATECALL the analysis finds on the dispatcher path takes its
   target from one storage slot or from the return of the aragonOS
   kernel's `getApp(APP_BASES_NAMESPACE, sload(appIdSlot))`; at least one
   of them forwards the caller's calldata (selector from calldata offset
   0, length `calldatasize`); no dispatcher-path CALL to the same target
   takes its input from calldata (Synthetix's CALL-or-DELEGATECALL
   proxy). Even then it is held (`followed: false`, the reason,
   `force_with: --follow-proxy`) when the target is a whole address packed
   in a slot at a byte offset (the high 160 bits, shr 96, or old solc's
   `sload(s) / 0x100 & mask`, bits 8..168; `resolved.bits` says which),
   when a call opcode was not reached by the analysis, when
   the fallback path creates contracts, self-destructs, calls anything but
   itself or the proxy's slots or writes storage, or when a function does
   any of that or writes other slots (a multisig that delegates unknown
   calls to an `implementation` variable is held for this reason: it has
   logic of its own). Another packed layout is only noted as unsupported.
   Writes to fixed slots and an upgrade-and-call to the stored
   implementation are allowed after a msg.sender check that lets only the
   admin through (the other branch reverts) against
   storage, a constant or an admin read by STATICCALL, and a STATICCALL in
   a view function or feeding that check is not counted. A library
   call (constant selector or `msg.data[4:]`) and CALL-forwarding are not
   treated as proxies; a note says why. `target.proxy_slot.source` is
   `storage` or `aragon_kernel` and `resolved` gives the reads used.
   When `dispatcher` says
   "fallback-only / packed-calldata executor", there are no ABI functions:
   rebuild a `fallback()` that decodes the packed calldata at
   `calldata_reads`.
2. **Interface and storage from facts.** `ethcli contract inspect <addr> --json`
   (functions, mutability, proxy), `ethcli contract selectors <impl> --lookup`
   (evmole types), `ethcli contract analyze <impl> --dispatcher --checks`, and
   storage reads (`ethcli rpc storage`). Selectors that have no known name
   keep their exact selector: mine a name with that selector or use a
   placeholder name whose keccak gives it, never change the selector.
3. **Optional draft.** `ethcli contract decompile <impl>` (add
   `--backend all` to also run heimdall, which is better at guard
   conditions and revert strings and worse at parameter types). Read the
   cross-check block first. Each warning there is something the draft does
   not show (for example `tx.origin` guards, storage writes, creation of
   contracts). "Not found by evmdec" is not a fact. Use the draft for control
   flow and constants only.
4. **Write Solidity.** Plain Solidity only. **No inline assembly**, unless the
   fingerprint and disassembly show the original used it, and then keep it to
   the smallest block and say so. Assembly can copy bytecode without
   explaining it, so a match reached through assembly proves little.
   A high inline-assembly share is acceptable when the disassembly shows
   the original used assembly: raw `calldatacopy` loops, `revert(0, 0)`
   with no return data copied, calls with no `extcodesize` check,
   `BALANCE(ADDRESS)` where Solidity would emit `SELFBALANCE`. Then say the
   match proves the bytes, not the intent.
   In assembly-heavy code, statement order, the order variables are
   declared (their final stack order) and pointer style (a `p := add(p, n)`
   cursor against computed offsets) change the bytes even when the meaning
   is the same: the legacy optimizer's common-subexpression step only
   rewrites a block when its version is cheaper. Try the cursor style and
   reorder declarations before changing logic.
5. **Build.** `forge build --ast --build-info` (the ASTs of every source
   let `contract match` name functions, including inherited modifiers in
   other files, and measure the inline-assembly share).
6. **Match.** `ethcli contract match <addr> --artifact out/<File>.sol/<Name>.json --json`.
   - `mismatch`: read `match.mismatch.function`, the bytes around
     `first_difference_offset` and the per-function table. Add `--diff` to
     see the artifact and deployed opcodes side by side for each function
     marked different (`functions[].diff`: equal, changed, inserted,
     deleted rows, with pcs in hex as `disassemble` prints them); `changed`
     DUP/SWAP rows with the same other opcodes point to statement or
     declaration order. When the sizes differ, `equal_shifted` functions
     have the same instructions and their jumps reach corresponding
     JUMPDESTs through the alignment; they only moved. Fix the `different`
     ones first (`--only-different` lists just those); an `equal_shifted`
     function can still differ in data it reads from elsewhere. Fix only the
     function named `different`, rebuild, and match again. When the size
     differs, fix size-changing differences first (missing or extra
     statements, types, checked vs unchecked arithmetic).
   - When only compiler-generated or contract-level code differs, look for
     the constant or text shown, the ABI coder version, error strings,
     optimizer runs and function order.
   - Check `selectors.in_artifact_not_deployed` and
     `deployed_not_in_artifact` before anything else.
7. **Repeat** steps 4 to 6.

Library sources: when local checkouts of dependencies are used, pin them
to the version the target imported. In a blind trial never use a checkout
of the target project itself; it gives the answer away.

## Stop rules

- Stop with success at `exact`, or at `partial` when the original metadata
  cannot be reproduced (it hashes the original file names, comments and
  settings).
- Stop and report when the same function stays `different` after five
  attempts, when the size difference does not shrink over three builds, or
  when matching would need inline assembly that the original likely did not
  have.
- Do not tune settings at random. Change one, rebuild, match, and keep it
  only if the match improves.

## Reporting

- `exact`: "the rebuild compiles to the deployed code, metadata included".
- `partial`: "matches the deployed code ignoring metadata".
- `partial_unverified_metadata`: as `partial`, but say that the deployed
  metadata bytes could hide reachable code; never report it as a clean
  partial match. Never call a
  partial match verified, matching or equivalent without that qualifier.
- Always state the compiler settings, the inline-assembly share
  (`inline_assembly`) and any immutable or library values found
  (`match.immutables`, `match.libraries`), including whether all positions
  agreed.

## Verifying on Sourcify (manual)

Sourcify accepts the sources plus the compiler settings and reports a
"full" (exact) or "partial" match. Submit through its web UI or API
yourself, with the user's agreement: it publishes the sources. A partial
match there means the same as `partial` here. ethcli does not submit
anything.
