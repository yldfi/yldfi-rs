//! Types for the Folders API

use serde::{Deserialize, Serialize};

/// A folder in the library
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Folder {
    /// Folder ID
    pub folder_id: String,
    /// Folder name
    pub name: Option<String>,
    /// Full path from the library root
    pub path: Option<String>,
    /// Parent folder ID; `None` for a folder at the library root
    pub parent_folder_id: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Icon name
    pub icon: Option<String>,
    /// Color: a preset name or a hex value
    pub color: Option<String>,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// A folder as listed inside another folder or at the library root
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct FolderSummary {
    /// Folder ID
    pub folder_id: String,
    /// Folder name
    pub name: Option<String>,
    /// Full path from the library root
    pub path: Option<String>,
    /// Number of queries and dashboards directly inside the folder
    pub item_count: Option<i64>,
}

/// Response from listing folders
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ListFoldersResponse {
    /// Folders, ordered by name
    #[serde(default)]
    pub folders: Vec<FolderSummary>,
    /// Offset of the next page; `None` on the last page
    pub next_offset: Option<i64>,
}

/// A query or dashboard inside a folder
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct FolderItem {
    /// Query ID or dashboard ID, depending on `item_type`
    pub id: i64,
    /// `query` or `dashboard`
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    /// Name
    pub name: Option<String>,
    /// Whether the item is private
    pub is_private: Option<bool>,
    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// A folder with its direct subfolders and items
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct FolderContents {
    /// The folder
    pub folder: Option<Folder>,
    /// Direct subfolders (at most 100)
    #[serde(default)]
    pub subfolders: Vec<FolderSummary>,
    /// Whether there are more than 100 subfolders; list them with
    /// [`FoldersApi::list`](super::FoldersApi::list)
    pub has_more_subfolders: Option<bool>,
    /// Queries and dashboards directly inside, not archived, ordered by name
    #[serde(default)]
    pub items: Vec<FolderItem>,
    /// Total number of items
    pub total_items: Option<i64>,
    /// Offset of the next page of items; `None` on the last page
    pub next_offset: Option<i64>,
}

/// Options for listing folders
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ListFoldersOptions {
    /// List the subfolders of this folder; `None` for the library root
    pub parent_folder_id: Option<String>,
    /// Case-insensitive substring filter on the folder name
    pub name_contains: Option<String>,
    /// Maximum number of folders (default 50, max 100)
    pub limit: Option<u32>,
    /// Pagination offset; use `next_offset` from the previous page
    pub offset: Option<i64>,
}

impl ListFoldersOptions {
    /// Options with nothing set
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Query pairs for the request (values are URL-encoded by the HTTP client)
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();
        if let Some(ref id) = self.parent_folder_id {
            params.push(("parent_folder_id", id.clone()));
        }
        if let Some(ref name) = self.name_contains {
            params.push(("name_contains", name.clone()));
        }
        if let Some(limit) = self.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(offset) = self.offset {
            params.push(("offset", offset.to_string()));
        }
        params
    }
}

/// Request to create a folder (`POST /v1/folders`)
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct CreateFolderRequest {
    /// Folder name
    pub name: String,
    /// Parent folder; `None` creates the folder at the library root
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<String>,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Icon (Dune defaults to `folder`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Color: a preset (green, yellow, red, blue-light, gray, orange, blue,
    /// pink, mauve, amber) or a hex value such as `#E5477D` (Dune defaults to
    /// `gray`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl CreateFolderRequest {
    /// A folder named `name` at the library root
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent_folder_id: None,
            description: None,
            icon: None,
            color: None,
        }
    }

    /// Create the folder inside `parent_folder_id`
    #[must_use]
    pub fn parent(mut self, parent_folder_id: impl Into<String>) -> Self {
        self.parent_folder_id = Some(parent_folder_id.into());
        self
    }

    /// Set the description
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Request to update a folder (`PATCH /v1/folders/{folder_id}`)
///
/// Fields left `None` keep their value.
#[derive(Debug, Clone, Default, Serialize)]
#[non_exhaustive]
pub struct UpdateFolderRequest {
    /// New name; renaming also renames the paths of every folder below it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New description; an empty string clears it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New icon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// New color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl UpdateFolderRequest {
    /// A request that changes nothing until fields are set
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Rename the folder
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Body of `POST /v1/folders/{folder_id}/move`
#[derive(Debug, Clone, Serialize)]
pub(crate) struct MoveFolderRequest<'a> {
    /// Always sent; `null` moves the folder to the library root
    pub parent_folder_id: Option<&'a str>,
}

/// Response from deleting a folder
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DeleteFolderResponse {
    /// Deleted folder ID
    pub folder_id: Option<String>,
    /// Path the folder had
    pub path: Option<String>,
    /// Folders deleted: the folder itself and every subfolder
    pub deleted_folder_count: Option<i64>,
    /// Queries archived by a recursive delete
    pub archived_query_count: Option<i64>,
    /// Dashboards archived by a recursive delete
    pub archived_dashboard_count: Option<i64>,
}

/// Request to move queries and dashboards (`POST /v1/content/move`)
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct MoveContentRequest {
    /// Destination folder; `None` takes the content out of folders (always
    /// sent, so a missing value never moves content by accident)
    pub target_folder_id: Option<String>,
    /// Queries to move (at most 100)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub query_ids: Vec<i64>,
    /// Dashboards to move (at most 100)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dashboard_ids: Vec<i64>,
}

impl MoveContentRequest {
    /// Move content into `target_folder_id`, or out of folders with `None`
    #[must_use]
    pub fn new(target_folder_id: Option<String>) -> Self {
        Self {
            target_folder_id,
            query_ids: Vec::new(),
            dashboard_ids: Vec::new(),
        }
    }

    /// Move these queries
    #[must_use]
    pub fn queries(mut self, query_ids: Vec<i64>) -> Self {
        self.query_ids = query_ids;
        self
    }

    /// Move these dashboards
    #[must_use]
    pub fn dashboards(mut self, dashboard_ids: Vec<i64>) -> Self {
        self.dashboard_ids = dashboard_ids;
        self
    }
}

/// Response from moving queries and dashboards
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MoveContentResponse {
    /// The destination folder; `None` when content moved to the library root
    pub target_folder: Option<Folder>,
    /// Queries moved
    #[serde(default)]
    pub moved_query_ids: Vec<i64>,
    /// Dashboards moved
    #[serde(default)]
    pub moved_dashboard_ids: Vec<i64>,
}
