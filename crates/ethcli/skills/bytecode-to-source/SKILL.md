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
   or `unknown` when the evidence is weak. Sweep coarse values, keyed on
   the executable size: try the bucket's value (200, 1000 or 10000) and
   its neighbours, keep the run count whose executable size equals the
   deployed one, then refine. Optimizer-off code also writes constants out.
   If the proxy was followed, the output names the implementation: rebuild
   that. A set EIP-1967 slot is not proof of a proxy: some contracts keep an
   address there and run their own dispatcher, delegating only the
   selectors it does not match from the fallback (Yearn V3 tokenized
   strategies do this). Fingerprint, match and decompile follow the value
   only when the call analysis confirms every unmatched call runs that
   code with no dispatcher of its own; otherwise `target.proxy_slot`
   shows the value, its source and why it was not followed, and
   `--follow-proxy` forces it. Proxies with no standard slot (Compound
   delegators such as cDAI, Compound's Unitroller, aragonOS app proxies
   such as stETH) are followed when all of these hold:
   every DELEGATECALL the analysis finds on the dispatcher path takes its
   target from one whole storage slot (not shifted or masked out of a
   packed slot) or from the return of the aragonOS kernel's
   `getApp(APP_BASES_NAMESPACE, sload(appIdSlot))`; at least one of them
   forwards the caller's calldata (input from calldata with no constant
   selector, length `calldatasize`); no dispatcher-path CALL forwards the
   calldata to the same slot (Synthetix's CALL-or-DELEGATECALL proxy);
   every CALL/CALLCODE/DELEGATECALL/STATICCALL opcode in the executable
   code was reached by the analysis; and the own functions that do not
   check msg.sender against storage call only the contract itself or the
   proxy's slots and write only the implementation slot, slots compared
   with msg.sender (admin, pending admin) or the EIP-1967 admin slot.
   `target.proxy_slot.source` is then `storage` or `aragon_kernel` and
   `resolved` gives the reads used. A shape that fails the last two checks
   is reported with `followed: false`, the reason and `force_with:
   --follow-proxy`; one that fails the others (a library call with a
   constant selector, a packed slot, CALL-forwarding) is not treated as a
   proxy and a note says why. When `dispatcher` says
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
