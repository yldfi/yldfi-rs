//! Types for the Tables (uploads) API

use serde::{Deserialize, Serialize};

/// Table column definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TableColumn {
    /// Column name
    pub name: String,
    /// Column type (varchar, integer, double, boolean, timestamp, etc.)
    #[serde(rename = "type")]
    pub column_type: String,
    /// Whether the column is nullable
    #[serde(default)]
    pub nullable: bool,
}

impl TableColumn {
    /// Create a new column definition
    #[must_use]
    pub fn new(name: &str, column_type: &str) -> Self {
        Self {
            name: name.to_string(),
            column_type: column_type.to_string(),
            nullable: true,
        }
    }

    /// Create a non-nullable column
    #[must_use]
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }
}

/// Table column info (from list response)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TableColumnInfo {
    /// Column name
    pub name: Option<String>,
    /// Column type
    #[serde(rename = "type")]
    pub column_type: Option<String>,
}

/// Table owner info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TableOwner {
    /// Owner handle
    pub handle: Option<String>,
    /// Owner type (user, team)
    #[serde(rename = "type")]
    pub owner_type: Option<String>,
}

/// Table list element
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Table {
    /// Full table name (catalog.schema.table)
    pub full_name: Option<String>,
    /// Table columns
    #[serde(default)]
    pub columns: Vec<TableColumnInfo>,
    /// Whether the table is private
    pub is_private: Option<bool>,
    /// Owner information
    pub owner: Option<TableOwner>,
    /// Table size in bytes
    pub table_size_bytes: Option<String>,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Last update timestamp
    pub updated_at: Option<String>,
    /// Purge timestamp
    pub purged_at: Option<String>,
}

/// Request to create a table
#[derive(Debug, Clone, Serialize)]
pub struct CreateTableRequest {
    /// Namespace (your username or team handle)
    pub namespace: String,
    /// Table name
    pub table_name: String,
    /// Table schema (columns)
    pub schema: Vec<TableColumn>,
    /// Table description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the table is private
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
}

impl CreateTableRequest {
    /// Create a new table request
    #[must_use]
    pub fn new(namespace: &str, table_name: &str, schema: Vec<TableColumn>) -> Self {
        Self {
            namespace: namespace.to_string(),
            table_name: table_name.to_string(),
            schema,
            description: None,
            is_private: None,
        }
    }

    /// Set the description
    #[must_use]
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set as private
    #[must_use]
    pub fn private(mut self, is_private: bool) -> Self {
        self.is_private = Some(is_private);
        self
    }
}

/// Response from creating a table
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateTableResponse {
    /// Full table name
    pub full_name: Option<String>,
    /// Namespace
    pub namespace: Option<String>,
    /// Table name
    pub table_name: Option<String>,
    /// Whether table already existed
    pub already_existed: Option<bool>,
    /// Example query to use the table
    pub example_query: Option<String>,
    /// Message
    pub message: Option<String>,
}

/// Request to upload CSV data
#[derive(Debug, Clone, Serialize)]
pub struct UploadCsvRequest {
    /// Table name
    pub table_name: String,
    /// CSV data
    pub data: String,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the table is private
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
}

impl UploadCsvRequest {
    /// Create a new CSV upload request
    #[must_use]
    pub fn new(table_name: &str, csv_data: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            data: csv_data.to_string(),
            description: None,
            is_private: None,
        }
    }
}

/// Response from CSV upload
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UploadCsvResponse {
    /// Whether upload was successful
    pub success: Option<bool>,
    /// Table name
    pub table_name: Option<String>,
}

/// Response from inserting data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InsertResponse {
    /// Table name
    pub name: Option<String>,
    /// Number of rows written
    pub rows_written: Option<i64>,
    /// Number of bytes written
    pub bytes_written: Option<i64>,
}

/// Response from clearing a table
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClearTableResponse {
    /// Success message
    pub message: Option<String>,
}

/// Response from deleting a table
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteTableResponse {
    /// Success message
    pub message: Option<String>,
}

/// Response from listing tables
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListTablesResponse {
    /// List of tables
    #[serde(default)]
    pub tables: Vec<Table>,
    /// Next page offset
    pub next_offset: Option<i64>,
}

/// Options for listing tables
#[derive(Debug, Clone, Default)]
pub struct ListTablesOptions {
    /// Maximum number of results
    pub limit: Option<u32>,
    /// Pagination offset
    pub offset: Option<i64>,
}

impl ListTablesOptions {
    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(offset) = self.offset {
            params.push(format!("offset={offset}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}
