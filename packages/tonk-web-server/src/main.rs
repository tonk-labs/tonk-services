use actix_web::{get, post, web, App, HttpResponse, HttpRequest, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct TestObject {
    message: String
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/test")]
async fn test(item: web::Json<TestObject>, req: HttpRequest) -> HttpResponse {
    println!("request: {req:?}");
    println!("model: {item:?}");
    
    HttpResponse::Ok().json("Nothing")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::JsonConfig::default().limit(4096))
            .service(hello)
            .service(test)
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}