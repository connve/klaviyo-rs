//! Run with:
//!
//! ```bash
//! export KLAVIYO_API_KEY=pk_...
//! cargo run --example track_event --features events -- ada@example.com ORDER-1001 129.90
//! ```
//!
//! Tracks a `Placed Order` event, which also creates or updates the profile.
//! `unique_id` is the order number, so tracking an order twice counts once.

use std::env;

use anyhow::{Context, Result};
use klaviyo::serde_json::{Map, Value};
use klaviyo::{types, Client};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let api_key =
        env::var("KLAVIYO_API_KEY").context("set KLAVIYO_API_KEY to a private API key")?;

    let mut args = env::args().skip(1);
    let email = args
        .next()
        .context("pass the customer email as the first CLI arg")?;
    let order_id = args
        .next()
        .context("pass the order ID as the second CLI arg")?;
    let value: f64 = args
        .next()
        .context("pass the order value as the third CLI arg")?
        .parse()
        .context("order value must be a number")?;

    let client = Client::builder().api_key(api_key).build()?;

    let event = types::EventCreateQueryV2 {
        data: types::EventCreateQueryV2ResourceObject {
            attributes: types::EventCreateQueryV2ResourceObjectAttributes {
                properties: Map::from_iter([("OrderId".to_owned(), Value::from(order_id.clone()))]),
                value: Some(value),
                unique_id: Some(order_id.clone()),
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
                            email: Some(email.clone()),
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

    client.events().create(&event).send().await?;
    println!("queued Placed Order {order_id} for {email}");

    Ok(())
}
