//! Integration tests for the template endpoints.
//!
//! The lifecycle test deletes the template it creates, even when a check
//! fails.
//!
//! Runs against the Klaviyo test account whose private API key is in
//! `KLAVIYO_API_KEY`, so every test is `#[ignore]`d; run them with
//! `cargo test -- --ignored`.

mod common;

use klaviyo::serde_json::{Map, Value};
use klaviyo::types;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn template_lifecycle() -> Result {
    let client = common::client()?;
    let name = unique_name!("template");

    let request = types::TemplateCreateHtmlOrDndQuery {
        data: types::TemplateCreateHtmlOrDndQueryResourceObject {
            attributes: types::TemplateCreateHtmlOrDndQueryResourceObjectAttributes {
                name: name.clone(),
                editor_type: "CODE".to_owned(),
                html: Some("<html><body><p>Hello {{ first_name }}</p></body></html>".to_owned()),
                ..Default::default()
            },
            ..Default::default()
        },
    };
    let template_id = client.templates().create(&request).send().await?.data.id;

    let checks: Result = async {
        let fetched = client.templates().get(&template_id).send().await?;
        ensure!(
            fetched.data.attributes.name == name,
            "fetched {:?}, created {name:?}",
            fetched.data.attributes.name
        );

        let update = types::TemplateUpdateHtmlOrDndQuery {
            data: types::TemplateUpdateHtmlOrDndQueryResourceObject {
                attributes: types::TemplateUpdateHtmlOrDndQueryResourceObjectAttributes {
                    html: Some(
                        "<html><body><p>Hello again {{ first_name }}</p></body></html>".to_owned(),
                    ),
                    ..Default::default()
                },
                ..Default::default()
            },
        };
        client
            .templates()
            .update(&template_id, &update)
            .send()
            .await?;

        let render = types::TemplateRenderQuery {
            data: types::TemplateRenderQueryResourceObject {
                attributes: types::TemplateRenderQueryResourceObjectAttributes {
                    context: Map::from_iter([("first_name".to_owned(), Value::from("Ada"))]),
                },
                ..Default::default()
            },
        };
        let rendered = client
            .templates()
            .render(&template_id, &render)
            .send()
            .await?;
        let html = rendered.data.attributes.html;
        ensure!(html.contains("Hello again Ada"), "rendered: {html}");

        Ok(())
    }
    .await;

    let cleanup = client.templates().delete(&template_id).send().await;
    checks?;
    cleanup?;
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn list_universal_content() -> Result {
    let client = common::client()?;
    client.templates().list_universal_content().send().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn get_unknown_template_is_rejected() -> Result {
    let client = common::client()?;
    ensure_rejected!(
        client
            .templates()
            .get(&unique_name!("template"))
            .send()
            .await
    );
    Ok(())
}
