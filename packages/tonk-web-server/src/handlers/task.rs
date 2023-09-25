use actix_web::{web, Error, HttpResponse, HttpRequest};
use tonk_shared_lib::Task;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskQuery {
    player_id: String,
    secret_key: String 
}

// RETURNS TASK AND IF IT DOESNT EXIST THEN RANDOMLY ASSIGNS NEW TASK
pub async fn get_task(_query: web::Query<Option<TaskQuery>>, req: HttpRequest) -> Result<HttpResponse, Error> {
    // req.path().split('/').last()
    Ok(HttpResponse::Ok().finish())
}

// USED TO CONFIRM SUCCESSFUL COMPLETION OF TASK
pub async fn post_task(_id: web::Json<Task>, _query: web::Query<Option<TaskQuery>>, req: HttpRequest) -> Result<HttpResponse, Error> {
    // req.path().split('/').last()
    Ok(HttpResponse::Ok().finish())
}