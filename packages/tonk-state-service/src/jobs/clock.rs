use tonk_shared_lib::{Game, GameStatus, Action, Time};
use tonk_shared_lib::redis_helper::*;

use super::error::JobError;

pub struct Clock {
    redis: RedisHelper
}

// All this does is advance the clock
impl Clock {
    pub fn new(redis: RedisHelper) -> Self {
        Self { redis }
    }

    pub async fn run(&self) -> Result<(), JobError> {
        let game: Game = self.redis.get_key("game").await?;
        if game.status == GameStatus::Null || game.status == GameStatus::Lobby {
            return Ok(());
        }
        let time = game.time.unwrap();

        if time.timer == 0 {
            // we should just wait
        } else {
            let next_game = Game {
                id: game.id,
                status: game.status,
                time: Some(Time {
                    timer: time.timer - 1,
                    round: time.round
                }),
                win_result: game.win_result 
            };
            let _ = self.redis.set_key("game", &next_game).await?;
        }
        Ok(())
    }
}
