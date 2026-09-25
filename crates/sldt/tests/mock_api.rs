//! HTTP-level tests against a wiremock server, using a fixture built from the
//! Cyfrin Solodit Findings API specification's Success Response schema.

use serde_json::{json, Value};
use sldt::{Client, Error, Impact, ReportedPeriod, SearchFilter};
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: &str = "sk_test_key_123";

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/findings_response.json")).unwrap()
}

async fn setup() -> (MockServer, Client) {
    let server = MockServer::start().await;
    let client = Client::with_base_url(KEY, server.uri()).unwrap();
    (server, client)
}

#[tokio::test]
async fn success_parses_spec_fixture() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/findings"))
        .and(header("X-Cyfrin-API-Key", KEY))
        .and(header("Content-Type", "application/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixture())
                .insert_header("X-RateLimit-Limit", "20")
                .insert_header("X-RateLimit-Remaining", "18")
                .insert_header("X-RateLimit-Reset", "1735689600"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let results = client.search("reentrancy").await.unwrap();

    assert_eq!(results.findings.len(), 2);
    assert_eq!(results.total, 1234);
    assert_eq!(results.page, 1);
    assert_eq!(results.page_size, 2);
    assert_eq!(results.total_pages, 617);
    assert_eq!(results.elapsed, Some(0.123));
    assert!(results.has_more());

    // Body rateLimit wins; headers are exposed separately.
    assert_eq!(results.rate_limit.limit, 20);
    assert_eq!(results.rate_limit.remaining, 19);
    assert_eq!(results.rate_limit.reset, 1_735_689_600);
    let hdr = results.rate_limit_headers.unwrap();
    assert_eq!(hdr.remaining, 18);

    let f = &results.findings[0];
    assert_eq!(f.id.as_deref(), Some("58291"));
    assert_eq!(f.impact_level(), Impact::High);
    assert_eq!(f.firm(), Some("Code4rena"));
    assert_eq!(f.protocol(), Some("Example Vault"));
    assert_eq!(f.finder_handles(), vec!["alice", "bob"]);
    assert_eq!(f.tags(), vec!["Reentrancy"]);
    assert_eq!(f.finders_count, Some(2));
    assert_eq!(f.general_score, Some(3.0));
    assert_eq!(f.report_date.as_deref(), Some("2024-03-15T00:00:00.000Z"));
    assert!(f.solodit_url().unwrap().ends_with(
        "/issues/h-01-reentrancy-in-withdraw-allows-draining-vault-code4rena-example-git"
    ));

    // Second finding: numeric BigInt id, report_date {}, firm only in nested object.
    let g = &results.findings[1];
    assert_eq!(g.id.as_deref(), Some("9007199254740993"));
    assert_eq!(g.report_date, None);
    assert_eq!(g.firm(), Some("Cyfrin"));
    assert_eq!(g.protocol(), None);
    assert_eq!(g.tags(), vec!["Oracle", "Stale Price"]);
    assert_eq!(g.pdf_page_from, Some(12));
}

#[tokio::test]
async fn success_with_headers_only_rate_limit() {
    let (server, client) = setup().await;
    let mut body = fixture();
    body.as_object_mut().unwrap().remove("rateLimit");
    Mock::given(method("POST"))
        .and(path("/findings"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(body)
                .insert_header("X-RateLimit-Limit", "20")
                .insert_header("X-RateLimit-Remaining", "7")
                .insert_header("X-RateLimit-Reset", "1735689600"),
        )
        .mount(&server)
        .await;

    let results = client.search("x").await.unwrap();
    assert_eq!(results.rate_limit.remaining, 7);
    assert_eq!(results.rate_limit.limit, 20);
}

#[tokio::test]
async fn one_odd_finding_does_not_fail_the_page() {
    let (server, client) = setup().await;
    let mut body = fixture();
    let findings = body["findings"].as_array_mut().unwrap();
    findings[0]["quality_score"] = json!("not-a-number");
    findings[0]["issues_issue_finders"] = json!(null);
    findings[0]["protocols_protocol"] = json!([]);
    findings.push(json!("garbage"));
    Mock::given(method("POST"))
        .and(path("/findings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let results = client.search("x").await.unwrap();
    assert_eq!(results.findings.len(), 2);
    assert_eq!(results.findings[0].quality_score, None);
    assert!(results.findings[0].finder_handles().is_empty());
}

#[tokio::test]
async fn request_body_shape_matches_spec() {
    let (server, client) = setup().await;
    let expected = json!({
        "page": 3,
        "pageSize": 25,
        "filters": {
            "keywords": "oracle",
            "impact": ["HIGH", "MEDIUM"],
            "firms": [{"value": "Cyfrin"}, {"value": "Sherlock"}],
            "tags": [{"value": "Oracle"}],
            "protocol": "Aave",
            "protocolCategory": [{"value": "DeFi"}],
            "forked": [{"value": "Compound"}],
            "languages": [{"value": "Solidity"}, {"value": "Vyper"}],
            "user": "alice",
            "minFinders": "1",
            "maxFinders": "5",
            "reported": {"value": "after"},
            "reportedAfter": "2024-01-01",
            "qualityScore": 3,
            "rarityScore": 2,
            "sortField": "Quality",
            "sortDirection": "Asc"
        }
    });
    Mock::given(method("POST"))
        .and(path("/findings"))
        .and(body_partial_json(expected))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture()))
        .expect(1)
        .mount(&server)
        .await;

    let filter = SearchFilter::new("oracle")
        .page(3)
        .page_size(25)
        .impact(Impact::High)
        .impact(Impact::Medium)
        .firm("Cyfrin")
        .firm("Sherlock")
        .tag("Oracle")
        .protocol("Aave")
        .protocol_category("DeFi")
        .forked("Compound")
        .language("Solidity")
        .language("Vyper")
        .user("alice")
        .finders_range(Some(1), Some(5))
        .reported_after("2024-01-01")
        .min_quality(3)
        .min_rarity(2)
        .sort_by_quality()
        .ascending();
    client.search_with_filter(filter).await.unwrap();
}

#[tokio::test]
async fn request_body_reported_period() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/findings"))
        .and(body_partial_json(
            json!({"filters": {"reported": {"value": "30"}}}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture()))
        .expect(1)
        .mount(&server)
        .await;

    let filter = SearchFilter::empty().reported(ReportedPeriod::Days30);
    client.search_with_filter(filter).await.unwrap();

    // reportedAfter must not be sent when not set
    let reqs = server.received_requests().await.unwrap();
    let sent: Value = serde_json::from_slice(&reqs[0].body).unwrap();
    assert!(sent["filters"].get("reportedAfter").is_none());
    assert!(sent["filters"].get("keywords").is_none());
}

#[tokio::test]
async fn unauthorized_missing_key_message() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/findings"))
        .respond_with(
            ResponseTemplate::new(401).set_body_json(json!({"message": "Missing API key"})),
        )
        .mount(&server)
        .await;

    let err = client.search("x").await.unwrap_err();
    assert!(err.is_unauthorized());
    assert!(err.is_missing_api_key());
    let msg = err.to_string();
    assert!(msg.contains("Missing API key"), "{msg}");
    assert!(!msg.contains(KEY));
}

#[tokio::test]
async fn unauthorized_invalid_key_message_with_hint() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/findings"))
        .respond_with(
            ResponseTemplate::new(401).set_body_json(json!({"message": "Invalid API key"})),
        )
        .mount(&server)
        .await;

    let err = client.search("x").await.unwrap_err();
    assert!(err.is_invalid_api_key());
    let msg = err.to_string();
    assert!(msg.contains("Invalid API key"), "{msg}");
    assert!(msg.contains("Profile > API Keys"), "{msg}");
    assert!(msg.contains("invalidates the previous one"), "{msg}");
    assert!(!msg.contains(KEY));
}

#[tokio::test]
async fn rate_limited_includes_reset() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/findings"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({"message": "Rate limit exceeded"}))
                .insert_header("X-RateLimit-Limit", "20")
                .insert_header("X-RateLimit-Remaining", "0")
                .insert_header("X-RateLimit-Reset", "1735689600"),
        )
        .mount(&server)
        .await;

    let err = client.search("x").await.unwrap_err();
    assert!(err.is_rate_limited());
    assert_eq!(err.rate_limit_reset(), Some(1_735_689_600));
    match &err {
        Error::RateLimited {
            limit, remaining, ..
        } => {
            assert_eq!(*limit, Some(20));
            assert_eq!(*remaining, Some(0));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    let msg = err.to_string();
    assert!(msg.contains("Rate limit exceeded"), "{msg}");
    assert!(msg.contains("1735689600"), "{msg}");
}

#[tokio::test]
async fn bad_request_includes_server_message() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/findings"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(json!({"message": "Invalid request parameters"})),
        )
        .mount(&server)
        .await;

    let err = client.search("x").await.unwrap_err();
    assert_eq!(err.status(), Some(400));
    assert!(err.to_string().contains("Invalid request parameters"));
}

#[tokio::test]
async fn get_by_slug_matches_exact_slug_and_id() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/findings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture()))
        .mount(&server)
        .await;

    let slug = "m-02-stale-oracle-price-cyfrin-none-example-markdown";
    let f = client.get_by_slug(slug).await.unwrap();
    assert_eq!(f.slug.as_deref(), Some(slug));

    let f = client
        .get_by_slug(&format!("https://solodit.cyfrin.io/issues/{slug}"))
        .await
        .unwrap();
    assert_eq!(f.slug.as_deref(), Some(slug));

    let f = client.get_by_slug("58291").await.unwrap();
    assert_eq!(f.id.as_deref(), Some("58291"));

    // First request used the raw slug as keywords with pageSize 100.
    let reqs = server.received_requests().await.unwrap();
    let sent: Value = serde_json::from_slice(&reqs[0].body).unwrap();
    assert_eq!(sent["filters"]["keywords"], json!(slug));
    assert_eq!(sent["pageSize"], json!(100));
}

#[tokio::test]
async fn get_by_slug_not_found_after_fallback_query() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/findings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture()))
        .expect(2)
        .mount(&server)
        .await;

    let err = client.get_by_slug("no-such-finding").await.unwrap_err();
    assert!(err.is_not_found());

    let reqs = server.received_requests().await.unwrap();
    let second: Value = serde_json::from_slice(&reqs[1].body).unwrap();
    assert_eq!(second["filters"]["keywords"], json!("no such finding"));
}

/// Live smoke test against the real API. Run with:
/// `SOLODIT_API_KEY=sk_... cargo test -p sldt --test mock_api -- --ignored`
#[tokio::test]
#[ignore = "requires SOLODIT_API_KEY and network access"]
async fn live_search_smoke() {
    let Ok(key) = std::env::var("SOLODIT_API_KEY") else {
        eprintln!("SOLODIT_API_KEY not set; skipping");
        return;
    };
    let client = Client::new(key).unwrap();
    let results = client
        .search_with_filter(
            SearchFilter::new("reentrancy")
                .page_size(5)
                .impact(Impact::High),
        )
        .await
        .unwrap();
    assert!(results.findings.len() <= 5);
    assert!(results.rate_limit.limit > 0 || results.rate_limit_headers.is_some());
    for f in &results.findings {
        assert!(f.id.is_some() || f.slug.is_some());
    }
}
