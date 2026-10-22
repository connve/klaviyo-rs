# Changelog

All notable changes are documented here. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0]

### Changed
- A template's `definition` is `serde_json::Value` in requests and responses, replacing the generated `TemplateDefinition` types. A fetched definition passes back into `create` or `update` unchanged.

### Fixed
- `templates().get` and `templates().list` no longer fail to deserialize drag-and-drop templates.

## [0.3.0]

### Changed
- Target Klaviyo API revision `2026-10-15`.
- Campaigns follow Klaviyo's new campaign model: `create` takes a `definition` with inline `audiences` and `messages`, each message carrying its `variations`; `update` takes a `definition`. Responses are `*OmniCampaign*` types, without `status`, `send_strategy`, `send_options` or `tracking_options`.
- `campaigns().list()` takes no filter. Klaviyo no longer requires a channel filter and dropped `messages.channel`, `status` and `scheduled_at` from filtering and sorting; filter with `.filter(..)`.
- Campaign messages are `*Message*` types; `update_message` takes `MessagePartialUpdateQuery`, which no longer carries the subject, preview text or sender: those belong to the message's variations. Message status is `definition.status`.
- Campaign messages relate to `campaign-variations` instead of `template` and `image`, in both `include` and sparse fieldsets.
- `fields[campaign]` names `definition.*` fields.
- `clone_campaign` uses the `clone_campaign` operation (was `create_campaign_clone`); its body takes `name` instead of `new_name`.
- Universal content uses the `*V1` request and response types and accepts every block type. An update's `definition` replaces the whole block.
- `PushActionButton` in flow push notifications is `PushActionButtonV2`.

### Added
- `campaigns().variations(message_id)` and `campaigns().update_variation(variation_id, ..)`. A variation's template is `template_id` in its email `details`, set at `create` or with `update_variation`.
- `fields[campaign-audience]` and `include=campaign-audiences` on campaign reads, `page_cursor` and `pages` on `messages`.
- Events filter on `properties`.
- Flow A/B test actions report `concluded_experiment_id` and `winning_variation_id`.
- `profiles().unsubscribe` accepts a profile `id`.

### Removed
- `campaigns().assign_template` and `campaigns().message_template`. On revision `2026-10-15` the assign endpoint answers 404 for messages of campaigns created with the new model; set `template_id` on the variation instead.

### Security
- Bump `rustls-webpki` to 0.103.16 (RUSTSEC-2026-0382) and `webpki-root-certs` to 1.0.10 (RUSTSEC-2026-0383).

## [0.2.0]

### Changed
- README lists the SDK methods per feature, each linked to Klaviyo's API reference.
- Releases bump both crates together and keep the `=x.y.z` pin on `klaviyo_generated` in step.

## [0.1.0]

### Added
- Initial release, targeting Klaviyo API revision `2026-07-15` (`klaviyo::API_REVISION`).
- `klaviyo` wrapper over `klaviyo_generated`, which is generated from Klaviyo's published `openapi.json`.
- `Client` with `Klaviyo-API-Key` auth from `Builder::api_key` or a deserializable `Credentials`.
- Features: `accounts`, `campaigns`, `events`, `flows`, `lists`, `metrics`, `profiles`, `reporting`, `segments`, `templates` (endpoints in the README).
- Request builders typed by per-operation markers (`klaviyo::operations`), offering only the query options each operation accepts: `fields`, `additional_fields`, `include`, `filter`, `sort`, `page_size`, `page_cursor`.
- `pages()` over collections and reports, following the `links.next` cursor.
- `Error::is_retryable()` and `Error::retry_after()`; `Error::Api` carries Klaviyo's JSON:API `errors`.
- `campaigns().assign_template(message_id, template_id)`, `send(campaign_id)` and `lists().add_profiles(list_id, ids)` take IDs instead of documents.
- `fan_out_campaign` and `track_event` examples.
