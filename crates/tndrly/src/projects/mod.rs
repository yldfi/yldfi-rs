//! Projects API
//!
//! List, create, read and update the projects of the configured account.
//! These are account-level routes: the project slug configured on the
//! [`Client`](crate::Client) is not used, except as a convenient argument.
//!
//! # Example
//!
//! ```ignore
//! use tndrly::Client;
//! use tndrly::projects::{CreateProjectRequest, UpdateProjectRequest};
//!
//! let client = Client::from_env()?;
//!
//! for project in client.projects().list().await? {
//!     println!("{} ({})", project.name, project.slug);
//! }
//!
//! let project = client
//!     .projects()
//!     .create(&CreateProjectRequest::new("Mainnet Monitoring"))
//!     .await?;
//!
//! // Renaming keeps the slug
//! let update = UpdateProjectRequest::new().name("Mainnet Monitoring v2");
//! client.projects().update(&project.slug, &update).await?;
//! ```

mod api;
mod types;

pub use api::ProjectsApi;
pub use types::*;
