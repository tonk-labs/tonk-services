use tonk_shared_lib::{Game, GameStatus, Action, Time};
use tonk_shared_lib::redis_helper::*;
use serde::{Deserialize,Serialize};

use super::error::JobError;

pub struct Clock {
    redis: RedisHelper
}

#[derive(Deserialize, Debug)]
pub struct ClockTestInjection {
    time: Time,
    status: GameStatus
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
                demo_play: game.demo_play,
                corrupted_players: game.corrupted_players,
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

    pub async fn mock_run(&self) -> Result<(), JobError> {
        let game: Game = self.redis.get_key("game").await?;
        if game.status == GameStatus::Null || game.status == GameStatus::Lobby {
            return Ok(());
        }
        let raw = self.redis.get_key_test("clock").await;
        if raw.is_err() {
            return Ok(());
        } else {
            let clk: ClockTestInjection = serde_json::from_str(&raw.unwrap()).map_err(|_| RedisHelperError::Unknown)?;
            if clk.time.timer != game.time.as_ref().unwrap().timer || clk.time.round != game.time.as_ref().unwrap().round || clk.status != game.status {
                let next_game = Game {
                    id: game.id,
                    status: clk.status,
                    demo_play: game.demo_play,
                    corrupted_players: game.corrupted_players,
                    time: Some(clk.time.clone()),
                    win_result: game.win_result 
                };
                let _ = self.redis.set_key("game", &next_game).await?;
                self.redis.clear_key("clock").await?;
            }
            Ok(())
        }
    }
}
