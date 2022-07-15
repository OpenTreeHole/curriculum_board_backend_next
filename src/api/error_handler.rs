use std::convert::Infallible;
use actix_web::HttpResponse;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessage {
    pub(crate) message: String,
}