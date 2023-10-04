use actix_web::{web, Error, HttpResponse};
use tonk_shared_lib::{Player, Game, Action, Task, Vote, GameStatus, Role};
use serde::{Deserialize, Serialize};
use tonk_shared_lib::redis_helper::*;
use log::*;
// use ethers_rs::{H256, keccak256};


#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerQuery {
    onchain_hash: String,
    secret_key: String,
}

// Used to establish a new player and is registered by the tonk item
pub async fn post_player(_id: web::Json<Player>, _path: web::Path<String>) -> Result<HttpResponse, Error> {
    // check if the player already exists
    let redis = RedisHelper::init().await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;
    //TODO: IMPLEMENT LATER
    // let onchain_hash = _query.onchain_hash.as_str().clone();
    // let secret = _query.secret_key.as_str().clone();
    // we need to double check here the onchain_hash is actually the same

    let player_obj = _id.0.clone();

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
                mobile_unit_id: player_obj.mobile_unit_id,
                display_name: player_obj.display_name,
                role: None,
                used_action: Some(false),
                // secret_key: Some(secret.clone().to_string()),
                secret_key: None,
                location: None,
            };
            let _ = redis.set_key(&player_key, &registered_player).await.map_err(|e| {
                error!("{:?}", e);
                actix_web::error::ErrorInternalServerError(e)
            })?;

            let _ = redis.add_to_index("player:index", &player_key).await.map_err(|e| {
                error!("{:?}", e);
                actix_web::error::ErrorInternalServerError(e)
            })?;
            return Ok(HttpResponse::Ok().finish());
        // }
    } else if player.is_ok() && player_obj.display_name != player.as_ref().unwrap().display_name {
        let cp = player.as_ref().unwrap().clone();
        let registered_player = Player {
            id: cp.id,
            nearby_buildings: cp.nearby_buildings,
            nearby_players: cp.nearby_players,
            mobile_unit_id: cp.mobile_unit_id,
            display_name: player_obj.display_name,
            role: cp.role,
            used_action: cp.used_action,
            // secret_key: Some(secret.clone().to_string()),
            secret_key: cp.secret_key,
            location: cp.location,
        };
        let _ = redis.set_key(&player_key, &registered_player).await.map_err(|e| {
            error!("{:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
        return Ok(HttpResponse::Ok().finish());
    }

    Err(actix_web::error::ErrorInternalServerError("unknown error"))
}

pub async fn get_player(_id: web::Path<String>) -> Result<HttpResponse, Error> {
    let redis = RedisHelper::init().await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;
    let player_key = format!("player:{}", _id.to_string());
    let mut player: Result<Player, _> = redis.get_key(&player_key).await;

    let game: Game = redis.get_key("game").await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;


    if let Err(RedisHelperError::MissingKey) = player {
        Ok(HttpResponse::Ok().json(Player {
            id: "".to_string(),
            nearby_buildings: None,
            nearby_players: None,
            role: None,
            used_action: None,
            display_name: None,
            mobile_unit_id: None,
            secret_key: None,
            location: None
        }))
    } else if let Ok(registered_player) = player {
        let mut wrapper_player = registered_player.clone();

        let index_key = format!("game:{}:player_index", game.id);
        let game_players: Vec<Player> = redis.get_index(&index_key).await.map_err(|e| {
            error!("{:?}", e);
            actix_web::error::ErrorInternalServerError("There was an unknown error")
        })?;
        let is_in_game = game_players.iter().find(|p| p.id == registered_player.id).is_some();

        if game.status == GameStatus::Tasks && is_in_game {
            let role = registered_player.role.as_ref().unwrap();

            if *role == Role::Bugged {
                let action_key = format!("action:{}:{}:{}", game.id, game.time.as_ref().unwrap().round, registered_player.id);
                let took_action: Result<Action, RedisHelperError> = redis.get_key(&action_key).await;
                if took_action.is_ok() {
                    wrapper_player.used_action = Some(true);
                }
            } else {
                let task_key = format!("task:{}:{}:{}", game.id, game.time.as_ref().unwrap().round, registered_player.id);
                let task: Result<Task, RedisHelperError> = redis.get_key(&task_key).await;
                if task.is_ok() {
                    wrapper_player.used_action = Some(task.as_ref().unwrap().complete);
                }
            }
        }

        if game.status == GameStatus::Vote && is_in_game {
            let vote_key = format!("vote:{}:{}:{}", game.id, game.time.as_ref().unwrap().round, registered_player.id);
            let cast_vote: Result<Vote, RedisHelperError> = redis.get_key(&vote_key).await;
            if cast_vote.is_ok() {
                wrapper_player.used_action = Some(true);
            }
        }
        Ok(HttpResponse::Ok().json(wrapper_player))
    } else {
        error!("Couldn't get the player key from redis");
        Err(actix_web::error::ErrorInternalServerError("unknown error"))
    }

}