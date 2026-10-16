# Changelog

All notable changes are documented here. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
