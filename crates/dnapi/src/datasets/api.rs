//! Datasets API implementation

use super::types::{
    Dataset, ListDatasetsOptions, ListDatasetsResponse, SearchByContractRequest,
    SearchDatasetsRequest, SearchDatasetsResponse,
};
use crate::client::Client;
use crate::error::Result;

/// Datasets API
pub struct DatasetsApi<'a> {
    client: &'a Client,
}

impl<'a> DatasetsApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List datasets, optionally filtered by owner and type
    ///
    /// Calls `GET /v1/datasets`.
    pub async fn list(&self, options: &ListDatasetsOptions) -> Result<ListDatasetsResponse> {
        let url = self.client.url("/v1/datasets");
        let request = self
            .client
            .http()
            .get(&url)
            .query(&options.to_query_pairs());
        self.client.send_json(request, None).await
    }

    /// Get a dataset, with its columns, by slug (e.g. `dex.trades`)
    ///
    /// Calls `GET /v1/datasets/{slug}`.
    pub async fn get(&self, slug: &str) -> Result<Dataset> {
        let url = self.client.url(&format!("/v1/datasets/{slug}"));
        self.client
            .send_json(
                self.client.http().get(&url),
                Some(format!("Dataset {slug}")),
            )
            .await
    }

    /// Search the catalog by text, category, blockchain, schema and more
    ///
    /// Calls `POST /v1/datasets/search`.
    pub async fn search(&self, request: &SearchDatasetsRequest) -> Result<SearchDatasetsResponse> {
        let url = self.client.url("/v1/datasets/search");
        self.client
            .send_json(self.client.http().post(&url).json(request), None)
            .await
    }

    /// Find the decoded datasets of a contract address
    ///
    /// Calls `POST /v1/datasets/search-by-contract`.
    pub async fn search_by_contract(
        &self,
        request: &SearchByContractRequest,
    ) -> Result<SearchDatasetsResponse> {
        let url = self.client.url("/v1/datasets/search-by-contract");
        self.client
            .send_json(self.client.http().post(&url).json(request), None)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use crate::test_util::serve;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path, query_param};
    use wiremock::Mock;

    fn dataset_json() -> serde_json::Value {
        json!({
            "full_name": "dex.trades",
            "type": "spell",
            "is_private": false,
            "owner": {"handle": "dune", "type": "team"},
            "columns": [{
                "name": "block_time", "type": "timestamp", "nullable": false,
                "metadata": {"description": "Block time", "filtering_column": true}
            }],
            "metadata": null,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-02T00:00:00Z"
        })
    }

    fn search_json() -> serde_json::Value {
        json!({
            "results": [{
                "full_name": "uniswap_v3_ethereum.pair_evt_swap",
                "category": "decoded",
                "dataset_type": "decoded_table",
                "blockchains": ["ethereum"],
                "visibility": "public",
                "schema": [{"name": "amount0", "type": "int256"}],
                "metadata": {"project_name": "uniswap_v3", "contract_name": "Pair",
                             "abi_type": "event", "page_rank_score": 0.75}
            }],
            "total": 1,
            "pagination": {"limit": 10, "offset": 0, "has_more": false}
        })
    }

    #[tokio::test]
    async fn list_sends_filters() {
        let (_server, client) = serve(
            Mock::given(method("GET"))
                .and(path("/v1/datasets"))
                .and(query_param("limit", "10"))
                .and(query_param("owner_handle", "dune"))
                .and(query_param("type", "spell,dune_table")),
            json!({"datasets": [dataset_json()], "total": 1}),
        )
        .await;
        let mut options = ListDatasetsOptions::new();
        options.limit = Some(10);
        options.owner_handle = Some("dune".into());
        options.dataset_type = Some("spell,dune_table".into());
        let r = client.datasets().list(&options).await.unwrap();
        assert_eq!(r.total, Some(1));
        let d = &r.datasets[0];
        assert_eq!(d.dataset_type.as_deref(), Some("spell"));
        assert_eq!(d.columns[0].nullable, Some(false));
        assert_eq!(
            d.columns[0].metadata.as_ref().unwrap().filtering_column,
            Some(true)
        );
    }

    #[tokio::test]
    async fn get_reads_dataset() {
        let (_server, client) = serve(
            Mock::given(method("GET")).and(path("/v1/datasets/dex.trades")),
            dataset_json(),
        )
        .await;
        let d = client.datasets().get("dex.trades").await.unwrap();
        assert_eq!(d.full_name.as_deref(), Some("dex.trades"));
        assert_eq!(d.owner.unwrap().handle.as_deref(), Some("dune"));
    }

    #[tokio::test]
    async fn search_sends_body() {
        let (_server, client) = serve(
            Mock::given(method("POST"))
                .and(path("/v1/datasets/search"))
                .and(body_json(json!({
                    "query": "uniswap swaps",
                    "blockchains": ["ethereum"],
                    "include_metadata": true,
                    "limit": 10,
                    "offset": 0
                }))),
            search_json(),
        )
        .await;
        let mut request = SearchDatasetsRequest::new("uniswap swaps")
            .blockchains(vec!["ethereum".into()])
            .page(10, 0);
        request.include_metadata = Some(true);
        let r = client.datasets().search(&request).await.unwrap();
        let hit = &r.results[0];
        assert_eq!(hit.blockchains, vec!["ethereum".to_string()]);
        assert_eq!(hit.metadata.as_ref().unwrap().page_rank_score, Some(0.75));
        assert!(hit.schema.as_ref().unwrap().is_array());
        assert_eq!(r.pagination.unwrap().has_more, Some(false));
    }

    #[tokio::test]
    async fn search_by_contract_sends_address() {
        let (_server, client) = serve(
            Mock::given(method("POST"))
                .and(path("/v1/datasets/search-by-contract"))
                .and(body_json(json!({
                    "contract_address": "0x1111111111111111111111111111111111111111",
                    "blockchains": ["ethereum"]
                }))),
            search_json(),
        )
        .await;
        let request = SearchByContractRequest::new("0x1111111111111111111111111111111111111111")
            .blockchains(vec!["ethereum".into()]);
        let r = client
            .datasets()
            .search_by_contract(&request)
            .await
            .unwrap();
        assert_eq!(r.total, Some(1));
        assert_eq!(r.results[0].dataset_type.as_deref(), Some("decoded_table"));
    }
}
