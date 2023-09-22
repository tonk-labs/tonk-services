use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::common::Game;

pub async fn put_game(_id: web::Json<Game>, req: HttpRequest) -> HttpResponse {
    // req.path().split('/').last()
    HttpResponse::Ok().json(_id.message.as_str())
}

pub async fn post_game(_id: web::Json<Game>, req: HttpRequest) -> HttpResponse {
    // req.path().split('/').last()
    HttpResponse::Ok().json(_id.message.as_str())
}

pub async fn get_game(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}