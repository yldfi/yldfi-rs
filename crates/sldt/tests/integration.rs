//! Integration tests for sldt client
//!
//! These tests verify the client construction and configuration without
//! making actual API calls (which require a valid API key).

use sldt::{Client, Impact, SearchFilter, SortDirection, SortField};

#[test]
fn test_client_creation() {
    let client = Client::new("test_api_key");
    assert!(client.is_ok());
}

#[test]
fn test_client_rejects_empty_api_key() {
    assert!(Client::new("").is_err());
    assert!(Client::new("   ").is_err());
    assert!(Client::new("\t\n").is_err());
}

#[test]
fn test_client_with_timeout() {
    use std::time::Duration;
    let client = Client::with_timeout("test_key", Duration::from_secs(60));
    assert!(client.is_ok());
}

#[test]
fn test_client_with_custom_base_url() {
    let client = Client::with_base_url("test_key", "https://custom.example.com/api");
    assert!(client.is_ok());
}

#[test]
fn test_client_debug_does_not_expose_key() {
    let client = Client::new("super_secret_key").unwrap();
    let debug_output = format!("{:?}", client);
    assert!(!debug_output.contains("super_secret_key"));
    assert!(debug_output.contains("[REDACTED]"));
}

#[test]
fn test_search_filter_builder() {
    let filter = SearchFilter::new("reentrancy")
        .impact(Impact::High)
        .impact(Impact::Medium)
        .firm("Cyfrin")
        .tag("Reentrancy")
        .page_size(100)
        .page(2);

    assert_eq!(filter.keywords, Some("reentrancy".to_string()));
    assert_eq!(filter.impacts.len(), 2);
    assert!(filter.impacts.contains(&Impact::High));
    assert!(filter.impacts.contains(&Impact::Medium));
    assert_eq!(filter.firms.len(), 1);
    assert_eq!(filter.firms[0].value, "Cyfrin");
    assert_eq!(filter.tags.len(), 1);
    assert_eq!(filter.tags[0].value, "Reentrancy");
    assert_eq!(filter.page_size, 100);
    assert_eq!(filter.page, 2);
}

#[test]
fn test_search_filter_sort_options() {
    let filter = SearchFilter::new("test")
        .sort_by_quality()
        .ascending();

    assert_eq!(filter.sort_field.as_str(), "Quality");
    assert_eq!(filter.sort_direction.as_str(), "Asc");
}

#[test]
fn test_search_filter_sort_by_rarity() {
    let filter = SearchFilter::new("test").sort_by_rarity();
    assert_eq!(filter.sort_field.as_str(), "Rarity");
}

#[test]
fn test_search_filter_with_protocol() {
    let filter = SearchFilter::new("oracle")
        .protocol("Aave")
        .protocol_category("DeFi")
        .language("Solidity");

    assert_eq!(filter.protocol, Some("Aave".to_string()));
    assert_eq!(filter.protocol_categories.len(), 1);
    assert_eq!(filter.protocol_categories[0].value, "DeFi");
    assert_eq!(filter.languages.len(), 1);
    assert_eq!(filter.languages[0].value, "Solidity");
}

#[test]
fn test_search_filter_with_user_constraints() {
    let filter = SearchFilter::new("test")
        .user("auditor_name")
        .finders_range(Some(1), Some(5));

    assert_eq!(filter.user, Some("auditor_name".to_string()));
    assert_eq!(filter.min_finders, Some(1));
    assert_eq!(filter.max_finders, Some(5));
}

#[test]
fn test_search_filter_with_quality_scores() {
    let filter = SearchFilter::new("test")
        .min_quality(3)
        .min_rarity(2);

    assert_eq!(filter.quality_score, Some(3));
    assert_eq!(filter.rarity_score, Some(2));
}

#[test]
fn test_search_filter_empty() {
    let filter = SearchFilter::empty();
    assert!(filter.keywords.is_none());
    assert!(filter.impacts.is_empty());
}

#[test]
fn test_impact_as_str() {
    assert_eq!(Impact::High.as_str(), "HIGH");
    assert_eq!(Impact::Medium.as_str(), "MEDIUM");
    assert_eq!(Impact::Low.as_str(), "LOW");
    assert_eq!(Impact::Gas.as_str(), "GAS");
    assert_eq!(Impact::Unknown.as_str(), "UNKNOWN");
}

#[test]
fn test_sort_field_as_str() {
    assert_eq!(SortField::Recency.as_str(), "Recency");
    assert_eq!(SortField::Quality.as_str(), "Quality");
    assert_eq!(SortField::Rarity.as_str(), "Rarity");
}

#[test]
fn test_sort_direction_as_str() {
    assert_eq!(SortDirection::Asc.as_str(), "Asc");
    assert_eq!(SortDirection::Desc.as_str(), "Desc");
}

#[test]
fn test_search_filter_with_page() {
    let filter = SearchFilter::new("test").page(5).page_size(25);
    let new_filter = filter.with_page(10);

    assert_eq!(new_filter.page, 10);
    assert_eq!(new_filter.page_size, 25);
    assert_eq!(new_filter.keywords, Some("test".to_string()));
}

#[test]
fn test_search_filter_page_size_clamped() {
    // Page size should be clamped to max 100
    let filter = SearchFilter::new("test").page_size(200);
    assert_eq!(filter.page_size, 100);

    // Page size should be at least 1
    let filter = SearchFilter::new("test").page_size(0);
    assert_eq!(filter.page_size, 1);
}

#[test]
fn test_search_filter_page_at_least_one() {
    let filter = SearchFilter::new("test").page(0);
    assert_eq!(filter.page, 1);
}
