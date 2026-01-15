use super::types::{DryRunFormat, SimulationType, TenderlyArgs};
use super::utils::{
    build_calldata, create_tenderly_client, format_request, get_tenderly_credentials, value_to_hex,
};
use crate::utils::address::resolve_label;
use tndrly::simulation::{
    AccessListEntry, BlockHeaderOverride, BundleSimulationRequest, SimulationRequest, StateOverride,
};

/// Simulate using Tenderly API
#[allow(clippy::too_many_arguments)]
pub async fn simulate_via_tenderly(
    to: &str,
    sig: &Option<String>,
    data: &Option<String>,
    args: &[String],
    from: &Option<String>,
    value: &str,
    block: &str,
    gas: Option<u64>,
    gas_price: Option<u64>,
    balance_overrides: &[String],
    storage_overrides: &[String],
    code_overrides: &[String],
    block_timestamp: Option<u64>,
    simulation_type: SimulationType,
    save: bool,
    tenderly_args: &TenderlyArgs,
    dry_run: Option<DryRunFormat>,
    show_secrets: bool,
    quiet: bool,
    network_id: &Option<String>,
    transaction_index: Option<u64>,
    estimate_gas: bool,
    generate_access_list: bool,
    access_list: &Option<String>,
    l1_block_number: Option<u64>,
    l1_timestamp: Option<u64>,
    l1_message_sender: &Option<String>,
    deposit_tx: bool,
    system_tx: bool,
    block_gas_limit: Option<u64>,
    block_coinbase: &Option<String>,
    block_difficulty: Option<u64>,
    block_base_fee: Option<u64>,
) -> anyhow::Result<()> {
    // Resolve target address
    let resolved_to = resolve_label(to);
    let calldata = build_calldata(sig, data, args)?;
    let from_addr = from
        .clone()
        .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());

    // Parse block number
    let block_number = if block == "latest" {
        None
    } else {
        Some(block.parse::<u64>()?)
    };

    let value_wei = value_to_hex(value)?;

    // Build tndrly SimulationRequest using the builder pattern
    let mut request = SimulationRequest::new(&from_addr, &resolved_to, &calldata)
        .value(&value_wei)
        .simulation_type(simulation_type.to_tndrly())
        .save(save);

    if let Some(bn) = block_number {
        request = request.block_number(bn);
    }

    if let Some(g) = gas {
        request = request.gas(g);
    }

    if let Some(gp) = gas_price {
        request = request.gas_price(gp);
    }

    // Apply state overrides
    for override_str in balance_overrides {
        let parts: Vec<&str> = override_str.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid balance override format: {}. Use address=wei",
                override_str
            ));
        }
        request = request.override_balance(parts[0], parts[1]);
    }

    for override_str in storage_overrides {
        let parts: Vec<&str> = override_str.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid storage override format: {}. Use address:slot=value",
                override_str
            ));
        }
        let addr_slot: Vec<&str> = parts[0].splitn(2, ':').collect();
        if addr_slot.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid storage override format: {}. Use address:slot=value",
                override_str
            ));
        }
        request = request.override_storage(addr_slot[0], addr_slot[1], parts[1]);
    }

    for override_str in code_overrides {
        let parts: Vec<&str> = override_str.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid code override format: {}. Use address=bytecode",
                override_str
            ));
        }
        request = request.override_code(parts[0], parts[1]);
    }

    // Build block header overrides if any are specified
    let has_block_header_overrides = block_timestamp.is_some()
        || block_gas_limit.is_some()
        || block_coinbase.is_some()
        || block_difficulty.is_some()
        || block_base_fee.is_some();

    if has_block_header_overrides {
        let mut header = BlockHeaderOverride::default();

        if let Some(ts) = block_timestamp {
            header.timestamp = Some(format!("0x{:x}", ts));
        }
        if let Some(gas_limit) = block_gas_limit {
            header.gas_limit = Some(format!("0x{:x}", gas_limit));
        }
        if let Some(ref coinbase) = block_coinbase {
            header.miner = Some(coinbase.clone());
        }
        if let Some(difficulty) = block_difficulty {
            header.difficulty = Some(format!("0x{:x}", difficulty));
        }
        if let Some(base_fee) = block_base_fee {
            header.base_fee_per_gas = Some(format!("0x{:x}", base_fee));
        }

        request.block_header = Some(header);
    }

    // Apply network ID if specified
    if let Some(nid) = network_id {
        request = request.network_id(nid);
    }

    // Apply new Tenderly API parameters (tndrly 0.2+)
    if let Some(ti) = transaction_index {
        request = request.transaction_index(ti);
    }

    if estimate_gas {
        request = request.estimate_gas(true);
    }

    if generate_access_list {
        request = request.generate_access_list(true);
    }

    // Parse access list if provided (JSON format)
    if let Some(al_str) = access_list {
        let al_json = if let Some(path) = al_str.strip_prefix('@') {
            // Load from file
            std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("Failed to read access list file {}: {}", path, e))?
        } else {
            al_str.clone()
        };
        let entries: Vec<AccessListEntry> = serde_json::from_str(&al_json)
            .map_err(|e| anyhow::anyhow!("Invalid access list JSON: {}", e))?;
        request = request.access_list(entries);
    }

    // L2/Optimism parameters
    if let Some(l1_bn) = l1_block_number {
        request = request.l1_block_number(l1_bn);
    }

    if let Some(l1_ts) = l1_timestamp {
        request = request.l1_timestamp(l1_ts);
    }

    if let Some(l1_sender) = l1_message_sender {
        request = request.l1_message_sender(l1_sender);
    }

    if deposit_tx {
        request = request.deposit_tx(true);
    }

    if system_tx {
        request = request.system_tx(true);
    }

    // Handle dry-run mode - output request without executing
    if let Some(format) = dry_run {
        let (api_key, account, project) = get_tenderly_credentials(tenderly_args)?;
        let url = format!(
            "https://api.tenderly.co/api/v1/account/{}/project/{}/simulate",
            account, project
        );
        let json_request = serde_json::to_value(&request)?;
        let headers = vec![
            ("X-Access-Key", api_key.as_str()),
            ("Content-Type", "application/json"),
        ];
        let output = format_request(&url, "POST", &headers, &json_request, format, show_secrets);
        println!("{}", output);
        return Ok(());
    }

    let has_state_overrides = !balance_overrides.is_empty()
        || !storage_overrides.is_empty()
        || !code_overrides.is_empty();

    if !quiet {
        eprintln!("Simulating via Tenderly API...");
        if save {
            eprintln!("  Saving simulation to Tenderly");
        }
        if has_state_overrides {
            let count = balance_overrides.len() + storage_overrides.len() + code_overrides.len();
            eprintln!("  State overrides: {} addresses", count);
        }
    }

    // Create tndrly client and execute simulation
    let client = create_tenderly_client(tenderly_args)?;
    let result = client
        .simulation()
        .simulate(&request)
        .await
        .map_err(|e| anyhow::anyhow!("Tenderly API error: {}", e))?;

    // If saved, show the simulation ID prominently
    if save {
        eprintln!("Simulation ID: {}", result.simulation.id);
    }

    // Display generated access list prominently if requested
    if generate_access_list {
        if let Some(ref access_list) = result.generated_access_list {
            if !access_list.is_empty() {
                eprintln!("\n=== Generated Access List ===");
                for entry in access_list {
                    eprintln!("Address: {}", entry.address);
                    if !entry.storage_keys.is_empty() {
                        for key in &entry.storage_keys {
                            eprintln!("  Storage: {}", key);
                        }
                    }
                }
                eprintln!();
            }
        }
    }

    // Pretty print the result
    let json_result = serde_json::to_value(&result)?;
    println!("{}", serde_json::to_string_pretty(&json_result)?);

    Ok(())
}

/// Trace existing tx via Tenderly
pub async fn trace_tx_via_tenderly(
    hash: &str,
    tenderly_args: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    if !quiet {
        eprintln!("Fetching trace from Tenderly API...");
    }

    let client = create_tenderly_client(tenderly_args)?;
    let result = client
        .simulation()
        .trace(hash)
        .await
        .map_err(|e| anyhow::anyhow!("Tenderly API error: {}", e))?;

    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

/// Simulate a bundle of transactions via Tenderly API
#[allow(clippy::too_many_arguments)]
pub async fn simulate_bundle_tenderly(
    txs_json: &str,
    block: &str,
    balance_overrides: &[String],
    storage_overrides: &[String],
    code_overrides: &[String],
    save: bool,
    tenderly_args: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    // Parse block number
    let block_number = if block == "latest" {
        None
    } else {
        Some(block.parse::<u64>()?)
    };

    // Load transactions from JSON string or file
    let txs_data = if let Some(path) = txs_json.strip_prefix('@') {
        // Load from file
        std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read bundle file {}: {}", path, e))?
    } else {
        txs_json.to_string()
    };

    // Parse JSON
    let txs_val: serde_json::Value = serde_json::from_str(&txs_data)
        .map_err(|e| anyhow::anyhow!("Invalid bundle JSON: {}", e))?;

    // Manually construct SimulationRequest objects
    let mut sims = Vec::new();
    if let Some(arr) = txs_val.as_array() {
        for val in arr {
            let from = val["from"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing 'from' in bundle tx"))?;
            let to = val["to"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing 'to' in bundle tx"))?;
            // Accept either input or data
            let input = val["input"]
                .as_str()
                .or(val["data"].as_str())
                .unwrap_or("0x");

            let mut req = SimulationRequest::new(from, to, input);
            if let Some(v) = val["value"].as_str() {
                req = req.value(v);
            }
            if let Some(g) = val["gas"].as_u64() {
                req = req.gas(g);
            }

            // Apply bundle-wide block number if set, or tx-specific if not
            if let Some(bn) = block_number {
                req = req.block_number(bn);
            } else if let Some(bn) = val["blockNumber"].as_u64() {
                req = req.block_number(bn);
            }

            req.save = save;
            sims.push(req);
        }
    } else {
        return Err(anyhow::anyhow!(
            "Bundle must be a JSON array of transactions"
        ));
    }

    let mut request = BundleSimulationRequest::new(sims);

    // Apply state overrides
    if !balance_overrides.is_empty() || !storage_overrides.is_empty() || !code_overrides.is_empty()
    {
        let mut state_objects = std::collections::HashMap::new();

        for o in balance_overrides {
            let parts: Vec<&str> = o.splitn(2, '=').collect();
            if parts.len() == 2 {
                let address = parts[0].to_lowercase();
                let balance = parts[1].to_string();
                let entry = state_objects
                    .entry(address)
                    .or_insert_with(StateOverride::default);
                entry.balance = Some(balance);
            }
        }

        for o in storage_overrides {
            let parts: Vec<&str> = o.splitn(2, '=').collect();
            if parts.len() == 2 {
                let addr_slot: Vec<&str> = parts[0].splitn(2, ':').collect();
                if addr_slot.len() == 2 {
                    let address = addr_slot[0].to_lowercase();
                    let slot = addr_slot[1].to_string();
                    let value = parts[1].to_string();
                    let entry = state_objects
                        .entry(address)
                        .or_insert_with(StateOverride::default);
                    let storage = entry
                        .storage
                        .get_or_insert_with(std::collections::HashMap::new);
                    storage.insert(slot, value);
                }
            }
        }

        for o in code_overrides {
            let parts: Vec<&str> = o.splitn(2, '=').collect();
            if parts.len() == 2 {
                let address = parts[0].to_lowercase();
                let code = parts[1].to_string();
                let entry = state_objects
                    .entry(address)
                    .or_insert_with(StateOverride::default);
                entry.code = Some(code);
            }
        }

        if !state_objects.is_empty() {
            request.state_objects = Some(state_objects);
        }
    }

    if !quiet {
        eprintln!(
            "Simulating bundle of {} transactions via Tenderly...",
            request.simulations.len()
        );
    }

    let client = create_tenderly_client(tenderly_args)?;
    let result = client
        .simulation()
        .simulate_bundle(&request)
        .await
        .map_err(|e| anyhow::anyhow!("Tenderly API error: {}", e))?;

    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

pub async fn list_simulations_tenderly(
    limit: u32,
    page: u32,
    tenderly_args: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    if !quiet {
        eprintln!("Listing Tenderly simulations...");
    }
    let client = create_tenderly_client(tenderly_args)?;
    let result = client.simulation().list(limit, page).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn get_simulation_tenderly(
    id: &str,
    tenderly_args: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    if !quiet {
        eprintln!("Fetching Tenderly simulation {}...", id);
    }
    let client = create_tenderly_client(tenderly_args)?;
    let result = client.simulation().get(id).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn get_simulation_info_tenderly(
    id: &str,
    tenderly_args: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    if !quiet {
        eprintln!("Fetching info for Tenderly simulation {}...", id);
    }
    let client = create_tenderly_client(tenderly_args)?;
    let result = client.simulation().info(id).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn share_simulation_tenderly(
    id: &str,
    tenderly_args: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    if !quiet {
        eprintln!("Sharing Tenderly simulation {}...", id);
    }
    let client = create_tenderly_client(tenderly_args)?;
    let result = client.simulation().share(id).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn unshare_simulation_tenderly(
    id: &str,
    tenderly_args: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    if !quiet {
        eprintln!("Unsharing Tenderly simulation {}...", id);
    }
    let client = create_tenderly_client(tenderly_args)?;
    client.simulation().unshare(id).await?;
    println!("Simulation {} is now private.", id);
    Ok(())
}
