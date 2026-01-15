//! Main log fetcher coordinator

use crate::abi::{AbiFetcher, DecodedLog, EventSignature, LogDecoder};
use crate::checkpoint::CheckpointManager;
use crate::config::{BlockNumber, Config};
use crate::error::{AbiError, Error, Result, RpcError};
use crate::rpc::RpcPool;
use alloy::primitives::{Address, B256};
use alloy::rpc::types::{Filter, Log};
use futures::stream::{self, StreamExt};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Parse event strings into topic hashes
///
/// Accepts either:
/// - A topic hash (0x-prefixed, 66 chars): parsed directly
/// - An event signature (e.g., "Transfer(address,address,uint256)"): computed via keccak256
fn parse_event_topics(events: &[String]) -> Result<Vec<B256>> {
    events
        .iter()
        .map(|event_str| {
            // Check if it's already a topic hash
            if event_str.starts_with("0x") && event_str.len() == 66 {
                event_str
                    .parse()
                    .map_err(|_| Error::Abi(AbiError::InvalidEventSignature(event_str.clone())))
            } else {
                // Parse as signature and compute topic
                EventSignature::parse(event_str)
                    .map(|sig| sig.topic)
                    .map_err(|_| Error::Abi(AbiError::InvalidEventSignature(event_str.clone())))
            }
        })
        .collect()
}

/// Statistics about a fetch operation
#[derive(Debug, Clone, Default)]
pub struct FetchStats {
    /// Total chunks attempted
    pub chunks_total: usize,
    /// Successfully fetched chunks
    pub chunks_succeeded: usize,
    /// Failed chunks
    pub chunks_failed: usize,
    /// Block ranges that failed (start, end, error message)
    pub failed_ranges: Vec<(u64, u64, String)>,
}

impl FetchStats {
    /// Check if all chunks succeeded
    pub fn is_complete(&self) -> bool {
        self.chunks_failed == 0
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.chunks_total == 0 {
            100.0
        } else {
            (self.chunks_succeeded as f64 / self.chunks_total as f64) * 100.0
        }
    }
}

/// Result of a fetch operation
#[derive(Debug)]
pub struct FetchResult {
    /// The fetched logs
    pub logs: FetchLogs,
    /// Statistics about the fetch operation
    pub stats: FetchStats,
}

/// The actual log data
#[derive(Debug)]
pub enum FetchLogs {
    /// Raw logs (undecoded)
    Raw(Vec<Log>),
    /// Decoded logs
    Decoded(Vec<DecodedLog>),
}

impl FetchResult {
    pub fn len(&self) -> usize {
        match &self.logs {
            FetchLogs::Raw(logs) => logs.len(),
            FetchLogs::Decoded(logs) => logs.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if the fetch was complete (no failures)
    pub fn is_complete(&self) -> bool {
        self.stats.is_complete()
    }

    /// Get failed ranges if any
    pub fn failed_ranges(&self) -> &[(u64, u64, String)] {
        &self.stats.failed_ranges
    }
}

/// Progress callback type
pub type ProgressCallback = Box<dyn Fn(FetchProgress) + Send + Sync>;

/// Fetch progress information
#[derive(Debug, Clone)]
pub struct FetchProgress {
    /// Current block being processed
    pub current_block: u64,
    /// Total blocks to process
    pub total_blocks: u64,
    /// Logs fetched so far
    pub logs_fetched: u64,
    /// Percentage complete
    pub percent: f64,
    /// Blocks per second
    pub blocks_per_second: f64,
}

/// Main log fetcher
pub struct LogFetcher {
    /// Configuration
    config: Config,
    /// RPC pool
    pool: RpcPool,
    /// Log decoder (if not raw mode)
    decoder: Option<LogDecoder>,
    /// Progress callback
    progress_callback: Option<ProgressCallback>,
    /// Resolved event signatures/topics for filtering (empty = all events)
    resolved_events: Vec<String>,
}

impl LogFetcher {
    /// Create a new log fetcher from config
    pub async fn new(mut config: Config) -> Result<Self> {
        // Create RPC pool
        let pool = RpcPool::new(config.chain, &config.rpc)?;

        // Auto-detect from_block if needed
        if config.auto_from_block && config.block_range.from_block() == 0 {
            tracing::info!("Looking up contract creation block from Etherscan...");
            let fetcher = AbiFetcher::new(config.etherscan_key.clone())?;
            match fetcher
                .get_contract_creation(config.chain, &config.contract)
                .await
            {
                Ok(creation) => {
                    tracing::info!(
                        "Contract created at block {} (tx: {})",
                        creation.block_number,
                        creation.tx_hash
                    );
                    // Update the block range
                    config.block_range = crate::config::BlockRange::Range {
                        from: creation.block_number,
                        to: config.block_range.to_block(),
                    };
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to get contract creation block: {}. Starting from block 0.",
                        e
                    );
                }
            }
        }

        // Resolve event filters (names, signatures, or topic hashes)
        let mut resolved_events = Vec::new();
        if !config.events.is_empty() {
            let fetcher = AbiFetcher::new(config.etherscan_key.clone())?;

            for event_str in &config.events {
                // Check if it's a topic hash (0x + 64 hex chars = 66 chars)
                if event_str.starts_with("0x") && event_str.len() == 66 {
                    // It's already a topic hash, use as-is
                    tracing::debug!("Using topic hash: {}", event_str);
                    resolved_events.push(event_str.clone());
                } else if event_str.contains('(') {
                    // It's a full signature
                    tracing::debug!("Using event signature: {}", event_str);
                    resolved_events.push(event_str.clone());
                } else {
                    // It's just an event name - resolve from Etherscan ABI
                    tracing::info!("Resolving event name '{}' from contract ABI...", event_str);
                    let resolved = fetcher
                        .resolve_event_name(config.chain, &config.contract, event_str)
                        .await?;
                    tracing::info!("Resolved '{}' to: {}", event_str, resolved);
                    resolved_events.push(resolved);
                }
            }
        }

        // Set up decoder if not raw mode
        let decoder = if config.raw {
            None
        } else {
            Some(Self::setup_decoder(&config, &resolved_events).await?)
        };

        Ok(Self {
            config,
            pool,
            decoder,
            progress_callback: None,
            resolved_events,
        })
    }

    /// Set up the log decoder
    async fn setup_decoder(config: &Config, resolved_events: &[String]) -> Result<LogDecoder> {
        // Filter out topic hashes (they're not signatures we can decode)
        let signatures: Vec<&str> = resolved_events
            .iter()
            .filter(|s| s.contains('(')) // Only signatures, not raw topic hashes
            .map(|s| s.as_str())
            .collect();

        // If we have resolved event signatures, use them
        if !signatures.is_empty() {
            let mut decoder = LogDecoder::new();
            for sig_str in signatures {
                let sig = EventSignature::parse(sig_str)?;
                decoder.add_signature(&sig)?;
            }
            return Ok(decoder);
        }

        // If ABI file provided, load it
        if let Some(abi_path) = &config.abi_path {
            let fetcher = AbiFetcher::new(None)?;
            let abi = fetcher.load_from_file(abi_path)?;
            return LogDecoder::from_abi(&abi);
        }

        // Try to fetch ABI from Etherscan
        let fetcher = AbiFetcher::new(config.etherscan_key.clone())?;
        let abi = fetcher
            .fetch_from_etherscan(config.chain, &config.contract)
            .await?;
        LogDecoder::from_abi(&abi)
    }

    /// Set progress callback
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(FetchProgress) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Fetch all logs
    ///
    /// **Warning:** This method collects ALL logs into memory before returning.
    /// For large block ranges or high-activity contracts, this can cause out-of-memory errors.
    /// Consider using `StreamingFetcher::fetch_streaming()` for large datasets, which
    /// processes logs in chunks and writes them incrementally to disk.
    pub async fn fetch_all(&self) -> Result<FetchResult> {
        let end_block = self.resolve_end_block().await?;
        let from_block = self.config.block_range.from_block();

        // Calculate chunks based on endpoint capabilities
        let max_range = self.pool.max_block_range();
        let chunks = Self::calculate_chunks(from_block, end_block, max_range);

        tracing::info!(
            "Fetching logs from block {} to {} ({} chunks)",
            from_block,
            end_block,
            chunks.len()
        );

        // Build base filter
        let address: Address = self
            .config
            .contract
            .parse()
            .map_err(|_| Error::from("Invalid contract address"))?;

        let mut base_filter = Filter::new().address(address);

        // Add event topics if we have specific events (works in both raw and decoded modes)
        // Multiple topics create an OR filter (matches any of the specified events)
        if !self.resolved_events.is_empty() {
            let topics = parse_event_topics(&self.resolved_events)?;
            base_filter = base_filter.event_signature(topics);
        }

        // Fetch chunks in parallel
        let concurrency = self.config.rpc.concurrency;
        let max_retries = self.config.rpc.max_retries;
        let logs_count = Arc::new(AtomicU64::new(0));
        let blocks_completed = Arc::new(AtomicU64::new(0));
        let start_time = std::time::Instant::now();

        // Note: buffer_unordered doesn't preserve order, so we carry chunk info through
        let results: Vec<(u64, u64, Result<Vec<Log>>)> = stream::iter(chunks.clone())
            .map(|(from, to)| {
                let filter = base_filter.clone().from_block(from).to_block(to);
                let pool = &self.pool;
                let logs_count = logs_count.clone();
                let blocks_completed = blocks_completed.clone();
                let callback = &self.progress_callback;
                let total_blocks = end_block - from_block + 1;

                async move {
                    let result =
                        Self::fetch_chunk_with_retry(pool, &filter, from, to, max_retries).await;

                    if let Ok(ref logs) = result {
                        let count = logs_count.fetch_add(logs.len() as u64, Ordering::Relaxed);
                        // Track actual blocks completed (chunk size), not position
                        let chunk_size = to - from + 1;
                        let blocks_done =
                            blocks_completed.fetch_add(chunk_size, Ordering::Relaxed) + chunk_size;

                        if let Some(cb) = callback {
                            let elapsed = start_time.elapsed().as_secs_f64();
                            cb(FetchProgress {
                                current_block: to,
                                total_blocks,
                                logs_fetched: count + logs.len() as u64,
                                percent: (blocks_done as f64 / total_blocks as f64) * 100.0,
                                blocks_per_second: if elapsed > 0.0 {
                                    blocks_done as f64 / elapsed
                                } else {
                                    0.0
                                },
                            });
                        }
                    }

                    // Return chunk bounds with result to preserve attribution
                    (from, to, result)
                }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;

        // Collect all logs and track failures
        let mut all_logs = Vec::new();
        let mut stats = FetchStats {
            chunks_total: chunks.len(),
            chunks_succeeded: 0,
            chunks_failed: 0,
            failed_ranges: Vec::new(),
        };

        // Process results - chunk bounds are carried with each result for correct attribution
        for (chunk_from, chunk_to, result) in results {
            match result {
                Ok(logs) => {
                    all_logs.extend(logs);
                    stats.chunks_succeeded += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        "Chunk fetch failed for blocks {}-{}: {}",
                        chunk_from,
                        chunk_to,
                        e
                    );
                    stats.chunks_failed += 1;
                    stats
                        .failed_ranges
                        .push((chunk_from, chunk_to, e.to_string()));
                }
            }
        }

        // Log summary if there were failures
        if stats.chunks_failed > 0 {
            tracing::warn!(
                "Fetch completed with {} failed chunks out of {} ({:.1}% success rate)",
                stats.chunks_failed,
                stats.chunks_total,
                stats.success_rate()
            );
        }

        // Sort by block number and log index
        all_logs.sort_by(|a, b| {
            let block_cmp = a.block_number.cmp(&b.block_number);
            if block_cmp == std::cmp::Ordering::Equal {
                a.log_index.cmp(&b.log_index)
            } else {
                block_cmp
            }
        });

        // Decode if needed
        let logs = if let Some(decoder) = &self.decoder {
            let decoded: Vec<DecodedLog> = all_logs
                .iter()
                .filter_map(|log| match decoder.decode(log) {
                    Ok(decoded) => Some(decoded),
                    Err(e) => {
                        tracing::debug!("Failed to decode log: {}", e);
                        None
                    }
                })
                .collect();

            FetchLogs::Decoded(decoded)
        } else {
            FetchLogs::Raw(all_logs)
        };

        Ok(FetchResult { logs, stats })
    }

    /// Fetch a single chunk with retry and adaptive splitting
    async fn fetch_chunk_with_retry(
        pool: &RpcPool,
        filter: &Filter,
        from: u64,
        to: u64,
        max_retries: u32,
    ) -> Result<Vec<Log>> {
        let mut current_from = from;
        let mut current_to = to;
        let mut all_logs = Vec::new();
        let mut retries = 0;

        while current_from <= to {
            let chunk_filter = filter.clone().from_block(current_from).to_block(current_to);

            match pool.get_logs(&chunk_filter).await {
                Ok(logs) => {
                    all_logs.extend(logs);
                    current_from = current_to + 1;
                    current_to = to;
                    retries = 0;
                }
                Err(Error::Rpc(RpcError::BlockRangeTooLarge { .. }))
                | Err(Error::Rpc(RpcError::ResponseTooLarge(_))) => {
                    // Split the range in half
                    let mid = (current_from + current_to) / 2;
                    if mid == current_from {
                        // Can't split further
                        return Err(RpcError::ResponseTooLarge(0).into());
                    }
                    current_to = mid;
                    tracing::debug!(
                        "Range too large, splitting: {} - {} -> {} - {}",
                        current_from,
                        to,
                        current_from,
                        current_to
                    );
                }
                Err(Error::Rpc(RpcError::RateLimited(_))) => {
                    // Wait and retry with exponential backoff (capped at 60s)
                    retries += 1;
                    if retries > max_retries {
                        return Err(
                            RpcError::RateLimited("Max retries exceeded".to_string()).into()
                        );
                    }
                    // Use saturating_pow to prevent overflow, cap at 60 seconds
                    let backoff_secs = 2u64.saturating_pow(retries).min(60);
                    tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;
                }
                Err(e) => {
                    retries += 1;
                    if retries > max_retries {
                        return Err(e);
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }

        Ok(all_logs)
    }

    /// Resolve end block number
    async fn resolve_end_block(&self) -> Result<u64> {
        match self.config.block_range.to_block() {
            BlockNumber::Number(n) => Ok(n),
            BlockNumber::Latest => self.pool.get_block_number().await,
        }
    }

    /// Calculate optimal chunks for fetching
    fn calculate_chunks(from: u64, to: u64, max_range: u64) -> Vec<(u64, u64)> {
        // Handle unlimited range (max_range == 0) - return single chunk
        if max_range == 0 {
            return vec![(from, to)];
        }

        let mut chunks = Vec::new();
        let mut current = from;

        while current <= to {
            // Use saturating_add to prevent overflow, then subtract 1
            // This calculates: current + max_range - 1, clamped to not exceed to
            let chunk_end = current.saturating_add(max_range.saturating_sub(1)).min(to);
            chunks.push((current, chunk_end));
            // Prevent infinite loop if chunk_end == u64::MAX
            if chunk_end == u64::MAX {
                break;
            }
            current = chunk_end + 1;
        }

        chunks
    }

    /// Get the RPC pool
    pub fn pool(&self) -> &RpcPool {
        &self.pool
    }

    /// Get endpoint count
    pub fn endpoint_count(&self) -> usize {
        self.pool.endpoint_count()
    }
}

/// Streaming fetcher for large datasets with checkpoint support
pub struct StreamingFetcher {
    fetcher: LogFetcher,
    checkpoint_manager: Option<std::sync::Arc<parking_lot::Mutex<CheckpointManager>>>,
}

impl StreamingFetcher {
    /// Create a streaming fetcher
    pub async fn new(config: Config) -> Result<Self> {
        let fetcher = LogFetcher::new(config).await?;
        Ok(Self {
            fetcher,
            checkpoint_manager: None,
        })
    }

    /// Enable checkpointing
    pub fn with_checkpoint(mut self, path: &Path) -> Result<Self> {
        let config = &self.fetcher.config;
        // For checkpoint, join multiple events with comma
        let event_filter = if config.events.is_empty() {
            None
        } else {
            Some(config.events.join(","))
        };
        let manager = CheckpointManager::load_or_create(
            path,
            &config.contract,
            config.chain.chain_id(),
            event_filter.as_deref(),
            config.block_range.from_block(),
            match config.block_range.to_block() {
                BlockNumber::Number(n) => Some(n),
                BlockNumber::Latest => None,
            },
        )?;

        self.checkpoint_manager = Some(std::sync::Arc::new(parking_lot::Mutex::new(manager)));
        Ok(self)
    }

    /// Get the RPC pool
    pub fn pool(&self) -> &RpcPool {
        &self.fetcher.pool
    }

    /// Get endpoint count
    pub fn endpoint_count(&self) -> usize {
        self.fetcher.pool.endpoint_count()
    }

    /// Fetch logs with streaming output, calling handler for each chunk
    /// Returns aggregated stats
    ///
    /// # Output Order
    ///
    /// **Important:** Chunks are processed in completion order, not block order.
    /// This means logs from later blocks may be written before logs from earlier blocks.
    /// This is a design choice to maximize throughput - using ordered streams would
    /// significantly reduce parallelism benefits.
    ///
    /// If you need block-ordered output:
    /// - Use `LogFetcher::fetch_all()` which sorts results before returning
    /// - Or post-process the output file to sort by block number
    ///
    /// Uses parallel fetching with `buffer_unordered` for improved performance
    /// while maintaining sequential handler calls for checkpoint consistency.
    pub async fn fetch_streaming<F>(&mut self, mut handler: F) -> Result<FetchStats>
    where
        F: FnMut(FetchResult) -> Result<()>,
    {
        let end_block = self.fetcher.resolve_end_block().await?;
        let from_block = self.fetcher.config.block_range.from_block();

        // Get remaining ranges if resuming
        let ranges = if let Some(ref manager) = self.checkpoint_manager {
            manager.lock().remaining_ranges(end_block)
        } else {
            vec![(from_block, end_block)]
        };

        if ranges.is_empty() {
            tracing::info!("All ranges already completed");
            return Ok(FetchStats::default());
        }

        let max_range = self.fetcher.pool.max_block_range();

        // Calculate all chunks across all ranges
        let all_chunks: Vec<(u64, u64)> = ranges
            .iter()
            .flat_map(|(from, to)| LogFetcher::calculate_chunks(*from, *to, max_range))
            .collect();

        let total_chunks = all_chunks.len();

        let mut stats = FetchStats {
            chunks_total: total_chunks,
            chunks_succeeded: 0,
            chunks_failed: 0,
            failed_ranges: Vec::new(),
        };

        // Build base filter
        let address: Address = self
            .fetcher
            .config
            .contract
            .parse()
            .map_err(|_| Error::from("Invalid contract address"))?;

        let mut base_filter = Filter::new().address(address);

        // Add event topics if we have specific events (use resolved events for filtering)
        // Multiple topics create an OR filter (matches any of the specified events)
        if !self.fetcher.resolved_events.is_empty() {
            let topics = parse_event_topics(&self.fetcher.resolved_events)?;
            base_filter = base_filter.event_signature(topics);
        }

        let concurrency = self.fetcher.config.rpc.concurrency;
        let max_retries = self.fetcher.config.rpc.max_retries;

        // Create parallel fetch stream
        let mut result_stream = stream::iter(all_chunks)
            .map(|(chunk_from, chunk_to)| {
                let filter = base_filter
                    .clone()
                    .from_block(chunk_from)
                    .to_block(chunk_to);
                let pool = &self.fetcher.pool;

                async move {
                    let result = LogFetcher::fetch_chunk_with_retry(
                        pool,
                        &filter,
                        chunk_from,
                        chunk_to,
                        max_retries,
                    )
                    .await;
                    (chunk_from, chunk_to, result)
                }
            })
            .buffer_unordered(concurrency);

        // Process results as they arrive (handler called sequentially)
        while let Some((chunk_from, chunk_to, result)) = result_stream.next().await {
            match result {
                Ok(logs) => {
                    let logs_count = logs.len() as u64;

                    let fetch_logs = if let Some(decoder) = &self.fetcher.decoder {
                        let mut decode_errors = 0u64;
                        let decoded: Vec<DecodedLog> = logs
                            .iter()
                            .filter_map(|log| match decoder.decode(log) {
                                Ok(decoded) => Some(decoded),
                                Err(e) => {
                                    decode_errors += 1;
                                    tracing::debug!(
                                        "Failed to decode log at block {:?}: {}",
                                        log.block_number,
                                        e
                                    );
                                    None
                                }
                            })
                            .collect();

                        if decode_errors > 0 {
                            tracing::warn!(
                                "Chunk {}-{}: {} logs failed to decode out of {}",
                                chunk_from,
                                chunk_to,
                                decode_errors,
                                logs_count
                            );
                        }
                        FetchLogs::Decoded(decoded)
                    } else {
                        FetchLogs::Raw(logs)
                    };

                    let fetch_result = FetchResult {
                        logs: fetch_logs,
                        stats: FetchStats {
                            chunks_total: 1,
                            chunks_succeeded: 1,
                            chunks_failed: 0,
                            failed_ranges: Vec::new(),
                        },
                    };

                    // Call handler to process/write the chunk
                    handler(fetch_result)?;

                    // Update checkpoint
                    if let Some(ref manager) = self.checkpoint_manager {
                        if let Err(e) = manager
                            .lock()
                            .mark_completed(chunk_from, chunk_to, logs_count)
                        {
                            tracing::warn!("Failed to update checkpoint: {}", e);
                        }
                    }

                    stats.chunks_succeeded += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        "Chunk fetch failed for blocks {}-{}: {}",
                        chunk_from,
                        chunk_to,
                        e
                    );
                    stats.chunks_failed += 1;
                    stats
                        .failed_ranges
                        .push((chunk_from, chunk_to, e.to_string()));
                }
            }
        }

        // Final checkpoint save
        if let Some(ref manager) = self.checkpoint_manager {
            if let Err(e) = manager.lock().save_now() {
                tracing::warn!("Failed to save final checkpoint: {}", e);
            }
        }

        Ok(stats)
    }

    /// Stream logs through a channel (for async consumers)
    ///
    /// Note: Uses `try_send` to avoid potential deadlock with `blocking_send`.
    /// If channel is full, returns an error. Consider using an unbounded channel
    /// or increasing channel capacity if you see send failures.
    pub async fn stream(mut self, tx: mpsc::Sender<Result<FetchResult>>) -> Result<FetchStats> {
        self.fetch_streaming(|result| {
            // Use try_send to avoid deadlock - blocking_send can deadlock if
            // the channel buffer is full and receiver is on the same runtime
            tx.try_send(Ok(result))
                .map_err(|e| Error::from(format!("Channel send failed (buffer full?): {}", e)))
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_chunks() {
        let chunks = LogFetcher::calculate_chunks(0, 100, 30);
        assert_eq!(chunks, vec![(0, 29), (30, 59), (60, 89), (90, 100)]);

        let chunks = LogFetcher::calculate_chunks(0, 10, 100);
        assert_eq!(chunks, vec![(0, 10)]);

        let chunks = LogFetcher::calculate_chunks(50, 50, 10);
        assert_eq!(chunks, vec![(50, 50)]);
    }

    #[test]
    fn test_fetch_result_len() {
        let result = FetchResult {
            logs: FetchLogs::Raw(vec![]),
            stats: FetchStats::default(),
        };
        assert!(result.is_empty());
        assert_eq!(result.len(), 0);
        assert!(result.is_complete());
    }

    #[test]
    fn test_fetch_stats() {
        let stats = FetchStats {
            chunks_total: 10,
            chunks_succeeded: 8,
            chunks_failed: 2,
            failed_ranges: vec![(100, 200, "error".to_string())],
        };
        assert!(!stats.is_complete());
        assert!((stats.success_rate() - 80.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_event_topics_valid_signature() {
        let events = vec!["Transfer(address,address,uint256)".to_string()];
        let topics = parse_event_topics(&events).unwrap();
        assert_eq!(topics.len(), 1);
        // Transfer topic hash
        assert_eq!(
            format!("{:#x}", topics[0]),
            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
        );
    }

    #[test]
    fn test_parse_event_topics_valid_hash() {
        let events =
            vec!["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string()];
        let topics = parse_event_topics(&events).unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(
            format!("{:#x}", topics[0]),
            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
        );
    }

    #[test]
    fn test_parse_event_topics_invalid_signature() {
        let events = vec!["InvalidSignature(".to_string()];
        let result = parse_event_topics(&events);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid event signature"));
    }

    #[test]
    fn test_parse_event_topics_invalid_hash() {
        // Too short
        let events = vec!["0x1234".to_string()];
        let result = parse_event_topics(&events);
        // This is treated as a signature, not a hash, so it should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_event_topics_empty() {
        let events: Vec<String> = vec![];
        let topics = parse_event_topics(&events).unwrap();
        assert!(topics.is_empty());
    }

    #[test]
    fn test_parse_event_topics_multiple() {
        let events = vec![
            "Transfer(address,address,uint256)".to_string(),
            "Approval(address,address,uint256)".to_string(),
        ];
        let topics = parse_event_topics(&events).unwrap();
        assert_eq!(topics.len(), 2);
    }
}
