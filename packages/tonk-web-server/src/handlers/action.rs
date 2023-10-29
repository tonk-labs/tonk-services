use actix_web::{web, Error, HttpResponse, HttpRequest};
use tonk_shared_lib::{Game, Player, Action, GameStatus, Task, Role, PlayerProximity};
use tonk_shared_lib::redis_helper::*;
use serde::{Deserialize, Serialize};
use log::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionQuery {
    player_id: String,
    secret_key: String 
}

// USED TO POISON OTHER PLAYERS DURING THE TASK ROUND
pub async fn post_action(_id: web::Json<Action>, _query: web::Query<ActionQuery>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut redis = RedisHelper::init().await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let action = _id.0;
    let game: Game = redis.get_key("game").await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;
    let round = game.time.unwrap().round;
    if round != action.round {
        return Err(actix_web::error::ErrorBadRequest("Improper round in request"));
    }
    if game.status != GameStatus::Tasks {
        return Err(actix_web::error::ErrorForbidden("The game is not in the task round"));
    }
    
    let player_id = &_query.player_id;
    let player_key = format!("player:{}", player_id);
    let player: Player = redis.get_key(&player_key).await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;

    let mut updated_player = player.clone();

    if *player.role.as_ref().unwrap() != Role::Bugged {
        return Err(actix_web::error::ErrorForbidden("You cannot take this action"));
    }

    let player_proximity_key = format!("player:{}:proximity", player_id);
    let proximity: PlayerProximity = redis.get_key(&player_proximity_key).await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;
    let nearby_players = proximity.nearby_players.unwrap();


    let target_is_near = nearby_players.iter().find(|e| {
        e.id == action.poison_target.id
    });
    if target_is_near.is_none() {
        return Err(actix_web::error::ErrorForbidden("The target is not within range"));
    }

    let target_proximity_key = format!("player:{}:proximity", target_is_near.as_ref().unwrap().id);
    let target_proximity: PlayerProximity = redis.get_key(&target_proximity_key).await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;
    if *target_is_near.as_ref().unwrap().role.as_ref().unwrap() == Role::Bugged {
        return Err(actix_web::error::ErrorForbidden("Bugs cannot bug another bug"));
    }
    if *target_proximity.immune.as_ref().unwrap() {
        return Err(actix_web::error::ErrorForbidden("You cannot bug someone within 3 tiles of the tower"));
    }

    let action_key = format!("action:{}:{}:{}", game.id, round, player.id);
    let exists: Result<Action, _> = redis.get_key(&action_key).await;
    if exists.is_err() {
        let mut updated_action = action.clone();
        let interrupted_task_key = format!("task:{}:{}:{}", game.id, round, action.poison_target.id);
        let task_result: Task = redis.get_key(&interrupted_task_key).await.map_err(|e| {
            error!("{:?}", e);
            actix_web::error::ErrorInternalServerError("Unknown error")
        })?;
        if !task_result.complete {
            updated_action.interrupted_task = true;
        }

        let _ = redis.set_key(&action_key, &updated_action).await.map_err(|e| {
            error!("{:?}", e);
            actix_web::error::ErrorInternalServerError("Unknown error")
        })?;

        updated_player.used_action = Some(true);
        redis.set_key(&player_key, &updated_player).await.map_err(|e| {
            error!("{:?}", e);
            actix_web::error::ErrorInternalServerError("Unknown error")
        })?;
        redis.add_to_index("game:actions", &action_key).await.map_err(|e| {
            error!("{:?}", e);
            actix_web::error::ErrorInternalServerError("Unknown error")
        })?;
    } else {
        return Err(actix_web::error::ErrorForbidden("You have already taken an action this round"));
    }

    Ok(HttpResponse::Ok().finish())
}