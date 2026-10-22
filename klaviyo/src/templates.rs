//! Templates, rendering, cloning, and universal content blocks.
//!
//! An account can hold at most 1,000 templates created through the API, so
//! update templates in place rather than creating one per campaign.
//!
//! ```no_run
//! use klaviyo::Client;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::builder()
//!     .api_key(std::env::var("KLAVIYO_API_KEY")?)
//!     .build()?;
//!
//! let mut pages = client
//!     .templates()
//!     .list()
//!     .filter("contains(name,'[master]')")
//!     .pages();
//! while let Some(page) = pages.next_page().await {
//!     for template in page?.data {
//!         println!("master template {}", template.id);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use crate::client::Client;
use crate::http::json_body;
use crate::operations;
use crate::request::{NoContent, RequestBuilder};
use crate::types;

/// Accessor returned by [`Client::templates`](crate::Client::templates).
#[derive(Debug)]
pub struct Templates<'a> {
    client: &'a Client,
}

impl<'a> Templates<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `GET /api/templates` — list templates.
    ///
    /// Filter on `id`, `name`, `created`, `updated`. Up to 10 per page. Ask
    /// for `additional_fields("template", &["definition"])` to get the full
    /// drag-and-drop definition. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_templates
    pub fn list(&self) -> ListTemplates<'a> {
        RequestBuilder::new(self.client, operations::GetTemplates::path())
    }

    /// `GET /api/templates/{id}` — get one template, including its HTML.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_template
    pub fn get(&self, template_id: &str) -> GetTemplate<'a> {
        RequestBuilder::new(self.client, operations::GetTemplate::path(template_id))
    }

    /// `POST /api/templates` — create an HTML or drag-and-drop template.
    ///
    /// Fails once the account has 1,000 templates created through the API.
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/create_template
    pub fn create(&self, request: &types::TemplateCreateHtmlOrDndQuery) -> CreateTemplate<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::CreateTemplate::path(),
            json_body(request, None),
        )
    }

    /// `PATCH /api/templates/{id}` — update a template's name or content.
    ///
    /// `data.id` in the body is set to `template_id`. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/update_template
    pub fn update(
        &self,
        template_id: &str,
        request: &types::TemplateUpdateHtmlOrDndQuery,
    ) -> UpdateTemplate<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::UpdateTemplate::path(template_id),
            json_body(request, Some(template_id)),
        )
    }

    /// `DELETE /api/templates/{id}` — delete a template.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/delete_template
    pub fn delete(&self, template_id: &str) -> DeleteTemplate<'a> {
        RequestBuilder::new(self.client, operations::DeleteTemplate::path(template_id))
    }

    /// `POST /api/template-render` — render a template with a context, as
    /// AMP, HTML, and plain text.
    ///
    /// Variables resolve against `attributes.context` as at send time.
    /// `data.id` in the body is set to `template_id`. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/render_template
    pub fn render(
        &self,
        template_id: &str,
        request: &types::TemplateRenderQuery,
    ) -> RenderTemplate<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::RenderTemplate::path(),
            json_body(request, Some(template_id)),
        )
    }

    /// `POST /api/template-clone` — copy a template within the same account.
    ///
    /// `data.id` in the body is set to `template_id`, the template to copy.
    /// Counts towards the 1,000-template API limit. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/clone_template
    pub fn clone_template(
        &self,
        template_id: &str,
        request: &types::TemplateCloneQuery,
    ) -> CloneTemplate<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::CloneTemplate::path(),
            json_body(request, Some(template_id)),
        )
    }

    /// `GET /api/template-universal-content` — list universal content blocks.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_all_universal_content
    pub fn list_universal_content(&self) -> ListUniversalContent<'a> {
        RequestBuilder::new(self.client, operations::GetAllUniversalContent::path())
    }

    /// `GET /api/template-universal-content/{id}` — get one universal
    /// content block.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_universal_content
    pub fn get_universal_content(&self, content_id: &str) -> GetUniversalContent<'a> {
        RequestBuilder::new(
            self.client,
            operations::GetUniversalContent::path(content_id),
        )
    }

    /// `POST /api/template-universal-content` — create a universal content
    /// block.
    ///
    /// Any block type, or a whole section. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/create_universal_content
    pub fn create_universal_content(
        &self,
        request: &types::UniversalContentCreateQueryV1,
    ) -> CreateUniversalContent<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::CreateUniversalContent::path(),
            json_body(request, None),
        )
    }

    /// `PATCH /api/template-universal-content/{id}` — update a universal
    /// content block.
    ///
    /// A `definition` replaces the whole block; leave it out to only rename.
    /// `data.id` in the body is set to `content_id`. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/update_universal_content
    pub fn update_universal_content(
        &self,
        content_id: &str,
        request: &types::UniversalContentPartialUpdateQueryV1,
    ) -> UpdateUniversalContent<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::UpdateUniversalContent::path(content_id),
            json_body(request, Some(content_id)),
        )
    }

    /// `DELETE /api/template-universal-content/{id}` — delete a universal
    /// content block.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/delete_universal_content
    pub fn delete_universal_content(&self, content_id: &str) -> DeleteUniversalContent<'a> {
        RequestBuilder::new(
            self.client,
            operations::DeleteUniversalContent::path(content_id),
        )
    }
}

/// Builder for [`Templates::list`].
pub type ListTemplates<'a> =
    RequestBuilder<'a, types::GetTemplateDndResponseCollection, operations::GetTemplates>;

/// Builder for [`Templates::get`].
pub type GetTemplate<'a> =
    RequestBuilder<'a, types::GetTemplateDndResponse, operations::GetTemplate>;

/// Builder for [`Templates::create`].
pub type CreateTemplate<'a> =
    RequestBuilder<'a, types::PostTemplateDndResponse, operations::CreateTemplate>;

/// Builder for [`Templates::update`].
pub type UpdateTemplate<'a> =
    RequestBuilder<'a, types::PatchTemplateDndResponse, operations::UpdateTemplate>;

/// Builder for [`Templates::delete`].
pub type DeleteTemplate<'a> = RequestBuilder<'a, NoContent, operations::DeleteTemplate>;

/// Builder for [`Templates::render`].
pub type RenderTemplate<'a> =
    RequestBuilder<'a, types::PostTemplateResponse, operations::RenderTemplate>;

/// Builder for [`Templates::clone_template`].
pub type CloneTemplate<'a> =
    RequestBuilder<'a, types::PostTemplateResponse, operations::CloneTemplate>;

/// Builder for [`Templates::list_universal_content`].
pub type ListUniversalContent<'a> = RequestBuilder<
    'a,
    types::GetUniversalContentResponseV1Collection,
    operations::GetAllUniversalContent,
>;

/// Builder for [`Templates::get_universal_content`].
pub type GetUniversalContent<'a> =
    RequestBuilder<'a, types::GetUniversalContentResponseV1, operations::GetUniversalContent>;

/// Builder for [`Templates::create_universal_content`].
pub type CreateUniversalContent<'a> =
    RequestBuilder<'a, types::PostUniversalContentResponseV1, operations::CreateUniversalContent>;

/// Builder for [`Templates::update_universal_content`].
pub type UpdateUniversalContent<'a> =
    RequestBuilder<'a, types::PatchUniversalContentResponseV1, operations::UpdateUniversalContent>;

/// Builder for [`Templates::delete_universal_content`].
pub type DeleteUniversalContent<'a> =
    RequestBuilder<'a, NoContent, operations::DeleteUniversalContent>;
