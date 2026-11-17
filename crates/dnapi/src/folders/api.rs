//! Folders API implementation

use super::types::{
    CreateFolderRequest, DeleteFolderResponse, Folder, FolderContents, ListFoldersOptions,
    ListFoldersResponse, MoveContentRequest, MoveContentResponse, MoveFolderRequest,
    UpdateFolderRequest,
};
use crate::client::Client;
use crate::error::Result;
use crate::PageOptions;

/// Folders API
pub struct FoldersApi<'a> {
    client: &'a Client,
}

impl<'a> FoldersApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List the folders at the library root, or inside
    /// `options.parent_folder_id`, ordered by name
    ///
    /// Calls `GET /v1/folders`. Page with `limit` and the `next_offset` of
    /// the previous response.
    pub async fn list(&self, options: &ListFoldersOptions) -> Result<ListFoldersResponse> {
        let url = self.client.url("/v1/folders");
        let request = self
            .client
            .http()
            .get(&url)
            .query(&options.to_query_pairs());
        self.client.send_json(request, None).await
    }

    /// Get a folder with its direct subfolders and the (non-archived) queries
    /// and dashboards directly inside it
    ///
    /// Calls `GET /v1/folders/{folder_id}`; `page` pages through the items.
    pub async fn get(&self, folder_id: &str, page: &PageOptions) -> Result<FolderContents> {
        let url = self.client.url(&format!("/v1/folders/{folder_id}"));
        let request = self.client.http().get(&url).query(&page.to_query_pairs());
        self.client
            .send_json(request, Some(format!("Folder {folder_id}")))
            .await
    }

    /// Create a folder at the library root or inside a parent folder
    ///
    /// Calls `POST /v1/folders`. Team folders need an editor or admin behind
    /// the API key. A folder that already exists at the same path is refused
    /// with a 409 (`folder_already_exists`).
    pub async fn create(&self, request: &CreateFolderRequest) -> Result<Folder> {
        let url = self.client.url("/v1/folders");
        self.client
            .send_json(self.client.http().post(&url).json(request), None)
            .await
    }

    /// Change a folder's name, description, icon or color
    ///
    /// Calls `PATCH /v1/folders/{folder_id}`; unset fields keep their value.
    pub async fn update(&self, folder_id: &str, request: &UpdateFolderRequest) -> Result<Folder> {
        let url = self.client.url(&format!("/v1/folders/{folder_id}"));
        self.client
            .send_json(
                self.client.http().patch(&url).json(request),
                Some(format!("Folder {folder_id}")),
            )
            .await
    }

    /// Move a folder, with everything inside it, under `parent_folder_id`,
    /// or to the library root with `None`
    ///
    /// Calls `POST /v1/folders/{folder_id}/move`. A folder cannot move into
    /// itself or one of its subfolders.
    pub async fn move_folder(
        &self,
        folder_id: &str,
        parent_folder_id: Option<&str>,
    ) -> Result<Folder> {
        let url = self.client.url(&format!("/v1/folders/{folder_id}/move"));
        let body = MoveFolderRequest { parent_folder_id };
        self.client
            .send_json(
                self.client.http().post(&url).json(&body),
                Some(format!("Folder {folder_id}")),
            )
            .await
    }

    /// Delete a folder
    ///
    /// Calls `DELETE /v1/folders/{folder_id}`. An empty folder is deleted
    /// right away. A folder that still holds anything is refused with a 409
    /// (`folder_not_empty`) unless `recursive` is true, which **deletes every
    /// subfolder and archives the queries and dashboards inside**. Move the
    /// content out first with [`move_content`](Self::move_content) to keep
    /// it in the library.
    pub async fn delete(&self, folder_id: &str, recursive: bool) -> Result<DeleteFolderResponse> {
        let url = self.client.url(&format!("/v1/folders/{folder_id}"));
        let mut request = self.client.http().delete(&url);
        if recursive {
            request = request.query(&[("recursive", "true")]);
        }
        self.client
            .send_json(request, Some(format!("Folder {folder_id}")))
            .await
    }

    /// Move up to 100 queries and 100 dashboards into a folder, or out of
    /// folders when `request.target_folder_id` is `None`
    ///
    /// Calls `POST /v1/content/move`. All or nothing: every item must belong
    /// to the API key's library and be editable through it, otherwise
    /// nothing moves.
    pub async fn move_content(&self, request: &MoveContentRequest) -> Result<MoveContentResponse> {
        let url = self.client.url("/v1/content/move");
        self.client
            .send_json(self.client.http().post(&url).json(request), None)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use crate::test_util::{failing, is_not_found, serve};
    use crate::PageOptions;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
    use wiremock::Mock;

    fn folder_json() -> serde_json::Value {
        json!({
            "folder_id": "f1",
            "name": "Test folder",
            "path": "Parent/Test folder",
            "parent_folder_id": null,
            "description": "",
            "icon": "folder",
            "color": "gray",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-02T00:00:00Z"
        })
    }

    #[tokio::test]
    async fn list_sends_filters() {
        let (_server, client) = serve(
            Mock::given(method("GET"))
                .and(path("/v1/folders"))
                .and(query_param("parent_folder_id", "f0"))
                .and(query_param("name_contains", "a b"))
                .and(query_param("limit", "10"))
                .and(query_param("offset", "20")),
            json!({
                "folders": [{"folder_id": "f1", "name": "Test folder", "path": "Test folder", "item_count": 3}],
                "next_offset": 30
            }),
        )
        .await;
        let mut options = ListFoldersOptions::new();
        options.parent_folder_id = Some("f0".into());
        options.name_contains = Some("a b".into());
        options.limit = Some(10);
        options.offset = Some(20);
        let r = client.folders().list(&options).await.unwrap();
        assert_eq!(r.folders[0].folder_id, "f1");
        assert_eq!(r.folders[0].item_count, Some(3));
        assert_eq!(r.next_offset, Some(30));
    }

    #[tokio::test]
    async fn get_reads_contents() {
        let (_server, client) = serve(
            Mock::given(method("GET"))
                .and(path("/v1/folders/f1"))
                .and(query_param("limit", "5"))
                .and(query_param_is_missing("offset")),
            json!({
                "folder": folder_json(),
                "subfolders": [{"folder_id": "f2", "name": "Sub", "path": "Test folder/Sub", "item_count": 0}],
                "has_more_subfolders": false,
                "items": [{"id": 1234, "type": "query", "name": "Q", "is_private": false, "updated_at": "2026-01-02T00:00:00Z"}],
                "total_items": 1
            }),
        )
        .await;
        let r = client
            .folders()
            .get("f1", &PageOptions::new(Some(5), None))
            .await
            .unwrap();
        assert_eq!(r.folder.unwrap().folder_id, "f1");
        assert_eq!(r.subfolders[0].folder_id, "f2");
        assert_eq!(r.items[0].id, 1234);
        assert_eq!(r.items[0].item_type.as_deref(), Some("query"));
        assert_eq!(r.next_offset, None);
    }

    #[tokio::test]
    async fn get_maps_404_to_not_found() {
        let (_server, client) = failing(404).await;
        let err = client
            .folders()
            .get("f1", &PageOptions::default())
            .await
            .unwrap_err();
        assert!(is_not_found(&err), "{err}");
    }

    #[tokio::test]
    async fn create_sends_body() {
        let (_server, client) = serve(
            Mock::given(method("POST"))
                .and(path("/v1/folders"))
                .and(body_json(json!({
                    "name": "Test folder",
                    "parent_folder_id": "f0",
                    "description": "about"
                }))),
            folder_json(),
        )
        .await;
        let request = CreateFolderRequest::new("Test folder")
            .parent("f0")
            .description("about");
        let f = client.folders().create(&request).await.unwrap();
        assert_eq!(f.name.as_deref(), Some("Test folder"));
        assert_eq!(f.parent_folder_id, None);
    }

    #[tokio::test]
    async fn update_patches_only_set_fields() {
        let (_server, client) = serve(
            Mock::given(method("PATCH"))
                .and(path("/v1/folders/f1"))
                .and(body_json(json!({"name": "Renamed"}))),
            folder_json(),
        )
        .await;
        let request = UpdateFolderRequest::new().name("Renamed");
        let f = client.folders().update("f1", &request).await.unwrap();
        assert_eq!(f.folder_id, "f1");
    }

    #[tokio::test]
    async fn move_folder_sends_parent() {
        let (_server, client) = serve(
            Mock::given(method("POST"))
                .and(path("/v1/folders/f1/move"))
                .and(body_json(json!({"parent_folder_id": "f0"}))),
            folder_json(),
        )
        .await;
        let f = client
            .folders()
            .move_folder("f1", Some("f0"))
            .await
            .unwrap();
        assert_eq!(f.folder_id, "f1");
    }

    #[tokio::test]
    async fn move_folder_to_root_sends_null() {
        let (_server, client) = serve(
            Mock::given(method("POST"))
                .and(path("/v1/folders/f1/move"))
                .and(body_json(json!({"parent_folder_id": null}))),
            folder_json(),
        )
        .await;
        client.folders().move_folder("f1", None).await.unwrap();
    }

    #[tokio::test]
    async fn delete_is_not_recursive_by_default() {
        let (_server, client) = serve(
            Mock::given(method("DELETE"))
                .and(path("/v1/folders/f1"))
                .and(query_param_is_missing("recursive")),
            json!({"folder_id": "f1", "path": "Test folder", "deleted_folder_count": 1}),
        )
        .await;
        let r = client.folders().delete("f1", false).await.unwrap();
        assert_eq!(r.deleted_folder_count, Some(1));
        assert_eq!(r.archived_query_count, None);
    }

    #[tokio::test]
    async fn delete_recursive_sets_flag() {
        let (_server, client) = serve(
            Mock::given(method("DELETE"))
                .and(path("/v1/folders/f1"))
                .and(query_param("recursive", "true")),
            json!({
                "folder_id": "f1", "path": "Test folder", "deleted_folder_count": 2,
                "archived_query_count": 3, "archived_dashboard_count": 1
            }),
        )
        .await;
        let r = client.folders().delete("f1", true).await.unwrap();
        assert_eq!(r.archived_query_count, Some(3));
        assert_eq!(r.archived_dashboard_count, Some(1));
    }

    #[tokio::test]
    async fn move_content_sends_ids() {
        let (_server, client) = serve(
            Mock::given(method("POST"))
                .and(path("/v1/content/move"))
                .and(body_json(json!({
                    "target_folder_id": "f1",
                    "query_ids": [1, 2],
                    "dashboard_ids": [3]
                }))),
            json!({
                "target_folder": folder_json(),
                "moved_query_ids": [1, 2],
                "moved_dashboard_ids": [3]
            }),
        )
        .await;
        let request = MoveContentRequest::new(Some("f1".into()))
            .queries(vec![1, 2])
            .dashboards(vec![3]);
        let r = client.folders().move_content(&request).await.unwrap();
        assert_eq!(r.target_folder.unwrap().folder_id, "f1");
        assert_eq!(r.moved_query_ids, vec![1, 2]);
        assert_eq!(r.moved_dashboard_ids, vec![3]);
    }

    #[tokio::test]
    async fn move_content_to_root_sends_null_target() {
        let (_server, client) = serve(
            Mock::given(method("POST"))
                .and(path("/v1/content/move"))
                .and(body_json(
                    json!({"target_folder_id": null, "query_ids": [1]}),
                )),
            json!({"target_folder": null, "moved_query_ids": [1], "moved_dashboard_ids": []}),
        )
        .await;
        let request = MoveContentRequest::new(None).queries(vec![1]);
        let r = client.folders().move_content(&request).await.unwrap();
        assert!(r.target_folder.is_none());
    }
}
