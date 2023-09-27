use actix_web::{web, Error, HttpResponse};
use tonk_shared_lib::Player;
use serde::{Deserialize, Serialize};
use tonk_shared_lib::redis_helper::*;
use ethers_rs::{H256, keccak256};


#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerQuery {
    onchain_hash: String,
    secret_key: String,
}

// Used to establish a new player and is registered by the tonk item
pub async fn post_player(_path: web::Path<String>) -> Result<HttpResponse, Error> {
    // check if the player already exists
    let redis = RedisHelper::init().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    //TODO: IMPLEMENT LATER
    // let onchain_hash = _query.onchain_hash.as_str().clone();
    // let secret = _query.secret_key.as_str().clone();
    // we need to double check here the onchain_hash is actually the same

    let player_key = format!("player:{}", _path.to_string());
    let player: Result<Player, _> = redis.get_key(&player_key).await;
    if let Err(RedisHelperError::MissingKey) = player {
        // let secret_bytes = secret.as_bytes();
        // let hash: H256 = ethers_rs::BytesM(keccak256(secret_bytes));
        // let hash_str = hash.to_string();
        
        //TODO: implement protected call here to issue some kind of session-key, etc thingy
        // players id is the hash of the secret_key
        // query is the secret_key to the hash
        // if hash_str == onchain_hash {
            // here we need to make interface somehow with the blockchain to verify
            let registered_player = Player {
                id: _path.to_string(),
                nearby_buildings: None,
                nearby_players: None,
                display_name: None,
                // secret_key: Some(secret.clone().to_string()),
                secret_key: None,
                location: None,
            };
            let _ = redis.set_key(&player_key, &registered_player).await.map_err(|e| {
                actix_web::error::ErrorInternalServerError(e)
            })?;

            let _ = redis.set_index("player:index", &player_key).await.map_err(|e| {
                actix_web::error::ErrorInternalServerError(e)
            })?;
            return Ok(HttpResponse::Ok().finish());
        // }
    }

    Err(actix_web::error::ErrorInternalServerError("unknown error"))
}

pub async fn get_player(_id: web::Path<String>) -> Result<HttpResponse, Error> {
    let redis = RedisHelper::init().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    let player_key = format!("player:{}", _id.to_string());
    let player: Result<Player, _> = redis.get_key(&player_key).await;
    if let Err(RedisHelperError::MissingKey) = player {
        Ok(HttpResponse::Ok().json(Player {
            id: "".to_string(),
            nearby_buildings: None,
            nearby_players: None,
            display_name: None,
            secret_key: None,
            location: None
        }))
    } else if let Ok(registered_player) = player {
        Ok(HttpResponse::Ok().json(registered_player))
    } else {
        Err(actix_web::error::ErrorInternalServerError("unknown error"))
    }

}