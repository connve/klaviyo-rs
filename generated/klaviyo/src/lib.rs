//! Generated Klaviyo REST API client.
//!
//! Built by `progenitor` from the operations in `build.rs`, out of Klaviyo's
//! `openapi.json`. Don't edit the output; change `build.rs`.

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

/// A marker type per operation, named after its `operationId`
/// (`get_campaigns` → `GetCampaigns`), and the traits `build.rs` implements
/// for it from the spec.
pub mod operations {
    /// An API operation.
    pub trait Operation {
        /// Klaviyo `operationId`.
        const ID: &'static str;
        /// HTTP method.
        const METHOD: reqwest::Method;
    }

    /// Accepts `fields[<type>]`.
    pub trait Fields: Operation {}

    /// Accepts `additional-fields[<type>]`.
    pub trait AdditionalFields: Operation {}

    /// Accepts `include`.
    pub trait Include: Operation {}

    /// Accepts `filter`.
    pub trait Filter: Operation {}

    /// Accepts `sort`.
    pub trait Sort: Operation {}

    /// Accepts `page[size]`.
    pub trait PageSize: Operation {}

    /// Pages its results with a cursor.
    pub trait Paginated: Operation {
        /// Query parameter that carries the cursor.
        const CURSOR_PARAM: &'static str;
    }

    /// The path of operation `O`. Only the marker's `path` function builds
    /// one, so a path can't be paired with the wrong operation.
    #[derive(Debug)]
    pub struct Path<'p, O> {
        segments: Vec<&'p str>,
        operation: std::marker::PhantomData<fn() -> O>,
    }

    impl<'p, O> Path<'p, O> {
        fn new(segments: Vec<&'p str>) -> Self {
            Self {
                segments,
                operation: std::marker::PhantomData,
            }
        }

        /// The path segments, e.g. `["api", "campaigns", "01J..."]`.
        pub fn segments(&self) -> &[&'p str] {
            &self.segments
        }
    }

    include!(concat!(env!("OUT_DIR"), "/operations.rs"));
}
