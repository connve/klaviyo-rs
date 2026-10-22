//! End-to-end test of the per-market flow in one account: a list and a
//! template, then a draft campaign for the list whose variation uses the
//! template, set either at creation or with `update_variation` afterwards.
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

/// How often to look for the variation's template, one second apart.
const TEMPLATE_ATTEMPTS: u32 = 10;

#[derive(Clone, Copy)]
enum Template {
    AtCreation,
    Updated,
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn draft_campaign_with_variation_template() -> Result {
    fan_out(Template::AtCreation).await
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn draft_campaign_with_updated_variation_template() -> Result {
    fan_out(Template::Updated).await
}

async fn fan_out(how: Template) -> Result {
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

    let checks = draft_campaign(&client, &name, &list_id, &template_id, how).await;

    let template_cleanup = client.templates().delete(&template_id).send().await;
    let list_cleanup = client.lists().delete(&list_id).send().await;
    checks?;
    template_cleanup?;
    list_cleanup?;
    Ok(())
}

async fn draft_campaign(
    client: &Client,
    name: &str,
    list_id: &str,
    template_id: &str,
    how: Template,
) -> Result {
    let inline_template_id = match how {
        Template::AtCreation => Some(template_id.to_owned()),
        Template::Updated => None,
    };
    let variation = types::VariationInlineCreate {
        definition: types::VariationDefinition {
            name: Some(name.to_owned()),
            details: Some(
                types::EmailChannelDetails {
                    channel: Some(types::EmailChannelDetailsChannel::Email),
                    subject: Some(name.to_owned()),
                    preview_text: Some(MARKER.to_owned()),
                    template_id: inline_template_id,
                    ..Default::default()
                }
                .into(),
            ),
        },
    };
    let request = types::CampaignCreateQuery {
        data: types::CampaignCreateQueryResourceObject {
            attributes: types::CampaignCreateQueryResourceObjectAttributes {
                definition: types::CampaignDefinition {
                    name: Some(name.to_owned()),
                    ..Default::default()
                },
                audiences: Some(vec![types::AudienceInlineCreate {
                    temporary_id: "audience".to_owned(),
                    included: Some(vec![list_id.to_owned()]),
                    ..Default::default()
                }]),
                messages: Some(vec![types::MessageInlineCreate {
                    audience_temporary_id: "audience".to_owned(),
                    name: Some(name.to_owned()),
                    variations: Some(vec![variation]),
                    ..Default::default()
                }]),
            },
            ..Default::default()
        },
    };
    let campaign_id = client.campaigns().create(&request).send().await?.data.id;

    let checks: Result = async {
        let messages = client.campaigns().messages(&campaign_id).send().await?;
        let message = messages.data.first().ok_or("the campaign has no message")?;

        if let Template::Updated = how {
            let variations = client.campaigns().variations(&message.id).send().await?;
            let variation = variations
                .data
                .first()
                .ok_or("the message has no variation")?;
            let request = types::VariationPartialUpdateQuery {
                data: types::VariationPartialUpdateQueryResourceObject {
                    attributes: types::VariationPartialUpdateQueryResourceObjectAttributes {
                        definition: types::VariationDefinition {
                            details: Some(
                                types::EmailChannelDetails {
                                    channel: Some(types::EmailChannelDetailsChannel::Email),
                                    template_id: Some(template_id.to_owned()),
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
            client
                .campaigns()
                .update_variation(&variation.id, &request)
                .send()
                .await?;
        }

        let html = variation_template_html(client, &message.id).await?;
        ensure!(html.contains(MARKER), "variation template: {html}");

        let variations = client.campaigns().variations(&message.id).send().await?;
        let subject = variations.data.first().and_then(|variation| {
            match &variation.attributes.definition.details {
                Some(types::VariationDefinitionDetails::EmailChannelDetails(details)) => {
                    details.subject.clone()
                }
                _ => None,
            }
        });
        ensure!(
            subject.as_deref() == Some(name),
            "variation subject {subject:?}"
        );

        let message = client.campaigns().get_message(&message.id).send().await?;
        ensure!(
            message.data.attributes.definition.status
                == Some(types::MessageDefinitionStatus::Draft),
            "message status {:?}",
            message.data.attributes.definition.status
        );
        Ok(())
    }
    .await;

    let cleanup = client.campaigns().delete(&campaign_id).send().await;
    checks?;
    cleanup?;
    Ok(())
}

async fn variation_template_html(client: &Client, message_id: &str) -> Result<String> {
    for _ in 0..TEMPLATE_ATTEMPTS {
        let message = client
            .campaigns()
            .get_message(message_id)
            .include(&["campaign-variations"])
            .send()
            .await?;
        match variation_template_id(&message) {
            Some(template_id) => {
                let template = client.templates().get(&template_id).send().await?;
                return Ok(template.data.attributes.html);
            }
            None => tokio::time::sleep(Duration::from_secs(1)).await,
        }
    }
    Err(format!("the variation has no template after {TEMPLATE_ATTEMPTS} attempts").into())
}

fn variation_template_id(message: &types::GetMessageResponseCompoundDocument) -> Option<String> {
    message.included.iter().find_map(|item| match item {
        types::GetMessageResponseCompoundDocumentIncludedItem::VariationResponseObjectResource(
            variation,
        ) => match &variation.attributes.definition.details {
            Some(types::VariationDefinitionDetails::EmailChannelDetails(details)) => {
                details.template_id.clone()
            }
            _ => None,
        },
        _ => None,
    })
}
