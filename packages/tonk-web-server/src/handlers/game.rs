use actix_web::{web, Error, HttpResponse, HttpRequest};
use tonk_shared_lib::{Game, Player, deserialize_struct, GameStatus, serialize_struct, Building};
use tonk_shared_lib::redis_helper::*;

// START GAME
// CALL PUT WITHOUT ANY DATA 
pub async fn post_game() -> Result<HttpResponse, Error> {
    let redis = RedisHelper::init().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    let game_result: Result<Game, RedisHelperError> = redis.get_key("game").await;
    match game_result {
        Ok(game) => {
            let mut current_game = game; 
            if current_game.status != GameStatus::Lobby {
                return Err(actix_web::error::ErrorForbidden("Game is already started"))
            }

            // check buildings exists
            let buildings: Vec<Building> = redis.get_index("building:index").await.map_err(|e| {
                actix_web::error::ErrorInternalServerError(e)
            })?;

            let mut found_tower = false;
            buildings.iter().for_each(|e| {
                if e.is_tower {
                    found_tower = true;
                }
            });

            if !found_tower || buildings.len() <= 3 {
                return Err(actix_web::error::ErrorForbidden("Need to register all the proper buildings before the game can begin"));
            }

            let index_key = format!("game:{}:player_index", current_game.id);
            let players: Vec<Player> = redis.get_index(&index_key).await.map_err(|_| { 
                actix_web::error::ErrorInternalServerError("unknown error")
            })?;

            if players.len() < 3 {
                return Err(actix_web::error::ErrorForbidden("More players need to join the game before we can start"));
            }

            // give tasks to all the players
            // update status
            current_game.status = GameStatus::Tasks;
            redis.set_key("game", &current_game).await.map_err(|e| {
                actix_web::error::ErrorInternalServerError(e)
            })?;

            Ok(HttpResponse::Ok().finish())
        }
        Err(e) => {
            println!("{}", e);
            Err(actix_web::error::ErrorInternalServerError("If you are seeing this error, the game is likely in a corrupted state"))
        }
    }
}

pub async fn get_time() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

// GET STATUS OF GAME
pub async fn get_game() -> Result<HttpResponse, Error> {
    let redis = RedisHelper::init().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    let current_game: Result<Game, RedisHelperError> = redis.get_key("game").await;
    match current_game {
        Ok(game) => {
            Ok(HttpResponse::Ok().json(game))
        }
        Err(e) => {
            // the game doesn't exist
            let empty_game = Game {
                id: "".to_string(),
                status: GameStatus::Null,
                time: None,
            };
            Ok(HttpResponse::Ok().json(empty_game))
        }
    }
}

pub async fn get_game_players() -> Result<HttpResponse, Error> {
    let redis = RedisHelper::init().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    let game: Game = redis.get_key("game").await.map_err(|_| { 
        actix_web::error::ErrorInternalServerError("unknown error")
    })?;
    let index_key = format!("game:{}:player_index", game.id);
    let players: Vec<Player> = redis.get_index(&index_key).await.map_err(|_| { 
        actix_web::error::ErrorInternalServerError("unknown error")
    })?;
    Ok(HttpResponse::Ok().json(players))
}

pub async fn post_player(_id: web::Json<Player>) -> Result<HttpResponse, Error> {
    let player = _id.0;
    let redis = RedisHelper::init().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    let game: Game = redis.get_key("game").await.map_err(|_| { 
        actix_web::error::ErrorInternalServerError("unknown error")
    })?;
    let player_key = format!("game:{}:player:{}", game.id, player.id);
    let redis_player: Result<Player, _> = redis.get_key(&player_key).await;
    match redis_player {
        Ok(_) => {
            Err(actix_web::error::ErrorForbidden("Player already in the game"))
        }
        Err(e) => {
            if let Ok(_) = redis.set_key(&player_key, &player).await {
                let index_key = format!("game:{}:player_index", game.id);
                let _ = redis.set_index(&index_key, &player_key).await.map_err(|_| { 
                    actix_web::error::ErrorInternalServerError("unknown error")
                })?;
                Ok(HttpResponse::Ok().json(player))
            } else {
                Err(actix_web::error::ErrorInternalServerError("Unknown error"))
            }
        }
    }
}