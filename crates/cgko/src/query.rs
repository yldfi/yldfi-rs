//! Query-string building shared by the request option types

use std::fmt::Display;

/// Percent-encode a value for use in a query string
pub(crate) fn encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

/// Builds `?a=1&b=x`, encoding every text value
#[derive(Default)]
pub(crate) struct Query(Vec<String>);

impl Query {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Add a text parameter, percent-encoded
    pub(crate) fn text(mut self, key: &str, value: Option<&str>) -> Self {
        if let Some(v) = value {
            self.0.push(format!("{key}={}", encode(v)));
        }
        self
    }

    /// Add a number or boolean parameter
    pub(crate) fn value(mut self, key: &str, value: Option<impl Display>) -> Self {
        if let Some(v) = value {
            self.0.push(format!("{key}={v}"));
        }
        self
    }

    /// Add a comma-separated list, each item percent-encoded
    pub(crate) fn list(mut self, key: &str, values: Option<&[String]>) -> Self {
        if let Some(vs) = values {
            let joined: Vec<String> = vs.iter().map(|v| encode(v)).collect();
            self.0.push(format!("{key}={}", joined.join(",")));
        }
        self
    }

    /// `?` followed by the parameters, or empty when there are none
    pub(crate) fn finish(self) -> String {
        if self.0.is_empty() {
            String::new()
        } else {
            format!("?{}", self.0.join("&"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Query;

    #[test]
    fn builds_and_encodes() {
        assert_eq!(
            Query::new()
                .text("a", None)
                .value("b", None::<u32>)
                .finish(),
            ""
        );
        let ids = vec!["a b".to_string(), "c&d".to_string()];
        let q = Query::new()
            .text("t", Some("x&y=z"))
            .value("n", Some(1.5))
            .value("b", Some(true))
            .list("ids", Some(&ids))
            .finish();
        assert_eq!(q, "?t=x%26y%3Dz&n=1.5&b=true&ids=a+b,c%26d");
    }
}
