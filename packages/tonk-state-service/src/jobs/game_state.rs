use tonk_shared_lib::{Game, GameStatus, Action, Time};
use tonk_shared_lib::redis_helper::*;
use uuid::Uuid;
use crate::jobs::error::*;

pub struct GameState {
    redis: RedisHelper
}


impl GameState {
    pub fn new(redis: RedisHelper) -> Self {
        Self { redis }
    }

    pub async fn run(&self) -> Result<(), JobError> {
        // get the game
        let game_result: Result<Game, _> = self.redis.get_key("game").await;
        // if there is no game, we should create one
        if let Err(RedisHelperError::MissingKey) = game_result {
            return self.create_game().await;
        } 

        if game_result.is_ok() {
            return self.update_logic(game_result.unwrap()).await;
        } else {
            return Err(game_result.unwrap_err().into())
        }
    }

    async fn create_game(&self) -> Result<(), JobError> {
        // Handle the MissingKey error case
        let game = Game {
            id: Uuid::new_v4().as_simple().to_string(),
            status: GameStatus::Lobby,
            time: Some(Time {
                timer: 0,
                round: 0
            }),
        };
        let _ = self.redis.set_key("game", &game).await?;
        Ok(())
    }


    async fn update_logic(&self, game: Game) -> Result<(), JobError> {
        match game.status {
        // if the game is in the Lobby, we do nothing
            GameStatus::Lobby => {
                // we are just waiting around for someone to start the game
                Ok(())
            }
            GameStatus::Tasks => {
                // if the game is in the task phase, we move the game into task result phase at the right time
                // we need to update the summary for that round
                let time = game.time.unwrap();


                // let actions: Vec<Action> = self.redis.get_index("game:actions").await;

                // let summary = {

                // }

                if time.timer >= 120 {
                    let next_game = Game {
                        id: game.id,
                        status: GameStatus::TaskResult,
                        time: Some(Time {
                            timer: 0,
                            round: time.round + 1
                        }),
                    };
                    let _ = self.redis.set_key("game", &next_game).await?;
                }
                Ok(())
            }
            GameStatus::TaskResult => {
            // if the game is in task result phase, we move game into vote phase at the right time
                // unless the game is over, then we move to the end phase

                Ok(())
            }
            GameStatus::Vote => {
                // if the game is in the vote phase, we move the game into the vote result phase at the right time
                Ok(())
            }
            GameStatus::VoteResult => {
            // if the game is in the vote result phase, we move the game into the task phase at the right time
                // unless the game is over, then we move to the end phase

                Ok(())
            }
            GameStatus::End => {
                // if the game is at the end, and timer is up, we should reset all the state and create a new game 
                Ok(())
            }
            GameStatus::Null => {
                // Something has gone wrong here?
                Ok(())
            }

        }
    }
}
