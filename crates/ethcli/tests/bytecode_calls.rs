//! Facts `contract calls` must recover from real mainnet runtime bytecode
//! (fixtures in tests/fixtures/bytecode, fetched once with `ethcli rpc code`).

use ethcli::bytecode::{analyze_calls, CallsBudget, CallsReport};

fn fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/tests/fixtures/bytecode/{name}.hex",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).expect("fixture");
    hex::decode(text.trim().trim_start_matches("0x")).expect("hex")
}

/// Step budget only, no wall clock, so results do not depend on machine speed
fn budget() -> CallsBudget {
    CallsBudget {
        max_time: std::time::Duration::from_secs(3600),
        ..CallsBudget::default()
    }
}

fn report(name: &str) -> CallsReport {
    analyze_calls(&fixture(name), budget())
}

fn guards(r: &CallsReport) -> Vec<(String, ethcli::bytecode::Guard)> {
    r.functions
        .iter()
        .flat_map(|f| {
            f.guards
                .iter()
                .map(move |g| (f.function.clone(), g.clone()))
        })
        .collect()
}

fn has_value(ps: &[ethcli::bytecode::Provenance], kind: &str, value: &str) -> bool {
    ps.iter()
        .any(|p| p.kind == kind && p.value.as_deref() == Some(value))
}

/// WETH9 withdraw(uint256): `msg.sender.transfer(wad)`
#[test]
fn weth9_withdraw_pays_msg_sender() {
    let r = report("weth9");
    let site = r
        .call_sites
        .iter()
        .find(|c| c.functions.contains(&"0x2e1a7d4d".to_string()))
        .expect("withdraw call");
    assert_eq!(site.kind, "CALL");
    assert_eq!(site.target[0].kind, "msg_sender");
    assert_eq!(site.value[0].kind, "calldata");
    assert_eq!(site.value[0].offset.as_deref(), Some("0x4"));
    assert!(r.opcode_sites.iter().all(|s| s.opcode != "ORIGIN"));
    // balanceOf[msg.sender] >= wad is not an access-control guard
    assert!(!guards(&r).iter().any(|(_, g)| g.check == "mapping_lookup"));
    assert_eq!(
        r.coverage.cfg_blocks_analysed,
        r.coverage.cfg_reachable_blocks
    );
}

/// USDC FiatTokenV2_2: onlyOwner and the other roles compare msg.sender
/// with a storage slot; permit/receiveWithAuthorization DELEGATECALL the
/// SignatureChecker library at a fixed address
#[test]
fn usdc_roles_and_library_delegatecall() {
    let r = report("usdc_impl");
    // transferOwnership(address) 0xf2fde38b: owner is slot 0
    assert!(guards(&r).iter().any(|(f, g)| f == "0xf2fde38b"
        && g.subject == "msg_sender"
        && g.against.kind == "storage"
        && g.against.slot.as_deref() == Some("0x0")));
    assert!(r.call_sites.iter().any(|c| c.kind == "DELEGATECALL"
        && has_value(
            &c.target,
            "code_constant",
            "0x800c32eaa2a6c93cf4cb51794450ed77fbfbb172"
        )));
}

/// Uniswap V3 pool: noDelegateCall compares address(this) with the
/// immutable `original`; swap/mint/flash call back msg.sender
#[test]
fn univ3_no_delegate_call_and_callbacks() {
    let r = report("univ3_pool");
    let pool = "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640";
    // swap(...) 0x128acb08
    assert!(guards(&r).iter().any(|(f, g)| f == "0x128acb08"
        && g.subject == "address_this"
        && g.against.value.as_deref() == Some(pool)));
    for (function, callback) in [
        ("0x128acb08", "0xfa461e33"), // uniswapV3SwapCallback
        ("0x3c8a7d8d", "0xd3487997"), // uniswapV3MintCallback
        ("0x490e6cbc", "0xe9cbafb0"), // uniswapV3FlashCallback
    ] {
        let site = r
            .call_sites
            .iter()
            .find(|c| {
                c.functions.contains(&function.to_string())
                    && c.target.iter().any(|t| t.kind == "msg_sender")
            })
            .unwrap_or_else(|| panic!("{function}: no call to msg.sender"));
        assert!(
            has_value(&site.selector, "constant", callback),
            "{function}: callback selector {callback}, got {:?}",
            site.selector
        );
    }
    // Hard-coded tokens (immutables token0/token1)
    assert!(r.call_sites.iter().any(|c| has_value(
        &c.target,
        "constant",
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
    )));
}

/// Banana Gun router: privileged functions require msg.sender == a
/// hard-coded owner
#[test]
fn bananagun_owner_constant() {
    let r = report("bananagun_router");
    let owner = "0x37aab97476ba8dc785476611006fd5dda4eed66b";
    assert!(guards(&r)
        .iter()
        .any(|(_, g)| g.subject == "msg_sender" && g.against.value.as_deref() == Some(owner)));
}

/// Wintermute bot: hard-coded admin checks and calls to calldata targets
#[test]
fn wintermute_guards_and_calldata_targets() {
    let r = report("wintermute_bot");
    assert!(guards(&r).iter().any(|(_, g)| g.subject == "msg_sender"
        && g.against.value.as_deref() == Some("0xce466fc694c971b95bb1193e494942e22197f759")));
    assert!(r
        .call_sites
        .iter()
        .any(|c| c.target.iter().any(|t| t.kind == "calldata")));
}

/// MEV executor (unverified, calldata-indexed jump table): three ORIGIN
/// uses, the CALLER == constant guard at the entry, tx.origin checks in the
/// callback path, calldata-driven targets and hard-coded tokens. Facts
/// derived by hand from the disassembly (pcs 0..48 and 13359..13735).
#[test]
fn mev_executor_facts() {
    let r = report("mev_executor");
    let operator = "0xae2fc483527b8ef99eb5d9b44875f005ba1fae13";
    let origins: Vec<usize> = r
        .opcode_sites
        .iter()
        .filter(|s| s.opcode == "ORIGIN")
        .map(|s| s.pc)
        .collect();
    assert_eq!(origins, vec![13360, 13582, 13723]);
    assert!(r
        .opcode_sites
        .iter()
        .filter(|s| s.opcode == "ORIGIN")
        .all(|s| s.reached));
    let g = guards(&r);
    assert!(g.iter().any(|(_, g)| g.pc == 35
        && g.subject == "msg_sender"
        && g.against.value.as_deref() == Some(operator)));
    assert!(g
        .iter()
        .any(|(_, g)| g.subject == "tx_origin" && g.against.value.as_deref() == Some(operator)));
    assert!(r.jump_tables.iter().any(|t| t.pc == 48 && t.entries > 0));
    assert!(r
        .call_sites
        .iter()
        .any(|c| c.target.iter().any(|t| t.kind == "calldata")));
    for token in [
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
    ] {
        assert!(
            r.call_sites
                .iter()
                .any(|c| has_value(&c.target, "code_constant", token)),
            "{token}"
        );
    }
    // transfer selectors read back from codecopied calldata templates
    assert!(r
        .call_sites
        .iter()
        .any(|c| has_value(&c.selector, "constant", "0xa9059cbb")));
    // A calldata-chosen 16-bit jump target stays unresolved
    assert!(r.coverage.unresolved_jumps.contains(&13492));
    assert!(!r.coverage.complete);
}

/// onlyOwner-style checks compiled by solc 0.8.28 (--optimize, and
/// --via-ir --optimize, no metadata hash): the optimiser turns `==` / `!=`
/// before a JUMPI into SUB, so guards must not depend on EQ. Source:
/// `owner` in slot 0; setIf / killIf (`if (msg.sender == owner)`),
/// revertIfNot (`if (msg.sender != owner) revert()`), requireOwner.
#[test]
fn solc_sub_peephole_owner_checks_are_guards() {
    for name in ["owned_solc0828_opt", "owned_solc0828_viair"] {
        let r = report(name);
        for selector in ["0x8c99b041", "0x5eb5c00c", "0xe4fa15a7", "0x9599ac06"] {
            assert!(
                guards(&r).iter().any(|(f, g)| f == selector
                    && g.subject == "msg_sender"
                    && g.against.kind == "storage"
                    && g.against.slot.as_deref() == Some("0x0")),
                "{name} {selector}: {:?}",
                guards(&r)
                    .iter()
                    .filter(|(f, _)| f == selector)
                    .collect::<Vec<_>>()
            );
        }
    }
}

/// Banana Gun router: the UniV3 `swap` call in 0x70fef1da is reached with
/// a resolved selector only by the second pass (capped values keep an
/// identity, states over the cap are widened)
#[test]
fn bananagun_swap_recovered_by_second_pass() {
    let r = report("bananagun_router");
    let site = r
        .call_sites
        .iter()
        .find(|c| c.pc == 16124 && c.functions.iter().any(|f| f == "0x70fef1da"))
        .expect("site 16124 under 0x70fef1da");
    assert!(has_value(&site.selector, "constant", "0x128acb08"));
    assert!(site.args[0].iter().any(|p| p.kind == "address_this"));
}

#[test]
#[ignore]
fn dump() {
    for name in [
        "weth9",
        "usdc_impl",
        "univ3_pool",
        "bananagun_router",
        "wintermute_bot",
        "mev_executor",
    ] {
        let t = std::time::Instant::now();
        let r = report(name);
        println!(
            "=== {name} {:?}\n{}",
            t.elapsed(),
            serde_json::to_string(&r).unwrap()
        );
    }
}
