use std::convert::Infallible;
use actix_web::{HttpResponse, error};
use actix_web::error::InternalError;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessage {
    pub message: String,
}

// 预定义了一些常用的错误信息。
pub fn internal_server_error(error: String) -> InternalError<String> {
    InternalError::from_response(error, HttpResponse::InternalServerError().json(ErrorMessage { message: error.clone() }))
}

pub fn not_found(error: String) -> HttpResponse {
    HttpResponse::NotFound().json(ErrorMessage { message: error })
}