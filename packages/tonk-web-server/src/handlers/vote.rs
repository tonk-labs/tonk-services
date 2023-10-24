use actix_web::{web, Error, HttpResponse, HttpRequest};
use tonk_shared_lib::{Vote, Game, Player, GameStatus};
use serde::{Deserialize, Serialize};
use tonk_shared_lib::redis_helper::*;
use log::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct VoteQuery {
    player_id: String,
    secret_key: String 
}

// USED TO CONFIRM SUCCESSFUL COMPLETION OF TASK
pub async fn post_vote(_id: web::Json<Vote>, _query: web::Query<VoteQuery>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut redis = RedisHelper::init().await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let game: Game = redis.get_key("game").await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;
    let round = game.time.unwrap().round;
    if game.status != GameStatus::Vote {
        return Err(actix_web::error::ErrorForbidden("The game is not in the voting round"));
    }

    let mut vote = _id.0.clone();
    let player_id = &_query.player_id;
    let player_key = format!("player:{}", player_id);

    let index_key = format!("game:{}:player_index", game.id);
    let player_keys: Vec<String> = redis.get_index_keys(&index_key).await.map_err(|e| { 
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;

    if player_keys.iter().find(|k| **k == player_key).is_none() {
        return Err(actix_web::error::ErrorForbidden("Player is not in the game"));
    }


    let candidate_key = format!("player:{}", vote.candidate.id);
    let vote_key = format!("vote:{}:{}:{}", game.id, round, player_id);

    let candidate: Player = redis.get_key(&candidate_key).await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;

    let mut player: Player = redis.get_key(&player_key).await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;

    let saved_vote: Result<Vote, _> = redis.get_key(&vote_key).await;
    match saved_vote {
        Err(RedisHelperError::MissingKey) => {
            vote.candidate.display_name = candidate.display_name.clone();
            let _ = redis.set_key(&vote_key, &vote).await.map_err(|e| {
                error!("{:?}", e);
                actix_web::error::ErrorInternalServerError("Unknown error")
            })?;
            player.used_action = Some(true);
            player.last_round_action = Some(round);
            redis.set_key(&player_key, &player).await.map_err(|e| {
                error!("{:?}", e);
                actix_web::error::ErrorInternalServerError("Unknown error")
            })?;
            redis.add_to_index("game:votes", &vote_key).await.map_err(|e| {
                error!("{:?}", e);
                actix_web::error::ErrorInternalServerError("Unknown error")
            })?;
        } Ok(_) => {
            return Err(actix_web::error::ErrorForbidden("You have already made your vote this round"));
        } _ => {
            return Err(actix_web::error::ErrorInternalServerError("Unknown error"));
        }
    }

    Ok(HttpResponse::Ok().finish())
}