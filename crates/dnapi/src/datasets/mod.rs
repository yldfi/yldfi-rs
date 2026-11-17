//! Datasets API for listing and searching the Dune catalog
//!
//! Uploaded tables are managed through [`crate::tables`].

mod api;
mod types;

pub use api::DatasetsApi;
pub use types::*;
