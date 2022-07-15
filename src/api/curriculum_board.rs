use actix_web::{get, post, HttpResponse, Responder, web, HttpRequest};
use sea_orm::{DatabaseConnection, EntityTrait};
use crate::api::auth::require_authentication;
use crate::entity::coursegroup as CourseGroup;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::InternalServerError().body("Hello world!")
}

#[get("/courses")]
pub async fn get_course_groups(req: HttpRequest, db: web::Data<DatabaseConnection>) -> impl Responder {
    let user_info = require_authentication(&req).await;
    if let Err(e) = user_info {
        return e;
    }

    let result: Result<Vec<serde_json::Value>, sea_orm::DbErr> =
        CourseGroup::Entity::find().into_json().all(db.get_ref()).await;
    match result {
        Ok(groups) => HttpResponse::Ok().json(groups),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    }
}

