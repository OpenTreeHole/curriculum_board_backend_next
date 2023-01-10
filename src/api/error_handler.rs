use actix_web::{HttpResponse, Error};
use actix_web::error::InternalError;
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ErrorMessage {
    pub message: String,
}

// 预定义了一些常用的错误信息。
pub fn internal_server_error(error: String) -> Error {
    InternalError::from_response(error.clone(),
                                 HttpResponse::InternalServerError().json(ErrorMessage { message: error })).into()
}

pub fn not_found(error: String) -> Error {
    InternalError::from_response(error.clone(),
                                 HttpResponse::NotFound().json(ErrorMessage { message: error })).into()
}

pub fn unauthorized(error: String) -> Error {
    InternalError::from_response(error.clone(),
                                 HttpResponse::Unauthorized().json(ErrorMessage { message: error })).into()
}


pub fn bad_request(error: String) -> Error {
    InternalError::from_response(error.clone(),
                                 HttpResponse::BadRequest().json(ErrorMessage { message: error })).into()
}

pub fn conflict(error: String) -> Error {
    InternalError::from_response(error.clone(),
                                 HttpResponse::Conflict().json(ErrorMessage { message: error })).into()
}

pub fn forbidden(error: String) -> Error {
    InternalError::from_response(error.clone(),
                                 HttpResponse::Forbidden().json(ErrorMessage { message: error })).into()
}