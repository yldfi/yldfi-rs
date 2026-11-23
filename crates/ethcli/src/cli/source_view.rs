//! Parts of verified source code: file list, one file, matching lines, or
//! one function's text
//!
//! A verified contract can be dozens of files and tens of thousands of lines.
//! These helpers return only the part asked for, so a caller does not have to
//! read or filter the whole source.

/// One source file: its path in the verified sources and its text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: String,
    pub content: String,
}

/// A line that matched a search
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LineMatch {
    pub path: String,
    pub line: usize,
    pub text: String,
}

/// A function, modifier, constructor, fallback or receive found in the source
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FoundFunction {
    pub path: String,
    /// First line of the definition (after any NatSpec comment), 1-based
    pub line: usize,
    /// The definition, with the NatSpec comment above it
    pub text: String,
}

/// Strip the directory prefix shared by every file when the paths are
/// absolute. Verified sources often carry the deployer's own paths
/// (`/home/<user>/project/contracts/A.sol`); what matters is `contracts/A.sol`.
pub fn strip_common_absolute_prefix(files: &mut [SourceFile]) {
    let is_absolute = |p: &str| {
        p.starts_with('/') || (p.len() > 2 && p.as_bytes()[1] == b':' && p.contains('\\'))
    };
    // Only the absolute paths are rewritten; relative ones such as
    // `@openzeppelin/...` are already fine
    let absolute: Vec<usize> = (0..files.len())
        .filter(|&i| is_absolute(&files[i].path))
        .collect();
    if absolute.len() < 2 {
        return;
    }
    let split =
        |p: &str| -> Vec<String> { p.split(['/', '\\']).map(str::to_string).collect::<Vec<_>>() };
    let first = split(&files[absolute[0]].path);
    // Shared directories only, never the file name itself
    let mut shared = first.len().saturating_sub(1);
    for &i in &absolute[1..] {
        let parts = split(&files[i].path);
        let common = first
            .iter()
            .zip(&parts)
            .take(parts.len().saturating_sub(1))
            .take_while(|(a, b)| a == b)
            .count();
        shared = shared.min(common);
    }
    // Keep the last shared folder (usually `contracts` or `src`)
    let shared = shared.saturating_sub(1);
    if shared == 0 {
        return;
    }
    for &i in &absolute {
        files[i].path = split(&files[i].path)[shared..].join("/");
    }
}

/// Files sorted by path, with line and byte counts, for `--list`
pub fn list(files: &[SourceFile]) -> Vec<serde_json::Value> {
    files
        .iter()
        .map(|f| {
            serde_json::json!({
                "path": f.path,
                "lines": f.content.lines().count(),
                "bytes": f.content.len(),
            })
        })
        .collect()
}

/// The files matching `name`: an exact path, or else every file whose path
/// ends with `/name` (so `Vault.sol` finds `src/core/Vault.sol` but not
/// `IVault.sol`). Case is ignored.
pub fn find_files<'a>(files: &'a [SourceFile], name: &str) -> Vec<&'a SourceFile> {
    let wanted = name.trim().to_ascii_lowercase();
    if let Some(exact) = files.iter().find(|f| f.path.to_ascii_lowercase() == wanted) {
        return vec![exact];
    }
    files
        .iter()
        .filter(|f| {
            let path = f.path.to_ascii_lowercase();
            path.ends_with(&format!("/{wanted}"))
        })
        .collect()
}

/// Lines matching a regular expression, in file and line order
pub fn grep(files: &[SourceFile], pattern: &str) -> anyhow::Result<Vec<LineMatch>> {
    let regex = regex::Regex::new(pattern)
        .map_err(|e| anyhow::anyhow!("Invalid --grep pattern '{pattern}': {e}"))?;
    Ok(files
        .iter()
        .flat_map(|f| {
            let regex = &regex;
            f.content
                .lines()
                .enumerate()
                .filter(move |(_, text)| regex.is_match(text))
                .map(move |(index, text)| LineMatch {
                    path: f.path.clone(),
                    line: index + 1,
                    text: text.trim_end().to_string(),
                })
        })
        .collect())
}

/// Byte index just past the `}` matching the `{` at `open`, skipping braces
/// inside strings and comments
fn matching_brace(text: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = open;
    while i < text.len() {
        match text[i] {
            b'/' if text.get(i + 1) == Some(&b'/') => {
                while i < text.len() && text[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if text.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i + 1 < text.len() && !(text[i] == b'*' && text[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
            quote @ (b'"' | b'\'') => {
                i += 1;
                while i < text.len() && text[i] != quote {
                    if text[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Every definition named `name`: `function name(`, `modifier name`, or for
/// `constructor`, `fallback` and `receive` those keywords. An interface
/// declaration (ending in `;`) is returned as just that line. The NatSpec
/// comment directly above a definition is included.
pub fn functions(files: &[SourceFile], name: &str) -> anyhow::Result<Vec<FoundFunction>> {
    let name = name.trim();
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        anyhow::bail!("'{name}' is not a function name");
    }
    let pattern = match name {
        "constructor" | "fallback" | "receive" => format!(r"\b{name}\s*\("),
        _ => format!(r"\b(?:function|modifier)\s+{name}\s*[\(\{{]"),
    };
    let regex = regex::Regex::new(&pattern)?;

    let mut found = Vec::new();
    for file in files {
        let text = file.content.as_bytes();
        for m in regex.find_iter(&file.content) {
            // Skip matches inside a line comment
            let line_start = file.content[..m.start()].rfind('\n').map_or(0, |i| i + 1);
            if file.content[line_start..m.start()].contains("//") {
                continue;
            }
            // The body starts at the first `{` after the match, unless a `;`
            // comes first (a declaration with no body)
            let rest = &file.content[m.end() - 1..];
            let brace = rest.find('{').map(|i| i + m.end() - 1);
            let semicolon = rest.find(';').map(|i| i + m.end() - 1);
            let end = match (brace, semicolon) {
                (Some(b), Some(s)) if s < b => s + 1,
                (Some(b), _) => match matching_brace(text, b) {
                    Some(end) => end,
                    None => continue,
                },
                (None, Some(s)) => s + 1,
                (None, None) => continue,
            };

            // Include the NatSpec or plain comment block directly above
            let mut start = line_start;
            while start > 0 {
                let previous_end = start - 1; // the newline ending the line above
                let previous_start = file.content[..previous_end]
                    .rfind('\n')
                    .map_or(0, |i| i + 1);
                let previous = file.content[previous_start..previous_end].trim();
                let is_doc = previous.starts_with("///")
                    || previous.starts_with("/**")
                    || previous.starts_with('*');
                if !is_doc {
                    break;
                }
                start = previous_start;
            }

            let line = file.content[..line_start].matches('\n').count() + 1;
            found.push(FoundFunction {
                path: file.path.clone(),
                line,
                text: file.content[start..end].to_string(),
            });
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files() -> Vec<SourceFile> {
        vec![
            SourceFile {
                path: "src/Vault.sol".to_string(),
                content: r#"pragma solidity ^0.8.0;

contract Vault {
    /// @notice Withdraw shares
    /// @dev Reverts when paused
    function withdraw(uint256 shares) external whenNotPaused {
        string memory s = "a } brace in a string";
        // a } brace in a comment
        if (shares > 0) { emit Withdrawn(shares); }
    }

    modifier whenNotPaused() {
        require(!paused, "paused");
        _;
    }

    constructor(address owner) { _owner = owner; }
}
"#
                .to_string(),
            },
            SourceFile {
                path: "src/interfaces/IVault.sol".to_string(),
                content: "interface IVault {\n    function withdraw(uint256 shares) external;\n}\n"
                    .to_string(),
            },
            SourceFile {
                path: "lib/oz/Ownable.sol".to_string(),
                content: "// function withdraw(uint256) is not here\ncontract Ownable {}\n"
                    .to_string(),
            },
        ]
    }

    #[test]
    fn finds_functions_with_their_natspec_and_skips_comments() {
        let found = functions(&files(), "withdraw").unwrap();
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!(found[0].path, "src/Vault.sol");
        assert_eq!(found[0].line, 6);
        assert!(
            found[0].text.starts_with("    /// @notice Withdraw shares"),
            "{}",
            found[0].text
        );
        assert!(
            found[0]
                .text
                .trim_end()
                .ends_with("emit Withdrawn(shares); }\n    }"),
            "{}",
            found[0].text
        );
        assert_eq!(found[1].path, "src/interfaces/IVault.sol");
        assert_eq!(
            found[1].text.trim(),
            "function withdraw(uint256 shares) external;"
        );

        let modifier = functions(&files(), "whenNotPaused").unwrap();
        assert_eq!(modifier.len(), 1);
        assert!(modifier[0].text.contains("require(!paused"));

        let constructor = functions(&files(), "constructor").unwrap();
        assert_eq!(
            constructor[0].text.trim(),
            "constructor(address owner) { _owner = owner; }"
        );

        assert!(functions(&files(), "nothing").unwrap().is_empty());
        assert!(functions(&files(), "a(b)").is_err());
    }

    #[test]
    fn finds_files_by_path_or_name_and_greps_lines() {
        let all = files();
        assert_eq!(find_files(&all, "src/Vault.sol").len(), 1);
        assert_eq!(find_files(&all, "vault.sol").len(), 1);
        assert_eq!(
            find_files(&all, "IVault.sol")[0].path,
            "src/interfaces/IVault.sol"
        );
        assert_eq!(
            find_files(&all, "Ownable.sol")[0].path,
            "lib/oz/Ownable.sol"
        );
        assert!(find_files(&all, "Missing.sol").is_empty());

        let hits = grep(&all, r"function\s+withdraw").unwrap();
        assert_eq!(
            hits.iter()
                .map(|h| (h.path.as_str(), h.line))
                .collect::<Vec<_>>(),
            vec![
                ("src/Vault.sol", 6),
                ("src/interfaces/IVault.sol", 2),
                ("lib/oz/Ownable.sol", 1)
            ]
        );
        assert!(grep(&all, "(").is_err());

        let mut absolute = vec![
            SourceFile {
                path: "/Users/dev/repo/contracts/v1/A.sol".to_string(),
                content: String::new(),
            },
            SourceFile {
                path: "/Users/dev/repo/contracts/v2/B.sol".to_string(),
                content: String::new(),
            },
            SourceFile {
                path: "/Users/dev/repo/lib/C.sol".to_string(),
                content: String::new(),
            },
            SourceFile {
                path: "@openzeppelin/contracts/D.sol".to_string(),
                content: String::new(),
            },
        ];
        strip_common_absolute_prefix(&mut absolute);
        assert_eq!(
            absolute.iter().map(|f| f.path.as_str()).collect::<Vec<_>>(),
            vec![
                "repo/contracts/v1/A.sol",
                "repo/contracts/v2/B.sol",
                "repo/lib/C.sol",
                "@openzeppelin/contracts/D.sol"
            ]
        );
        let mut relative = all.clone();
        strip_common_absolute_prefix(&mut relative);
        assert_eq!(relative, all);

        let listed = list(&all);
        assert_eq!(listed[0]["path"], "src/Vault.sol");
        assert_eq!(listed[1]["lines"], 3);
    }
}
