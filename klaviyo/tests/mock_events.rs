//! Offline tests for the event endpoints.
//!
//! Runs against a local `wiremock` server and needs nothing else, so it
//! isn't `#[ignore]`d.

use klaviyo::serde_json::{Map, Value};
use klaviyo::{types, Client};
use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn client(server: &MockServer) -> klaviyo::Result<Client> {
    Client::builder()
        .api_key("pk_test")
        .base_url(server.uri())
        .build()
}

#[tokio::test]
async fn create_accepts_an_empty_202() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/events"))
        .and(header("content-type", "application/vnd.api+json"))
        .and(body_partial_json(json!({
            "data": {
                "type": "event",
                "attributes": {
                    "properties": { "OrderId": "ORDER-1001" },
                    "value": 129.9,
                    "unique_id": "ORDER-1001",
                    "metric": { "data": { "type": "metric", "attributes": { "name": "Placed Order" } } },
                    "profile": { "data": { "type": "profile", "attributes": { "email": "ada@example.com" } } }
                }
            }
        })))
        .respond_with(ResponseTemplate::new(202))
        .expect(1)
        .mount(&server)
        .await;

    let request = types::EventCreateQueryV2 {
        data: types::EventCreateQueryV2ResourceObject {
            attributes: types::EventCreateQueryV2ResourceObjectAttributes {
                properties: Map::from_iter([("OrderId".to_owned(), Value::from("ORDER-1001"))]),
                value: Some(129.9),
                unique_id: Some("ORDER-1001".to_owned()),
                metric: types::EventCreateQueryV2ResourceObjectAttributesMetric {
                    data: types::MetricCreateQueryResourceObject {
                        attributes: types::MetricCreateQueryResourceObjectAttributes {
                            name: "Placed Order".to_owned(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                profile: types::EventCreateQueryV2ResourceObjectAttributesProfile {
                    data: types::EventProfileCreateQueryResourceObject {
                        attributes: types::EventProfileCreateQueryResourceObjectAttributes {
                            email: Some("ada@example.com".to_owned()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
            ..Default::default()
        },
    };
    client(&server)?.events().create(&request).send().await?;
    Ok(())
}

#[tokio::test]
async fn list_sorts_and_sizes_pages() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/events"))
        .and(query_param("sort", "-datetime"))
        .and(query_param("page[size]", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "links": { "self": "https://a.klaviyo.com/api/events" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    client(&server)?
        .events()
        .list()
        .sort("-datetime")
        .page_size(1000)
        .send()
        .await?;
    Ok(())
}
