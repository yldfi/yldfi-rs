//! Projects API operations

use super::types::{
    CreateProjectRequest, ListProjectsResponse, Project, ProjectResponse, UpdateProjectRequest,
};
use crate::client::{encode_path_segment, Client};
use crate::error::Result;

/// Projects API client
pub struct ProjectsApi<'a> {
    client: &'a Client,
}

impl<'a> ProjectsApi<'a> {
    /// Create a new Projects API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List the account's projects plus projects shared with the requester
    ///
    /// `GET /v1/account/{account}/projects`
    pub async fn list(&self) -> Result<Vec<Project>> {
        let response: ListProjectsResponse = self.client.get_account("/projects").await?;
        Ok(response.projects)
    }

    /// Get a project by slug or ID
    ///
    /// `GET /v1/account/{account}/project/{project}`. Pass
    /// `client.project()` for the configured project.
    pub async fn get(&self, project: &str) -> Result<Project> {
        let url = self
            .client
            .account_url(&format!("/project/{}", encode_path_segment(project)));
        let response: ProjectResponse = self.client.get_url(&url).await?;
        Ok(response.project)
    }

    /// Create a project owned by the account
    ///
    /// `POST /v1/account/{account}/project`. Creates a new project; its slug
    /// is derived from the name and cannot be changed afterwards. Requires
    /// the create-project permission on the account.
    pub async fn create(&self, request: &CreateProjectRequest) -> Result<Project> {
        let url = self.client.account_url("/project");
        let response: ProjectResponse = self.client.post_url(&url, request).await?;
        Ok(response.project)
    }

    /// Partially update a project
    ///
    /// `POST /v1/account/{account}/project/{project}`. Changes only the
    /// fields set on `request` (renaming keeps the slug) and returns the
    /// updated project.
    ///
    /// # Errors
    /// An invalid-parameter error, without sending anything if
    /// `request` sets no field.
    pub async fn update(&self, project: &str, request: &UpdateProjectRequest) -> Result<Project> {
        request.validate().map_err(crate::error::invalid_param)?;
        let url = self
            .client
            .account_url(&format!("/project/{}", encode_path_segment(project)));
        let response: ProjectResponse = self.client.post_url(&url, request).await?;
        Ok(response.project)
    }
}

#[cfg(test)]
mod tests {
    use crate::projects::{CreateProjectRequest, UpdateProjectRequest};
    use crate::test_util::mock_client;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, ResponseTemplate};

    fn project_json(name: &str) -> serde_json::Value {
        json!({
            "id": "d66fd37f-e2f8-46c3-b549-0160d60eec96",
            "name": name,
            "slug": "mainnet-monitoring",
            "owner_id": "49309f73-1d60-48fc-94b3-120fd9982381",
            "public": false,
            "restricted": true,
            "created_at": "2026-09-07T16:12:22.895718Z",
            "last_push_at": null,
            "number_of_users": 1,
            "options": null,
            "owner": { "id": "49309f73", "username": "acme", "type": "organization" },
            "permissions": { "add_contract": true, "manage_collaborators": false },
            "members": [{ "id": "2f6a3c4e", "username": "jane", "accepted": true }]
        })
    }

    #[tokio::test]
    async fn test_list_projects() {
        let (server, client) = mock_client().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/account/acct/projects"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({ "projects": [project_json("Mainnet Monitoring")] })),
            )
            .expect(1)
            .mount(&server)
            .await;

        let projects = client.projects().list().await.unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].slug, "mainnet-monitoring");
        assert!(projects[0].restricted);
        assert_eq!(projects[0].number_of_users, Some(1));
        let owner = projects[0].owner.as_ref().unwrap();
        assert_eq!(owner.owner_type.as_deref(), Some("organization"));
        assert!(projects[0].permissions.as_ref().unwrap().add_contract);
    }

    #[tokio::test]
    async fn test_list_projects_null_is_empty() {
        let (server, client) = mock_client().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/account/acct/projects"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "projects": null })))
            .mount(&server)
            .await;
        assert!(client.projects().list().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_project() {
        let (server, client) = mock_client().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/account/acct/project/my%20proj"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({ "project": project_json("Mainnet Monitoring") })),
            )
            .expect(1)
            .mount(&server)
            .await;

        let project = client.projects().get("my proj").await.unwrap();
        assert_eq!(project.name, "Mainnet Monitoring");
        let members = project.members.unwrap();
        assert_eq!(members[0].username.as_deref(), Some("jane"));
        assert_eq!(members[0].accepted, Some(true));
    }

    #[tokio::test]
    async fn test_create_project() {
        let (server, client) = mock_client().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/account/acct/project"))
            .and(body_json(json!({
                "name": "Treasury Ops",
                "restricted": true,
                "members": ["2f6a3c4e"]
            })))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(json!({ "project": project_json("Treasury Ops") })),
            )
            .expect(1)
            .mount(&server)
            .await;

        let request = CreateProjectRequest::new("Treasury Ops")
            .restricted(true)
            .member("2f6a3c4e");
        let project = client.projects().create(&request).await.unwrap();
        assert_eq!(project.name, "Treasury Ops");
    }

    #[tokio::test]
    async fn test_update_project() {
        let (server, client) = mock_client().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/account/acct/project/mainnet%2Dmonitoring"))
            .and(body_json(json!({ "name": "Mainnet Monitoring v2" })))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({ "project": project_json("Mainnet Monitoring v2") })),
            )
            .expect(1)
            .mount(&server)
            .await;

        let request = UpdateProjectRequest::new().name("Mainnet Monitoring v2");
        let project = client
            .projects()
            .update("mainnet-monitoring", &request)
            .await
            .unwrap();
        assert_eq!(project.name, "Mainnet Monitoring v2");
        assert_eq!(project.slug, "mainnet-monitoring");
    }

    #[tokio::test]
    async fn test_update_project_rejects_empty_update() {
        let (server, client) = mock_client().await;
        let err = client
            .projects()
            .update("p", &UpdateProjectRequest::new())
            .await
            .unwrap_err();
        assert!(
            matches!(
                err,
                crate::Error::Domain(crate::error::DomainError::InvalidParam(_))
            ),
            "{err}"
        );
        assert!(server.received_requests().await.unwrap().is_empty());
    }
}
