//! Chainlink Data Streams REST client

use crate::utils::{get_shared_http_client, unix_timestamp_ms};
use chainlink_data_streams_report::{feed_id::ID, report::Report};
use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, InvalidHeaderValue};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{digest::InvalidLength, Digest, Sha256};
use thiserror::Error;

pub const DEFAULT_STREAMS_REST_URL: &str = "https://api.testnet-dataengine.chain.link";

const API_V1_FEEDS: &str = "/api/v1/feeds";
const API_V1_REPORTS: &str = "/api/v1/reports";
const API_V1_REPORTS_BULK: &str = "/api/v1/reports/bulk";
const API_V1_REPORTS_PAGE: &str = "/api/v1/reports/page";
const API_V1_REPORTS_LATEST: &str = "/api/v1/reports/latest";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Error)]
pub enum DataStreamsError {
    #[error("API key cannot be empty")]
    EmptyApiKey,

    #[error("API secret cannot be empty")]
    EmptyApiSecret,

    #[error("HTTP client initialization failed: {0}")]
    HttpClientInit(String),

    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("HMAC generation failed: {0}")]
    Hmac(#[from] HmacError),

    #[error("Failed to serialize query string: {0}")]
    QuerySerialization(#[from] serde_urlencoded::ser::Error),

    #[error("Invalid response format: {0}")]
    InvalidResponseFormat(#[from] serde_json::Error),

    #[error("API error: {0}")]
    Api(String),
}

#[derive(Debug, Error)]
pub enum HmacError {
    #[error("Invalid key length: {0}")]
    InvalidKeyLength(#[from] InvalidLength),

    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataStreamsFeed {
    #[serde(rename = "feedID")]
    pub feed_id: ID,
}

#[derive(Debug, Deserialize)]
struct FeedsResponse {
    feeds: Vec<DataStreamsFeed>,
}

#[derive(Debug, Deserialize)]
struct ReportResponse {
    report: Report,
}

#[derive(Debug, Deserialize)]
struct ReportsResponse {
    reports: Vec<Report>,
}

pub struct DataStreamsClient {
    api_key: String,
    api_secret: String,
    rest_url: String,
    http: reqwest::Client,
}

impl DataStreamsClient {
    pub fn new(
        api_key: impl Into<String>,
        api_secret: impl Into<String>,
        rest_url: impl Into<String>,
    ) -> Result<Self, DataStreamsError> {
        let api_key = api_key.into();
        if api_key.trim().is_empty() {
            return Err(DataStreamsError::EmptyApiKey);
        }

        let api_secret = api_secret.into();
        if api_secret.trim().is_empty() {
            return Err(DataStreamsError::EmptyApiSecret);
        }

        let http = get_shared_http_client()
            .map_err(DataStreamsError::HttpClientInit)?
            .clone();

        Ok(Self {
            api_key,
            api_secret,
            rest_url: rest_url.into().trim_end_matches('/').to_string(),
            http,
        })
    }

    pub async fn get_feeds(&self) -> Result<Vec<DataStreamsFeed>, DataStreamsError> {
        let response: FeedsResponse = self.get(API_V1_FEEDS, Vec::new()).await?;
        Ok(response.feeds)
    }

    pub async fn get_latest_report(&self, feed_id: ID) -> Result<Report, DataStreamsError> {
        let response: ReportResponse = self
            .get(
                API_V1_REPORTS_LATEST,
                vec![("feedID", feed_id.to_hex_string())],
            )
            .await?;
        Ok(response.report)
    }

    pub async fn get_report(
        &self,
        feed_id: ID,
        timestamp: u128,
    ) -> Result<Report, DataStreamsError> {
        let response: ReportResponse = self
            .get(
                API_V1_REPORTS,
                vec![
                    ("feedID", feed_id.to_hex_string()),
                    ("timestamp", timestamp.to_string()),
                ],
            )
            .await?;
        Ok(response.report)
    }

    pub async fn get_reports_bulk(
        &self,
        feed_ids: &[ID],
        timestamp: u128,
    ) -> Result<Vec<Report>, DataStreamsError> {
        let feed_ids_joined = feed_ids
            .iter()
            .map(ID::to_hex_string)
            .collect::<Vec<_>>()
            .join(",");

        let response: ReportsResponse = self
            .get(
                API_V1_REPORTS_BULK,
                vec![
                    ("feedIDs", feed_ids_joined),
                    ("timestamp", timestamp.to_string()),
                ],
            )
            .await?;
        Ok(response.reports)
    }

    pub async fn get_reports_page_with_limit(
        &self,
        feed_id: ID,
        start_timestamp: u128,
        limit: usize,
    ) -> Result<Vec<Report>, DataStreamsError> {
        let response: ReportsResponse = self
            .get(
                API_V1_REPORTS_PAGE,
                vec![
                    ("feedID", feed_id.to_hex_string()),
                    ("startTimestamp", start_timestamp.to_string()),
                    ("limit", limit.to_string()),
                ],
            )
            .await?;
        Ok(response.reports)
    }

    async fn get<T>(
        &self,
        endpoint: &str,
        query_params: Vec<(&str, String)>,
    ) -> Result<T, DataStreamsError>
    where
        T: DeserializeOwned,
    {
        let path = build_request_path(endpoint, &query_params)?;
        let headers = generate_auth_headers(
            "GET",
            &path,
            b"",
            &self.api_key,
            &self.api_secret,
            unix_timestamp_ms(),
        )?;
        let url = format!("{}{}", self.rest_url, endpoint);

        let request = self.http.get(url).headers(headers);
        let request = if query_params.is_empty() {
            request
        } else {
            request.query(&query_params)
        };

        let response = request.send().await?;
        let response = response
            .error_for_status()
            .map_err(|e| DataStreamsError::Api(e.to_string()))?;
        let body = response.bytes().await?;

        Ok(serde_json::from_slice(&body)?)
    }
}

fn build_request_path(
    endpoint: &str,
    query_params: &[(&str, String)],
) -> Result<String, DataStreamsError> {
    if query_params.is_empty() {
        return Ok(endpoint.to_string());
    }

    let query_string = serde_urlencoded::to_string(query_params)?;
    Ok(format!("{endpoint}?{query_string}"))
}

fn generate_auth_headers(
    method: &str,
    path: &str,
    body: &[u8],
    client_id: &str,
    user_secret: &str,
    timestamp: u128,
) -> Result<HeaderMap, HmacError> {
    let mut headers = HeaderMap::new();
    let hmac = generate_hmac(method, path, body, client_id, timestamp, user_secret)?;

    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(client_id)?,
    );
    headers.insert(
        HeaderName::from_static("x-authorization-timestamp"),
        HeaderValue::from_str(&timestamp.to_string())?,
    );
    headers.insert(
        HeaderName::from_static("x-authorization-signature-sha256"),
        HeaderValue::from_str(&hmac)?,
    );

    Ok(headers)
}

fn generate_hmac(
    method: &str,
    path: &str,
    body: &[u8],
    client_id: &str,
    timestamp: u128,
    user_secret: &str,
) -> Result<String, HmacError> {
    let mut hasher = Sha256::new();
    hasher.update(body);
    let body_hash = hex::encode(hasher.finalize());

    let signing_payload = format!("{method} {path} {body_hash} {client_id} {timestamp}");

    let mut mac = HmacSha256::new_from_slice(user_secret.as_bytes())?;
    mac.update(signing_payload.as_bytes());

    Ok(hex::encode(mac.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{header, header_exists, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    const FEED_ID: &str = "0x000359843a543ee2fe414dc14c7e7920ef10f4372990b79d6361cdc0dd1ba782";

    #[test]
    fn generate_hmac_matches_sdk_test_vectors() {
        assert_eq!(
            generate_hmac(
                "GET",
                API_V1_FEEDS,
                b"",
                "clientId",
                1718885772,
                "userSecret"
            )
            .unwrap(),
            "e9b2aa1deb13b2abd078353a5e335b2f50307159ad28b433157d2c74dbab2072"
        );
        assert_eq!(
            generate_hmac("POST", API_V1_FEEDS, b"", "clientId1", 12000, "secret1").unwrap(),
            "31b48ebdb13802b58978cd89eca0c3c68ddccf85392e703b55942544e7203d3d"
        );
        assert_eq!(
            generate_hmac(
                "POST",
                API_V1_REPORTS_BULK,
                br#"{"attr1": "value1","attr2": [1,2,3]}"#,
                "clientId2",
                1718885772,
                "secret2"
            )
            .unwrap(),
            "37190febe20b6f3662f6abbfa3a7085ad705ac64e88bde8c1a01a635859e6cf7"
        );
    }

    #[test]
    fn build_request_path_percent_encodes_bulk_feed_ids() {
        let path = build_request_path(
            API_V1_REPORTS_BULK,
            &[
                ("feedIDs", format!("{FEED_ID},{FEED_ID}")),
                ("timestamp", "1718885772".to_string()),
            ],
        )
        .unwrap();

        assert_eq!(
            path,
            format!(
                "{API_V1_REPORTS_BULK}?feedIDs={}%2C{}&timestamp=1718885772",
                FEED_ID, FEED_ID
            )
        );
    }

    #[tokio::test]
    async fn latest_report_parses_mock_response() {
        let server = MockServer::start().await;
        let client = DataStreamsClient::new("client-id", "client-secret", server.uri()).unwrap();

        Mock::given(method("GET"))
            .and(path(API_V1_REPORTS_LATEST))
            .and(query_param("feedID", FEED_ID))
            .and(header("authorization", "client-id"))
            .and(header_exists("x-authorization-timestamp"))
            .and(header_exists("x-authorization-signature-sha256"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "report": {
                    "feedID": FEED_ID,
                    "validFromTimestamp": 1718885772,
                    "observationsTimestamp": 1718885772,
                    "fullReport": "0x1234"
                }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let report = client
            .get_latest_report(ID::from_hex_str(FEED_ID).unwrap())
            .await
            .unwrap();

        assert_eq!(report.feed_id.to_hex_string(), FEED_ID);
        assert_eq!(report.valid_from_timestamp, 1718885772);
        assert_eq!(report.observations_timestamp, 1718885772);
        assert_eq!(report.full_report, "0x1234");
    }
}
