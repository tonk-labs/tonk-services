use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::common::Action;

pub async fn post_action(_id: web::Json<Action>, req: HttpRequest) -> HttpResponse {
    // req.path().split('/').last()
    HttpResponse::Ok().json(_id.message.as_str())
}