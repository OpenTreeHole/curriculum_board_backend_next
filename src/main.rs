mod api;
mod entity;
mod constant;

use std::env;
use api::curriculum_board;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, middleware};
use dotenv::dotenv;
use sea_orm::{Database, DatabaseConnection};
use crate::entity::coursegroup as CourseGroup;
use sea_orm::EntityTrait;

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(curriculum_board::hello)
        .service(curriculum_board::get_course_groups)
        .service(curriculum_board::get_course_group)
        .service(curriculum_board::add_course);
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
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}