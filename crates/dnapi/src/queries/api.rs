//! Queries API implementation

use super::types::{
    CreateQueryRequest, CreateQueryResponse, ListQueriesOptions, ListQueriesResponse, Query,
    QueryIdResponse, UpdateQueryRequest, UpdateQueryResponse,
};
use crate::client::Client;
use crate::error::{self, Result};

/// Queries API
pub struct QueriesApi<'a> {
    client: &'a Client,
}

impl<'a> QueriesApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create a new query
    pub async fn create(&self, request: &CreateQueryRequest) -> Result<CreateQueryResponse> {
        let url = format!("{}/v1/query", self.client.base_url());
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// Get a query by ID
    pub async fn get(&self, query_id: i64) -> Result<Query> {
        let url = format!("{}/v1/query/{}", self.client.base_url(), query_id);
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {query_id}")))
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// Update a query
    pub async fn update(
        &self,
        query_id: i64,
        request: &UpdateQueryRequest,
    ) -> Result<UpdateQueryResponse> {
        let url = format!("{}/v1/query/{}", self.client.base_url(), query_id);
        let response = self.client.http().patch(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {query_id}")))
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// List queries
    pub async fn list(&self) -> Result<ListQueriesResponse> {
        self.list_with_options(&ListQueriesOptions::default()).await
    }

    /// List queries with options
    pub async fn list_with_options(
        &self,
        options: &ListQueriesOptions,
    ) -> Result<ListQueriesResponse> {
        let url = format!(
            "{}/v1/queries{}",
            self.client.base_url(),
            options.to_query_string()
        );
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// Archive a query
    ///
    /// Calls `POST /v1/query/{query_id}/archive`. The query is not deleted
    /// but can no longer be edited or executed. Only the owner's API key
    /// works. Ignores the response body; [`set_archived`](Self::set_archived)
    /// returns it.
    pub async fn archive(&self, query_id: i64) -> Result<()> {
        let url = format!("{}/v1/query/{}/archive", self.client.base_url(), query_id);
        let response = self.client.http().post(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {query_id}")))
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// Unarchive a query
    ///
    /// Calls `POST /v1/query/{query_id}/unarchive`. Only the owner's API key
    /// works.
    pub async fn unarchive(&self, query_id: i64) -> Result<()> {
        let url = format!("{}/v1/query/{}/unarchive", self.client.base_url(), query_id);
        let response = self.client.http().post(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {query_id}")))
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// Make a query private
    ///
    /// Calls `POST /v1/query/{query_id}/private`. Only the owner's API key
    /// works. Ignores the response body; [`set_private`](Self::set_private)
    /// returns it.
    pub async fn make_private(&self, query_id: i64) -> Result<()> {
        let url = format!("{}/v1/query/{}/private", self.client.base_url(), query_id);
        let response = self.client.http().post(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {query_id}")))
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// Make a query public (unprivate)
    ///
    /// Calls `POST /v1/query/{query_id}/unprivate`. Only the owner's API key
    /// works.
    pub async fn make_public(&self, query_id: i64) -> Result<()> {
        let url = format!("{}/v1/query/{}/unprivate", self.client.base_url(), query_id);
        let response = self.client.http().post(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {query_id}")))
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// Archive (`true`) or unarchive (`false`) a query, returning the
    /// response
    ///
    /// Calls `POST /v1/query/{query_id}/archive` or `.../unarchive`. An
    /// archived query is not deleted but can no longer be edited or
    /// executed. Only the owner's API key works.
    pub async fn set_archived(&self, query_id: i64, archived: bool) -> Result<QueryIdResponse> {
        let action = if archived { "archive" } else { "unarchive" };
        self.post_action(query_id, action).await
    }

    /// Make a query private (`true`) or public (`false`), returning the
    /// response
    ///
    /// Calls `POST /v1/query/{query_id}/private` or `.../unprivate`. Only
    /// the owner's API key works.
    pub async fn set_private(&self, query_id: i64, private: bool) -> Result<QueryIdResponse> {
        let action = if private { "private" } else { "unprivate" };
        self.post_action(query_id, action).await
    }

    async fn post_action(&self, query_id: i64, action: &str) -> Result<QueryIdResponse> {
        let url = self.client.url(&format!("/v1/query/{query_id}/{action}"));
        self.client
            .send_json(
                self.client.http().post(&url),
                Some(format!("Query {query_id}")),
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use crate::test_util::{failing, is_not_found, serve};
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::Mock;

    #[tokio::test]
    async fn list_with_options_pages() {
        let (_server, client) = serve(
            Mock::given(method("GET"))
                .and(path("/v1/queries"))
                .and(query_param("limit", "5"))
                .and(query_param("offset", "10")),
            json!({"queries": [{"id": 1234, "name": "Test query", "tags": []}], "total": 11}),
        )
        .await;
        let options = ListQueriesOptions {
            limit: Some(5),
            offset: Some(10),
        };
        let r = client.queries().list_with_options(&options).await.unwrap();
        assert_eq!(r.queries[0].id, 1234);
        assert_eq!(r.total, Some(11));
    }

    #[tokio::test]
    async fn set_archived_posts_archive_and_unarchive() {
        for (archived, action) in [(true, "archive"), (false, "unarchive")] {
            let (_server, client) = serve(
                Mock::given(method("POST")).and(path(format!("/v1/query/1234/{action}"))),
                json!({"query_id": 1234}),
            )
            .await;
            let r = client.queries().set_archived(1234, archived).await.unwrap();
            assert_eq!(r.query_id, Some(1234));
        }
    }

    #[tokio::test]
    async fn set_private_posts_private_and_unprivate() {
        for (private, action) in [(true, "private"), (false, "unprivate")] {
            let (_server, client) = serve(
                Mock::given(method("POST")).and(path(format!("/v1/query/1234/{action}"))),
                json!({"query_id": 1234}),
            )
            .await;
            let r = client.queries().set_private(1234, private).await.unwrap();
            assert_eq!(r.query_id, Some(1234));
        }
    }

    #[tokio::test]
    async fn existing_actions_post_to_their_routes() {
        for action in ["archive", "unarchive", "private", "unprivate"] {
            let (_server, client) = serve(
                Mock::given(method("POST")).and(path(format!("/v1/query/1234/{action}"))),
                json!({"query_id": 1234}),
            )
            .await;
            let queries = client.queries();
            match action {
                "archive" => queries.archive(1234).await,
                "unarchive" => queries.unarchive(1234).await,
                "private" => queries.make_private(1234).await,
                _ => queries.make_public(1234).await,
            }
            .unwrap();
        }
    }

    #[tokio::test]
    async fn set_archived_maps_404_to_not_found() {
        let (_server, client) = failing(404).await;
        let err = client.queries().set_archived(1234, true).await.unwrap_err();
        assert!(is_not_found(&err), "{err}");
    }
}
