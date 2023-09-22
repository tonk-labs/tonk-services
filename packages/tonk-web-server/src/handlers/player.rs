use actix_web::{web, Error, HttpResponse};
use crate::common::PlayerQuery;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    id: String,
    query: String 
}

pub async fn get_player(_id: web::Path<String>, _query: web::Query<PlayerQuery>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(Player {
        id: _id.to_string(),
        query: _query.privateKey.clone()
    }))
}