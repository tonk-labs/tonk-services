use tonk_shared_lib::{Game, Player, GameStatus, Action, Time, Task, RoundResult, Vote, Role};
use tonk_shared_lib::redis_helper::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::cmp::Eq;
use std::collections::HashSet;
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

    async fn set_vote_result(&self, game: &Game) -> Result<(), JobError> {
        let mut vote_result = RoundResult {
            round_type: GameStatus::Vote,
            eliminated: None,
            tasks_completed: None
        };
        let votes: Vec<Vote> = self.redis.get_index("game:votes").await.map_err(|_| JobError::RedisError)?;

        // count the votes
        let vote_counts = votes.iter().fold(HashMap::new(), |mut acc, vote| {
            *acc.entry(vote.candidate.clone()).or_insert(0) += 1;
            acc
        });

        let mut max_candidate: Option<Player> = None; 
        let mut max_count = 0;
        for (candidate, count) in vote_counts.iter() {
            if *count > max_count {
                max_count = *count;
                max_candidate = Some(candidate.clone());
            }
        }

        if max_candidate.is_some() {
            vote_result.eliminated  = Some(vec![max_candidate.as_ref().unwrap().clone()]);
        }

        let result_key = format!("result:{}:{}", game.id, game.time.as_ref().unwrap().round);
        let _ = self.redis.set_key(&result_key, &vote_result).await.map_err(|e| JobError::RedisError)?;

        Ok(())
    }

    async fn set_task_result(&self, game: &Game) -> Result<(), JobError> {
        let mut task_result = RoundResult {
            round_type: GameStatus::TaskResult,
            eliminated: None,
            tasks_completed: None,
        };

        // we need to count all the players eliminated
        let actions: Vec<Action> = self.redis.get_index("game:actions").await.map_err(|e| JobError::RedisError)?;
        let eliminated_players: Vec<Player> = actions.iter().map(|a| {
            a.poison_target.clone()
        }).collect();
        let interrupted_ids: HashSet<String> = actions
            .iter()
            .filter(|a| a.interrupted_task)
            .map(|a| a.poison_target.id.clone())
            .collect();

        // and we need to count all the tasks completed
        let tasks: Vec<Task> = self.redis.get_index("game:tasks").await.map_err(|e| JobError::RedisError)?;
        let filtered_tasks = tasks
            .iter()
            .filter(|t| {
                let interrupted = interrupted_ids.contains(t.assignee.as_ref().unwrap().id.as_str());
                !interrupted && t.complete
            }) 
            .map(|t| {
            Task {
                assignee: None,
                destination: None,
                round: t.round,
                complete: t.complete 
            }
        }).collect();
        task_result.eliminated = Some(eliminated_players);
        task_result.tasks_completed = Some(filtered_tasks);
        
        let result_key = format!("result:{}:{}", game.id, game.time.as_ref().unwrap().round);
        let _ = self.redis.set_key(&result_key, &task_result).await.map_err(|e| JobError::RedisError)?;

        Ok(())
    }

    async fn reset_round(&self, game: &Game) -> Result<(), JobError> {
        // remove players who were eliminated in the prior result
        let result_key = format!("result:{}:{}", game.id, game.time.as_ref().unwrap().round); 
        let prior_result: RoundResult = self.redis.get_key(&result_key).await?;

        let player_index_key = format!("game:{}:player_index", game.id);
        if prior_result.eliminated.is_some() {
            for player in prior_result.eliminated.as_ref().unwrap() {
                let player_key = format!("player:{}", player.id);
                self.redis.remove_from_index(&player_index_key, &player_key).await?;
            }
        }

        if game.status == GameStatus::TaskResult {
            let action_keys: Vec<String> = self.redis.get_index_keys("game:actions").await?;
            let task_keys: Vec<String> = self.redis.get_index_keys("game:tasks").await?;
            for key in action_keys {
                self.redis.clear_key(&key).await?;
            }
            for key in task_keys {
                self.redis.clear_key(&key).await?;
            }
        }

        if game.status == GameStatus::VoteResult {
            let vote_keys: Vec<String> = self.redis.get_index_keys("game:votes").await?;
            for key in vote_keys {
                self.redis.clear_key(&key).await?;
            }
        }

        // clear out actions, tasks, votes
        self.redis.clear_index("game:actions").await?;
        self.redis.clear_index("game:tasks").await?;
        self.redis.clear_index("game:votes").await?;

        Ok(())
    }

    async fn reset_to_new_game(&self, game: &Game) -> Result<(), JobError> {

        // clear out all individual results 
        for i in 0..game.time.as_ref().unwrap().round {
            let result_key = format!("result:{}:{}", game.id, i);
            self.redis.clear_key(&result_key).await?;
        }

        // remove all final players
        let game_player_index = format!("game:{}:player_index", game.id);
        self.redis.clear_index(&game_player_index).await?;

        // create a new game
        self.create_game().await?;

        Ok(())
    }

    async fn check_end_game_condition(&self, game: &Game) -> Result<bool, JobError> {
        let result_key = format!("result:{}:{}", game.id, game.time.as_ref().unwrap().round);
        let result: RoundResult = self.redis.get_key(&result_key).await?;
        let game_player_index = format!("game:{}:player_index", game.id);
        let players: Vec<Player> = self.redis.get_index(&game_player_index).await?;

        // all tasks were completed
        // if result.round_type == GameStatus::TaskResult {
        //     let tasks: Vec<Task> = self.redis.get_index("game:tasks").await.map_err(|e| JobError::RedisError)?;

        //     if tasks.len() == result.tasks_completed.as_ref().unwrap().len() {
        //         // find all the saboteurs
        //         for player in players {
        //             if *player.role.as_ref().unwrap() == Role::Bugged {
        //                 let player_key = format!("player:{}", player.id);
        //                 let _ =self.redis.remove_from_index(&game_player_index, &player_key).await?;
        //             }
        //         }
        //         return Ok(true);
        //     }
        // }
        
        let eliminated_ids: HashSet<String> = result
            .eliminated
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|player| player.id.clone())
            .collect();

        let remaining_players: Vec<&Player> = players
            .iter()
            .filter(|&p| !eliminated_ids.contains(&p.id))
            .collect();

        // everyone is killed, only saboteurs remain
        let no_villagers_remain = remaining_players.iter().fold(true, |acc, e| {
            if *e.role.as_ref().unwrap() == Role::Normal {
                false
            } else {
                acc
            }
        });
        // all saboteurs are voted off
        let no_saboteurs_remain = remaining_players.iter().fold(true, |acc, e| {
            if *e.role.as_ref().unwrap() == Role::Bugged {
                false
            } else {
                acc
            }
        });

        if no_saboteurs_remain || no_villagers_remain {
            return Ok(true);
        }

        Ok(false)
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
                let time = game.time.as_ref().unwrap();

                if time.timer == 0 {
                    self.set_task_result(&game).await?;
                    let next_game = Game {
                        id: game.id,
                        status: GameStatus::TaskResult,
                        time: Some(Time {
                            timer: 15,
                            round: time.round
                        }),
                    };
                    let _ = self.redis.set_key("game", &next_game).await?;
                }
                Ok(())
            }
            GameStatus::TaskResult => {
                // if the game is in task result phase, we move game into vote phase at the right time
                // unless the game is over, then we move to the end phase
                let time = game.time.as_ref().unwrap();

                if time.timer == 0 {
                // CHECK END CONDITIONS
                let is_end = self.check_end_game_condition(&game).await?;
                    self.reset_round(&game).await?;
                    if is_end {
                        let next_game = Game {
                            id: game.id.clone(),
                            status: GameStatus::End,
                            time: Some(Time {
                                timer: 30,
                                round: time.round 
                            }),
                        };
                        let _ = self.redis.set_key("game", &next_game).await?;
                    } else {
                        let next_game = Game {
                            id: game.id.clone(),
                            status: GameStatus::Vote,
                            time: Some(Time {
                                timer: 30,
                                round: time.round + 1
                            }),
                        };
                        let _ = self.redis.set_key("game", &next_game).await?;
                    }
                }
                Ok(())
            }
            GameStatus::Vote => {
                // if the game is in the vote phase, we move the game into the vote result phase at the right time
                let time = game.time.as_ref().unwrap();

                if time.timer == 0 {
                    self.set_vote_result(&game).await?;
                    let next_game = Game {
                        id: game.id,
                        status: GameStatus::VoteResult,
                        time: Some(Time {
                            timer: 15,
                            round: time.round
                        }),
                    };
                    let _ = self.redis.set_key("game", &next_game).await?;
                }
                Ok(())
            }
            GameStatus::VoteResult => {
                // if the game is in the vote result phase, we move the game into the task phase at the right time
                // unless the game is over, then we move to the end phase
                
                // CHECK END CONDITIONS
                let time = game.time.as_ref().unwrap();

                if time.timer == 0 {
                    let is_end = self.check_end_game_condition(&game).await?;
                    self.reset_round(&game.clone()).await?;
                    if is_end {
                        let next_game = Game {
                            id: game.id.clone(),
                            status: GameStatus::End,
                            time: Some(Time {
                                timer: 30,
                                round: time.round
                            }),
                        };
                        let _ = self.redis.set_key("game", &next_game).await?;
                    } else {
                        let next_game = Game {
                            id: game.id.clone(),
                            status: GameStatus::Tasks,
                            time: Some(Time {
                                timer: 30,
                                round: time.round + 1
                            }),
                        };
                        let _ = self.redis.set_key("game", &next_game).await?;
                    }
                }

                Ok(())
            }
            GameStatus::End => {
                // if the game is at the end, and timer is up, we should reset all the state and create a new game 
                let time = game.time.as_ref().unwrap();
                if time.timer == 0 {
                    self.reset_to_new_game(&game).await?;
                }
                Ok(())
            }
            GameStatus::Null => {
                // Something has gone wrong here?
                Ok(())
            }

        }
    }
}
