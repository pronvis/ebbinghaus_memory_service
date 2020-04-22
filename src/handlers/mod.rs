use actix_web::{Error, HttpRequest, HttpResponse};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::prelude::*;

#[derive(Fail, Debug)]
pub enum ServiceError {
    #[fail(display = "External Service Error")]
    ExternalServiceError,

    #[fail(display = "informative error: {}", message)]
    InformativeError { message: String },
}

impl actix_web::error::ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::ExternalServiceError => HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("external service error"),

            ServiceError::InformativeError { message } => HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(message),
        }
    }
}

pub fn json_error_handler(err: actix_web::error::JsonPayloadError, _: &HttpRequest) -> Error {
    actix_web::error::InternalError::from_response(
        "",
        HttpResponse::BadRequest()
            .content_type("application/json")
            .body(format!(r#"{{"error": "{}"}}"#, err)),
    )
    .into()
}
