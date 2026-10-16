//! Offline tests for the metric endpoints.
//!
//! Runs against a local `wiremock` server and needs nothing else, so it
//! isn't `#[ignore]`d.

use klaviyo::Client;
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn client(server: &MockServer) -> klaviyo::Result<Client> {
    Client::builder()
        .api_key("pk_test")
        .base_url(server.uri())
        .build()
}

#[tokio::test]
async fn list_filters_by_integration() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/metrics"))
        .and(query_param("filter", "equals(integration.name,'Shopify')"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "links": { "self": "https://a.klaviyo.com/api/metrics" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    client(&server)?
        .metrics()
        .list()
        .filter("equals(integration.name,'Shopify')")
        .send()
        .await?;
    Ok(())
}
