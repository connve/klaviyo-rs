//! Offline tests for the template endpoints.
//!
//! Runs against a local `wiremock` server and needs nothing else, so it
//! isn't `#[ignore]`d.

use klaviyo::serde_json::{Map, Value};
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
async fn list_asks_for_additional_fields_and_filters() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/templates"))
        .and(query_param("additional-fields[template]", "definition"))
        .and(query_param("filter", "equals(name,'Autumn [PL]')"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "links": { "self": "https://a.klaviyo.com/api/templates" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    client(&server)?
        .templates()
        .list()
        .additional_fields("template", &["definition"])
        .filter("equals(name,'Autumn [PL]')")
        .send()
        .await?;
    Ok(())
}

#[tokio::test]
async fn update_sets_data_id_from_the_path() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/templates/RipRmi"))
        .and(body_partial_json(json!({
            "data": {
                "type": "template",
                "id": "RipRmi",
                "attributes": { "html": "<p>Cześć</p>" }
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let request = types::TemplateUpdateHtmlOrDndQuery {
        data: types::TemplateUpdateHtmlOrDndQueryResourceObject {
            attributes: types::TemplateUpdateHtmlOrDndQueryResourceObjectAttributes {
                html: Some("<p>Cześć</p>".to_owned()),
                ..Default::default()
            },
            ..Default::default()
        },
    };
    let result = client(&server)?
        .templates()
        .update("RipRmi", &request)
        .send()
        .await;
    routed(result);
    Ok(())
}

#[tokio::test]
async fn render_sets_the_template_id_and_passes_the_context() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/template-render"))
        .and(body_partial_json(json!({
            "data": {
                "type": "template",
                "id": "RipRmi",
                "attributes": { "context": { "first_name": "Ada" } }
            }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let request = types::TemplateRenderQuery {
        data: types::TemplateRenderQueryResourceObject {
            attributes: types::TemplateRenderQueryResourceObjectAttributes {
                context: Map::from_iter([("first_name".to_owned(), Value::from("Ada"))]),
            },
            ..Default::default()
        },
    };
    let result = client(&server)?
        .templates()
        .render("RipRmi", &request)
        .send()
        .await;
    routed(result);
    Ok(())
}

#[tokio::test]
async fn delete_universal_content_accepts_an_empty_204() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/template-universal-content/01JBLOCK"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    client(&server)?
        .templates()
        .delete_universal_content("01JBLOCK")
        .send()
        .await?;
    Ok(())
}
