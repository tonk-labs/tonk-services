use std::borrow::BorrowMut;

use actix_web::{web, Error, HttpResponse, HttpRequest};
use tonk_shared_lib::{Task, Building, Game, Player, GameStatus};
use serde::{Deserialize, Serialize};
use crate::redis_helper::*;
use rand::Rng;

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskQuery {
    player_id: String,
    secret_key: String 
}

async fn get_random_depot(redis: &mut RedisHelper) -> Result<Building, Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let buildings: Vec<Building> = redis.get_index("building:index").await.map_err(|e| {
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
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let game: Game = redis.get_key("game").await?;
    if game.status != GameStatus::Tasks {
        return Err(actix_web::error::ErrorForbidden("The game is not in the task round"));
    }
    let player_id = &_query.player_id;
    let round = game.time.unwrap().round;
    let task_key = format!("task:{}:{}:{}", game.id, round, player_id);
    let task_result: Result<Task, Box<dyn std::error::Error>> = redis.get_key(&task_key).await;
    match task_result {
        Ok(task) => {
            Ok(HttpResponse::Ok().json(task))
        }
        Err(e) => {
            match *e.downcast().unwrap() {
                RedisHelperError::MissingKey => {
                    let depot = get_random_depot(redis.borrow_mut()).await?;
                    let random_task = Task {
                        assignee: Player { id: player_id.clone(), nearby_buildings: None, nearby_players: None, display_name: None, secret_key: None, location: None },
                        destination: depot,
                        round: round,
                        complete: false
                    };
                    let _ = redis.set_key(&task_key, &random_task).await?;
                    redis.set_index("game:tasks", &task_key).await?;
                    Ok(HttpResponse::Ok().json(random_task))
                }
                _ => {
                    Err(actix_web::error::ErrorInternalServerError("An unexpected error occurred."))
                }
            }
        }
    }
}

// USED TO CONFIRM SUCCESSFUL COMPLETION OF TASK
pub async fn post_task(_id: web::Json<Task>, _query: web::Query<TaskQuery>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut redis = RedisHelper::init().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let game: Game = redis.get_key("game").await?;
    let round = game.time.unwrap().round;
    if game.status != GameStatus::Tasks {
        return Err(actix_web::error::ErrorForbidden("The game is not in the task round"));
    }

    let player_id = &_query.player_id;
    let player_key = format!("player:{}", player_id);
    let task_key = format!("task:{}:{}:{}", game.id, round, player_id);

    let player: Player = redis.get_key(&player_key).await?;
    let task: Task = redis.get_key(&task_key).await?;

    if let Some(buildings) = player.nearby_buildings {
        for building in buildings {
            if building.id == _id.0.destination.id {
                let mut completed_task = task.clone();
                completed_task.complete = true;
                redis.set_key(&task_key, &completed_task).await?;
                return Ok(HttpResponse::Ok().json(completed_task));
            }
        }
        return Err(actix_web::error::ErrorForbidden("Player is not near the task building"));
    } else {
        return Err(actix_web::error::ErrorForbidden("Player is not near any buildings"));
    }
}