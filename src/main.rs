mod api;
mod entity;
mod constant;

use std::env;
use api::curriculum_board;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use sea_orm::{Database, DatabaseConnection};
use crate::entity::coursegroup as CourseGroup;
use sea_orm::EntityTrait;

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(curriculum_board::hello)
        .service(curriculum_board::get_course_groups)
        .service(curriculum_board::get_course_group);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化 dotenv
    dotenv().ok();

    let db: DatabaseConnection = Database::connect(env::var(constant::ENV_DB_URL).unwrap()).await.unwrap();
    HttpServer::new(move || {
        App::new()
            .configure(config)
            .app_data(web::Data::new(db.clone()))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}