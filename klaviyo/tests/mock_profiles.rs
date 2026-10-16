//! Offline tests for the profile endpoints.
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

/// Single-resource responses need a full JSON:API document to deserialize.
/// Tests that only check what was sent accept a body that doesn't.
fn routed<T: std::fmt::Debug>(result: klaviyo::Result<T>) {
    assert!(
        matches!(result, Ok(_) | Err(Error::Deserialize { .. })),
        "{result:?}"
    );
}

#[tokio::test]
async fn list_sends_additional_fields_filter_and_page_size() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/profiles"))
        .and(query_param("additional-fields[profile]", "subscriptions"))
        .and(query_param("filter", "equals(email,'ada@example.com')"))
        .and(query_param("page[size]", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "links": { "self": "https://a.klaviyo.com/api/profiles" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    client(&server)?
        .profiles()
        .list()
        .additional_fields("profile", &["subscriptions"])
        .filter("equals(email,'ada@example.com')")
        .page_size(100)
        .send()
        .await?;
    Ok(())
}

#[tokio::test]
async fn upsert_posts_to_profile_import() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/profile-import"))
        .and(body_partial_json(json!({
            "data": { "type": "profile", "attributes": { "email": "ada@example.com" } }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let request = types::ProfileUpsertQuery {
        data: types::ProfileUpsertQueryResourceObject {
            attributes: types::ProfileUpsertQueryResourceObjectAttributes {
                email: Some("ada@example.com".to_owned()),
                first_name: Some("Ada".to_owned()),
                ..Default::default()
            },
            ..Default::default()
        },
    };
    let result = client(&server)?.profiles().upsert(&request).send().await;
    routed(result);
    Ok(())
}

#[tokio::test]
async fn bulk_import_job_status_and_errors_use_the_job_id() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/profile-bulk-import-jobs/job-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/profile-bulk-import-jobs/job-1/import-errors"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "links": { "self": "https://a.klaviyo.com/api/profile-bulk-import-jobs/job-1/import-errors" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server)?;
    routed(client.profiles().get_bulk_import_job("job-1").send().await);
    client.profiles().bulk_import_errors("job-1").send().await?;
    Ok(())
}
