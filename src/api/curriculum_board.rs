use actix_web::{get, post, HttpResponse, Responder, web};
use sea_orm::{DatabaseConnection, EntityTrait};
use crate::entity::coursegroup as CourseGroup;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/courses")]
pub async fn get_course_groups(db: web::Data<DatabaseConnection>) -> impl Responder {
    let result: Result<Vec<serde_json::Value>, sea_orm::DbErr> =
        CourseGroup::Entity::find().into_json().all(db.get_ref()).await;
    match result {
        Ok(groups) => HttpResponse::Ok().json(groups),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    }
}
