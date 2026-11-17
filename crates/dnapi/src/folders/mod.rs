//! Folders API for organizing queries and dashboards in the library
//!
//! A team API key works on the team's folders as the team member who created
//! the key; a personal key works on the user's own folders.

mod api;
mod types;

pub use api::FoldersApi;
pub use types::*;
