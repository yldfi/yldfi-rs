//! Comprehensive test of ALL API endpoints
//!
//! Run with: TENDERLY_ACCESS_KEY=... TENDERLY_ACCOUNT=... TENDERLY_PROJECT=... cargo run --example test_all_endpoints

use tndrly::alerts::AlertHistoryQuery;
use tndrly::contracts::ListContractsQuery;
use tndrly::simulation::{BundleSimulationRequest, SimulationRequest};
use tndrly::vnets::{ListVNetTransactionsQuery, UpdateVNetRequest};
use tndrly::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    println!("=== Comprehensive API Endpoint Test ===\n");

    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    // ========== NETWORKS API (8 methods) ==========
    println!("--- Networks API ---");

    // 1. supported()
    print!("  networks.supported()... ");
    match client.networks().supported().await {
        Ok(n) => {
            println!("✓ ({} networks)", n.len());
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 2. get()
    print!("  networks.get()... ");
    match client.networks().get("1").await {
        Ok(n) => {
            println!("✓ ({:?})", n.map(|x| x.network_name));
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 3. get_by_chain_id()
    print!("  networks.get_by_chain_id()... ");
    match client.networks().get_by_chain_id(1).await {
        Ok(n) => {
            println!("✓ ({:?})", n.map(|x| x.network_name));
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 4. get_by_slug()
    print!("  networks.get_by_slug()... ");
    match client.networks().get_by_slug("mainnet").await {
        Ok(n) => {
            println!("✓ ({:?})", n.map(|x| x.network_name));
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 5. mainnets()
    print!("  networks.mainnets()... ");
    match client.networks().mainnets().await {
        Ok(n) => {
            println!("✓ ({} networks)", n.len());
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 6. testnets()
    print!("  networks.testnets()... ");
    match client.networks().testnets().await {
        Ok(n) => {
            println!("✓ ({} networks)", n.len());
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 7. with_simulation_support()
    print!("  networks.with_simulation_support()... ");
    match client.networks().with_simulation_support().await {
        Ok(n) => {
            println!("✓ ({} networks)", n.len());
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 8. with_vnet_support()
    print!("  networks.with_vnet_support()... ");
    match client.networks().with_vnet_support().await {
        Ok(n) => {
            println!("✓ ({} networks)", n.len());
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // ========== DELIVERY CHANNELS API (3 methods) ==========
    println!("\n--- Delivery Channels API ---");

    // 1. list_account()
    print!("  delivery_channels.list_account()... ");
    match client.delivery_channels().list_account().await {
        Ok(r) => {
            println!("✓ ({} channels)", r.delivery_channels.len());
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 2. list_project()
    print!("  delivery_channels.list_project()... ");
    match client.delivery_channels().list_project().await {
        Ok(r) => {
            println!("✓ ({} channels)", r.delivery_channels.len());
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 3. list_all()
    print!("  delivery_channels.list_all()... ");
    match client.delivery_channels().list_all().await {
        Ok(c) => {
            println!("✓ ({} channels)", c.len());
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // ========== SIMULATION API (9 methods) ==========
    println!("\n--- Simulation API ---");

    // 1. list()
    print!("  simulation.list()... ");
    let sim_list = match client.simulation().list(1, 10).await {
        Ok(r) => {
            println!("✓ ({} simulations)", r.simulations.len());
            passed += 1;
            Some(r)
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
            None
        }
    };

    // 2. get() - need a simulation ID
    print!("  simulation.get()... ");
    if let Some(ref list) = sim_list {
        if let Some(sim) = list.simulations.first() {
            match client.simulation().get(&sim.id).await {
                Ok(s) => {
                    println!("✓ (id={})", s.simulation.id);
                    passed += 1;
                }
                Err(e) => {
                    println!("✗ {}", e);
                    failed += 1;
                }
            }
        } else {
            println!("SKIP (no simulations)");
            skipped += 1;
        }
    } else {
        println!("SKIP (list failed)");
        skipped += 1;
    }

    // 3. get_full()
    print!("  simulation.get_full()... ");
    if let Some(ref list) = sim_list {
        if let Some(sim) = list.simulations.first() {
            match client.simulation().get_full(&sim.id).await {
                Ok(s) => {
                    println!("✓ (id={})", s.simulation.id);
                    passed += 1;
                }
                Err(e) => {
                    println!("✗ {}", e);
                    failed += 1;
                }
            }
        } else {
            println!("SKIP (no simulations)");
            skipped += 1;
        }
    } else {
        println!("SKIP (list failed)");
        skipped += 1;
    }

    // 4. info()
    print!("  simulation.info()... ");
    if let Some(ref list) = sim_list {
        if let Some(sim) = list.simulations.first() {
            match client.simulation().info(&sim.id).await {
                Ok(v) => {
                    println!("✓ ({} keys)", v.as_object().map(|o| o.len()).unwrap_or(0));
                    passed += 1;
                }
                Err(e) => {
                    println!("✗ {}", e);
                    failed += 1;
                }
            }
        } else {
            println!("SKIP (no simulations)");
            skipped += 1;
        }
    } else {
        println!("SKIP (list failed)");
        skipped += 1;
    }

    // 5. simulate()
    print!("  simulation.simulate()... ");
    let sim_req = SimulationRequest::new(
        "0x0000000000000000000000000000000000000001",
        "0x0000000000000000000000000000000000000002",
        "0x",
    )
    .network_id("1");
    match client.simulation().simulate(&sim_req).await {
        Ok(r) => {
            println!("✓ (status={})", r.simulation.status);
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 6. simulate_bundle()
    print!("  simulation.simulate_bundle()... ");
    let bundle_req = BundleSimulationRequest::new(vec![sim_req.clone()]);
    match client.simulation().simulate_bundle(&bundle_req).await {
        Ok(r) => {
            println!("✓ ({} results)", r.simulation_results.len());
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 7-9. share/unshare/trace - skip as they modify state or need specific sims
    println!("  simulation.share()... SKIP (modifies state)");
    skipped += 1;
    println!("  simulation.unshare()... SKIP (modifies state)");
    skipped += 1;
    println!("  simulation.trace()... SKIP (needs real tx hash)");
    skipped += 1;

    // ========== CONTRACTS API (12 methods) ==========
    println!("\n--- Contracts API ---");

    // 1. list()
    print!("  contracts.list()... ");
    let contracts = match client.contracts().list(None).await {
        Ok(c) => {
            println!("✓ ({} contracts)", c.len());
            passed += 1;
            Some(c)
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
            None
        }
    };

    // 2. list() with query
    print!("  contracts.list(query)... ");
    let query = ListContractsQuery::new().network_id("1");
    match client.contracts().list(Some(query)).await {
        Ok(c) => {
            println!("✓ ({} contracts)", c.len());
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 3. get() - need an actual contract (not a wallet)
    print!("  contracts.get()... ");
    if let Some(ref c) = contracts {
        // Filter for actual contracts - the get() endpoint doesn't work for wallets
        if let Some(contract) = c.iter().find(|c| c.is_contract()) {
            if let (Some(net), Some(addr)) = (contract.network_id(), contract.address()) {
                match client.contracts().get(net, addr).await {
                    Ok(c) => {
                        println!("✓ ({:?})", c.address());
                        passed += 1;
                    }
                    Err(e) => {
                        println!("✗ {}", e);
                        failed += 1;
                    }
                }
            } else {
                println!("SKIP (missing network_id/address)");
                skipped += 1;
            }
        } else {
            println!("SKIP (no actual contracts, only wallets)");
            skipped += 1;
        }
    } else {
        println!("SKIP (list failed)");
        skipped += 1;
    }

    // 4. abi() - currently returns None (stub)
    print!("  contracts.abi()... ");
    if let Some(ref c) = contracts {
        // Use actual contracts for abi() as well
        if let Some(contract) = c.iter().find(|c| c.is_contract()) {
            if let (Some(net), Some(addr)) = (contract.network_id(), contract.address()) {
                match client.contracts().abi(net, addr).await {
                    Ok(a) => {
                        println!("✓ ({:?})", a.is_some());
                        passed += 1;
                    }
                    Err(e) => {
                        println!("✗ {}", e);
                        failed += 1;
                    }
                }
            } else {
                println!("SKIP (missing network_id/address)");
                skipped += 1;
            }
        } else {
            println!("SKIP (no actual contracts)");
            skipped += 1;
        }
    } else {
        println!("SKIP (list failed)");
        skipped += 1;
    }

    // 5-12. Skip write operations
    println!("  contracts.add()... SKIP (modifies state)");
    skipped += 1;
    println!("  contracts.update()... SKIP (modifies state)");
    skipped += 1;
    println!("  contracts.delete()... SKIP (modifies state)");
    skipped += 1;
    println!("  contracts.verify()... SKIP (needs source code)");
    skipped += 1;
    println!("  contracts.encode_state()... SKIP (needs specific input)");
    skipped += 1;
    println!("  contracts.add_tag()... SKIP (modifies state)");
    skipped += 1;
    println!("  contracts.remove_tag()... SKIP (modifies state)");
    skipped += 1;
    println!("  contracts.rename()... SKIP (modifies state)");
    skipped += 1;
    println!("  contracts.bulk_tag()... SKIP (modifies state)");
    skipped += 1;
    println!("  contracts.delete_tag()... SKIP (modifies state)");
    skipped += 1;

    // ========== WALLETS API (3 methods) ==========
    println!("\n--- Wallets API ---");

    // 1. list()
    print!("  wallets.list()... ");
    let wallets = match client.wallets().list().await {
        Ok(w) => {
            println!("✓ ({} wallets)", w.len());
            passed += 1;
            Some(w)
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
            None
        }
    };

    // 2. get()
    print!("  wallets.get()... ");
    if let Some(ref w) = wallets {
        if let Some(wallet) = w.first() {
            if let (Some(addr), Some(net)) = (wallet.address(), wallet.network_id()) {
                match client.wallets().get(addr, net).await {
                    Ok(w) => {
                        println!("✓ ({:?})", w.address());
                        passed += 1;
                    }
                    Err(e) => {
                        println!("✗ {}", e);
                        failed += 1;
                    }
                }
            } else {
                println!("SKIP (missing address/network_id)");
                skipped += 1;
            }
        } else {
            println!("SKIP (no wallets)");
            skipped += 1;
        }
    } else {
        println!("SKIP (list failed)");
        skipped += 1;
    }

    // 3. add()
    println!("  wallets.add()... SKIP (modifies state)");
    skipped += 1;

    // ========== ALERTS API (15 methods) ==========
    println!("\n--- Alerts API ---");

    // 1. list()
    print!("  alerts.list()... ");
    let alerts = match client.alerts().list().await {
        Ok(r) => {
            println!("✓ ({} alerts)", r.alerts.len());
            passed += 1;
            Some(r)
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
            None
        }
    };

    // 2. get()
    print!("  alerts.get()... ");
    if let Some(ref a) = alerts {
        if let Some(alert) = a.alerts.first() {
            match client.alerts().get(&alert.id).await {
                Ok(a) => {
                    println!("✓ (id={})", a.id);
                    passed += 1;
                }
                Err(e) => {
                    println!("✗ {}", e);
                    failed += 1;
                }
            }
        } else {
            println!("SKIP (no alerts)");
            skipped += 1;
        }
    } else {
        println!("SKIP (list failed)");
        skipped += 1;
    }

    // 3. history()
    print!("  alerts.history()... ");
    match client.alerts().history(None).await {
        Ok(h) => {
            println!("✓ ({} entries)", h.alert_history.len());
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 4. history() with query
    print!("  alerts.history(query)... ");
    let query = AlertHistoryQuery::new().per_page(5);
    match client.alerts().history(Some(query)).await {
        Ok(h) => {
            println!("✓ ({} entries)", h.alert_history.len());
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 5. list_webhooks()
    print!("  alerts.list_webhooks()... ");
    match client.alerts().list_webhooks().await {
        Ok(r) => {
            println!("✓ ({} webhooks)", r.webhooks.len());
            passed += 1;
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
        }
    }

    // 6-15. Skip write operations
    println!("  alerts.create()... SKIP (modifies state)");
    skipped += 1;
    println!("  alerts.update()... SKIP (modifies state)");
    skipped += 1;
    println!("  alerts.delete()... SKIP (modifies state)");
    skipped += 1;
    println!("  alerts.enable()... SKIP (modifies state)");
    skipped += 1;
    println!("  alerts.disable()... SKIP (modifies state)");
    skipped += 1;
    println!("  alerts.add_destination()... SKIP (modifies state)");
    skipped += 1;
    println!("  alerts.remove_destination()... SKIP (modifies state)");
    skipped += 1;
    println!("  alerts.create_webhook()... SKIP (modifies state)");
    skipped += 1;
    println!("  alerts.get_webhook()... SKIP (needs webhook id)");
    skipped += 1;
    println!("  alerts.delete_webhook()... SKIP (modifies state)");
    skipped += 1;
    println!("  alerts.test_webhook()... SKIP (modifies state)");
    skipped += 1;
    println!("  alerts.test_alert()... SKIP (needs specific input)");
    skipped += 1;

    // ========== ACTIONS API (16 methods) ==========
    println!("\n--- Actions API ---");

    // 1. list()
    print!("  actions.list()... ");
    let actions = match client.actions().list().await {
        Ok(r) => {
            println!("✓ ({} actions)", r.actions.len());
            passed += 1;
            Some(r)
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
            None
        }
    };

    // 2. get()
    print!("  actions.get()... ");
    if let Some(ref a) = actions {
        if let Some(action) = a.actions.first() {
            match client.actions().get(&action.id).await {
                Ok(a) => {
                    println!("✓ (id={})", a.id);
                    passed += 1;
                }
                Err(e) => {
                    println!("✗ {}", e);
                    failed += 1;
                }
            }
        } else {
            println!("SKIP (no actions)");
            skipped += 1;
        }
    } else {
        println!("SKIP (list failed)");
        skipped += 1;
    }

    // 3. source()
    print!("  actions.source()... ");
    if let Some(ref a) = actions {
        if let Some(action) = a.actions.first() {
            match client.actions().source(&action.id).await {
                Ok(s) => {
                    println!("✓ ({} bytes)", s.len());
                    passed += 1;
                }
                Err(e) => {
                    println!("✗ {}", e);
                    failed += 1;
                }
            }
        } else {
            println!("SKIP (no actions)");
            skipped += 1;
        }
    } else {
        println!("SKIP (list failed)");
        skipped += 1;
    }

    // 4. logs()
    print!("  actions.logs()... ");
    if let Some(ref a) = actions {
        if let Some(action) = a.actions.first() {
            match client.actions().logs(&action.id).await {
                Ok(l) => {
                    println!("✓ ({} logs)", l.logs.len());
                    passed += 1;
                }
                Err(e) => {
                    println!("✗ {}", e);
                    failed += 1;
                }
            }
        } else {
            println!("SKIP (no actions)");
            skipped += 1;
        }
    } else {
        println!("SKIP (list failed)");
        skipped += 1;
    }

    // 5. calls()
    print!("  actions.calls()... ");
    if let Some(ref a) = actions {
        if let Some(action) = a.actions.first() {
            match client.actions().calls(&action.id, None).await {
                Ok(c) => {
                    println!("✓ ({} executions)", c.executions.len());
                    passed += 1;
                }
                Err(e) => {
                    println!("✗ {}", e);
                    failed += 1;
                }
            }
        } else {
            println!("SKIP (no actions)");
            skipped += 1;
        }
    } else {
        println!("SKIP (list failed)");
        skipped += 1;
    }

    // 6-16. Skip write operations and ops needing specific IDs
    println!("  actions.create()... SKIP (modifies state)");
    skipped += 1;
    println!("  actions.update()... SKIP (modifies state)");
    skipped += 1;
    println!("  actions.delete()... SKIP (modifies state)");
    skipped += 1;
    println!("  actions.enable()... SKIP (modifies state)");
    skipped += 1;
    println!("  actions.disable()... SKIP (modifies state)");
    skipped += 1;
    println!("  actions.invoke()... SKIP (modifies state)");
    skipped += 1;
    println!("  actions.get_log()... SKIP (needs log id)");
    skipped += 1;
    println!("  actions.update_source()... SKIP (modifies state)");
    skipped += 1;
    println!("  actions.stop()... SKIP (modifies state)");
    skipped += 1;
    println!("  actions.resume()... SKIP (modifies state)");
    skipped += 1;
    println!("  actions.stop_many()... SKIP (modifies state)");
    skipped += 1;
    println!("  actions.resume_many()... SKIP (modifies state)");
    skipped += 1;
    println!("  actions.get_call()... SKIP (needs call id)");
    skipped += 1;

    // ========== VNETS API (14 methods) ==========
    println!("\n--- VNets API ---");

    // 1. list()
    print!("  vnets.list()... ");
    let vnets = match client.vnets().list(None).await {
        Ok(v) => {
            println!("✓ ({} vnets)", v.len());
            passed += 1;
            Some(v)
        }
        Err(e) => {
            println!("✗ {}", e);
            failed += 1;
            None
        }
    };

    let vnet_id = vnets.as_ref().and_then(|v| v.first()).map(|v| v.id.clone());

    // 2. get()
    print!("  vnets.get()... ");
    if let Some(ref id) = vnet_id {
        match client.vnets().get(id).await {
            Ok(v) => {
                println!("✓ ({})", v.display_name);
                passed += 1;
            }
            Err(e) => {
                println!("✗ {}", e);
                failed += 1;
            }
        }
    } else {
        println!("SKIP (no vnets)");
        skipped += 1;
    }

    // 3. transactions()
    print!("  vnets.transactions()... ");
    if let Some(ref id) = vnet_id {
        match client.vnets().transactions(id, None).await {
            Ok(t) => {
                println!("✓ ({} txs)", t.len());
                passed += 1;
            }
            Err(e) => {
                println!("✗ {}", e);
                failed += 1;
            }
        }
    } else {
        println!("SKIP (no vnets)");
        skipped += 1;
    }

    // 4. transactions() with query
    print!("  vnets.transactions(query)... ");
    if let Some(ref id) = vnet_id {
        let query = ListVNetTransactionsQuery::new().success();
        match client.vnets().transactions(id, Some(query)).await {
            Ok(t) => {
                println!("✓ ({} txs)", t.len());
                passed += 1;
            }
            Err(e) => {
                println!("✗ {}", e);
                failed += 1;
            }
        }
    } else {
        println!("SKIP (no vnets)");
        skipped += 1;
    }

    // 5. get_transaction()
    print!("  vnets.get_transaction()... ");
    if let Some(ref id) = vnet_id {
        let txs = client.vnets().transactions(id, None).await.ok();
        if let Some(tx) = txs
            .as_ref()
            .and_then(|t| t.iter().find(|tx| tx.tx_hash.is_some()))
        {
            let hash = tx.tx_hash.as_ref().unwrap();
            match client.vnets().get_transaction(id, hash).await {
                Ok(t) => {
                    println!("✓ ({:?})", t.tx_hash);
                    passed += 1;
                }
                Err(e) => {
                    println!("✗ {}", e);
                    failed += 1;
                }
            }
        } else {
            println!("SKIP (no txs with hash)");
            skipped += 1;
        }
    } else {
        println!("SKIP (no vnets)");
        skipped += 1;
    }

    // 6. rpc_urls()
    print!("  vnets.rpc_urls()... ");
    if let Some(ref id) = vnet_id {
        match client.vnets().rpc_urls(id).await {
            Ok(r) => {
                println!("✓ ({} endpoints)", r.endpoints.len());
                passed += 1;
            }
            Err(e) => {
                println!("✗ {}", e);
                failed += 1;
            }
        }
    } else {
        println!("SKIP (no vnets)");
        skipped += 1;
    }

    // 7. admin_rpc() + get_latest()
    print!("  vnets.admin_rpc().get_latest()... ");
    if let Some(ref id) = vnet_id {
        match client.vnets().admin_rpc(id).await {
            Ok(admin) => match admin.get_latest().await {
                Ok(b) => {
                    println!("✓ (block={:?})", b.block_number);
                    passed += 1;
                }
                Err(e) => {
                    println!("✗ admin_rpc OK but get_latest: {}", e);
                    failed += 1;
                }
            },
            Err(e) => {
                println!("✗ {}", e);
                failed += 1;
            }
        }
    } else {
        println!("SKIP (no vnets)");
        skipped += 1;
    }

    // 8. update() - with no-op update
    print!("  vnets.update()... ");
    if let Some(ref id) = vnet_id {
        let vnet = client.vnets().get(id).await.ok();
        if let Some(v) = vnet {
            let req = UpdateVNetRequest::new().display_name(&v.display_name);
            match client.vnets().update(id, &req).await {
                Ok(u) => {
                    println!("✓ ({})", u.display_name);
                    passed += 1;
                }
                Err(e) => {
                    println!("✗ {}", e);
                    failed += 1;
                }
            }
        } else {
            println!("SKIP (couldn't get vnet)");
            skipped += 1;
        }
    } else {
        println!("SKIP (no vnets)");
        skipped += 1;
    }

    // 9-14. Skip destructive/create operations
    println!("  vnets.create()... SKIP (modifies state)");
    skipped += 1;
    println!("  vnets.delete()... SKIP (modifies state)");
    skipped += 1;
    println!("  vnets.delete_many()... SKIP (modifies state)");
    skipped += 1;
    println!("  vnets.fork()... SKIP (modifies state)");
    skipped += 1;
    println!("  vnets.simulate()... SKIP (needs specific format)");
    skipped += 1;
    println!("  vnets.send_transaction()... SKIP (modifies state)");
    skipped += 1;

    // ========== SUMMARY ==========
    println!("\n=== Summary ===");
    println!("  Passed:  {}", passed);
    println!("  Failed:  {}", failed);
    println!("  Skipped: {} (write ops / need specific data)", skipped);
    println!("  Total:   {}", passed + failed + skipped);

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
