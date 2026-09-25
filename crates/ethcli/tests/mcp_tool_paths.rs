//! Static regression test: every ethcli-mcp tool must invoke a real ethcli
//! command path with flags that exist on that path.
//!
//! ethcli-mcp wraps the ethcli binary as a subprocess and builds argument
//! lists with `ArgsBuilder` in `crates/ethcli-mcp/src/tools.rs`. Nothing ties
//! those strings to the clap definitions, so CLI changes silently broke MCP
//! tools (e.g. `tenderly vnets` without the `list` leaf, or `--chain` passed
//! to a command that takes the chain positionally). This test parses
//! tools.rs and checks every statically-known path against the clap
//! `Command` tree built from the ethcli library, without running the binary.

use clap::CommandFactory;
use regex::Regex;

fn tools_source() -> Option<String> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../ethcli-mcp/src/tools.rs");
    std::fs::read_to_string(path).ok()
}

/// Find a direct subcommand by name or alias.
fn find_sub<'a>(cmd: &'a clap::Command, name: &str) -> Option<&'a clap::Command> {
    cmd.get_subcommands()
        .find(|c| c.get_name() == name || c.get_all_aliases().any(|a| a == name))
}

/// Does any command along `chain` define `flag` (long, short, or alias)?
fn flag_exists(chain: &[&clap::Command], flag: &str) -> bool {
    chain.iter().any(|cmd| {
        cmd.get_arguments().any(|arg| {
            if let Some(long) = flag.strip_prefix("--") {
                arg.get_long() == Some(long)
                    || arg
                        .get_all_aliases()
                        .is_some_and(|aliases| aliases.contains(&long))
            } else if let Some(short) = flag.strip_prefix('-') {
                let mut chars = short.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => {
                        arg.get_short() == Some(c)
                            || arg
                                .get_all_short_aliases()
                                .is_some_and(|aliases| aliases.contains(&c))
                    }
                    _ => false,
                }
            } else {
                false
            }
        })
    })
}

#[derive(Debug)]
enum Step {
    New(String),
    Sub(String),
    DynamicSub,
    Flag(String),
}

fn tokenize(body: &str) -> Vec<Step> {
    let re = Regex::new(
        r#"ArgsBuilder::new\("([^"]+)"\)|\.subcommand\(\s*"([^"]+)"\s*\)|\.subcommand\(|\.opt\(\s*"([^"]+)"|\.opt_flag\(\s*"([^"]+)"|\.chain\(|\.network\(|\.format_json\(\)"#,
    )
    .unwrap();
    re.captures_iter(body)
        .map(|c| {
            if let Some(m) = c.get(1) {
                Step::New(m.as_str().to_string())
            } else if let Some(m) = c.get(2) {
                Step::Sub(m.as_str().to_string())
            } else if let Some(m) = c.get(3).or_else(|| c.get(4)) {
                Step::Flag(m.as_str().to_string())
            } else {
                let t = c.get(0).unwrap().as_str();
                if t.starts_with(".subcommand(") {
                    Step::DynamicSub
                } else if t.starts_with(".chain(") {
                    Step::Flag("--chain".into())
                } else if t.starts_with(".network(") {
                    Step::Flag("-n".into())
                } else {
                    Step::Flag("-o".into())
                }
            }
        })
        .collect()
}

/// Validate one tool function; returns a list of problems.
fn check_function(root: &clap::Command, name: &str, body: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let mut path: Vec<String> = Vec::new();
    let mut dynamic = false;
    let mut saw_builder = false;

    let resolve = |path: &[String]| -> Result<Vec<&clap::Command>, String> {
        let mut chain = vec![root];
        for (i, seg) in path.iter().enumerate() {
            let parent = *chain.last().unwrap();
            match find_sub(parent, seg) {
                Some(c) => chain.push(c),
                None => {
                    return Err(format!(
                        "`{}` is not a subcommand of `ethcli {}`",
                        seg,
                        path[..i].join(" ")
                    ))
                }
            }
        }
        Ok(chain)
    };

    let finish = |path: &[String], dynamic: bool, problems: &mut Vec<String>| {
        if path.is_empty() || dynamic {
            return;
        }
        if let Ok(chain) = resolve(path) {
            let leaf = *chain.last().unwrap();
            // Ask clap itself whether the path is runnable without a further
            // subcommand (derive does not expose this via the Command flags).
            let argv = std::iter::once("ethcli".to_string()).chain(path.iter().cloned());
            let missing_sub = matches!(
                root.clone()
                    .try_get_matches_from(argv)
                    .map_err(|e| e.kind()),
                Err(clap::error::ErrorKind::MissingSubcommand)
                    | Err(clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand)
            );
            if leaf.has_subcommands() && missing_sub {
                let subs: Vec<_> = leaf.get_subcommands().map(|c| c.get_name()).collect();
                problems.push(format!(
                    "`ethcli {}` is a command group, not a leaf (needs one of {:?})",
                    path.join(" "),
                    subs
                ));
            }
        }
    };

    for step in tokenize(body) {
        match step {
            Step::New(cmd) => {
                finish(&path, dynamic, &mut problems);
                // e.g. the health check runs `ethcli --version`
                if cmd.starts_with('-') {
                    path.clear();
                    dynamic = true;
                    continue;
                }
                path = vec![cmd];
                dynamic = false;
                saw_builder = true;
            }
            Step::Sub(sub) => {
                if !dynamic {
                    path.push(sub);
                }
            }
            Step::DynamicSub => dynamic = true,
            Step::Flag(flag) => {
                if dynamic || path.is_empty() {
                    continue;
                }
                match resolve(&path) {
                    Ok(chain) => {
                        if !flag_exists(&chain, &flag) {
                            problems.push(format!(
                                "flag `{}` does not exist on `ethcli {}`",
                                flag,
                                path.join(" ")
                            ));
                        }
                        // The root `--chain` is global, so clap accepts it
                        // everywhere; but when the leaf takes the chain as a
                        // positional, the global flag is silently ignored.
                        if let Some(long) = flag.strip_prefix("--") {
                            let leaf = *chain.last().unwrap();
                            let positional_shadow = leaf.get_positionals().any(|a| {
                                a.get_id().as_str() == long.replace('-', "_")
                                    && !leaf.get_arguments().any(|o| o.get_long() == Some(long))
                            });
                            if positional_shadow {
                                problems.push(format!(
                                    "`ethcli {}` takes `{}` as a positional argument; `{}` is ignored",
                                    path.join(" "),
                                    long,
                                    flag
                                ));
                            }
                        }
                    }
                    Err(e) => problems.push(e),
                }
            }
        }
    }
    finish(&path, dynamic, &mut problems);
    // Path validity for builders without any flags
    if saw_builder && !dynamic && !path.is_empty() {
        if let Err(e) = resolve(&path) {
            if !problems.contains(&e) {
                problems.push(e);
            }
        }
    }
    problems
        .into_iter()
        .map(|p| format!("{name}: {p}"))
        .collect()
}

fn split_functions(src: &str) -> Vec<(String, String)> {
    let re = Regex::new(r"pub async fn (\w+)\s*[<(]").unwrap();
    let starts: Vec<(usize, String)> = re
        .captures_iter(src)
        .map(|c| (c.get(0).unwrap().start(), c[1].to_string()))
        .collect();
    let mut out = Vec::new();
    for (i, (start, name)) in starts.iter().enumerate() {
        let end = starts.get(i + 1).map(|(s, _)| *s).unwrap_or(src.len());
        out.push((name.clone(), src[*start..end].to_string()));
    }
    out
}

#[test]
fn every_mcp_tool_maps_to_an_existing_ethcli_command() {
    let Some(src) = tools_source() else {
        eprintln!("ethcli-mcp sources not found; skipping (not in workspace checkout)");
        return;
    };
    // Only look at the tool functions (skip the #[cfg(test)] module, if any).
    let src = src.split("#[cfg(test)]").next().unwrap_or(&src);
    let root = ethcli::cli::Cli::command();

    let functions = split_functions(src);
    assert!(
        functions.len() > 100,
        "expected to find the MCP tool functions in tools.rs, found {}",
        functions.len()
    );

    let problems: Vec<String> = functions
        .iter()
        .flat_map(|(name, body)| check_function(&root, name, body))
        .collect();

    assert!(
        problems.is_empty(),
        "{} MCP tool(s) call invalid ethcli command paths/flags:\n  {}",
        problems.len(),
        problems.join("\n  ")
    );
}

#[test]
fn checker_detects_group_without_leaf_and_bad_flag() {
    let root = ethcli::cli::Cli::command();
    // The pre-fix tenderly_vnets tool body.
    let body = r#"ArgsBuilder::new("tenderly").subcommand("vnets").execute()"#;
    let problems = check_function(&root, "tenderly_vnets", body);
    assert!(
        problems.iter().any(|p| p.contains("command group")),
        "{problems:?}"
    );

    // The pre-fix curve_crvusd_markets tool body (--chain is positional there).
    let body = r#"ArgsBuilder::new("curve").subcommand("crvusd").subcommand("markets");
        builder = builder.opt("--chain", Some(c));"#;
    let problems = check_function(&root, "curve_crvusd_markets", body);
    assert!(
        problems.iter().any(|p| p.contains("--chain")),
        "{problems:?}"
    );

    let body = r#"ArgsBuilder::new("nope").execute()"#;
    assert!(!check_function(&root, "x", body).is_empty());
}
