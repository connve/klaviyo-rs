//! Offline tests for the account endpoints.
//!
//! Runs against a local `wiremock` server and needs nothing else, so it
//! isn't `#[ignore]`d.

use klaviyo::Client;
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn list_returns_the_key_owners_account() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/accounts"))
        .and(header("authorization", "Klaviyo-API-Key pk_market_pl"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "links": { "self": "https://a.klaviyo.com/api/accounts" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("pk_market_pl")
        .base_url(server.uri())
        .build()?;
    client.accounts().list().send().await?;
    Ok(())
}
