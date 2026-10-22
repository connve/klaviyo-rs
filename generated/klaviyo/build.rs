//! Generates the Klaviyo types and operation markers from `openapi.json`.
//!
//! The spec is normalised first so progenitor can generate it: unused
//! operations are dropped, media types and error responses are unified, and
//! request objects are made `Default` where their fields allow it.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::PathBuf;

use heck::{ToPascalCase, ToSnakeCase};
use openapiv3::OpenAPI;
use serde_json::{json, Map, Value};

/// Operations to generate, by `operationId`. An ID missing from the spec
/// fails the build.
const OPERATIONS: &[&str] = &[
    // Accounts
    "get_accounts",
    "get_account",
    // Campaigns
    "get_campaigns",
    "get_campaign",
    "create_campaign",
    "update_campaign",
    "delete_campaign",
    "clone_campaign",
    "get_messages_for_campaign",
    "get_campaign_message",
    "update_campaign_message",
    "get_variations_for_campaign_message",
    "update_campaign_variation",
    "send_campaign",
    "get_campaign_send_job",
    "cancel_campaign_send",
    "refresh_campaign_recipient_estimation",
    "get_campaign_recipient_estimation_job",
    "get_campaign_recipient_estimation",
    // Events
    "get_events",
    "get_event",
    "create_event",
    "bulk_create_events",
    // Flows
    "get_flows",
    "get_flow",
    "update_flow",
    // Lists
    "get_lists",
    "get_list",
    "create_list",
    "update_list",
    "delete_list",
    "get_profiles_for_list",
    "add_profiles_to_list",
    "remove_profiles_from_list",
    // Metrics
    "get_metrics",
    "get_metric",
    "query_metric_aggregates",
    // Profiles
    "get_profiles",
    "get_profile",
    "create_profile",
    "update_profile",
    "create_or_update_profile",
    "bulk_import_profiles",
    "get_bulk_import_profiles_job",
    "get_errors_for_bulk_import_profiles_job",
    "bulk_subscribe_profiles",
    "bulk_unsubscribe_profiles",
    "bulk_suppress_profiles",
    "bulk_unsuppress_profiles",
    // Reporting
    "query_campaign_values",
    "query_flow_values",
    "query_flow_series",
    // Segments
    "get_segments",
    "get_segment",
    "get_profiles_for_segment",
    // Templates
    "get_templates",
    "get_template",
    "create_template",
    "update_template",
    "delete_template",
    "render_template",
    "clone_template",
    "get_all_universal_content",
    "get_universal_content",
    "create_universal_content",
    "update_universal_content",
    "delete_universal_content",
];

/// The keys of an OpenAPI path item that hold an operation.
const HTTP_METHODS: &[&str] = &[
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

/// The prefix of a `$ref` pointing at a component schema.
const SCHEMA_REF_PREFIX: &str = "#/components/schemas/";

/// The component every 4XX and 5XX response is pointed at.
const ERROR_SCHEMA: &str = "ErrorDocument";

/// Components generated as `serde_json::Value`. Klaviyo returns drag-and-drop
/// template definitions that the generated types reject, and a fetched
/// definition must pass back into a request unchanged.
const UNTYPED: &[&str] = &["TemplateDefinition"];

type BuildResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> BuildResult<()> {
    let spec_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("openapi.json");
    let mut spec: Value = serde_json::from_str(&fs::read_to_string(&spec_path)?)?;

    keep_operations(&mut spec)?;
    let operations = operation_markers(&spec)?;
    rewrite_media_types(&mut spec);
    name_error_schema(&mut spec)?;
    untype_refs(&mut spec);
    prune_schemas(&mut spec);
    default_single_value_enums(&mut spec);
    let request_schemas = hoist_request_objects(&mut spec);
    strip_optional_defaults(&mut spec, &request_schemas);
    let default_types = defaultable_with_required_fields(&spec, &request_schemas);

    let revision = spec["info"]["version"]
        .as_str()
        .ok_or("spec has no `info.version`")?
        .to_owned();
    let spec: OpenAPI = serde_json::from_value(spec)?;

    // typify may PascalCase a schema name and ignores patches for names it
    // doesn't use, so register both spellings.
    let mut default_patch = progenitor::TypePatch::default();
    default_patch.with_derive("Default");
    let mut settings = progenitor::GenerationSettings::default();
    for name in &default_types {
        settings.with_patch(name, &default_patch);
        let pascal = name.to_pascal_case();
        if &pascal != name {
            settings.with_patch(pascal, &default_patch);
        }
    }

    let mut generator = progenitor::Generator::new(&settings);
    let tokens = generator.generate_tokens(&spec)?;

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    fs::write(
        out_dir.join("generated.rs"),
        format!(
            "{tokens}\n\
             /// The spec's API revision.\n\
             pub const API_REVISION: &str = {revision:?};\n"
        ),
    )?;
    fs::write(out_dir.join("operations.rs"), operations)?;

    println!("cargo:rerun-if-changed=openapi.json");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

/// Drops every operation not in [`OPERATIONS`], then every path left empty.
fn keep_operations(spec: &mut Value) -> BuildResult<()> {
    let wanted: BTreeSet<&str> = OPERATIONS.iter().copied().collect();
    let mut found = BTreeSet::new();

    let paths = spec
        .get_mut("paths")
        .and_then(Value::as_object_mut)
        .ok_or("spec has no `paths` object")?;
    for item in paths.values_mut() {
        let Some(item) = item.as_object_mut() else {
            continue;
        };
        item.retain(|method, operation| {
            if !HTTP_METHODS.contains(&method.as_str()) {
                return true;
            }
            match operation.get("operationId").and_then(Value::as_str) {
                Some(id) if wanted.contains(id) => {
                    found.insert(id.to_owned());
                    true
                }
                _ => false,
            }
        });
    }
    paths.retain(|_, item| {
        item.as_object()
            .is_some_and(|item| item.keys().any(|k| HTTP_METHODS.contains(&k.as_str())))
    });

    let missing: Vec<_> = wanted.iter().filter(|id| !found.contains(**id)).collect();
    if !missing.is_empty() {
        return Err(format!("operations not found in openapi.json: {missing:?}").into());
    }
    Ok(())
}

/// One marker type per operation, with its method, path, and accepted query
/// options. The traits are in `src/lib.rs`.
fn operation_markers(spec: &Value) -> BuildResult<String> {
    let paths = spec
        .get("paths")
        .and_then(Value::as_object)
        .ok_or("spec has no `paths` object")?;

    let mut source = String::new();
    for (path, item) in paths {
        let Some(item) = item.as_object() else {
            continue;
        };
        let shared = item.get("parameters").and_then(Value::as_array);
        for (method, operation) in item
            .iter()
            .filter(|(method, _)| HTTP_METHODS.contains(&method.as_str()))
        {
            let id = operation["operationId"]
                .as_str()
                .ok_or_else(|| format!("{method} {path} has no operationId"))?;
            let parameters: Vec<&Value> = shared
                .into_iter()
                .flatten()
                .chain(operation["parameters"].as_array().into_iter().flatten())
                .collect();
            source.push_str(&operation_marker(id, method, path, &parameters));
        }
    }
    Ok(source)
}

fn operation_marker(id: &str, method: &str, path: &str, parameters: &[&Value]) -> String {
    let name = id.to_pascal_case();
    let http_method = method.to_uppercase();

    let mut arguments = Vec::new();
    let mut segments = Vec::new();
    for segment in path.split('/').filter(|segment| !segment.is_empty()) {
        match segment
            .strip_prefix('{')
            .and_then(|segment| segment.strip_suffix('}'))
        {
            Some(parameter) => {
                let argument = rust_identifier(&parameter.to_snake_case());
                segments.push(argument.clone());
                arguments.push(argument);
            }
            None => segments.push(format!("{segment:?}")),
        }
    }
    let (generics, signature, lifetime) = if arguments.is_empty() {
        ("", String::new(), "'static")
    } else {
        let signature = arguments
            .iter()
            .map(|argument| format!("{argument}: &'p str"))
            .collect::<Vec<_>>()
            .join(", ");
        ("<'p>", signature, "'p")
    };

    let mut source = format!(
        "/// `{http_method} {path}`\n\
         #[derive(Debug)]\n\
         pub enum {name} {{}}\n\
         impl {name} {{\n\
         /// The path with its parameters filled in.\n\
         pub fn path{generics}({signature}) -> Path<{lifetime}, Self> {{\n\
         Path::new(::std::vec![{segments}])\n\
         }}\n\
         }}\n\
         impl Operation for {name} {{\n\
         const ID: &'static str = {id:?};\n\
         const METHOD: ::reqwest::Method = ::reqwest::Method::{http_method};\n\
         }}\n",
        segments = segments.join(", "),
    );

    let query: BTreeSet<&str> = parameters
        .iter()
        .filter(|parameter| parameter["in"] == "query")
        .filter_map(|parameter| parameter["name"].as_str())
        .collect();
    let mut capability = |accepted: bool, trait_name: &str| {
        if accepted {
            source.push_str(&format!("impl {trait_name} for {name} {{}}\n"));
        }
    };
    capability(query.iter().any(|q| q.starts_with("fields[")), "Fields");
    capability(
        query.iter().any(|q| q.starts_with("additional-fields[")),
        "AdditionalFields",
    );
    capability(query.contains("include"), "Include");
    capability(query.contains("filter"), "Filter");
    capability(query.contains("sort"), "Sort");
    capability(query.contains("page[size]"), "PageSize");
    if let Some(cursor) = ["page[cursor]", "page_cursor"]
        .into_iter()
        .find(|cursor| query.contains(cursor))
    {
        source.push_str(&format!(
            "impl Paginated for {name} {{ const CURSOR_PARAM: &'static str = {cursor:?}; }}\n"
        ));
    }
    source
}

fn rust_identifier(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "as", "async", "await", "box", "break", "const", "continue", "crate", "dyn", "else",
        "enum", "extern", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "static", "struct", "trait", "type", "unsafe", "use",
        "where", "while", "yield",
    ];
    if KEYWORDS.contains(&name) {
        format!("r#{name}")
    } else {
        name.to_owned()
    }
}

/// progenitor doesn't treat `application/vnd.api+json` as JSON.
fn rewrite_media_types(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::Object(content)) = map.get_mut("content") {
                if let Some(media) = content.remove("application/vnd.api+json") {
                    content.insert("application/json".to_owned(), media);
                }
            }
            map.values_mut().for_each(rewrite_media_types);
        }
        Value::Array(items) => items.iter_mut().for_each(rewrite_media_types),
        _ => {}
    }
}

/// progenitor names an inline error schema `<OperationId>Response`, which
/// collides with success schemas such as `GetAccountResponse`.
fn name_error_schema(spec: &mut Value) -> BuildResult<()> {
    if spec["components"]["schemas"].get(ERROR_SCHEMA).is_some() {
        return Err(format!("openapi.json already defines `{ERROR_SCHEMA}`").into());
    }

    let mut error_schema = None;
    if let Some(responses) = spec["components"]["responses"].as_object_mut() {
        for response in responses.values_mut() {
            let Some(content) = response.get_mut("content").and_then(Value::as_object_mut) else {
                continue;
            };
            for media in content.values_mut() {
                if let Some(schema) = media.get_mut("schema") {
                    if schema.get("$ref").is_none() {
                        let inline = std::mem::replace(
                            schema,
                            json!({ "$ref": format!("{SCHEMA_REF_PREFIX}{ERROR_SCHEMA}") }),
                        );
                        error_schema.get_or_insert(inline);
                    }
                }
            }
        }
    }

    let error_schema = error_schema.ok_or("no inline error response schema found")?;
    spec["components"]["schemas"]
        .as_object_mut()
        .ok_or("spec has no `components/schemas` object")?
        .insert(ERROR_SCHEMA.to_owned(), error_schema);
    Ok(())
}

/// Otherwise progenitor generates all ~1300 Klaviyo schemas.
fn prune_schemas(spec: &mut Value) {
    let mut reachable = BTreeSet::new();
    let mut pending = Vec::new();
    collect_schema_refs(&spec["paths"], &mut pending);
    collect_schema_refs(&spec["components"]["responses"], &mut pending);

    while let Some(name) = pending.pop() {
        if !reachable.insert(name.clone()) {
            continue;
        }
        collect_schema_refs(&spec["components"]["schemas"][&name], &mut pending);
    }

    if let Some(schemas) = spec["components"]["schemas"].as_object_mut() {
        schemas.retain(|name, _| reachable.contains(name));
    }
}

fn collect_schema_refs(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(name) = schema_ref(value) {
                out.push(name.to_owned());
            }
            map.values()
                .for_each(|child| collect_schema_refs(child, out));
        }
        Value::Array(items) => items
            .iter()
            .for_each(|child| collect_schema_refs(child, out)),
        _ => {}
    }
}

fn schema_ref(value: &Value) -> Option<&str> {
    value
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|r| r.strip_prefix(SCHEMA_REF_PREFIX))
}

/// A `default` on single-value enums such as JSON:API `type` (`"campaign"`)
/// makes typify implement `Default` for them. Optional properties are
/// skipped: typify always serializes a property that has a `default`.
fn default_single_value_enums(spec: &mut Value) {
    if let Some(schemas) = spec["components"]["schemas"].as_object_mut() {
        for schema in schemas.values_mut() {
            add_enum_default(schema);
            default_required_inline_enums(schema);
        }
    }
    default_required_inline_enums(&mut spec["paths"]);
}

fn add_enum_default(schema: &mut Value) {
    let Some(map) = schema.as_object_mut() else {
        return;
    };
    if map.contains_key("default") {
        return;
    }
    let single = match map.get("enum").and_then(Value::as_array) {
        Some(values) if values.len() == 1 => values[0].clone(),
        _ => return,
    };
    map.insert("default".to_owned(), single);
}

fn default_required_inline_enums(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let required = required_names(map);
            if let Some(Value::Object(properties)) = map.get_mut("properties") {
                for (name, property) in properties.iter_mut() {
                    if required.contains(name) {
                        add_enum_default(property);
                    }
                }
            }
            map.values_mut().for_each(default_required_inline_enums);
        }
        Value::Array(items) => items.iter_mut().for_each(default_required_inline_enums),
        _ => {}
    }
}

fn required_names(schema: &Map<String, Value>) -> BTreeSet<String> {
    match schema.get("required") {
        Some(Value::Array(names)) => names
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
        _ => BTreeSet::new(),
    }
}

/// Names the inline objects reachable from request bodies the way typify
/// would, and moves them into components so they can be patched. Returns
/// every schema reachable from a request body. Nullable objects stay inline,
/// since a `$ref` loses `nullable`.
fn hoist_request_objects(spec: &mut Value) -> BTreeSet<String> {
    let mut pending = Vec::new();
    if let Some(paths) = spec["paths"].as_object() {
        for operation in paths
            .values()
            .filter_map(Value::as_object)
            .flat_map(|item| {
                item.iter()
                    .filter(|(method, _)| HTTP_METHODS.contains(&method.as_str()))
                    .map(|(_, operation)| operation)
            })
        {
            if let Some(content) = operation["requestBody"]["content"].as_object() {
                for media in content.values() {
                    if let Some(name) = schema_ref(&media["schema"]) {
                        pending.push(name.to_owned());
                    }
                }
            }
        }
    }

    let mut reachable = BTreeSet::new();
    let Some(schemas) = spec["components"]["schemas"].as_object_mut() else {
        return reachable;
    };
    while let Some(name) = pending.pop() {
        if reachable.contains(&name) {
            continue;
        }
        let Some(mut schema) = schemas.remove(&name) else {
            continue;
        };
        reachable.insert(name.clone());

        let mut hoisted = Vec::new();
        hoist_properties(&mut schema, &name, schemas, &mut hoisted);
        collect_schema_refs(&schema, &mut pending);
        schemas.insert(name, schema);
        for (hoisted_name, hoisted_schema) in hoisted {
            schemas.insert(hoisted_name.clone(), hoisted_schema);
            pending.push(hoisted_name);
        }
    }
    reachable
}

fn hoist_properties(
    schema: &mut Value,
    parent: &str,
    taken: &Map<String, Value>,
    hoisted: &mut Vec<(String, Value)>,
) {
    let Some(Value::Object(properties)) = schema.get_mut("properties") else {
        return;
    };
    for (property, child) in properties.iter_mut() {
        hoist_value(
            child,
            format!("{parent}{}", property.to_pascal_case()),
            taken,
            hoisted,
        );
    }
}

fn hoist_value(
    slot: &mut Value,
    name: String,
    taken: &Map<String, Value>,
    hoisted: &mut Vec<(String, Value)>,
) {
    if is_inline_object(slot) && slot.get("nullable") != Some(&Value::Bool(true)) {
        let name = unique_name(name, taken, hoisted);
        let schema = std::mem::replace(
            slot,
            json!({ "$ref": format!("{SCHEMA_REF_PREFIX}{name}") }),
        );
        hoisted.push((name, schema));
    } else if slot.get("type").and_then(Value::as_str) == Some("array") {
        if let Some(items) = slot.get_mut("items") {
            hoist_value(items, format!("{name}Item"), taken, hoisted);
        }
    }
}

fn is_inline_object(schema: &Value) -> bool {
    schema.get("$ref").is_none()
        && schema.get("type").and_then(Value::as_str) == Some("object")
        && schema.get("properties").is_some()
}

fn unique_name(name: String, taken: &Map<String, Value>, hoisted: &[(String, Value)]) -> String {
    let is_free = |candidate: &str| {
        !taken.contains_key(candidate) && hoisted.iter().all(|(n, _)| n != candidate)
    };
    if is_free(&name) {
        return name;
    }
    let mut n = 2;
    loop {
        let candidate = format!("{name}{n}");
        if is_free(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// The untyped components are then unreachable, and pruned.
fn untype_refs(value: &mut Value) {
    if schema_ref(value).is_some_and(|name| UNTYPED.contains(&name)) {
        let mut untyped = Map::new();
        if let Some(description) = value.get("description") {
            untyped.insert("description".to_owned(), description.clone());
        }
        *value = Value::Object(untyped);
        return;
    }
    match value {
        Value::Object(map) => map.values_mut().for_each(untype_refs),
        Value::Array(items) => items.iter_mut().for_each(untype_refs),
        _ => {}
    }
}

/// typify always serializes an optional property that has a `default`, so a
/// PATCH built with `..Default::default()` would overwrite settings such as
/// `use_smart_sending`.
fn strip_optional_defaults(spec: &mut Value, request: &BTreeSet<String>) {
    let Some(schemas) = spec["components"]["schemas"].as_object_mut() else {
        return;
    };
    for (name, schema) in schemas.iter_mut() {
        if request.contains(name) {
            strip_defaults_of_optional_properties(schema);
        }
    }
}

fn strip_defaults_of_optional_properties(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let required = required_names(map);
            if let Some(Value::Object(properties)) = map.get_mut("properties") {
                for (name, property) in properties.iter_mut() {
                    if !required.contains(name) {
                        if let Some(property) = property.as_object_mut() {
                            property.remove("default");
                        }
                    }
                }
            }
            map.values_mut()
                .for_each(strip_defaults_of_optional_properties);
        }
        Value::Array(items) => items
            .iter_mut()
            .for_each(strip_defaults_of_optional_properties),
        _ => {}
    }
}

/// Request-side objects to patch with `Default`: those with a required field,
/// all of whose required fields implement `Default`. typify implements it
/// itself only for objects without required fields.
fn defaultable_with_required_fields(spec: &Value, request: &BTreeSet<String>) -> Vec<String> {
    let schemas = &spec["components"]["schemas"];
    let mut defaultable: BTreeSet<&str> = request
        .iter()
        .map(String::as_str)
        .filter(|name| is_object_schema(&schemas[*name]))
        .collect();

    // Start from every candidate and drop the ones that fail until nothing
    // changes, so schemas that refer to themselves are kept rather than
    // rejected.
    loop {
        let snapshot = defaultable.clone();
        defaultable.retain(|name| required_fields_default(&schemas[*name], schemas, &snapshot));
        if defaultable.len() == snapshot.len() {
            break;
        }
    }

    defaultable
        .into_iter()
        .filter(|name| has_required_field(&schemas[*name]))
        .map(str::to_owned)
        .collect()
}

fn is_object_schema(schema: &Value) -> bool {
    schema.get("properties").is_some()
        && schema.get("oneOf").is_none()
        && schema.get("anyOf").is_none()
        && schema.get("allOf").is_none()
}

fn required_fields_default(schema: &Value, schemas: &Value, defaultable: &BTreeSet<&str>) -> bool {
    let Some(map) = schema.as_object() else {
        return false;
    };
    required_names(map)
        .iter()
        .all(|name| field_has_default(&schema["properties"][name], schemas, defaultable))
}

fn has_required_field(schema: &Value) -> bool {
    schema
        .as_object()
        .is_some_and(|map| !required_names(map).is_empty())
}

/// Whether typify's type for `field` implements `Default`. Errs towards
/// `false`: a wrong `true` fails the build.
fn field_has_default(field: &Value, schemas: &Value, defaultable: &BTreeSet<&str>) -> bool {
    if field.is_null() {
        return false;
    }
    // openapiv3 drops the siblings of a `$ref`, `nullable` included, so the
    // field is the bare type, not an `Option`.
    if let Some(name) = schema_ref(field) {
        let target = &schemas[name];
        return defaultable.contains(name)
            || (!is_object_schema(target) && field_has_default(target, schemas, defaultable));
    }
    if field.get("nullable") == Some(&Value::Bool(true)) {
        return true;
    }
    if ["oneOf", "anyOf", "allOf"]
        .iter()
        .any(|k| field.get(*k).is_some())
    {
        return false;
    }
    let ty = field.get("type").and_then(Value::as_str);
    if field.get("enum").is_some() {
        // typify implements `Default` for a string enum with a `default`.
        return matches!(ty, Some("string") | None) && field.get("default").is_some();
    }
    match ty {
        // typify wraps constrained strings in newtypes without `Default`.
        Some("string") => ["pattern", "minLength", "maxLength"]
            .iter()
            .all(|k| field.get(*k).is_none()),
        // typify maps integers that can't be zero to `NonZero*`.
        Some("integer") => !cannot_be_zero(field),
        Some("number" | "boolean" | "array") => true,
        Some("object") => field.get("properties").is_none(),
        Some(_) => false,
        None => true,
    }
}

fn cannot_be_zero(field: &Value) -> bool {
    let exclusive = field.get("exclusiveMinimum") == Some(&Value::Bool(true));
    field
        .get("minimum")
        .and_then(Value::as_f64)
        .is_some_and(|minimum| minimum > 0.0 || (exclusive && minimum >= 0.0))
}
