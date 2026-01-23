//! Direct Dune Analytics API commands
//!
//! Provides 1:1 access to Dune Analytics API endpoints.

use crate::cli::OutputFormat;
use crate::config::ConfigFile;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct DuneArgs {
    /// Output format
    #[arg(long, short = 'o', default_value = "json")]
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
