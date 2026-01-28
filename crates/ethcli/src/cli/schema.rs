//! JSON schema generation for CLI commands
//!
//! Provides machine-readable command schema for LLM discoverability.

use clap::{Arg, ArgAction, Command, CommandFactory};
use serde::Serialize;

/// Schema for a command argument
#[derive(Debug, Clone, Serialize)]
pub struct ArgSchema {
    /// Argument name (long form)
    pub name: String,
    /// Short form (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<char>,
    /// Description from help text
    pub description: String,
    /// Whether the argument is required
    pub required: bool,
    /// Value type hint
    pub value_type: String,
    /// Default value (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// Environment variable (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    /// Whether this is a flag (no value)
    pub is_flag: bool,
    /// Whether this argument is global
    pub global: bool,
    /// Possible values (for enums)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub possible_values: Vec<String>,
}

/// Schema for a command
#[derive(Debug, Clone, Serialize)]
pub struct CommandSchema {
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Visible aliases
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    /// Positional arguments
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub positional_args: Vec<ArgSchema>,
    /// Named arguments (flags and options)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<ArgSchema>,
    /// Subcommands
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subcommands: Vec<CommandSchema>,
}

/// Root schema for the entire CLI
#[derive(Debug, Clone, Serialize)]
pub struct CliSchema {
    /// CLI name
    pub name: String,
    /// Version
    pub version: String,
    /// Description
    pub description: String,
    /// Global options
    pub global_options: Vec<ArgSchema>,
    /// Top-level commands
    pub commands: Vec<CommandSchema>,
}

impl ArgSchema {
    /// Build an ArgSchema from a clap Arg
    fn from_clap_arg(arg: &Arg) -> Self {
        let name = arg
            .get_long()
            .map(|s| s.to_string())
            .or_else(|| arg.get_id().as_str().to_string().into())
            .unwrap_or_default();

        let short = arg.get_short();

        let description = arg.get_help().map(|s| s.to_string()).unwrap_or_default();

        let required = arg.is_required_set();

        // Determine value type from action
        let (value_type, is_flag) = match arg.get_action() {
            ArgAction::SetTrue | ArgAction::SetFalse => ("bool".to_string(), true),
            ArgAction::Count => ("count".to_string(), true),
            ArgAction::Set | ArgAction::Append => {
                let type_hint = arg
                    .get_value_names()
                    .map(|names| {
                        names
                            .iter()
                            .map(|n| n.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_else(|| "string".to_string());
                (type_hint, false)
            }
            _ => ("string".to_string(), false),
        };

        let default = arg
            .get_default_values()
            .first()
            .map(|v| v.to_string_lossy().to_string());

        let env = arg.get_env().map(|e| e.to_string_lossy().to_string());

        let global = arg.is_global_set();

        let possible_values: Vec<String> = arg
            .get_possible_values()
            .iter()
            .map(|v| v.get_name().to_string())
            .collect();

        ArgSchema {
            name,
            short,
            description,
            required,
            value_type,
            default,
            env,
            is_flag,
            global,
            possible_values,
        }
    }
}

impl CommandSchema {
    /// Build a CommandSchema from a clap Command
    fn from_clap_command(cmd: &Command) -> Self {
        let name = cmd.get_name().to_string();

        let description = cmd.get_about().map(|s| s.to_string()).unwrap_or_default();

        let aliases: Vec<String> = cmd.get_visible_aliases().map(|s| s.to_string()).collect();

        let mut positional_args = Vec::new();
        let mut options = Vec::new();

        for arg in cmd.get_arguments() {
            // Skip help and version
            if arg.get_id() == "help" || arg.get_id() == "version" {
                continue;
            }

            let schema = ArgSchema::from_clap_arg(arg);

            // Positional args don't have long/short names
            if arg.get_long().is_none() && arg.get_short().is_none() && !arg.is_global_set() {
                positional_args.push(schema);
            } else {
                options.push(schema);
            }
        }

        let subcommands: Vec<CommandSchema> = cmd
            .get_subcommands()
            .filter(|sc| sc.get_name() != "help")
            .map(CommandSchema::from_clap_command)
            .collect();

        CommandSchema {
            name,
            description,
            aliases,
            positional_args,
            options,
            subcommands,
        }
    }
}

impl CliSchema {
    /// Build the complete CLI schema from the Cli type
    pub fn from_cli<T: CommandFactory>() -> Self {
        let cmd = T::command();

        let name = cmd.get_name().to_string();
        let version = cmd
            .get_version()
            .map(|s| s.to_string())
            .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
        let description = cmd.get_about().map(|s| s.to_string()).unwrap_or_default();

        // Collect global options
        let global_options: Vec<ArgSchema> = cmd
            .get_arguments()
            .filter(|arg| {
                arg.is_global_set() && arg.get_id() != "help" && arg.get_id() != "version"
            })
            .map(ArgSchema::from_clap_arg)
            .collect();

        // Collect subcommands
        let commands: Vec<CommandSchema> = cmd
            .get_subcommands()
            .filter(|sc| sc.get_name() != "help")
            .map(CommandSchema::from_clap_command)
            .collect();

        CliSchema {
            name,
            version,
            description,
            global_options,
            commands,
        }
    }

    /// Generate JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Generate compact JSON string
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Get schema for a specific subcommand path (e.g., "tx" or "account balance")
pub fn get_subcommand_schema<T: CommandFactory>(path: &[&str]) -> Option<CommandSchema> {
    let cmd = T::command();

    let mut current = &cmd;
    for name in path {
        current = current
            .get_subcommands()
            .find(|sc| sc.get_name() == *name)?;
    }

    Some(CommandSchema::from_clap_command(current))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;

    #[test]
    fn test_schema_generation() {
        let schema = CliSchema::from_cli::<Cli>();
        assert_eq!(schema.name, "ethcli");
        assert!(!schema.commands.is_empty());
        assert!(!schema.global_options.is_empty());
    }

    #[test]
    fn test_schema_to_json() {
        let schema = CliSchema::from_cli::<Cli>();
        let json = schema.to_json().expect("JSON generation should succeed");
        assert!(json.contains("ethcli"));
        assert!(json.contains("commands"));
    }

    #[test]
    fn test_subcommand_schema() {
        let schema = get_subcommand_schema::<Cli>(&["tx"]);
        assert!(schema.is_some());
        let tx_schema = schema.unwrap();
        assert_eq!(tx_schema.name, "tx");
    }
}
