# klaviyo-rs

[![Test Suite](https://github.com/connve/klaviyo-rs/actions/workflows/test.yml/badge.svg)](https://github.com/connve/klaviyo-rs/actions/workflows/test.yml)
[![Security Audit](https://github.com/connve/klaviyo-rs/actions/workflows/security.yml/badge.svg)](https://github.com/connve/klaviyo-rs/actions/workflows/security.yml)
[![Release](https://img.shields.io/github/v/release/connve/klaviyo-rs)](https://github.com/connve/klaviyo-rs/releases)
[![Crates.io](https://img.shields.io/crates/v/klaviyo.svg)](https://crates.io/crates/klaviyo)
[![Docs.rs](https://docs.rs/klaviyo/badge.svg)](https://docs.rs/klaviyo)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)

Unofficial Rust SDK for Klaviyo APIs.

> Not affiliated with or endorsed by Klaviyo, Inc.

## Workspace layout

```
klaviyo-rs/
├── klaviyo/                # User-facing crate
├── generated/
│   └── klaviyo/            # progenitor-generated types + Klaviyo's openapi.json
└── examples/
    └── klaviyo/            # Runnable examples
```

## Quick start

Every API sits behind a Cargo feature; none are enabled by default.

```toml
[dependencies]
klaviyo = { version = "0.1", features = ["campaigns", "profiles"] }
```

```rust
use klaviyo::Client;

let client = Client::builder()
    .api_key(std::env::var("KLAVIYO_API_KEY")?)
    .build()?;

let drafts = client
    .campaigns()
    .list("and(equals(messages.channel,'email'),equals(status,'Draft'))")
    .send()
    .await?;

let mut pages = client.profiles().list().page_size(100).pages();
while let Some(page) = pages.next_page().await {
    for profile in page?.data {
        println!("{:?}", profile.id);
    }
}
```

## Credentials

The client takes a private API key (`pk_...`), created under *Settings → API
keys*. Where the key comes from is up to the application: `Credentials`
deserializes from `{ "api_key": "pk_..." }` for config files and secret
stores.

A key belongs to one account, so use one `Client` per account.

## Examples

```bash
# One master email → a draft campaign in each market's account. Sends nothing.
cargo run --example fan_out_campaign --features campaigns,templates,lists -- markets.json

export KLAVIYO_API_KEY=pk_...
cargo run --example track_event --features events -- ada@example.com ORDER-1001 129.90
```

## Coverage

API revision `2026-07-15`.

| API | Feature | Endpoints |
| --- | --- | --- |
| Accounts | `accounts` | `GET /api/accounts`, `GET /api/accounts/{id}` |
| Campaigns | `campaigns` | `GET/POST /api/campaigns`, `GET/PATCH/DELETE /api/campaigns/{id}`, `POST /api/campaign-clone` |
| Campaigns | `campaigns` | `GET /api/campaigns/{id}/campaign-messages`, `GET/PATCH /api/campaign-messages/{id}`, `GET /api/campaign-messages/{id}/template` |
| Campaigns | `campaigns` | `POST /api/campaign-message-assign-template` |
| Campaigns | `campaigns` | `POST /api/campaign-send-jobs`, `GET/PATCH /api/campaign-send-jobs/{id}` |
| Campaigns | `campaigns` | `POST /api/campaign-recipient-estimation-jobs`, `GET .../{id}`, `GET /api/campaign-recipient-estimations/{id}` |
| Events | `events` | `GET/POST /api/events`, `GET /api/events/{id}`, `POST /api/event-bulk-create-jobs` |
| Flows | `flows` | `GET /api/flows`, `GET/PATCH /api/flows/{id}` |
| Lists | `lists` | `GET/POST /api/lists`, `GET/PATCH/DELETE /api/lists/{id}` |
| Lists | `lists` | `GET /api/lists/{id}/profiles`, `POST/DELETE /api/lists/{id}/relationships/profiles` |
| Metrics | `metrics` | `GET /api/metrics`, `GET /api/metrics/{id}`, `POST /api/metric-aggregates` |
| Profiles | `profiles` | `GET/POST /api/profiles`, `GET/PATCH /api/profiles/{id}`, `POST /api/profile-import` |
| Profiles | `profiles` | `POST /api/profile-bulk-import-jobs`, `GET .../{id}`, `GET .../{id}/import-errors` |
| Profiles | `profiles` | `POST /api/profile-subscription-bulk-{create,delete}-jobs`, `POST /api/profile-suppression-bulk-{create,delete}-jobs` |
| Reporting | `reporting` | `POST /api/campaign-values-reports`, `POST /api/flow-values-reports`, `POST /api/flow-series-reports` |
| Segments | `segments` | `GET /api/segments`, `GET /api/segments/{id}`, `GET /api/segments/{id}/profiles` |
| Templates | `templates` | `GET/POST /api/templates`, `GET/PATCH/DELETE /api/templates/{id}`, `POST /api/template-render`, `POST /api/template-clone` |
| Templates | `templates` | `GET/POST /api/template-universal-content`, `GET/PATCH/DELETE .../{id}` |

Catalogs, coupons, web feeds, images, tags and webhooks aren't covered yet.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MPL-2.0
