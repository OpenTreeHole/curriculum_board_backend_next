mod api;
mod entity;
mod constant;

use std::env;
use api::curriculum_board;
use api::r#static;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, middleware};
use dotenv::dotenv;
use sea_orm::{Database, DatabaseConnection};
use crate::entity::coursegroup as CourseGroup;
use sea_orm::EntityTrait;
use serde_json::json;

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(curriculum_board::hello)
        .route("/courses/hash", web::get().to(|| async { HttpResponse::Ok().json(json!({"hash":"unimplemented"})) }))
        .service(curriculum_board::get_course_groups)
        .service(curriculum_board::get_course_group)
        .service(curriculum_board::add_course)
        .service(curriculum_board::get_course)
        .service(curriculum_board::add_review)
        .service(curriculum_board::modify_review)
        .service(curriculum_board::vote_for_review)
        .service(curriculum_board::get_reviews)
        .service(r#static::cedict);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化 dotenv
    dotenv().ok();

    let db: DatabaseConnection = Database::connect(env::var(constant::ENV_DB_URL).unwrap()).await.unwrap();
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .configure(config)
            .app_data(web::Data::new(db.clone()))
    })
        .bind(("0.0.0.0", 11451))?
        .run()
        .await
}