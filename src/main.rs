mod api;
mod entity;

use api::curriculum_board;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};


fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(curriculum_board::hello)
        .service(curriculum_board::get_course_groups);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .configure(config)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}