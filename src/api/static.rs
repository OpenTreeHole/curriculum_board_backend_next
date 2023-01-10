use actix_web::{HttpRequest, Result, get};
use actix_files::NamedFile;

#[utoipa::path(
responses(
(status = 200, description = "Get the static dict file for cedict.", content_type = "text/plain"),
)
)]
#[get("/static/cedict_ts.u8")]
pub async fn cedict(_unused: HttpRequest) -> Result<NamedFile> {
    Ok(NamedFile::open("static/cedict_ts.u8")?.set_content_type(mime::TEXT_PLAIN_UTF_8).disable_content_disposition())
}