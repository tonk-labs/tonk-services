use actix_web::{web, Error, HttpResponse, HttpRequest};
use tonk_shared_lib::{Vote, Game, Player, GameStatus};
use serde::{Deserialize, Serialize};
use tonk_shared_lib::redis_helper::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct VoteQuery {
    player_id: String,
    secret_key: String 
}

// USED TO CONFIRM SUCCESSFUL COMPLETION OF TASK
pub async fn post_vote(_id: web::Json<Vote>, _query: web::Query<VoteQuery>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut redis = RedisHelper::init().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let game: Game = redis.get_key("game").await.map_err(|e| {
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;
    let round = game.time.unwrap().round;
    if game.status != GameStatus::Vote {
        return Err(actix_web::error::ErrorForbidden("The game is not in the voting round"));
    }

    let vote = _id.0;
    let player_id = &_query.player_id;
    // let player_key = format!("player:{}", player_id);
    let vote_key = format!("vote:{}:{}:{}", game.id, round, player_id);

    // let player: Player = redis.get_key(&player_key).await?;

    let exists: Result<Vote, _> = redis.get_key(&vote_key).await;
    if exists.is_err() {
        let _ = redis.set_key(&vote_key, &vote).await.map_err(|e| {
            actix_web::error::ErrorInternalServerError("Unknown error")
        })?;
        redis.set_index("game:votes", &vote_key).await.map_err(|e| {
            actix_web::error::ErrorInternalServerError("Unknown error")
        })?;
    } else {
        return Err(actix_web::error::ErrorForbidden("You have already made your vote this round"));
    }

    Ok(HttpResponse::Ok().finish())
}