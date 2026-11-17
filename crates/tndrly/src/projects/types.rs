//! Types for the Projects API

use serde::{Deserialize, Serialize};

/// A Tenderly project
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Project {
    /// Project ID
    #[serde(default)]
    pub id: String,

    /// Display name
    #[serde(default)]
    pub name: String,

    /// URL slug, derived from the name at creation and never changed
    #[serde(default)]
    pub slug: String,

    /// Principal ID of the owning user or organization
    #[serde(default)]
    pub owner_id: Option<String>,

    /// Whether the project is public
    #[serde(default)]
    pub public: bool,

    /// Whether only explicit members can see the project
    /// (organization projects only)
    #[serde(default)]
    pub restricted: bool,

    /// High-volume project
    #[serde(default)]
    pub high_volume: bool,

    /// Whether the project is being transferred to another account
    #[serde(default)]
    pub in_transfer: bool,

    /// Whether the project is a module
    #[serde(default)]
    pub is_module: bool,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Time of the last push (e.g. of contracts via the CLI)
    #[serde(default)]
    pub last_push_at: Option<String>,

    /// Number of users with access
    #[serde(default)]
    pub number_of_users: Option<u64>,

    /// Project options (free-form)
    #[serde(default)]
    pub options: Option<serde_json::Value>,

    /// Owning user or organization
    #[serde(default)]
    pub owner: Option<ProjectOwner>,

    /// The requester's effective permissions on the project
    #[serde(default)]
    pub permissions: Option<ProjectPermissions>,

    /// Project members (when returned)
    #[serde(default)]
    pub members: Option<Vec<ProjectMember>>,
}

/// Owner of a project
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProjectOwner {
    /// Principal ID
    #[serde(default)]
    pub id: Option<String>,

    /// Username or organization slug
    #[serde(default)]
    pub username: Option<String>,

    /// First name
    #[serde(default)]
    pub first_name: Option<String>,

    /// Last name
    #[serde(default)]
    pub last_name: Option<String>,

    /// Email
    #[serde(default)]
    pub email: Option<String>,

    /// `user` or `organization`
    #[serde(default, rename = "type")]
    pub owner_type: Option<String>,
}

/// The requester's effective permissions on a project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProjectPermissions {
    /// Can add contracts
    #[serde(default)]
    pub add_contract: bool,
    /// Can update contracts
    #[serde(default)]
    pub update_contract: bool,
    /// Can remove contracts
    #[serde(default)]
    pub remove_contract: bool,
    /// Can create alerts
    #[serde(default)]
    pub create_alert: bool,
    /// Can update alerts
    #[serde(default)]
    pub update_alert: bool,
    /// Can remove alerts
    #[serde(default)]
    pub remove_alert: bool,
    /// Can create and update analytics
    #[serde(default)]
    pub create_and_update_analytics: bool,
    /// Can manage collaborators
    #[serde(default)]
    pub manage_collaborators: bool,
}

/// A project member
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProjectMember {
    /// Principal ID
    #[serde(default)]
    pub id: Option<String>,

    /// Username
    #[serde(default)]
    pub username: Option<String>,

    /// First name
    #[serde(default)]
    pub first_name: Option<String>,

    /// Last name
    #[serde(default)]
    pub last_name: Option<String>,

    /// Email
    #[serde(default)]
    pub email: Option<String>,

    /// Whether the member accepted the invitation
    #[serde(default)]
    pub accepted: Option<bool>,

    /// When the member was added
    #[serde(default)]
    pub added_at: Option<String>,
}

/// `{"project": ...}` envelope returned by get, create and update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProjectResponse {
    /// The project
    pub project: Project,
}

/// `{"projects": [...]}` envelope returned by list
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ListProjectsResponse {
    /// The projects (`null` reads as empty)
    #[serde(default, deserialize_with = "null_as_empty")]
    pub projects: Vec<Project>,
}

fn null_as_empty<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

/// Request to create a project
///
/// ```
/// use tndrly::projects::CreateProjectRequest;
///
/// let request = CreateProjectRequest::new("Treasury Ops")
///     .restricted(true)
///     .member("2f6a3c4e-1b7d-4e0a-9c11-5d2e8a7b9f30");
/// ```
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct CreateProjectRequest {
    /// Project name, 1 to 100 characters; the slug is derived from it
    pub name: String,

    /// Organization accounts only: `true` creates a project visible to its
    /// explicit members alone
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restricted: Option<bool>,

    /// Organization accounts only, with `restricted: true`: principal IDs of
    /// organization members to add (the creator and the organization owner
    /// are always added)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<String>>,
}

impl CreateProjectRequest {
    /// Create a request for a project with this name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            restricted: None,
            members: None,
        }
    }

    /// Restrict the project to its explicit members (organization accounts)
    #[must_use]
    pub fn restricted(mut self, restricted: bool) -> Self {
        self.restricted = Some(restricted);
        self
    }

    /// Add a member by principal ID (organization accounts, restricted projects)
    #[must_use]
    pub fn member(mut self, principal_id: impl Into<String>) -> Self {
        self.members
            .get_or_insert_with(Vec::new)
            .push(principal_id.into());
        self
    }
}

/// Partial update of a project: only the fields set are sent, and at least
/// one must be set
///
/// ```
/// use tndrly::projects::UpdateProjectRequest;
///
/// let rename = UpdateProjectRequest::new().name("Mainnet Monitoring v2");
/// assert!(rename.validate().is_ok());
/// assert!(UpdateProjectRequest::new().validate().is_err());
/// ```
#[derive(Debug, Clone, Default, Serialize)]
#[non_exhaustive]
pub struct UpdateProjectRequest {
    /// New name (the slug stays unchanged)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Replaces the project options object
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,

    /// Organization projects only: restrict to explicit members (`true`) or
    /// open to every organization member (`false`). Not changeable with an
    /// access token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restricted: Option<bool>,

    /// Organization projects only: replaces the full member list (principal
    /// IDs). Not changeable with an access token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<String>>,
}

impl UpdateProjectRequest {
    /// Create an empty update
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Rename the project
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Replace the project options
    #[must_use]
    pub fn options(mut self, options: serde_json::Value) -> Self {
        self.options = Some(options);
        self
    }

    /// Restrict or open the project
    #[must_use]
    pub fn restricted(mut self, restricted: bool) -> Self {
        self.restricted = Some(restricted);
        self
    }

    /// Replace the member list
    #[must_use]
    pub fn members(mut self, principal_ids: Vec<String>) -> Self {
        self.members = Some(principal_ids);
        self
    }

    /// Check that at least one field is set
    ///
    /// # Errors
    /// If the update is empty.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_none()
            && self.options.is_none()
            && self.restricted.is_none()
            && self.members.is_none()
        {
            return Err("A project update needs at least one field".to_string());
        }
        Ok(())
    }
}
