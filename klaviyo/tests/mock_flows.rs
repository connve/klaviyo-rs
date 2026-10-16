//! Offline tests for the flow endpoints.
//!
//! Runs against a local `wiremock` server and needs nothing else, so it
//! isn't `#[ignore]`d.

use klaviyo::{types, Client, Error};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn client(server: &MockServer) -> klaviyo::Result<Client> {
    Client::builder()
        .api_key("pk_test")
        .base_url(server.uri())
        .build()
}

#[tokio::test]
async fn update_sets_the_flow_id_and_status() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/flows/XVTP5Q"))
        .and(body_partial_json(json!({
            "data": { "type": "flow", "id": "XVTP5Q", "attributes": { "status": "manual" } }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let request = types::FlowUpdateQuery {
        data: types::FlowUpdateQueryResourceObject {
            attributes: types::FlowUpdateQueryResourceObjectAttributes {
                status: "manual".to_owned(),
            },
            ..Default::default()
        },
    };
    let result = client(&server)?
        .flows()
        .update("XVTP5Q", &request)
        .send()
        .await;
    assert!(
        matches!(result, Ok(_) | Err(Error::Deserialize { .. })),
        "{result:?}"
    );
    Ok(())
}

#[tokio::test]
async fn list_filters_live_flows() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/flows"))
        .and(query_param("filter", "equals(status,'live')"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "links": { "self": "https://a.klaviyo.com/api/flows" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    client(&server)?
        .flows()
        .list()
        .filter("equals(status,'live')")
        .send()
        .await?;
    Ok(())
}
