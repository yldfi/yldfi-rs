//! URL redaction helpers for human-facing output.
//!
//! RPC URLs frequently embed credentials: basic-auth userinfo
//! (`https://user:pass@host`), API keys in the path
//! (`https://eth-mainnet.g.alchemy.com/v2/<key>`) or query
//! (`?apikey=<key>`). Anything printed to a terminal can end up in shell
//! history, CI logs, screenshots or LLM transcripts, so every URL shown to a
//! user should go through [`redact_url`].

use reqwest::Url;

/// Redact a URL down to `scheme://host[:port]`.
///
/// Userinfo, path, query and fragment are dropped entirely, because any of
/// them may contain a secret. Strings that do not parse as a URL with a host
/// are replaced by a fixed placeholder rather than echoed back.
///
/// # Examples
///
/// ```
/// use ethcli::utils::url::redact_url;
///
/// assert_eq!(
///     redact_url("https://user:pass@rpc.example.com:8545/v2/SECRET?key=x"),
///     "https://rpc.example.com:8545"
/// );
/// assert_eq!(redact_url("https://eth.llamarpc.com"), "https://eth.llamarpc.com");
/// ```
pub fn redact_url(raw: &str) -> String {
    let Ok(url) = Url::parse(raw.trim()) else {
        return "<invalid url>".to_string();
    };
    let Some(host) = url.host_str() else {
        return "<invalid url>".to_string();
    };
    match url.port() {
        Some(port) => format!("{}://{}:{}", url.scheme(), host, port),
        None => format!("{}://{}", url.scheme(), host),
    }
}

/// Returns true if [`redact_url`] would hide anything (userinfo, a
/// non-root path, a query or a fragment).
pub fn url_has_hidden_parts(raw: &str) -> bool {
    match Url::parse(raw.trim()) {
        Ok(url) => {
            !url.username().is_empty()
                || url.password().is_some()
                || (url.path() != "/" && !url.path().is_empty())
                || url.query().is_some()
                || url.fragment().is_some()
        }
        Err(_) => true,
    }
}

const URL_SCHEMES: &[&str] = &["https://", "http://", "wss://", "ws://"];

fn is_url_delimiter(c: char) -> bool {
    c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | ')' | ']' | '}' | '`' | ',')
}

/// Replace every URL embedded in free-form text (e.g. an error chain from
/// reqwest, which includes the full request URL) with [`redact_url`].
///
/// # Examples
///
/// ```
/// use ethcli::utils::url::redact_urls_in_text;
///
/// let msg = "error sending request for url (https://x.io/v2/SECRET?k=1)";
/// assert_eq!(
///     redact_urls_in_text(msg),
///     "error sending request for url (https://x.io)"
/// );
/// ```
pub fn redact_urls_in_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    loop {
        // Case-insensitive search; ASCII lowercasing preserves byte offsets.
        let lower = rest.to_ascii_lowercase();
        let next = URL_SCHEMES.iter().filter_map(|s| lower.find(s)).min();
        let Some(start) = next else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..start]);
        let tail = &rest[start..];
        let end = tail.find(is_url_delimiter).unwrap_or(tail.len());
        // Trailing punctuation is usually sentence punctuation, not URL.
        let candidate = tail[..end].trim_end_matches(['.', ';', ':']);
        let consumed = candidate.len();
        out.push_str(&redact_url(candidate));
        rest = &tail[consumed..];
        if consumed == 0 {
            // Defensive: avoid an infinite loop on a bare scheme.
            out.push_str(rest);
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_redaction_multiple_urls() {
        let msg = "a https://u:p@h1.io/k1, then wss://h2.io:8546/ws/k2. done";
        assert_eq!(
            redact_urls_in_text(msg),
            "a https://h1.io, then wss://h2.io:8546. done"
        );
    }

    #[test]
    fn text_redaction_no_urls() {
        assert_eq!(redact_urls_in_text("plain error"), "plain error");
    }

    #[test]
    fn strips_userinfo_path_query() {
        assert_eq!(
            redact_url("https://alice:hunter2@rpc.example.com/v2/abcdef123456?apikey=zzz#frag"),
            "https://rpc.example.com"
        );
    }

    #[test]
    fn keeps_port() {
        assert_eq!(
            redact_url("http://user:pw@127.0.0.1:8545/key"),
            "http://127.0.0.1:8545"
        );
    }

    #[test]
    fn default_port_elided() {
        assert_eq!(
            redact_url("https://rpc.example.com:443/x"),
            "https://rpc.example.com"
        );
    }

    #[test]
    fn websocket_scheme() {
        assert_eq!(
            redact_url("wss://mainnet.infura.io/ws/v3/0123456789abcdef"),
            "wss://mainnet.infura.io"
        );
    }

    #[test]
    fn invalid_is_not_echoed() {
        let out = redact_url("not a url with SECRET");
        assert!(!out.contains("SECRET"));
    }

    #[test]
    fn hidden_parts_detection() {
        assert!(!url_has_hidden_parts("https://eth.llamarpc.com"));
        assert!(!url_has_hidden_parts("https://eth.llamarpc.com/"));
        assert!(url_has_hidden_parts("https://rpc.ankr.com/eth"));
        assert!(url_has_hidden_parts("https://u:p@host.com"));
        assert!(url_has_hidden_parts("https://host.com?k=v"));
    }
}
