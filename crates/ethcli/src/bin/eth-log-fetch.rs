//! Backward compatibility wrapper for eth-log-fetch
//!
//! This binary forwards all arguments to `ethcli logs` for users who have
//! scripts or muscle memory using the old `eth-log-fetch` command.

use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!("Note: eth-log-fetch is now ethcli. Use `ethcli logs` instead.");
    eprintln!("      This wrapper will be removed in a future version.\n");

    // Get all args after the program name
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Build new command: ethcli logs <args...>
    let status = std::process::Command::new("ethcli")
        .arg("logs")
        .args(&args)
        .status();

    match status {
        Ok(s) => {
            if s.success() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(s.code().unwrap_or(1) as u8)
            }
        }
        Err(e) => {
            eprintln!("Failed to run ethcli: {}", e);
            ExitCode::FAILURE
        }
    }
}
