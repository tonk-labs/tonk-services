use actix_web::{web, Error, HttpResponse, HttpRequest};
use tonk_shared_lib::Action;

pub async fn post_action(_id: web::Json<Action>, req: HttpRequest) -> Result<HttpResponse, Error> {
    // req.path().split('/').last()
    Ok(HttpResponse::Ok().finish())
}