//! Run with:
//!
//! ```bash
//! cargo run --example fan_out_campaign --features campaigns,templates,lists -- markets.json
//! ```
//!
//! Turns one master email into a draft campaign in each market's account:
//! fills the master template's `[[placeholders]]` with the market's copy,
//! creates or updates the market's template, and creates a draft campaign
//! for the market's list with that template. Nothing is sent.
//!
//! `markets.json` names, for each account, the environment variable that
//! holds its private API key:
//!
//! ```json
//! {
//!   "master": { "api_key_env": "KLAVIYO_API_KEY_MASTER", "template_id": "RipRmi" },
//!   "campaign_name": "Autumn sale 2026",
//!   "from_email": "hello@example.com",
//!   "from_label": "Example",
//!   "markets": [
//!     {
//!       "market": "PL",
//!       "api_key_env": "KLAVIYO_API_KEY_PL",
//!       "list_name": "Newsletter",
//!       "subject": "Jesienna wyprzedaż",
//!       "preview_text": "Do -40% tylko do niedzieli",
//!       "content": { "headline": "Jesienna wyprzedaż", "cta_url": "https://example.com/pl/sale" }
//!     }
//!   ]
//! }
//! ```
//!
//! `[[name]]` leaves Klaviyo's own `{{ ... }}` tags for send time.

use std::collections::BTreeMap;
use std::env;

use anyhow::{bail, Context, Result};
use klaviyo::serde_json;
use klaviyo::{types, Client};
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    master: Master,
    campaign_name: String,
    from_email: String,
    from_label: String,
    markets: Vec<Market>,
}

#[derive(Deserialize)]
struct Master {
    api_key_env: String,
    template_id: String,
}

#[derive(Deserialize)]
struct Market {
    market: String,
    api_key_env: String,
    list_name: String,
    subject: String,
    preview_text: String,
    #[serde(default)]
    content: BTreeMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config_path = env::args()
        .nth(1)
        .context("pass the path to markets.json as the first CLI arg")?;
    let config: Config = serde_json::from_str(
        &std::fs::read_to_string(&config_path).with_context(|| format!("reading {config_path}"))?,
    )
    .with_context(|| format!("parsing {config_path}"))?;

    let master = account_client(&config.master.api_key_env)?;
    let template = master
        .templates()
        .get(&config.master.template_id)
        .send()
        .await
        .context("fetching the master template")?;
    let master_html = template.data.attributes.html;
    let master_text = template.data.attributes.text.unwrap_or_default();

    for market in &config.markets {
        let campaign_id = fan_out(&config, market, &master_html, &master_text)
            .await
            .with_context(|| format!("market {}", market.market))?;
        println!("{}: draft campaign {campaign_id}", market.market);
    }

    Ok(())
}

async fn fan_out(
    config: &Config,
    market: &Market,
    master_html: &str,
    master_text: &str,
) -> Result<String> {
    let client = account_client(&market.api_key_env)?;
    let name = format!("{} [{}]", config.campaign_name, market.market);

    let html = fill(master_html, &market.content)?;
    let text = fill(master_text, &market.content)?;
    let template_id = upsert_template(&client, &name, &html, &text).await?;
    let list_id = find_list(&client, &market.list_name).await?;

    let message = types::CampaignMessageCreateQueryResourceObject {
        attributes: types::CampaignMessageCreateQueryResourceObjectAttributes {
            definition: types::EmailMessageDefinition {
                label: Some(name.clone()),
                content: Some(types::EmailContent {
                    subject: Some(market.subject.clone()),
                    preview_text: Some(market.preview_text.clone()),
                    from_email: Some(config.from_email.clone()),
                    from_label: Some(config.from_label.clone()),
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
                name,
                audiences: types::Audiences {
                    included: vec![list_id],
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
    let campaign = client.campaigns().create(&request).send().await?;
    let campaign_id = campaign.data.id;

    let messages = client.campaigns().messages(&campaign_id).send().await?;
    for message in &messages.data {
        client
            .campaigns()
            .assign_template(&message.id, &template_id)
            .send()
            .await?;
    }

    Ok(campaign_id)
}

/// A client for the account whose private API key is in `api_key_env`.
fn account_client(api_key_env: &str) -> Result<Client> {
    let api_key = env::var(api_key_env).with_context(|| format!("reading {api_key_env}"))?;
    Ok(Client::builder().api_key(api_key).build()?)
}

/// Fails if a placeholder is left unfilled.
fn fill(master: &str, content: &BTreeMap<String, String>) -> Result<String> {
    let mut filled = master.to_owned();
    for (key, value) in content {
        filled = filled.replace(&format!("[[{key}]]"), value);
    }
    if let Some(start) = filled.find("[[") {
        let rest = &filled[start..];
        let end = match rest.find("]]") {
            Some(end) => end + 2,
            None => rest.len(),
        };
        bail!("no content for placeholder {}", &rest[..end]);
    }
    Ok(filled)
}

/// Reuses a template of the same name, to stay under Klaviyo's limit of
/// 1,000 API-created templates.
async fn upsert_template(client: &Client, name: &str, html: &str, text: &str) -> Result<String> {
    let existing = client
        .templates()
        .list()
        .filter(&format!("equals(name,{})", filter_string(name)?))
        .send()
        .await?;

    if let Some(template) = existing.data.first() {
        let request = types::TemplateUpdateHtmlOrDndQuery {
            data: types::TemplateUpdateHtmlOrDndQueryResourceObject {
                attributes: types::TemplateUpdateHtmlOrDndQueryResourceObjectAttributes {
                    html: Some(html.to_owned()),
                    text: Some(text.to_owned()),
                    ..Default::default()
                },
                ..Default::default()
            },
        };
        client
            .templates()
            .update(&template.id, &request)
            .send()
            .await?;
        return Ok(template.id.clone());
    }

    let request = types::TemplateCreateHtmlOrDndQuery {
        data: types::TemplateCreateHtmlOrDndQueryResourceObject {
            attributes: types::TemplateCreateHtmlOrDndQueryResourceObjectAttributes {
                name: name.to_owned(),
                editor_type: "CODE".to_owned(),
                html: Some(html.to_owned()),
                text: Some(text.to_owned()),
                ..Default::default()
            },
            ..Default::default()
        },
    };
    let created = client.templates().create(&request).send().await?;
    Ok(created.data.id)
}

async fn find_list(client: &Client, list_name: &str) -> Result<String> {
    let lists = client
        .lists()
        .list()
        .filter(&format!("equals(name,{})", filter_string(list_name)?))
        .send()
        .await?;
    match lists.data.first() {
        Some(list) => Ok(list.id.clone()),
        None => bail!("no list named {list_name:?}"),
    }
}

fn filter_string(value: &str) -> Result<String> {
    if value.contains('\'') {
        bail!("{value:?} contains a single quote, which a filter value can't hold");
    }
    Ok(format!("'{value}'"))
}
