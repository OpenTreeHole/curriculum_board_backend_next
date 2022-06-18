mod api;
mod entity;

use api::curriculum_board;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use sea_orm::{Database, DatabaseConnection};
use crate::entity::coursegroup as CourseGroup;
use sea_orm::EntityTrait;

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(curriculum_board::hello)
        .service(curriculum_board::get_course_groups);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db: DatabaseConnection = Database::connect("mysql://root:root@127.0.0.1:3306/board").await.unwrap();
    HttpServer::new(move || {
        App::new()
            .configure(config)
            .app_data(web::Data::new(db.clone()))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}