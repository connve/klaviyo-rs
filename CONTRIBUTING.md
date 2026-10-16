# Contributing to klaviyo-rs

## Repository shape

```
klaviyo-rs/
├── klaviyo/                # User-facing crate
│   ├── src/
│   │   ├── lib.rs          # Re-exports: API_REVISION, types, operations
│   │   ├── client.rs       # Client, Builder, Credentials
│   │   ├── error.rs        # Error, ApiError
│   │   ├── http.rs         # reqwest plumbing, JSON:API errors, next-page cursor
│   │   ├── headers.rs      # HeaderBag, reserved headers
│   │   ├── request.rs      # RequestBuilder, NoContent, Pages
│   │   └── <api>.rs        # One module per feature
│   └── tests/              # Live tests and mock_* tests
├── generated/
│   └── klaviyo/            # openapi.json, build.rs, operation traits in src/lib.rs
└── examples/klaviyo/
```

The wrapper sends requests itself. Paths, methods, and accepted query
options come from the spec through the operation markers.

## Auth

`Authorization: Klaviyo-API-Key <key>`. A key belongs to one account, so
multi-account callers build one `Client` per key.

The SDK takes the key as a value (`Builder::api_key`, or `Credentials`,
which deserializes from `{ "api_key": "pk_..." }`). It never reads files or
environment variables; where secrets come from is the application's call.
Tests and examples read `KLAVIYO_API_KEY`.

## Spec

`generated/klaviyo/openapi.json` is `openapi/stable.json` from
[klaviyo/openapi](https://github.com/klaviyo/openapi), committed verbatim.
Don't edit it or the generated output; change `build.rs`, which:

1. Keeps the operations in `OPERATIONS`. A missing ID fails the build.
2. Emits one marker per operation (`operations::GetCampaigns`) with its
   method, a `path(..)` taking one argument per path parameter, and a trait
   per accepted query option (`Include`, `Filter`, `Sort`, `PageSize`,
   `Paginated`, ...).
3. Rewrites `application/vnd.api+json` to `application/json`.
4. Points 4XX/5XX responses at one `ErrorDocument` schema. Left inline,
   progenitor names them `<OperationId>Response`, colliding with success
   schemas such as `GetAccountResponse`.
5. Prunes unreachable schemas.
6. Gives single-value enums (JSON:API `type`) a `default`, so they implement
   `Default`.
7. Hoists inline objects in request bodies into named components, with the
   names typify would use, so they can be patched.
8. Strips `default` from optional request properties; typify would always
   serialize them.
9. Patches `Default` onto request objects whose required fields all
   implement it.

### Upgrading the API revision

1. Replace `openapi.json` with the new `stable.json`.
2. `cargo build -p klaviyo_generated`; fix `OPERATIONS` for renamed or
   removed operations.
3. `cargo test --workspace --all-features`; renamed types, path parameters,
   and query options fail to compile in the wrapper.
4. A revision bump is at least a minor release.

## Request builders

Endpoint methods return `RequestBuilder<'a, Resp, Op>` under a per-endpoint
alias, where `Op` is the operation's marker. Builder methods are gated on the
marker's traits:

| Method | Trait |
| --- | --- |
| `fields` | `Fields` |
| `additional_fields` | `AdditionalFields` |
| `include` | `Include` |
| `filter` | `Filter` |
| `sort` | `Sort` |
| `page_size` | `PageSize` |
| `page_cursor`, `pages` | `Paginated` |

- Build requests from the marker: `RequestBuilder::new(client,
  operations::GetCampaign::path(id))`. `Path` is typed by the marker, so it
  can't be paired with another operation's alias.
- `202` / `204` operations return `NoContent`, never `()`.
- For a body whose `data.id` must match the path, use
  `http::json_body(&body, Some(id))`.
- When a body holds only IDs, take the IDs and build it from the generated
  types (`assign_template`, `send`, `add_profiles`).
- Reserved headers: `Authorization`, `revision`, `Content-Type`,
  `Content-Length`, `Transfer-Encoding`, `Accept`, `Host`, `User-Agent`.

`Pages` re-sends the original request with the cursor from `links.next`; it
never requests that URL. A `links.next` without a usable cursor is reported
as `Error::InvalidNextPage`.

## Errors

| Variant | Retryable |
| --- | --- |
| `Http` | on timeout, connect, request errors |
| `Api` | 5xx, 408, 425, 429 |
| everything else | no |

`Error::Api`'s message leaves out `body` and error details, which can contain
personal data. `ApiError` keeps unknown fields in `extra`.

- Every public error enum is `#[non_exhaustive]` and has `is_retryable()`.
- One variant per failure mode; no `Generic(String)`.
- `#[source]` with `.map_err(|source| Error::Variant { source })`, not
  `#[from]`.
- No `unwrap()`, `expect()` or `panic!()` outside tests.

## Adding an endpoint

1. Add the `operationId` to `OPERATIONS`.
2. Take the request and response types from the generated client method
   (`klaviyo_generated::Client::<operation_id>`); the marker is
   `operations::<OperationId in PascalCase>`.
3. Add the method and its builder alias to `klaviyo/src/<api>.rs`. A new
   module gets a feature enabling `_write` if it sends a body, `_api`
   otherwise.
4. Tests:
   - `tests/<api>.rs`: live tests against a test account. Run the real
     lifecycle and clean up even when a check fails: checks go in an `async`
     block using `ensure!` / `ensure_rejected!`, cleanup runs, then the first
     error is returned. Never send a campaign or track an event. For
     asynchronous operations, check acceptance only. Assert only what
     concurrent CI runs agree on.
   - `tests/mock_<api>.rs`: `wiremock` tests of method, path, query, and body.

   Live tests are `#[ignore = "needs a Klaviyo test account key in
   KLAVIYO_API_KEY"]`; mock tests aren't. Each test file opens with a
   `//!` saying what it covers, what it needs (the test account, or nothing),
   and why it is or isn't `#[ignore]`d.

   Build requests as typed literals ending in `..Default::default()`;
   `json!` is only for expected bodies and stubbed responses in mock tests.

   Register both in `klaviyo/Cargo.toml` with `required-features`.
5. Put the usage example in the module docs.
6. Bump the version and add a CHANGELOG entry in the same PR.

## Code style

- `module.rs` + `module/submodule.rs`, never `mod.rs`.
- `thiserror` in the library, `anyhow` only in examples.
- Comments only for a non-obvious *why*, written as complete sentences;
  none inside unit tests. Module intros, public items, struct fields and
  constants have rustdoc; examples needing credentials are `no_run`.
- Prefer `match` over `if`/`else if` chains and over combinator chains such
  as `.map(..).unwrap_or(..)`. When clippy asks for the combinator
  (`manual_unwrap_or_default`, ...), clippy wins.
- Data types derive `PartialEq, Clone, Debug, Default, Deserialize,
  Serialize` where they fit.
- Dates and times use `chrono`, never hand-computed from `SystemTime`.
- `#[cfg_attr(feature = "trace", tracing::instrument(skip_all))]` on public
  async methods.
- Re-export dependency types that appear in the public API.
- Lint suppressions are `#[expect(lint, reason = "...")]`; `#[allow]` is
  denied.
- Test behaviour, not derives, formatting or constants. Match errors with
  `matches!`, not message strings.

## Dependencies

- All versions live in the root `[workspace.dependencies]`.
- `klaviyo_generated` is pinned with `=x.y.z`; a release bumps both.
- `tokio` is a dev-dependency only.
- MSRV is 1.88.
- Every single-feature build must pass clippy without `#[expect]`; gate
  shared code on `_api` / `_write`.

## Checks

```bash
cargo fmt --all
cargo hack clippy -p klaviyo --each-feature --skip _api,_write --all-targets -- -D warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo package --workspace --locked
```

Live tests, against a test account:

```bash
KLAVIYO_API_KEY=pk_... cargo test -p klaviyo --all-features --tests -- --ignored
```

The pre-commit hook runs most of these:

```bash
chmod +x .githooks/pre-commit
git config core.hooksPath .githooks
```

## Releases

Conventional commits (`feat:`, `fix:`, `chore:`). Bump with
`cargo workspaces version --no-git-commit --yes <minor|patch>` and merge to
`main`; the release workflow tags and publishes.
