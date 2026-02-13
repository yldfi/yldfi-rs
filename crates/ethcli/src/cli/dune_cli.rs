//! Direct Dune Analytics API commands
//!
//! Provides 1:1 access to Dune Analytics API endpoints.

use crate::cli::OutputFormat;
use crate::config::ConfigFile;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct DuneArgs {
    /// Output format
    #[arg(long, short = 'o', visible_alias = "output", default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum DuneCommands {
    /// Execute queries and get results
    Query {
        #[command(subcommand)]
        action: QueryCommands,

        #[command(flatten)]
        args: DuneArgs,
    },

    /// Execute raw SQL
    Sql {
        #[command(subcommand)]
        action: SqlCommands,

        #[command(flatten)]
        args: DuneArgs,
    },

    /// Execution management
    Execution {
        #[command(subcommand)]
        action: ExecutionCommands,

        #[command(flatten)]
        args: DuneArgs,
    },

    /// Query management (CRUD)
    Queries {
        #[command(subcommand)]
        action: QueriesCommands,

        #[command(flatten)]
        args: DuneArgs,
    },

    /// User table management
    Tables {
        #[command(subcommand)]
        action: TablesCommands,

        #[command(flatten)]
        args: DuneArgs,
    },

    /// Materialized views management
    #[command(visible_alias = "mv")]
    Matviews {
        #[command(subcommand)]
        action: MatviewsCommands,

        #[command(flatten)]
        args: DuneArgs,
    },

    /// Pipeline execution
    Pipelines {
        #[command(subcommand)]
        action: PipelinesCommands,

        #[command(flatten)]
        args: DuneArgs,
    },

    /// Usage and billing information
    Usage {
        #[command(flatten)]
        args: DuneArgs,
    },
}

#[derive(Subcommand)]
pub enum QueryCommands {
    /// Execute a saved query by ID
    Execute {
        /// Query ID
        query_id: i64,
        /// Timeout in seconds (default: 300)
        #[arg(long, default_value = "300")]
        timeout: u64,
    },

    /// Get cached results for a query
    Results {
        /// Query ID
        query_id: i64,
        /// Limit number of rows
        #[arg(long)]
        limit: Option<u32>,
        /// Offset for pagination
        #[arg(long)]
        offset: Option<i64>,
    },

    /// Get results as CSV
    Csv {
        /// Query ID
        query_id: i64,
        /// Limit number of rows
        #[arg(long)]
        limit: Option<u32>,
    },
}

#[derive(Subcommand)]
pub enum SqlCommands {
    /// Execute raw SQL query
    Execute {
        /// SQL query to execute
        sql: String,
        /// Timeout in seconds (default: 300)
        #[arg(long, default_value = "300")]
        timeout: u64,
    },
}

#[derive(Subcommand)]
pub enum ExecutionCommands {
    /// Get execution status
    Status {
        /// Execution ID
        execution_id: String,
    },

    /// Get execution results
    Results {
        /// Execution ID
        execution_id: String,
        /// Limit number of rows
        #[arg(long)]
        limit: Option<u32>,
        /// Offset for pagination
        #[arg(long)]
        offset: Option<i64>,
    },

    /// Get execution results as CSV
    Csv {
        /// Execution ID
        execution_id: String,
    },

    /// Cancel a running execution
    Cancel {
        /// Execution ID
        execution_id: String,
    },
}

// ============================================================================
// Queries CRUD Commands
// ============================================================================

#[derive(Subcommand)]
pub enum QueriesCommands {
    /// Create a new saved query
    Create {
        /// Query name
        #[arg(long)]
        name: String,
        /// SQL query text
        #[arg(long)]
        sql: String,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Make query private
        #[arg(long)]
        private: bool,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },

    /// Get query details by ID
    Get {
        /// Query ID
        query_id: i64,
    },

    /// Update an existing query
    Update {
        /// Query ID
        query_id: i64,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New SQL
        #[arg(long)]
        sql: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },

    /// List all queries
    List {
        /// Limit results
        #[arg(long)]
        limit: Option<u32>,
        /// Offset for pagination
        #[arg(long)]
        offset: Option<i64>,
    },

    /// Archive a query
    Archive {
        /// Query ID
        query_id: i64,
    },

    /// Unarchive a query
    Unarchive {
        /// Query ID
        query_id: i64,
    },

    /// Make a query private
    MakePrivate {
        /// Query ID
        query_id: i64,
    },

    /// Make a query public
    MakePublic {
        /// Query ID
        query_id: i64,
    },
}

// ============================================================================
// Tables Commands
// ============================================================================

#[derive(Subcommand)]
pub enum TablesCommands {
    /// Create a new table
    Create {
        /// Namespace (your username or team handle)
        namespace: String,
        /// Table name
        table_name: String,
        /// Schema as JSON array: [{"name":"col1","column_type":"varchar","nullable":false}]
        #[arg(long)]
        schema: String,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Make table private
        #[arg(long)]
        private: bool,
    },

    /// Upload CSV data to a table
    UploadCsv {
        /// Table name (namespace.table_name)
        table_name: String,
        /// CSV file path or inline CSV data
        data: String,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Make table private
        #[arg(long)]
        private: bool,
        /// Read data from file instead of inline
        #[arg(long)]
        file: bool,
    },

    /// List all tables
    List {
        /// Limit results
        #[arg(long)]
        limit: Option<u32>,
        /// Offset for pagination
        #[arg(long)]
        offset: Option<i64>,
    },

    /// Get table details
    Get {
        /// Namespace
        namespace: String,
        /// Table name
        table_name: String,
    },

    /// Insert rows into a table
    Insert {
        /// Namespace
        namespace: String,
        /// Table name
        table_name: String,
        /// Data as JSON array: [{"col1":"val1","col2":123}]
        data: String,
    },

    /// Clear all data from a table
    Clear {
        /// Namespace
        namespace: String,
        /// Table name
        table_name: String,
    },

    /// Delete a table entirely
    Delete {
        /// Namespace
        namespace: String,
        /// Table name
        table_name: String,
    },
}

// ============================================================================
// Materialized Views Commands
// ============================================================================

#[derive(Subcommand)]
pub enum MatviewsCommands {
    /// Create or update a materialized view
    Upsert {
        /// View name
        name: String,
        /// Query ID to materialize
        #[arg(long)]
        query_id: i64,
        /// Cron expression for refresh schedule (e.g., "0 */6 * * *")
        #[arg(long)]
        cron: Option<String>,
        /// Expiration time (ISO timestamp)
        #[arg(long)]
        expires_at: Option<String>,
        /// Make view private
        #[arg(long)]
        private: bool,
        /// Performance tier: medium or large
        #[arg(long)]
        performance: Option<String>,
    },

    /// Get materialized view details
    Get {
        /// View name
        name: String,
    },

    /// List materialized views
    List {
        /// Limit results
        #[arg(long)]
        limit: Option<u32>,
        /// Offset for pagination
        #[arg(long)]
        offset: Option<i64>,
    },

    /// Refresh a materialized view
    Refresh {
        /// View name
        name: String,
        /// Performance tier: medium or large
        #[arg(long)]
        performance: Option<String>,
    },

    /// Delete a materialized view
    Delete {
        /// View name
        name: String,
    },
}

// ============================================================================
// Pipelines Commands
// ============================================================================

#[derive(Subcommand)]
pub enum PipelinesCommands {
    /// Execute a pipeline of queries/matviews
    Execute {
        /// Pipeline definition as JSON: {"nodes":[{"node_type":"query_execution","query_id":123}]}
        pipeline_json: String,
        /// Query parameters as JSON: {"key":"value"}
        #[arg(long)]
        params: Option<String>,
        /// Performance tier: medium or large
        #[arg(long)]
        performance: Option<String>,
    },

    /// Get pipeline execution status
    Status {
        /// Pipeline execution ID
        execution_id: String,
    },
}

// ============================================================================
// Handler
// ============================================================================

/// Handle Dune Analytics commands
pub async fn handle(command: &DuneCommands, quiet: bool) -> anyhow::Result<()> {
    use secrecy::ExposeSecret;

    // Try config first, then fall back to env var
    let api_key = if let Ok(Some(config)) = ConfigFile::load_default() {
        if let Some(ref dune_config) = config.dune {
            dune_config.api_key.expose_secret().to_string()
        } else {
            std::env::var("DUNE_API_KEY")
                .map_err(|_| anyhow::anyhow!("DUNE_API_KEY not set in config or environment"))?
        }
    } else {
        std::env::var("DUNE_API_KEY")
            .map_err(|_| anyhow::anyhow!("DUNE_API_KEY not set in config or environment"))?
    };

    let client = dnapi::Client::new(&api_key)?;

    match command {
        DuneCommands::Query { action, args } => handle_query(&client, action, args, quiet).await,
        DuneCommands::Sql { action, args } => handle_sql(&client, action, args, quiet).await,
        DuneCommands::Execution { action, args } => {
            handle_execution(&client, action, args, quiet).await
        }
        DuneCommands::Queries { action, args } => {
            handle_queries(&client, action, args, quiet).await
        }
        DuneCommands::Tables { action, args } => handle_tables(&client, action, args, quiet).await,
        DuneCommands::Matviews { action, args } => {
            handle_matviews(&client, action, args, quiet).await
        }
        DuneCommands::Pipelines { action, args } => {
            handle_pipelines(&client, action, args, quiet).await
        }
        DuneCommands::Usage { args } => handle_usage(&client, args, quiet).await,
    }
}

async fn handle_query(
    client: &dnapi::Client,
    action: &QueryCommands,
    args: &DuneArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        QueryCommands::Execute { query_id, timeout } => {
            if !quiet {
                eprintln!("Executing query {}...", query_id);
            }
            let response = client
                .executions()
                .run_query(*query_id, Some(*timeout))
                .await?;
            print_output(&response, args.format)?;
        }
        QueryCommands::Results {
            query_id,
            limit,
            offset,
        } => {
            if !quiet {
                eprintln!("Fetching results for query {}...", query_id);
            }
            let mut opts = dnapi::executions::GetResultsOptions::default();
            if let Some(l) = limit {
                opts.limit = Some(*l);
            }
            if let Some(o) = offset {
                opts.offset = Some(*o);
            }
            let response = client
                .executions()
                .query_results_with_options(*query_id, &opts)
                .await?;
            print_output(&response, args.format)?;
        }
        QueryCommands::Csv { query_id, limit } => {
            if !quiet {
                eprintln!("Fetching CSV results for query {}...", query_id);
            }
            let mut opts = dnapi::executions::GetResultsOptions::default();
            if let Some(l) = limit {
                opts.limit = Some(*l);
            }
            let response = client
                .executions()
                .query_results_csv_with_options(*query_id, &opts)
                .await?;
            println!("{}", response);
        }
    }
    Ok(())
}

async fn handle_sql(
    client: &dnapi::Client,
    action: &SqlCommands,
    args: &DuneArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        SqlCommands::Execute { sql, timeout } => {
            if !quiet {
                eprintln!("Executing SQL...");
            }
            let response = client.executions().run_sql(sql, Some(*timeout)).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_execution(
    client: &dnapi::Client,
    action: &ExecutionCommands,
    args: &DuneArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        ExecutionCommands::Status { execution_id } => {
            if !quiet {
                eprintln!("Fetching status for execution {}...", execution_id);
            }
            let response = client.executions().status(execution_id).await?;
            print_output(&response, args.format)?;
        }
        ExecutionCommands::Results {
            execution_id,
            limit,
            offset,
        } => {
            if !quiet {
                eprintln!("Fetching results for execution {}...", execution_id);
            }
            let mut opts = dnapi::executions::GetResultsOptions::default();
            if let Some(l) = limit {
                opts.limit = Some(*l);
            }
            if let Some(o) = offset {
                opts.offset = Some(*o);
            }
            let response = client
                .executions()
                .results_with_options(execution_id, &opts)
                .await?;
            print_output(&response, args.format)?;
        }
        ExecutionCommands::Csv { execution_id } => {
            if !quiet {
                eprintln!("Fetching CSV results for execution {}...", execution_id);
            }
            let response = client.executions().results_csv(execution_id).await?;
            println!("{}", response);
        }
        ExecutionCommands::Cancel { execution_id } => {
            if !quiet {
                eprintln!("Cancelling execution {}...", execution_id);
            }
            let response = client.executions().cancel(execution_id).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

// ============================================================================
// Queries CRUD Handler
// ============================================================================

async fn handle_queries(
    client: &dnapi::Client,
    action: &QueriesCommands,
    args: &DuneArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        QueriesCommands::Create {
            name,
            sql,
            description,
            private,
            tags,
        } => {
            if !quiet {
                eprintln!("Creating query '{}'...", name);
            }
            let mut request = dnapi::queries::CreateQueryRequest {
                name: name.clone(),
                query_sql: sql.clone(),
                description: description.clone(),
                parameters: None,
                is_private: if *private { Some(true) } else { None },
                tags: tags
                    .as_ref()
                    .map(|t| t.split(',').map(|s| s.trim().to_string()).collect()),
            };
            let _ = &mut request; // suppress unused mut warning if needed
            let response = client.queries().create(&request).await?;
            print_output(&response, args.format)?;
        }

        QueriesCommands::Get { query_id } => {
            if !quiet {
                eprintln!("Getting query {}...", query_id);
            }
            let response = client.queries().get(*query_id).await?;
            print_output(&response, args.format)?;
        }

        QueriesCommands::Update {
            query_id,
            name,
            sql,
            description,
            tags,
        } => {
            if !quiet {
                eprintln!("Updating query {}...", query_id);
            }
            let request = dnapi::queries::UpdateQueryRequest {
                name: name.clone(),
                query_sql: sql.clone(),
                description: description.clone(),
                parameters: None,
                tags: tags
                    .as_ref()
                    .map(|t| t.split(',').map(|s| s.trim().to_string()).collect()),
            };
            let response = client.queries().update(*query_id, &request).await?;
            print_output(&response, args.format)?;
        }

        QueriesCommands::List { limit, offset } => {
            if !quiet {
                eprintln!("Listing queries...");
            }
            if limit.is_some() || offset.is_some() {
                let opts = dnapi::queries::ListQueriesOptions {
                    limit: *limit,
                    offset: *offset,
                };
                let response = client.queries().list_with_options(&opts).await?;
                print_output(&response, args.format)?;
            } else {
                let response = client.queries().list().await?;
                print_output(&response, args.format)?;
            }
        }

        QueriesCommands::Archive { query_id } => {
            if !quiet {
                eprintln!("Archiving query {}...", query_id);
            }
            client.queries().archive(*query_id).await?;
            println!("Query {} archived.", query_id);
        }

        QueriesCommands::Unarchive { query_id } => {
            if !quiet {
                eprintln!("Unarchiving query {}...", query_id);
            }
            client.queries().unarchive(*query_id).await?;
            println!("Query {} unarchived.", query_id);
        }

        QueriesCommands::MakePrivate { query_id } => {
            if !quiet {
                eprintln!("Making query {} private...", query_id);
            }
            client.queries().make_private(*query_id).await?;
            println!("Query {} is now private.", query_id);
        }

        QueriesCommands::MakePublic { query_id } => {
            if !quiet {
                eprintln!("Making query {} public...", query_id);
            }
            client.queries().make_public(*query_id).await?;
            println!("Query {} is now public.", query_id);
        }
    }
    Ok(())
}

// ============================================================================
// Tables Handler
// ============================================================================

async fn handle_tables(
    client: &dnapi::Client,
    action: &TablesCommands,
    args: &DuneArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        TablesCommands::Create {
            namespace,
            table_name,
            schema,
            description,
            private,
        } => {
            if !quiet {
                eprintln!("Creating table {}.{}...", namespace, table_name);
            }
            let columns: Vec<dnapi::tables::TableColumn> = serde_json::from_str(schema)
                .map_err(|e| anyhow::anyhow!("Invalid schema JSON: {}", e))?;
            let request = dnapi::tables::CreateTableRequest {
                namespace: namespace.clone(),
                table_name: table_name.clone(),
                schema: columns,
                description: description.clone(),
                is_private: if *private { Some(true) } else { None },
            };
            let response = client.tables().create(&request).await?;
            print_output(&response, args.format)?;
        }

        TablesCommands::UploadCsv {
            table_name,
            data,
            description,
            private,
            file,
        } => {
            if !quiet {
                eprintln!("Uploading CSV to {}...", table_name);
            }
            let csv_data = if *file {
                std::fs::read_to_string(data)
                    .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", data, e))?
            } else {
                data.clone()
            };
            let request = dnapi::tables::UploadCsvRequest {
                table_name: table_name.clone(),
                data: csv_data,
                description: description.clone(),
                is_private: if *private { Some(true) } else { None },
            };
            let response = client.tables().upload_csv(&request).await?;
            print_output(&response, args.format)?;
        }

        TablesCommands::List { limit, offset } => {
            if !quiet {
                eprintln!("Listing tables...");
            }
            if limit.is_some() || offset.is_some() {
                let opts = dnapi::tables::ListTablesOptions {
                    limit: *limit,
                    offset: *offset,
                };
                let response = client.tables().list_with_options(&opts).await?;
                print_output(&response, args.format)?;
            } else {
                let response = client.tables().list().await?;
                print_output(&response, args.format)?;
            }
        }

        TablesCommands::Get {
            namespace,
            table_name,
        } => {
            if !quiet {
                eprintln!("Getting table {}.{}...", namespace, table_name);
            }
            let response = client.tables().get(namespace, table_name).await?;
            print_output(&response, args.format)?;
        }

        TablesCommands::Insert {
            namespace,
            table_name,
            data,
        } => {
            if !quiet {
                eprintln!("Inserting data into {}.{}...", namespace, table_name);
            }
            let json_data: serde_json::Value = serde_json::from_str(data)
                .map_err(|e| anyhow::anyhow!("Invalid data JSON: {}", e))?;
            let response = client
                .tables()
                .insert(namespace, table_name, &json_data)
                .await?;
            print_output(&response, args.format)?;
        }

        TablesCommands::Clear {
            namespace,
            table_name,
        } => {
            if !quiet {
                eprintln!("Clearing table {}.{}...", namespace, table_name);
            }
            let response = client.tables().clear(namespace, table_name).await?;
            print_output(&response, args.format)?;
        }

        TablesCommands::Delete {
            namespace,
            table_name,
        } => {
            if !quiet {
                eprintln!("Deleting table {}.{}...", namespace, table_name);
            }
            let response = client.tables().delete(namespace, table_name).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

// ============================================================================
// Materialized Views Handler
// ============================================================================

async fn handle_matviews(
    client: &dnapi::Client,
    action: &MatviewsCommands,
    args: &DuneArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        MatviewsCommands::Upsert {
            name,
            query_id,
            cron,
            expires_at,
            private,
            performance,
        } => {
            if !quiet {
                eprintln!("Creating/updating materialized view '{}'...", name);
            }
            let request = dnapi::matviews::UpsertMatviewRequest {
                name: name.clone(),
                query_id: *query_id,
                cron_expression: cron.clone(),
                expires_at: expires_at.clone(),
                is_private: if *private { Some(true) } else { None },
                performance: performance.clone(),
            };
            let response = client.matviews().upsert(&request).await?;
            print_output(&response, args.format)?;
        }

        MatviewsCommands::Get { name } => {
            if !quiet {
                eprintln!("Getting materialized view '{}'...", name);
            }
            let response = client.matviews().get(name).await?;
            print_output(&response, args.format)?;
        }

        MatviewsCommands::List { limit, offset } => {
            if !quiet {
                eprintln!("Listing materialized views...");
            }
            if limit.is_some() || offset.is_some() {
                let opts = dnapi::matviews::ListMatviewsOptions {
                    limit: *limit,
                    offset: *offset,
                };
                let response = client.matviews().list_with_options(&opts).await?;
                print_output(&response, args.format)?;
            } else {
                let response = client.matviews().list().await?;
                print_output(&response, args.format)?;
            }
        }

        MatviewsCommands::Refresh { name, performance } => {
            if !quiet {
                eprintln!("Refreshing materialized view '{}'...", name);
            }
            if let Some(perf) = performance {
                let request = dnapi::matviews::RefreshMatviewRequest {
                    performance: Some(perf.clone()),
                };
                let response = client
                    .matviews()
                    .refresh_with_options(name, &request)
                    .await?;
                print_output(&response, args.format)?;
            } else {
                let response = client.matviews().refresh(name).await?;
                print_output(&response, args.format)?;
            }
        }

        MatviewsCommands::Delete { name } => {
            if !quiet {
                eprintln!("Deleting materialized view '{}'...", name);
            }
            let response = client.matviews().delete(name).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

// ============================================================================
// Pipelines Handler
// ============================================================================

async fn handle_pipelines(
    client: &dnapi::Client,
    action: &PipelinesCommands,
    args: &DuneArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        PipelinesCommands::Execute {
            pipeline_json,
            params,
            performance,
        } => {
            if !quiet {
                eprintln!("Executing pipeline...");
            }
            let pipeline: dnapi::pipelines::Pipeline = serde_json::from_str(pipeline_json)
                .map_err(|e| anyhow::anyhow!("Invalid pipeline JSON: {}", e))?;
            let query_parameters = if let Some(p) = params {
                Some(
                    serde_json::from_str(p)
                        .map_err(|e| anyhow::anyhow!("Invalid params JSON: {}", e))?,
                )
            } else {
                None
            };
            let request = dnapi::pipelines::ExecutePipelineRequest {
                pipeline,
                query_parameters,
                performance: performance.clone(),
            };
            let response = client.pipelines().execute(&request).await?;
            print_output(&response, args.format)?;
        }

        PipelinesCommands::Status { execution_id } => {
            if !quiet {
                eprintln!("Getting pipeline status {}...", execution_id);
            }
            let response = client.pipelines().status(execution_id).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

// ============================================================================
// Usage Handler
// ============================================================================

async fn handle_usage(client: &dnapi::Client, args: &DuneArgs, quiet: bool) -> anyhow::Result<()> {
    if !quiet {
        eprintln!("Fetching usage information...");
    }
    let response = client.usage().get().await?;
    print_output(&response, args.format)?;
    Ok(())
}

fn print_output<T: serde::Serialize>(data: &T, format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        OutputFormat::Ndjson => {
            println!("{}", serde_json::to_string(data)?);
        }
        OutputFormat::Table => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    }
    Ok(())
}
