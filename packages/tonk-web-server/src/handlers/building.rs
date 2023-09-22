use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::common::Building;

pub async fn post_building(_id: web::Json<Building>, req: HttpRequest) -> HttpResponse {
    // req.path().split('/').last()
    HttpResponse::Ok().json(_id.message.as_str())
}