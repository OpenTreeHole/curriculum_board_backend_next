use std::env;
use actix_web::http::header::HeaderValue;
use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::http::StatusCode;
use if_chain::if_chain;
use lazy_static::lazy_static;
use moka::future::Cache;
use reqwest::{Error, Response};
use crate::api::error_handler::ErrorMessage;
use crate::constant;
use serde::Deserialize;


#[derive(Debug, Copy, Clone, Deserialize)]
pub struct UserInfo {
    id: i64,
    is_admin: bool,
}

lazy_static::lazy_static! {
    static ref GLOBAL_USER_CACHE: Cache<String,UserInfo> = {
        Cache::new(10000)
    };
}

async fn request_user_info(header: &str) -> Result<UserInfo, HttpResponse> {
    let header = header.to_string();
    let cached_value = GLOBAL_USER_CACHE.get(&header);
    if let Some(info) = cached_value {
        return Ok(info);
    }
    let client = reqwest::Client::new();
    let result =
        client.get(env::var(constant::ENV_USER_VERIFICATION_ADDRESS).unwrap()).header("Authorization", &header).send().await;
    match result {
        Ok(response) => {
            if let Ok(user) = response.json::<UserInfo>().await {
                GLOBAL_USER_CACHE.insert(header, user).await;
                Ok(user)
            } else {
                Err(HttpResponse::InternalServerError().json(ErrorMessage { message: "Internal Error: Cannot validate authorization information.".to_string() }))
            }
        }
        Err(e) => {
            if let Some(status_code) = e.status() {
                if status_code == StatusCode::UNAUTHORIZED {
                    return Err(HttpResponse::Unauthorized().json(ErrorMessage { message: "Authorization Failed.".to_string() }));
                }
            }
            Err(HttpResponse::InternalServerError().json(ErrorMessage { message: "Internal Error: Cannot validate authorization information.".to_string() }))
        }
    }
}

pub async fn require_authentication(req: &HttpRequest) -> Result<UserInfo, HttpResponse> {
    let authorization = req.headers().get("Authorization");
    if_chain! {
        if let Some(header) = authorization;
        if let Ok(header_value) = header.to_str();
        then {
            return request_user_info(header_value).await
        } else {
            return Err(HttpResponse::Unauthorized().json(ErrorMessage { message: "Authorization Information Needed.".to_string() }))
        }
    }
}