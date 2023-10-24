use actix_web::{http::header, get, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use dotenv::dotenv;
use std::env;

mod app_config;
mod handlers;

#[get("/ping")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    match env::var("TONK_SERVICES_STAGE") {
        Ok(stage) => {
            println!("Starting up tonk-web-server in stage: {}", stage);
            dotenv::from_filename(".env.production").ok();
        }
        Err(_) => {
            println!("Starting up tonk-web-server in stage: {}", "local");
            dotenv::from_filename(".env.local").ok();
        }
    }
    // let origin = env::var("ALLOWED_ORIGIN").unwrap();
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .send_wildcard()
            )
            .configure(app_config::config)
    })
    .bind(("0.0.0.0", 8082))?
    .run()
    .await
}
