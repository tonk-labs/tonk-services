use std::borrow::BorrowMut;

use actix_web::{web, Error, HttpResponse, HttpRequest};
use tonk_shared_lib::{Task, Building, Game, Player, GameStatus, Role, PlayerProximity};
use serde::{Deserialize, Serialize};
use tonk_shared_lib::redis_helper::*;
use rand::Rng;
use log::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskQuery {
    player_id: String,
    secret_key: String 
}

async fn get_random_depot(redis: &mut RedisHelper) -> Result<Building, Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let buildings: Vec<Building> = redis.get_index("building:index").await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let random_index = rng.gen_range(0..buildings.len()-1); 
    let depots: Vec<&Building> = buildings.iter().filter(|e| {
        !e.is_tower
    }).collect();
    let chosen_depot = depots[random_index].clone();
    Ok(chosen_depot)
}

// RETURNS TASK AND IF IT DOESNT EXIST THEN RANDOMLY ASSIGNS NEW TASK
pub async fn get_task(_query: web::Query<TaskQuery>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut redis = RedisHelper::init().await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let game: Game = redis.get_key("game").await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;
    if game.status != GameStatus::Tasks {
        return Err(actix_web::error::ErrorForbidden("The game is not in the task round"));
    }

    let index_key = format!("game:{}:player_index", game.id);
    let player_keys: Vec<String> = redis.get_index_keys(&index_key).await.map_err(|e| { 
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;

    let player_id = &_query.player_id;
    let player_key = format!("player:{}", player_id);

    if player_keys.iter().find(|k| **k == player_key).is_none() {
        return Err(actix_web::error::ErrorForbidden("Player is not in the game"));
    }

    let player: Player = redis.get_key(&player_key).await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;
    if *player.role.as_ref().unwrap() == Role::Bugged {
        let empty_task = Task {
            assignee: Some(player.clone()),
            destination: Some(Building { id: "".to_string(), readable_id: "".to_string(), location: None, task_message: "You have been corrupted and seek to attack others.".to_string(), is_tower: false }),
            round: game.time.as_ref().unwrap().round.clone(),
            dropped_off: false,
            complete: false
        };
        return Ok(HttpResponse::Ok().json(empty_task));
    }
    let round = game.time.unwrap().round;
    let task_key = format!("task:{}:{}:{}", game.id, round, player_id);
    let task_result: Result<Task, RedisHelperError> = redis.get_key(&task_key).await;
    match task_result {
        Ok(task) => {
            Ok(HttpResponse::Ok().json(task))
        }
        Err(RedisHelperError::MissingKey) => {
            let depot = get_random_depot(redis.borrow_mut()).await?;
            let random_task = Task {
                assignee: Some(Player { 
                    id: player_id.clone(), 
                    used_action: None,
                    last_round_action: None,
                    display_name: None, 
                    mobile_unit_id: None, 
                    secret_key: None, 
                    role: None,
                    eliminated: None,
                    proximity: None,
                }),
                destination: Some(depot),
                round: round,
                dropped_off: false,
                complete: false
            };
            let _ = redis.set_key(&task_key, &random_task).await.map_err(|e| {
                error!("{:?}", e);
                actix_web::error::ErrorInternalServerError("Unknown error")
            })?;
            redis.add_to_index("game:tasks", &task_key).await.map_err(|e| {
                error!("{:?}", e);
                actix_web::error::ErrorInternalServerError("Unknown error")
            })?;
            Ok(HttpResponse::Ok().json(random_task))
        }
        _ => {
            Err(actix_web::error::ErrorInternalServerError("An unexpected error occurred."))
        }
    }
}

// USED TO CONFIRM SUCCESSFUL COMPLETION OF TASK
pub async fn post_task(_id: web::Json<Task>, _query: web::Query<TaskQuery>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut redis = RedisHelper::init().await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let game: Game = redis.get_key("game").await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;
    let round = game.time.unwrap().round;
    if game.status != GameStatus::Tasks {
        return Err(actix_web::error::ErrorForbidden("The game is not in the task round"));
    }

    let player_id = &_query.player_id;
    let player_key = format!("player:{}", player_id);
    let task_key = format!("task:{}:{}:{}", game.id, round, player_id);

    let player: Player = redis.get_key(&player_key).await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;
    let mut updated_player = player.clone();

    let task: Task = redis.get_key(&task_key).await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;

    if task.complete {
        return Err(actix_web::error::ErrorForbidden("Task is already complete"));
    }

    let player_proximity_key = format!("player:{}:proximity", player_id);
    let proximity: PlayerProximity = redis.get_key(&player_proximity_key).await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;
    if let Some(buildings) = proximity.nearby_buildings {
        for building in buildings {
            if !task.dropped_off && building.id == _id.0.destination.as_ref().unwrap().id {
                let mut updated_task = task.clone();
                updated_task.dropped_off = true;
                redis.set_key(&task_key, &updated_task).await.map_err(|e| {
                    actix_web::error::ErrorInternalServerError("Unknown error")
                })?;
                return Ok(HttpResponse::Ok().json(updated_task));
            }
            if !task.complete && building.is_tower {
                let mut completed_task = task.clone();
                completed_task.complete = true;
                redis.set_key(&task_key, &completed_task).await.map_err(|e| {
                    actix_web::error::ErrorInternalServerError("Unknown error")
                })?;

                updated_player.used_action = Some(true);
                updated_player.last_round_action = Some(round);
                redis.set_key(&player_key, &updated_player).await.map_err(|e| {
                    error!("{:?}", e);
                    actix_web::error::ErrorInternalServerError("Unknown error")
                })?;

                return Ok(HttpResponse::Ok().json(completed_task));
            }
        }
        return Err(actix_web::error::ErrorForbidden("Player is not near the task building"));
    } else {
        return Err(actix_web::error::ErrorForbidden("Player is not near any buildings"));
    }
}