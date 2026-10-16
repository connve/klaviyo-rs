//! End-to-end test of the per-market flow in one account: a list and a
//! template, then a draft campaign for the list with the template assigned.
//! Everything is deleted again, even when a check fails. Nothing is sent.
//!
//! Runs against the Klaviyo test account whose private API key is in
//! `KLAVIYO_API_KEY`, so every test is `#[ignore]`d; run them with
//! `cargo test -- --ignored`.

mod common;

use std::time::Duration;

use klaviyo::{types, Client};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Text put into the template and preview text, to recognise them later.
const MARKER: &str = "klaviyo-rs fan-out marker";

/// How often to look for the assigned template, one second apart.
const TEMPLATE_ATTEMPTS: u32 = 10;

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn draft_campaign_with_assigned_template() -> Result {
    let client = common::client()?;
    let name = unique_name!("fan-out");

    let list = types::ListCreateQuery {
        data: types::ListCreateQueryResourceObject {
            attributes: types::ListCreateQueryResourceObjectAttributes {
                name: name.clone(),
                ..Default::default()
            },
            ..Default::default()
        },
    };
    let list_id = client.lists().create(&list).send().await?.data.id;

    let template = types::TemplateCreateHtmlOrDndQuery {
        data: types::TemplateCreateHtmlOrDndQueryResourceObject {
            attributes: types::TemplateCreateHtmlOrDndQueryResourceObjectAttributes {
                name: name.clone(),
                editor_type: "CODE".to_owned(),
                html: Some(format!("<html><body><p>{MARKER}</p></body></html>")),
                ..Default::default()
            },
            ..Default::default()
        },
    };
    let template_id = match client.templates().create(&template).send().await {
        Ok(created) => created.data.id,
        Err(error) => {
            client.lists().delete(&list_id).send().await?;
            return Err(error.into());
        }
    };

    let checks = draft_campaign(&client, &name, &list_id, &template_id).await;

    let template_cleanup = client.templates().delete(&template_id).send().await;
    let list_cleanup = client.lists().delete(&list_id).send().await;
    checks?;
    template_cleanup?;
    list_cleanup?;
    Ok(())
}

async fn draft_campaign(client: &Client, name: &str, list_id: &str, template_id: &str) -> Result {
    let message = types::CampaignMessageCreateQueryResourceObject {
        attributes: types::CampaignMessageCreateQueryResourceObjectAttributes {
            definition: types::EmailMessageDefinition {
                label: Some(name.to_owned()),
                content: Some(types::EmailContent {
                    subject: Some(name.to_owned()),
                    preview_text: Some(MARKER.to_owned()),
                    ..Default::default()
                }),
                ..Default::default()
            }
            .into(),
        },
        relationships: None,
        type_: Default::default(),
    };
    let request = types::CampaignCreateQuery {
        data: types::CampaignCreateQueryResourceObject {
            attributes: types::CampaignCreateQueryResourceObjectAttributes {
                name: name.to_owned(),
                audiences: types::Audiences {
                    included: vec![list_id.to_owned()],
                    excluded: None,
                },
                campaign_messages:
                    types::CampaignCreateQueryResourceObjectAttributesCampaignMessages {
                        data: vec![message],
                    },
                ..Default::default()
            },
            ..Default::default()
        },
    };
    let campaign_id = client.campaigns().create(&request).send().await?.data.id;

    let checks: Result = async {
        let messages = client.campaigns().messages(&campaign_id).send().await?;
        let message = messages.data.first().ok_or("the campaign has no message")?;

        ensure_rejected!(
            client
                .campaigns()
                .assign_template(&message.id, &unique_name!("template"))
                .send()
                .await
        );

        client
            .campaigns()
            .assign_template(&message.id, template_id)
            .send()
            .await?;
        let html = assigned_template_html(client, &message.id).await?;
        ensure!(html.contains(MARKER), "assigned template: {html}");

        let campaign = client.campaigns().get(&campaign_id).send().await?;
        ensure!(
            campaign.data.attributes.status
                == types::CampaignResponseObjectResourceAttributesStatus::Draft,
            "campaign status {:?}",
            campaign.data.attributes.status
        );
        Ok(())
    }
    .await;

    let cleanup = client.campaigns().delete(&campaign_id).send().await;
    checks?;
    cleanup?;
    Ok(())
}

async fn assigned_template_html(client: &Client, message_id: &str) -> Result<String> {
    for _ in 0..TEMPLATE_ATTEMPTS {
        let assigned = client
            .campaigns()
            .message_template(message_id)
            .send()
            .await?;
        match assigned.data {
            Some(template) => return Ok(template.attributes.html),
            None => tokio::time::sleep(Duration::from_secs(1)).await,
        }
    }
    Err(format!("the message has no template after {TEMPLATE_ATTEMPTS} attempts").into())
}
