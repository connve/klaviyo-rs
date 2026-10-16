//! Offline tests for the list endpoints, including pagination.
//!
//! Runs against a local `wiremock` server and needs nothing else, so it
//! isn't `#[ignore]`d.

use klaviyo::Client;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn client(server: &MockServer) -> klaviyo::Result<Client> {
    Client::builder()
        .api_key("pk_test")
        .base_url(server.uri())
        .build()
}

fn collection(next: Option<&str>) -> serde_json::Value {
    json!({
        "data": [],
        "links": { "self": "https://a.klaviyo.com/api/lists", "next": next }
    })
}

#[tokio::test]
async fn pages_follow_the_next_cursor_until_the_last_page() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/lists"))
        .and(query_param("page[size]", "2"))
        .and(query_param_is_missing("page[cursor]"))
        .respond_with(ResponseTemplate::new(200).set_body_json(collection(Some(
            "https://a.klaviyo.com/api/lists?page%5Bsize%5D=2&page%5Bcursor%5D=second",
        ))))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/lists"))
        .and(query_param("page[size]", "2"))
        .and(query_param("page[cursor]", "second"))
        .respond_with(ResponseTemplate::new(200).set_body_json(collection(None)))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server)?;
    let mut pages = client.lists().list().page_size(2).pages();
    let mut fetched = 0;
    while let Some(page) = pages.next_page().await {
        page?;
        fetched += 1;
    }
    assert_eq!(fetched, 2);
    Ok(())
}

#[tokio::test]
async fn a_next_link_to_another_host_only_contributes_its_cursor() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/lists"))
        .and(query_param_is_missing("page[cursor]"))
        .respond_with(ResponseTemplate::new(200).set_body_json(collection(Some(
            "https://attacker.example/steal?page%5Bcursor%5D=second",
        ))))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/lists"))
        .and(query_param("page[cursor]", "second"))
        .respond_with(ResponseTemplate::new(200).set_body_json(collection(None)))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server)?;
    let mut pages = client.lists().list().pages();
    while let Some(page) = pages.next_page().await {
        page?;
    }
    Ok(())
}

#[tokio::test]
async fn a_failed_page_can_be_retried() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/lists"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/lists"))
        .respond_with(ResponseTemplate::new(200).set_body_json(collection(None)))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server)?;
    let mut pages = client.lists().list().pages();
    let first = pages.next_page().await.expect("a first attempt");
    assert!(first.expect_err("503 is an error").is_retryable());

    pages.next_page().await.expect("a retry")?;
    assert!(pages.next_page().await.is_none());
    Ok(())
}

#[tokio::test]
async fn add_profiles_sends_resource_identifiers() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/lists/Y6nRLr/relationships/profiles"))
        .and(body_json(json!({
            "data": [
                { "type": "profile", "id": "01P1" },
                { "type": "profile", "id": "01P2" }
            ]
        })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    client(&server)?
        .lists()
        .add_profiles("Y6nRLr", &["01P1", "01P2"])
        .send()
        .await?;
    Ok(())
}

#[tokio::test]
async fn remove_profiles_sends_a_delete_with_a_body() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/lists/Y6nRLr/relationships/profiles"))
        .and(body_json(
            json!({ "data": [{ "type": "profile", "id": "01P1" }] }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    client(&server)?
        .lists()
        .remove_profiles("Y6nRLr", &["01P1"])
        .send()
        .await?;
    Ok(())
}
