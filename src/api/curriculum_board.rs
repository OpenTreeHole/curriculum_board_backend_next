use actix_web::{get, post,HttpResponse, Responder};

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/courses")]
pub async fn get_course_groups() -> impl Responder {
    HttpResponse::ServiceUnavailable().body("Poo!")
}