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
   `min_evm_version` and the pipeline guess. Set `solc_version`,
   `evm_version` and the hash type (`bytecode_hash = "ipfs"`, `"bzzr1"` or
   `"none"`) in `foundry.toml` to match. The optimizer runs and via-IR are not
   in the code; start with the optimizer on at 200 runs and the legacy
   pipeline, and change one setting at a time. If the proxy was followed, the
   output names the implementation: rebuild that.
2. **Interface and storage from facts.** `ethcli contract inspect <addr> --json`
   (functions, mutability, proxy), `ethcli contract selectors <impl> --lookup`
   (evmole types), `ethcli contract analyze <impl> --dispatcher --checks`, and
   storage reads (`ethcli rpc storage`). Selectors that have no known name
   keep their exact selector: mine a name with that selector or use a
   placeholder name whose keccak gives it, never change the selector.
3. **Optional draft.** `ethcli contract decompile <impl>`. Read the
   cross-check block first. Each warning there is something the draft does
   not show (for example `tx.origin` guards, storage writes, creation of
   contracts). "Not found by evmdec" is not a fact. Use the draft for control
   flow and constants only.
4. **Write Solidity.** Plain Solidity only. **No inline assembly**, unless the
   fingerprint and disassembly show the original used it, and then keep it to
   the smallest block and say so. Assembly can copy bytecode without
   explaining it, so a match reached through assembly proves little.
5. **Build.** `forge build --ast` (the AST lets `contract match` name
   functions and measure the inline-assembly share).
6. **Match.** `ethcli contract match <addr> --artifact out/<File>.sol/<Name>.json --json`.
   - `mismatch`: read `match.mismatch.function`, the bytes around
     `first_difference_offset` and the per-function table. Fix only the
     function named `different`, rebuild, and match again. When the size
     differs, fix size-changing differences first (missing or extra
     statements, types, checked vs unchecked arithmetic).
   - When only compiler-generated or contract-level code differs, look for
     the constant or text shown, the ABI coder version, error strings,
     optimizer runs and function order.
   - Check `selectors.in_artifact_not_deployed` and
     `deployed_not_in_artifact` before anything else.
7. **Repeat** steps 4 to 6.

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
- `partial`: "matches the deployed code ignoring metadata". Never call a
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
