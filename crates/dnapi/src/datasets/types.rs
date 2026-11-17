//! Types for the Datasets API

use crate::tables::TableOwner;
use serde::{Deserialize, Serialize};

/// Column metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DatasetColumnMetadata {
    /// Column description
    pub description: Option<String>,
    /// Whether the column is a good filter (e.g. a partition column)
    pub filtering_column: Option<bool>,
}

/// A dataset column
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DatasetColumn {
    /// Column name
    pub name: Option<String>,
    /// Column type
    #[serde(rename = "type")]
    pub column_type: Option<String>,
    /// Whether the column is nullable
    pub nullable: Option<bool>,
    /// Column metadata
    pub metadata: Option<DatasetColumnMetadata>,
}

/// A dataset with its columns
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Dataset {
    /// Full name, e.g. `dex.trades`
    pub full_name: Option<String>,
    /// Dataset type, e.g. `spell`, `decoded_table`, `uploaded_table`
    #[serde(rename = "type")]
    pub dataset_type: Option<String>,
    /// Whether the dataset is private
    pub is_private: Option<bool>,
    /// Owner
    pub owner: Option<TableOwner>,
    /// Columns
    #[serde(default)]
    pub columns: Vec<DatasetColumn>,
    /// Dataset metadata (shape depends on the dataset type)
    pub metadata: Option<serde_json::Value>,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// Response from listing datasets
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ListDatasetsResponse {
    /// Datasets
    #[serde(default)]
    pub datasets: Vec<Dataset>,
    /// Total number of matching datasets
    pub total: Option<i64>,
}

/// Options for listing datasets
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ListDatasetsOptions {
    /// Maximum number of results (default 50, max 250)
    pub limit: Option<u32>,
    /// Pagination offset
    pub offset: Option<i64>,
    /// Only datasets owned by this handle
    pub owner_handle: Option<String>,
    /// Only these dataset types, comma-separated: `transformation_view`,
    /// `transformation_table`, `uploaded_table`, `decoded_table`, `spell`,
    /// `dune_table`
    pub dataset_type: Option<String>,
}

impl ListDatasetsOptions {
    /// Options with nothing set
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Query pairs for the request (values are URL-encoded by the HTTP client)
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(offset) = self.offset {
            params.push(("offset", offset.to_string()));
        }
        if let Some(ref owner) = self.owner_handle {
            params.push(("owner_handle", owner.clone()));
        }
        if let Some(ref kind) = self.dataset_type {
            params.push(("type", kind.clone()));
        }
        params
    }
}

/// Request to search the catalog (`POST /v1/datasets/search`)
///
/// Every field is optional; unset fields are not sent.
#[derive(Debug, Clone, Default, Serialize)]
#[non_exhaustive]
pub struct SearchDatasetsRequest {
    /// Free-text search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Only these categories
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    /// Only these blockchains
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockchains: Option<Vec<String>>,
    /// Only these schemas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<Vec<String>>,
    /// Only these dataset types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_types: Option<Vec<String>>,
    /// Owner scope filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_scope: Option<String>,
    /// Include private datasets the API key can see
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_private: Option<bool>,
    /// Include each result's schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_schema: Option<bool>,
    /// Include each result's metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_metadata: Option<bool>,
    /// Maximum number of results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination offset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

impl SearchDatasetsRequest {
    /// Search for `query`
    #[must_use]
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: Some(query.into()),
            ..Self::default()
        }
    }

    /// Only these blockchains
    #[must_use]
    pub fn blockchains(mut self, blockchains: Vec<String>) -> Self {
        self.blockchains = Some(blockchains);
        self
    }

    /// Set the page size and offset
    #[must_use]
    pub fn page(mut self, limit: u32, offset: i64) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

/// Request to find the decoded datasets of a contract
/// (`POST /v1/datasets/search-by-contract`)
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct SearchByContractRequest {
    /// Contract address
    pub contract_address: String,
    /// Only these blockchains
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockchains: Option<Vec<String>>,
    /// Include each result's schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_schema: Option<bool>,
    /// Maximum number of results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination offset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

impl SearchByContractRequest {
    /// Find the datasets of `contract_address` on any chain
    #[must_use]
    pub fn new(contract_address: impl Into<String>) -> Self {
        Self {
            contract_address: contract_address.into(),
            blockchains: None,
            include_schema: None,
            limit: None,
            offset: None,
        }
    }

    /// Only these blockchains
    #[must_use]
    pub fn blockchains(mut self, blockchains: Vec<String>) -> Self {
        self.blockchains = Some(blockchains);
        self
    }
}

/// Metadata of a search result
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DatasetSearchMetadata {
    /// Description
    pub description: Option<String>,
    /// Project (namespace) of a decoded table
    pub project_name: Option<String>,
    /// Contract name of a decoded table
    pub contract_name: Option<String>,
    /// ABI type of a decoded table (e.g. event or call)
    pub abi_type: Option<String>,
    /// Spell type
    pub spell_type: Option<String>,
    /// Spell metadata (the spec gives no usable shape, so kept as JSON)
    pub spell_metadata: Option<serde_json::Value>,
    /// Ranking score
    pub page_rank_score: Option<f64>,
}

/// A dataset found by a search
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DatasetSearchResult {
    /// Full name, e.g. `uniswap_v3_ethereum.Pair_evt_Swap`
    pub full_name: Option<String>,
    /// Category
    pub category: Option<String>,
    /// Dataset type
    pub dataset_type: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Blockchains the dataset covers
    #[serde(default)]
    pub blockchains: Vec<String>,
    /// Owner scope
    pub owner_scope: Option<String>,
    /// Visibility
    pub visibility: Option<String>,
    /// Schema, when requested (the spec gives no usable shape, so kept as
    /// JSON)
    pub schema: Option<serde_json::Value>,
    /// Metadata, when requested
    pub metadata: Option<DatasetSearchMetadata>,
}

/// Search pagination
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SearchPagination {
    /// Page size
    pub limit: Option<i64>,
    /// Offset of this page
    pub offset: Option<i64>,
    /// Offset of the next page
    pub next_offset: Option<i64>,
    /// Whether more results exist
    pub has_more: Option<bool>,
}

/// Response from a dataset search
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SearchDatasetsResponse {
    /// Matching datasets
    #[serde(default)]
    pub results: Vec<DatasetSearchResult>,
    /// Total number of matches
    pub total: Option<i64>,
    /// Pagination
    pub pagination: Option<SearchPagination>,
}
