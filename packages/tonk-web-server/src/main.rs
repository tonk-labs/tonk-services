use actix_web::{web, get, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;

mod app_config;
mod handlers;
mod redis_helper;

#[get("/ping")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    HttpServer::new(|| {
        App::new()
            .app_data(web::JsonConfig::default().limit(4096))
            .configure(app_config::config)
    })
    .bind(("0.0.0.0", 8082))?
    .run()
    .await
}
