use actix_web::{web, Error, HttpResponse, HttpRequest};
use tonk_shared_lib::{Game, Player, deserialize_struct, GameStatus, serialize_struct, Building};
use crate::redis_helper::*;

// START GAME
// CALL POST WITHOUT ANY DATA BASICALLY
pub async fn post_game() -> Result<HttpResponse, Error> {
    let mut redis = RedisHelper::init().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    let game_result: Result<Game, Box<dyn std::error::Error>> = redis.get_key("game").await;
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

            let player_count = current_game.players.as_ref().map_or(0, |players| players.len());

            if player_count <= 3 {
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
            Err(actix_web::error::ErrorInternalServerError("If you are seeing this error, the game is likely in a corrupted state"))
        }
    }
}

pub async fn get_time() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

// GET STATUS OF GAME
pub async fn get_game() -> Result<HttpResponse, Error> {
    let mut redis = RedisHelper::init().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    let current_game: Result<Game, Box<dyn std::error::Error>> = redis.get_key("game").await;
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
                players: None
            };
            Ok(HttpResponse::Ok().json(empty_game))
        }
    }
}

pub async fn get_game_players(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

pub async fn post_player(_id: web::Json<Player>, req: HttpRequest) -> Result<HttpResponse, Error> {
    // req.path().split('/').last()
    Ok(HttpResponse::Ok().finish())
}