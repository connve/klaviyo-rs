# klaviyo-rs

[![Test Suite](https://github.com/connve/klaviyo-rs/actions/workflows/test.yml/badge.svg)](https://github.com/connve/klaviyo-rs/actions/workflows/test.yml)
[![Security Audit](https://github.com/connve/klaviyo-rs/actions/workflows/security.yml/badge.svg)](https://github.com/connve/klaviyo-rs/actions/workflows/security.yml)
[![Release](https://img.shields.io/github/v/release/connve/klaviyo-rs)](https://github.com/connve/klaviyo-rs/releases)
[![Crates.io](https://img.shields.io/crates/v/klaviyo.svg)](https://crates.io/crates/klaviyo)
[![Docs.rs](https://docs.rs/klaviyo/badge.svg)](https://docs.rs/klaviyo)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)

Unofficial Rust SDK for Klaviyo APIs.

> Not affiliated with or endorsed by Klaviyo, Inc.

## Quick start

```toml
[dependencies]
klaviyo = { version = "0.3", features = ["campaigns", "profiles"] }
```

```rust
let client = klaviyo::Client::builder()
    .api_key(api_key)
    .build()?;
```

## Coverage

API revision `2026-10-15`.

| API | Feature | Methods |
| --- | --- | --- |
| Accounts | `accounts` | [`list`](https://developers.klaviyo.com/en/reference/get_accounts), [`get`](https://developers.klaviyo.com/en/reference/get_account) |
| Campaigns | `campaigns` | [`list`](https://developers.klaviyo.com/en/reference/get_campaigns), [`get`](https://developers.klaviyo.com/en/reference/get_campaign), [`create`](https://developers.klaviyo.com/en/reference/create_campaign), [`update`](https://developers.klaviyo.com/en/reference/update_campaign), [`delete`](https://developers.klaviyo.com/en/reference/delete_campaign), [`clone_campaign`](https://developers.klaviyo.com/en/reference/clone_campaign), [`messages`](https://developers.klaviyo.com/en/reference/get_messages_for_campaign), [`get_message`](https://developers.klaviyo.com/en/reference/get_campaign_message), [`update_message`](https://developers.klaviyo.com/en/reference/update_campaign_message), [`variations`](https://developers.klaviyo.com/en/reference/get_variations_for_campaign_message), [`update_variation`](https://developers.klaviyo.com/en/reference/update_campaign_variation), [`send`](https://developers.klaviyo.com/en/reference/send_campaign), [`get_send_job`](https://developers.klaviyo.com/en/reference/get_campaign_send_job), [`cancel_send`](https://developers.klaviyo.com/en/reference/cancel_campaign_send), [`refresh_recipient_estimation`](https://developers.klaviyo.com/en/reference/refresh_campaign_recipient_estimation), [`get_recipient_estimation_job`](https://developers.klaviyo.com/en/reference/get_campaign_recipient_estimation_job), [`get_recipient_estimation`](https://developers.klaviyo.com/en/reference/get_campaign_recipient_estimation) |
| Events | `events` | [`list`](https://developers.klaviyo.com/en/reference/get_events), [`get`](https://developers.klaviyo.com/en/reference/get_event), [`create`](https://developers.klaviyo.com/en/reference/create_event), [`bulk_create`](https://developers.klaviyo.com/en/reference/bulk_create_events) |
| Flows | `flows` | [`list`](https://developers.klaviyo.com/en/reference/get_flows), [`get`](https://developers.klaviyo.com/en/reference/get_flow), [`update`](https://developers.klaviyo.com/en/reference/update_flow) |
| Lists | `lists` | [`list`](https://developers.klaviyo.com/en/reference/get_lists), [`get`](https://developers.klaviyo.com/en/reference/get_list), [`create`](https://developers.klaviyo.com/en/reference/create_list), [`update`](https://developers.klaviyo.com/en/reference/update_list), [`delete`](https://developers.klaviyo.com/en/reference/delete_list), [`profiles`](https://developers.klaviyo.com/en/reference/get_profiles_for_list), [`add_profiles`](https://developers.klaviyo.com/en/reference/add_profiles_to_list), [`remove_profiles`](https://developers.klaviyo.com/en/reference/remove_profiles_from_list) |
| Metrics | `metrics` | [`list`](https://developers.klaviyo.com/en/reference/get_metrics), [`get`](https://developers.klaviyo.com/en/reference/get_metric), [`query_aggregates`](https://developers.klaviyo.com/en/reference/query_metric_aggregates) |
| Profiles | `profiles` | [`list`](https://developers.klaviyo.com/en/reference/get_profiles), [`get`](https://developers.klaviyo.com/en/reference/get_profile), [`create`](https://developers.klaviyo.com/en/reference/create_profile), [`update`](https://developers.klaviyo.com/en/reference/update_profile), [`upsert`](https://developers.klaviyo.com/en/reference/create_or_update_profile), [`bulk_import`](https://developers.klaviyo.com/en/reference/bulk_import_profiles), [`get_bulk_import_job`](https://developers.klaviyo.com/en/reference/get_bulk_import_profiles_job), [`bulk_import_errors`](https://developers.klaviyo.com/en/reference/get_errors_for_bulk_import_profiles_job), [`subscribe`](https://developers.klaviyo.com/en/reference/bulk_subscribe_profiles), [`unsubscribe`](https://developers.klaviyo.com/en/reference/bulk_unsubscribe_profiles), [`suppress`](https://developers.klaviyo.com/en/reference/bulk_suppress_profiles), [`unsuppress`](https://developers.klaviyo.com/en/reference/bulk_unsuppress_profiles) |
| Reporting | `reporting` | [`campaign_values`](https://developers.klaviyo.com/en/reference/query_campaign_values), [`flow_values`](https://developers.klaviyo.com/en/reference/query_flow_values), [`flow_series`](https://developers.klaviyo.com/en/reference/query_flow_series) |
| Segments | `segments` | [`list`](https://developers.klaviyo.com/en/reference/get_segments), [`get`](https://developers.klaviyo.com/en/reference/get_segment), [`profiles`](https://developers.klaviyo.com/en/reference/get_profiles_for_segment) |
| Templates | `templates` | [`list`](https://developers.klaviyo.com/en/reference/get_templates), [`get`](https://developers.klaviyo.com/en/reference/get_template), [`create`](https://developers.klaviyo.com/en/reference/create_template), [`update`](https://developers.klaviyo.com/en/reference/update_template), [`delete`](https://developers.klaviyo.com/en/reference/delete_template), [`render`](https://developers.klaviyo.com/en/reference/render_template), [`clone_template`](https://developers.klaviyo.com/en/reference/clone_template), [`list_universal_content`](https://developers.klaviyo.com/en/reference/get_all_universal_content), [`get_universal_content`](https://developers.klaviyo.com/en/reference/get_universal_content), [`create_universal_content`](https://developers.klaviyo.com/en/reference/create_universal_content), [`update_universal_content`](https://developers.klaviyo.com/en/reference/update_universal_content), [`delete_universal_content`](https://developers.klaviyo.com/en/reference/delete_universal_content) |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MPL-2.0
