use actix_web::{http::header, get, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use dotenv::dotenv;

mod app_config;
mod handlers;

#[get("/ping")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST", "PUT"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .configure(app_config::config)
    })
    .bind(("0.0.0.0", 8082))?
    .run()
    .await
}
