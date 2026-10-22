//! Offline tests for the campaign endpoints.
//!
//! Runs against a local `wiremock` server and needs nothing else, so it
//! isn't `#[ignore]`d.

use std::time::Duration;

use klaviyo::header::{HeaderMap, HeaderValue};
use klaviyo::{types, Client, Error, API_REVISION};
use serde_json::json;
use wiremock::matchers::{
    body_json, body_partial_json, header, method, path, query_param, query_param_is_missing,
};
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
async fn list_sends_auth_revision_and_filter() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/campaigns"))
        .and(header("authorization", "Klaviyo-API-Key pk_test"))
        .and(header("revision", API_REVISION))
        .and(header("accept", "application/vnd.api+json"))
        .and(query_param("filter", "equals(archived,false)"))
        .and(query_param("sort", "-created_at"))
        .and(query_param("fields[campaign]", "definition.name"))
        .and(query_param(
            "include",
            "campaign-audiences,campaign-messages",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "links": { "self": "https://a.klaviyo.com/api/campaigns" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let response = client(&server)?
        .campaigns()
        .list()
        .filter("equals(archived,false)")
        .sort("-created_at")
        .fields("campaign", &["definition.name"])
        .include(&["campaign-audiences", "campaign-messages"])
        .send()
        .await?;

    assert!(response.data.is_empty());
    Ok(())
}

#[tokio::test]
async fn list_sends_no_filter_unless_asked() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/campaigns"))
        .and(query_param_is_missing("filter"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "links": { "self": "https://a.klaviyo.com/api/campaigns" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    client(&server)?.campaigns().list().send().await?;
    Ok(())
}

#[tokio::test]
async fn update_sets_data_id_from_the_path() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/campaigns/01JCAMPAIGN"))
        .and(header("content-type", "application/vnd.api+json"))
        .and(body_partial_json(json!({
            "data": { "type": "campaign", "id": "01JCAMPAIGN" }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let result = client(&server)?
        .campaigns()
        .update("01JCAMPAIGN", &types::CampaignPartialUpdateQuery::default())
        .send()
        .await;
    routed(result);
    Ok(())
}

#[tokio::test]
async fn clone_sets_the_source_campaign_id() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/campaign-clone"))
        .and(body_partial_json(json!({
            "data": { "type": "campaign", "id": "01JSOURCE" }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let result = client(&server)?
        .campaigns()
        .clone_campaign("01JSOURCE", &types::CampaignCloneQuery::default())
        .send()
        .await;
    routed(result);
    Ok(())
}

#[tokio::test]
async fn delete_accepts_an_empty_204() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/campaigns/01JCAMPAIGN"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    client(&server)?
        .campaigns()
        .delete("01JCAMPAIGN")
        .send()
        .await?;
    Ok(())
}

#[tokio::test]
async fn variations_lists_the_message_variations() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/api/campaign-messages/01JMESSAGE/campaign-variations",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [] })))
        .expect(1)
        .mount(&server)
        .await;

    let response = client(&server)?
        .campaigns()
        .variations("01JMESSAGE")
        .send()
        .await?;

    assert!(response.data.is_empty());
    Ok(())
}

#[tokio::test]
async fn update_variation_sends_only_the_given_details() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/campaign-variations/01JVARIATION"))
        .and(body_json(json!({
            "data": {
                "type": "campaign-variation",
                "id": "01JVARIATION",
                "attributes": {
                    "definition": {
                        "details": { "channel": "email", "template_id": "RipRmi" }
                    }
                }
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let request = types::VariationPartialUpdateQuery {
        data: types::VariationPartialUpdateQueryResourceObject {
            attributes: types::VariationPartialUpdateQueryResourceObjectAttributes {
                definition: types::VariationDefinition {
                    details: Some(
                        types::EmailChannelDetails {
                            channel: Some(types::EmailChannelDetailsChannel::Email),
                            template_id: Some("RipRmi".to_owned()),
                            ..Default::default()
                        }
                        .into(),
                    ),
                    ..Default::default()
                },
            },
            ..Default::default()
        },
    };
    let result = client(&server)?
        .campaigns()
        .update_variation("01JVARIATION", &request)
        .send()
        .await;
    routed(result);
    Ok(())
}

#[tokio::test]
async fn send_creates_a_send_job_for_the_campaign() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/campaign-send-jobs"))
        .and(body_json(json!({
            "data": { "type": "campaign-send-job", "id": "01JCAMPAIGN" }
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let result = client(&server)?
        .campaigns()
        .send("01JCAMPAIGN")
        .send()
        .await;
    routed(result);
    Ok(())
}

#[tokio::test]
async fn refresh_recipient_estimation_sends_the_job_document() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/campaign-recipient-estimation-jobs"))
        .and(body_json(json!({
            "data": { "type": "campaign-recipient-estimation-job", "id": "01JCAMPAIGN" }
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let result = client(&server)?
        .campaigns()
        .refresh_recipient_estimation("01JCAMPAIGN")
        .send()
        .await;
    routed(result);
    Ok(())
}

#[tokio::test]
async fn throttling_surfaces_errors_and_retry_after() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/campaigns/01JCAMPAIGN"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "3")
                .set_body_json(json!({
                    "errors": [{
                        "id": "e-1",
                        "status": 429,
                        "code": "throttled",
                        "title": "Request was throttled.",
                        "detail": "Request was throttled. Expected available in 3 seconds."
                    }]
                })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = client(&server)?
        .campaigns()
        .get("01JCAMPAIGN")
        .send()
        .await
        .expect_err("429 must be an error");

    assert!(err.is_retryable());
    assert_eq!(err.retry_after(), Some(Duration::from_secs(3)));
    match err {
        Error::Api { status, errors, .. } => {
            assert_eq!(status.as_u16(), 429);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].code, "throttled");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
    Ok(())
}

#[tokio::test]
async fn ids_are_percent_encoded_into_one_segment() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/campaigns/a%2F..%2Fb"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let result = client(&server)?.campaigns().get("a/../b").send().await;
    routed(result);
    Ok(())
}

#[tokio::test]
async fn ids_that_url_would_rewrite_are_rejected_without_a_request() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = client(&server)?;
    for campaign_id in ["", ".", "..", "..\t"] {
        let err = client
            .campaigns()
            .get(campaign_id)
            .send()
            .await
            .expect_err("campaign_id must be rejected");
        assert!(
            matches!(err, Error::InvalidPathSegment { ref segment } if segment == campaign_id),
            "{campaign_id:?}: got {err:?}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn per_call_headers_merge_over_client_defaults() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/campaigns/01JCAMPAIGN"))
        .and(header("x-tenant", "default"))
        .and(header("x-request-id", "per-call"))
        .and(header("x-trace-id", "kept"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let mut defaults = HeaderMap::new();
    defaults.insert("x-tenant", HeaderValue::from_static("default"));
    defaults.insert("x-request-id", HeaderValue::from_static("client-default"));

    let client = Client::builder()
        .api_key("pk_test")
        .base_url(server.uri())
        .default_headers(defaults)
        .build()?;

    let mut per_call = HeaderMap::new();
    per_call.insert("x-request-id", HeaderValue::from_static("per-call"));

    client
        .campaigns()
        .delete("01JCAMPAIGN")
        .header("X-Trace-Id", "kept")
        .headers(per_call)
        .send()
        .await?;
    Ok(())
}

#[tokio::test]
async fn reserved_per_call_header_is_rejected_without_a_request() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;

    for reserved in ["Authorization", "revision"] {
        let err = client(&server)?
            .campaigns()
            .delete("01JCAMPAIGN")
            .header(reserved, "x")
            .send()
            .await
            .expect_err("reserved header must be rejected");
        assert!(!err.is_retryable());
        assert!(
            matches!(err, Error::InvalidHeader { ref name } if name == &reserved.to_lowercase()),
            "{reserved}: got {err:?}"
        );
    }
    Ok(())
}
