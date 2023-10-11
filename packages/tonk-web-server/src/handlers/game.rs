use actix_web::{web, Error, HttpResponse, HttpRequest};
use tonk_shared_lib::{Game, Player, deserialize_struct, GameStatus, serialize_struct, Building, Role, RoundResult, Time};
use tonk_shared_lib::redis_helper::*;
use rand::{Rng, thread_rng, RngCore};
use rand::seq::SliceRandom;
use log::*;

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
                error!("{:?}", e);
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
            let players: Vec<Player> = redis.get_index(&index_key).await.map_err(|e| { 
                error!("{:?}", e);
                actix_web::error::ErrorInternalServerError("unknown error")
            })?;

            if players.len() < 2 {
                return Err(actix_web::error::ErrorForbidden("More players need to join the game before we can start"));
            }

            // we don't want the number of bugs to outnumber the players
            let max_bugs = (players.len() as f64 * 0.4).round() as usize;

            // the purely stochastic bug player setup
            let mut new_players: Vec<Player> = Vec::new();
            let mut rng = rand::thread_rng();
            let mut num_bugs = 0;

            // Step 2: Traverse and modify
            for i in 0..players.len() {
                let is_bug = rng.gen_ratio(1, 4);

                let mut newp = players[i].clone();
                if is_bug && max_bugs > num_bugs {
                    num_bugs += 1;
                    newp.role = Some(Role::Bugged);
                } else {
                    newp.role = Some(Role::Normal);
                }
                new_players.push(newp);
            }

            if num_bugs == 0 {
                // we need to choose at least one random person to be a bug
                new_players.shuffle(&mut rng);
                new_players[0].role = Some(Role::Bugged);
                new_players.shuffle(&mut rng);
            }

            for player in new_players {
                let player_key = format!("player:{}", player.id.to_string());
                let _ = redis.set_key(&player_key, &player).await;
            }

            // give tasks to all the players
            // update status
            current_game.status = GameStatus::Tasks;
            current_game.time = Some(Time {
                round: 0,
                timer: 90,
            });
            redis.set_key("game", &current_game).await.map_err(|e| {
                error!("{:?}", e);
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

// GET STATUS OF GAME
pub async fn get_game() -> Result<HttpResponse, Error> {
    let redis = RedisHelper::init().await.map_err(|e| {
        error!("{:?}", e);
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

pub async fn health_check() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello!"))
}

pub async fn get_game_players() -> Result<HttpResponse, Error> {
    let redis = RedisHelper::init().await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;
    let game: Game = redis.get_key("game").await.map_err(|e| { 
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("unknown error")
    })?;
    let index_key = format!("game:{}:player_index", game.id);
    let players: Vec<Player> = redis.get_index(&index_key).await.map_err(|e| { 
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("unknown error")
    })?;
    Ok(HttpResponse::Ok().json(players))
}

// Used to join the game
pub async fn post_player(_id: web::Json<Player>) -> Result<HttpResponse, Error> {
    let player = _id.0;
    let redis = RedisHelper::init().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    let game: Game = redis.get_key("game").await.map_err(|e| { 
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("unknown error")
    })?;
    if game.status != GameStatus::Lobby {
        return Err(actix_web::error::ErrorForbidden("You cannot join a game while it is in session"))
    }
    let registered_player_key = format!("player:{}", player.id);
    let registered_player: Player = redis.get_key(&registered_player_key).await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorForbidden("player does not have a tonk")
    })?;

    let index_key = format!("game:{}:player_index", game.id);
    let game_players: Vec<Player> = redis.get_index(&index_key).await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("There was an unknown error")
    })?;
    if game_players.iter().find(|p| p.id == player.id).is_some() {
        return Err(actix_web::error::ErrorForbidden("This player has already joined the game"));
    }
    let _ = redis.add_to_index(&index_key, &registered_player_key).await.map_err(|e| { 
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("There was an unknown error")
    })?;
    Ok(HttpResponse::Ok().json(registered_player))

    // let index_key = format!("game:{}:player_index", game.id);
    // let player_key = format!("game:{}:player:{}", game.id, player.id);
    // let redis_player: Result<Player, _> = redis.get_key(&player_key).await;
    // //TODO: for extra security, double check if the player is actually close to the tower or not

    // match redis_player {
    //     Ok(_) => {
    //         Err(actix_web::error::ErrorForbidden("Player already in the game"))
    //     }
    //     Err(e) => {
    //         if let Ok(_) = redis.set_key(&player_key, &registered_player).await {
    //             let index_key = format!("game:{}:player_index", game.id);
    //             let _ = redis.add_to_index(&index_key, &player_key).await.map_err(|_| { 
    //                 actix_web::error::ErrorInternalServerError("unknown error")
    //             })?;
    //             Ok(HttpResponse::Ok().json(player))
    //         } else {
    //             Err(actix_web::error::ErrorInternalServerError("Unknown error"))
    //         }
    //     }
    // }
}

pub async fn get_result() -> Result<HttpResponse, Error> {
    let redis = RedisHelper::init().await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;
    let game: Game = redis.get_key("game").await.map_err(|e| { 
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("unknown error")
    })?;

    let result_key = format!("result:{}:{}", game.id, game.time.as_ref().unwrap().round);
    let result: RoundResult = redis.get_key(&result_key).await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("unknown error")
    })?;

    Ok(HttpResponse::Ok().json(result))
}
