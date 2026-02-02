//! RPC endpoint management and parallel request handling

mod endpoint;
mod health;
pub mod multicall;
mod optimizer;
mod pool;
pub mod retry;
pub mod selector;

pub use endpoint::Endpoint;
pub use health::{EndpointHealth, HealthTracker};
pub use multicall::{MulticallBuilder, MulticallResult, MULTICALL3_ADDRESS};
pub use optimizer::{optimize_endpoint, test_connectivity, OptimizationResult};
pub use pool::RpcPool;
pub use retry::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};
pub use selector::{
    get_archive_endpoint, get_rpc_endpoint, get_rpc_endpoint_for_block, get_rpc_url,
    get_rpc_url_for_block, SelectionOptions,
};
