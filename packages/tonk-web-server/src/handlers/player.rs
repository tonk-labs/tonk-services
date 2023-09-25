use actix_web::{web, Error, HttpResponse};
use tonk_shared_lib::Player;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerQuery {
    secret_key: String
}

pub async fn get_player(_id: web::Path<String>, _query: web::Query<PlayerQuery>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(Player {
        id: _id.to_string(),
        nearby_buildings: None,
        nearby_players: None,
        display_name: None,
        secret_key: None,
        location: None
    }))
}