//! Beacon API for Ethereum consensus layer

mod api;
mod types;

pub use api::{BeaconApi, RandaoResponse, RootResponse, ValidatorBalance};
pub use types::*;
