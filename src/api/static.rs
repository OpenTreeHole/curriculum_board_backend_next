use actix_web::{HttpRequest, Result, get};
use actix_files::NamedFile;

#[get("/static/cedict_ts.u8")]
pub async fn cedict(_: HttpRequest) -> Result<NamedFile> {
    Ok(NamedFile::open("static/cedict_ts.u8")?.set_content_type(mime::TEXT_PLAIN_UTF_8).disable_content_disposition())
}