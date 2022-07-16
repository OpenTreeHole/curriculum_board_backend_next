use std::convert::Infallible;
use actix_web::HttpResponse;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessage {
    pub(crate) message: String,
}

pub fn internal_server_error(error: String) -> HttpResponse {
    HttpResponse::InternalServerError().json(ErrorMessage { message: error })
}

pub fn not_found(error: String) -> HttpResponse {
    HttpResponse::NotFound().json(ErrorMessage { message: error })
}