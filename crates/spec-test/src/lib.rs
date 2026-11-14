//! Checks a client crate's response types against a provider's OpenAPI spec.
//!
//! `scripts/spec-check/examples.py` writes, for every operation in a spec,
//! the spec's own example response and responses built from the schema with
//! every field filled in (`sample`, `sample2`). [`spec_tests!`] generates
//! tests that deserialize each of them into the type the crate returns for
//! that operation, so a type that no longer matches the spec fails
//! `cargo test` instead of failing for users at runtime.
//!
//! ```ignore
//! yldfi_spec_test::spec_tests! {
//!     file: "spec_examples/provider.json",
//!     checks: {
//!         "get-thing" => mycrate::Thing,
//!         "list-things" => Vec<mycrate::Thing>,
//!     },
//!     not_implemented: ["delete-thing"],
//!     // Optional: responses the spec gets wrong, each with the reason
//!     skip: [("list-things", "example", "the example shows the v1 shape")],
//! }
//! ```
//!
//! This crate is not published; client crates use it as a path-only
//! dev-dependency, which `cargo publish` drops.

use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::BTreeMap;

/// Deserializes a response into one type
pub type Check = fn(&Value) -> Result<(), String>;

/// A response not to check, where the spec is known to be wrong:
/// `(operation, kind, reason)`, with kind `example`, `sample` or `sample2`
pub type Skip = (&'static str, &'static str, &'static str);

/// A [`Check`] for type `T`
pub fn parse<T: DeserializeOwned>(value: &Value) -> Result<(), String> {
    // Name the field that failed, e.g. `result[0].usd_price: invalid type ...`
    serde_path_to_error::deserialize::<_, T>(value.clone())
        .map(drop)
        .map_err(|e| {
            let path = e.path().to_string();
            if path == "." {
                e.into_inner().to_string()
            } else {
                format!("{path}: {}", e.into_inner())
            }
        })
}

/// The responses for every operation in one spec, keyed by operation id
pub struct Spec {
    operations: BTreeMap<String, Value>,
}

impl Spec {
    /// Parse the JSON written by `examples.py`
    ///
    /// # Panics
    /// If `text` is not that JSON.
    #[must_use]
    pub fn from_json(text: &str) -> Self {
        Self {
            operations: serde_json::from_str(text).expect("spec examples JSON"),
        }
    }

    /// Deserialize the `kind` response (`example`, `sample` or `sample2`) of
    /// every checked operation that has one. Returns one line per failure.
    ///
    /// # Errors
    /// Lists every operation whose response failed to deserialize, or that
    /// is checked but missing from the spec.
    pub fn check(&self, kind: &str, checks: &[(&str, Check)], skip: &[Skip]) -> Result<(), String> {
        let mut failures: Vec<String> = skip
            .iter()
            .filter(|(op, _, _)| !checks.iter().any(|(c, _)| c == op))
            .map(|(op, _, _)| format!("{op}: skipped but not checked"))
            .collect();
        for (op, check) in checks {
            if skip.iter().any(|(s, k, _)| s == op && *k == kind) {
                continue;
            }
            match self.operations.get(*op) {
                None => failures.push(format!("{op}: not in the spec examples")),
                Some(entry) => {
                    if let Some(value) = entry.get(kind) {
                        if let Err(e) = check(value) {
                            failures.push(format!("{op}: {e}"));
                        }
                    }
                }
            }
        }
        report(&format!("spec {kind}(s) failed to deserialize"), failures)
    }

    /// Every operation in the spec must be either checked or listed as not
    /// implemented, so a new operation is noticed.
    ///
    /// # Errors
    /// Lists operations that are neither, and listed ones the spec no
    /// longer has.
    pub fn coverage(
        &self,
        checks: &[(&str, Check)],
        not_implemented: &[&str],
    ) -> Result<(), String> {
        let checked: Vec<&str> = checks.iter().map(|(op, _)| *op).collect();
        let mut failures: Vec<String> = self
            .operations
            .keys()
            .filter(|op| !checked.contains(&op.as_str()) && !not_implemented.contains(&op.as_str()))
            .map(|op| format!("{op}: neither checked nor listed as not implemented"))
            .collect();
        failures.extend(
            not_implemented
                .iter()
                .filter(|op| !self.operations.contains_key(**op))
                .map(|op| format!("{op}: listed as not implemented but not in the spec")),
        );
        report("spec operation(s) not accounted for", failures)
    }
}

fn report(what: &str, failures: Vec<String>) -> Result<(), String> {
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} {what}:\n  {}",
            failures.len(),
            failures.join("\n  ")
        ))
    }
}

/// Generates the spec tests for one spec file (relative to the calling test
/// file). Use it once per spec, inside a `mod` when a crate has several.
#[macro_export]
macro_rules! spec_tests {
    (
        file: $file:literal,
        checks: { $($op:literal => $ty:ty),* $(,)? },
        not_implemented: [$($ni:literal),* $(,)?]
        $(, skip: [$(($sop:literal, $skind:literal, $sreason:literal $(,)?)),* $(,)?])? $(,)?
    ) => {
        fn spec() -> $crate::Spec {
            $crate::Spec::from_json(include_str!($file))
        }

        fn checks() -> Vec<(&'static str, $crate::Check)> {
            vec![$(($op, $crate::parse::<$ty> as $crate::Check)),*]
        }

        fn skip() -> Vec<$crate::Skip> {
            vec![$($(($sop, $skind, $sreason)),*)?]
        }

        /// The spec's own example responses
        #[test]
        fn spec_examples_deserialize() {
            if let Err(e) = spec().check("example", &checks(), &skip()) {
                panic!("{e}");
            }
        }

        /// Responses built from the schema, with every field present
        #[test]
        fn spec_schema_samples_deserialize() {
            if let Err(e) = spec().check("sample", &checks(), &skip()) {
                panic!("{e}");
            }
        }

        /// The same, taking the other alternative where a field may be one
        /// of several types
        #[test]
        fn spec_schema_alternative_samples_deserialize() {
            if let Err(e) = spec().check("sample2", &checks(), &skip()) {
                panic!("{e}");
            }
        }

        #[test]
        fn every_spec_operation_is_checked_or_listed() {
            if let Err(e) = spec().coverage(&checks(), &[$($ni),*]) {
                panic!("{e}");
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct Thing {
        id: u64,
    }

    const SPEC: &str = r#"{
        "get-thing": {"path": "/things/{id}", "example": {"id": 1}, "sample": {"id": "1"}},
        "list-things": {"path": "/things", "example": [{"id": 1}]}
    }"#;

    #[test]
    fn reports_each_failing_operation() {
        let spec = Spec::from_json(SPEC);
        let checks: Vec<(&str, Check)> = vec![
            ("get-thing", parse::<Thing> as Check),
            ("list-things", parse::<Vec<Thing>> as Check),
        ];
        assert!(spec.check("example", &checks, &[]).is_ok());
        let err = spec.check("sample", &checks, &[]).unwrap_err();
        assert!(err.starts_with("1 spec sample(s)"), "{err}");
        assert!(err.contains("get-thing: id: invalid type: string"), "{err}");
        // An operation without that kind of response is skipped
        assert!(spec.check("sample2", &checks, &[]).is_ok());
        // A skip silences exactly that operation and kind
        assert!(spec
            .check(
                "sample",
                &checks,
                &[("get-thing", "sample", "spec is wrong")]
            )
            .is_ok());
        assert!(spec
            .check(
                "sample",
                &checks,
                &[("get-thing", "example", "spec is wrong")]
            )
            .is_err());
        // Skipping an operation that is not checked is reported
        let err = spec
            .check("example", &checks, &[("other", "example", "stale")])
            .unwrap_err();
        assert!(err.contains("other: skipped but not checked"), "{err}");
    }

    #[test]
    fn coverage_needs_every_operation_accounted_for() {
        let spec = Spec::from_json(SPEC);
        let checks: Vec<(&str, Check)> = vec![("get-thing", parse::<Thing> as Check)];
        let err = spec.coverage(&checks, &[]).unwrap_err();
        assert!(err.contains("list-things: neither"), "{err}");
        assert!(spec.coverage(&checks, &["list-things"]).is_ok());
        let err = spec
            .coverage(&checks, &["list-things", "gone"])
            .unwrap_err();
        assert!(
            err.contains("gone: listed as not implemented but not in the spec"),
            "{err}"
        );
    }

    #[test]
    fn unknown_operations_fail() {
        let spec = Spec::from_json(SPEC);
        let checks: Vec<(&str, Check)> = vec![("nope", parse::<Thing> as Check)];
        assert!(spec
            .check("example", &checks, &[])
            .unwrap_err()
            .contains("nope: not in"));
    }
}
