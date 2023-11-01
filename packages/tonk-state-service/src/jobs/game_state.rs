use redis::RedisError;
use tonk_shared_lib::{Game, Player, GameStatus, Action, Time, Task, RoundResult, Vote, Role, Elimination, EliminationReason, WinResult, PlayerProximity};
use tonk_shared_lib::redis_helper::*;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::hash::Hash;
use std::cmp::Eq;
use std::collections::HashSet;
use std::ops::Index;
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
            demo_play: false,
            corrupted_players: None,
            time: Some(Time {
                timer: 0,
                round: 0
            }),
            win_result: None
        };
        let _ = self.redis.set_key("game", &game).await?;
        Ok(())
    }

    async fn set_vote_result(&self, game: &Game) -> Result<Game, JobError> {
        let mut vote_result = RoundResult {
            round_type: GameStatus::Vote,
            eliminated: None,
            tasks_completed: None
        };
        let votes: Vec<Vote> = self.redis.get_index("game:votes").await.map_err(|_| JobError::RedisError)?;
        let mut new_corrupted: Vec<Player> = Vec::new();

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

        // check for inactive players
        let player_index_key = format!("game:{}:player_index", game.id);
        let players: Vec<Player> = self.redis.get_index(&player_index_key).await.map_err(|e| JobError::RedisError )?;

        let inactive_players: Vec<Elimination> = players.iter().filter(|p| {
            p.used_action.is_some() && *p.used_action.as_ref().unwrap_or(&tonk_shared_lib::ActionStatus::Unused) != tonk_shared_lib::ActionStatus::Voted
        }).map(|p| {
            Elimination {
                player: p.clone(),
                reason: EliminationReason::Inaction
            }
        }).collect();

        let mut eliminated_players: Vec<Elimination> = Vec::new(); 

        let mut max_candidate_id: String = "".to_string();
        if max_candidate.is_some() {
            max_candidate_id = max_candidate.as_ref().unwrap().id.clone();
            // println!("max_candidate: {:?}", max_candidate.as_ref().unwrap().role.as_ref().unwrap());
            if *max_candidate.as_ref().unwrap().role.as_ref().unwrap() == Role::Bugged {
                new_corrupted.push(max_candidate.as_ref().unwrap().clone());
            }
            eliminated_players.push(Elimination {
                player: max_candidate.as_ref().unwrap().clone(),
                reason: EliminationReason::VotedOut
            });
        }

        for inactive_player in inactive_players {
            if inactive_player.player.id != max_candidate_id && !game.demo_play {
                if *inactive_player.player.role.as_ref().unwrap() == Role::Bugged {
                    new_corrupted.push(inactive_player.player.clone())
                }
                eliminated_players.push(inactive_player);
            }
        }

        vote_result.eliminated = Some(eliminated_players);

        let mut new_game = game.clone();
        if new_corrupted.len() > 0 {
            // println!("Setting corrupted_players on new game");
            if game.corrupted_players.as_ref().is_some() {
                new_corrupted.append(game.clone().corrupted_players.as_mut().unwrap());
            } 
            new_game.corrupted_players = Some(new_corrupted);
            // println!("New game object {:?}", new_game);
            let _ = self.redis.set_key("game", &new_game).await?;
        }

        let result_key = format!("result:{}:{}", game.id, game.time.as_ref().unwrap().round);
        let _ = self.redis.set_key(&result_key, &vote_result).await.map_err(|e| JobError::RedisError)?;

        Ok(new_game)
    }

    async fn set_task_result(&self, game: &Game) -> Result<Game, JobError> {
        let mut task_result = RoundResult {
            round_type: GameStatus::TaskResult,
            eliminated: None,
            tasks_completed: None,
        };
        let mut new_corrupted: Vec<Player> = Vec::new();

        let mut eliminations: HashSet<String> = HashSet::new();

        // we need to count all the players eliminated
        let actions: Vec<Action> = self.redis.get_index("game:actions").await.map_err(|e| JobError::RedisError)?;
        println!("actions: {:?}", actions);
        let mut eliminated_players: Vec<Elimination> = actions.iter().filter(|a| {
            a.interrupted_task
        }).map(|a| {
            eliminations.insert(a.poison_target.id.clone());
            Elimination {
                player: a.poison_target.clone(),
                reason: EliminationReason::BuggedOut
            }
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
                second_destination: None,
                round: t.round,
                dropped_off: t.dropped_off,
                dropped_off_second: t.dropped_off_second,
                complete: t.complete 
            }
        }).collect();

        // check for inactive players
        let player_index_key = format!("game:{}:player_index", game.id);
        let players: Vec<Player> = self.redis.get_index(&player_index_key).await.map_err(|e| JobError::RedisError )?;

        let inactive_players: Vec<Elimination> = players.iter().filter(|p| {
            p.used_action.is_some() && *p.used_action.as_ref().unwrap_or(&tonk_shared_lib::ActionStatus::Unused) != tonk_shared_lib::ActionStatus::TaskComplete
        }).map(|p| {
            Elimination {
                player: p.clone(),
                reason: EliminationReason::Inaction
            }
        }).collect();

        for inactive_player in inactive_players {
            if !eliminations.contains(&inactive_player.player.id) && !game.demo_play {
                if *inactive_player.player.role.as_ref().unwrap() == Role::Bugged {
                    new_corrupted.push(inactive_player.player.clone())
                }
                eliminated_players.push(inactive_player);
            }
        }

        task_result.eliminated = Some(eliminated_players);
        task_result.tasks_completed = Some(filtered_tasks);
        
        let result_key = format!("result:{}:{}", game.id, game.time.as_ref().unwrap().round);
        let _ = self.redis.set_key(&result_key, &task_result).await.map_err(|e| JobError::RedisError)?;

        let mut new_game = game.clone();
        if new_corrupted.len() > 0 {
            if game.corrupted_players.as_ref().is_some() {
                new_corrupted.append(game.clone().corrupted_players.as_mut().unwrap());
            } 
            new_game.corrupted_players = Some(new_corrupted);
            let _ = self.redis.set_key("game", &new_game).await?;
        }

        Ok(new_game)
    }

    async fn reset_round(&self, game: &Game) -> Result<(), JobError> {
        // remove players who were eliminated in the prior result
        let result_key = format!("result:{}:{}", game.id, game.time.as_ref().unwrap().round); 
        let prior_result: RoundResult = self.redis.get_key(&result_key).await?;

        let player_index_key = format!("game:{}:player_index", game.id);
        let players: Vec<Player> = self.redis.get_index(&player_index_key).await?;

        // println!("Calling reset round!");

        for player in players {
                let player_key = format!("player:{}", player.id);
                let mut reset_player = player.clone();
                reset_player.used_action = Some(tonk_shared_lib::ActionStatus::Unused); 

                // println!("resetting used_action for player {}!", player_key);

                self.redis.set_key(&player_key, &reset_player).await?;
        }

        if prior_result.eliminated.is_some() {
            for elimination in prior_result.eliminated.as_ref().unwrap() {
                let eliminated_player = elimination.player.clone();
                let player_key = format!("player:{}", eliminated_player.id);
                let mut player: Player = self.redis.get_key(&player_key).await?;

                self.redis.remove_from_index(&player_index_key, &player_key).await?;

                player.eliminated = Some(true);
                self.redis.set_key(&player_key, &player).await?;
            }
        }

        if game.status == GameStatus::Tasks {
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

    async fn check_all_votes_in(&self, game: &Game) -> Result<bool, JobError> {
        let player_index_key = format!("game:{}:player_index", game.id);
        let player_keys = self.redis.get_index_keys(&player_index_key).await?;

        let vote_keys = self.redis.get_index_keys("game:votes").await?;

        Ok(player_keys.len() == vote_keys.len())
    }

    async fn check_all_tasks_in(&self, game: &Game) -> Result<bool, JobError> {
        let player_index_key = format!("game:{}:player_index", game.id);
        let players: Vec<Player> = self.redis.get_index(&player_index_key).await?;

        let tasks = self.redis.get_index_keys("game:tasks").await?;
        let actions = self.redis.get_index_keys("game:actions").await?;

        if (tasks.len() + actions.len()) == players.len() {
            let all_done = players.iter().fold(true, |acc, e| {
                acc && (*e.used_action.as_ref().unwrap_or(&tonk_shared_lib::ActionStatus::Unused) == tonk_shared_lib::ActionStatus::TaskComplete)
            });
            return Ok(all_done);
        }
        return Ok(false);
    }

    async fn clear_player_state(&self) -> Result<(), JobError> {
        let players: Vec<Player> = self.redis.get_index("player:index").await?;
        // println!("clearing players {:?}", players);
        for player in players {
            let clean_player = Player {
                id: player.id.clone(),
                mobile_unit_id: player.mobile_unit_id.clone(),
                display_name: player.display_name.clone(),
                secret_key: None,
                last_round_action: None,
                eliminated: None,
                proximity: None,
                role: None,
                used_action: None,
            };
            let player_key = format!("player:{}", player.id);
            let _ = self.redis.set_key(&player_key, &clean_player).await?;

            let proximity_key = format!("player:{}:proximity", player.id);
            let clean_proximity = PlayerProximity {
                location: None,
                nearby_buildings: None,
                nearby_players: None,
                immune: None
            };
            let _ = self.redis.set_key(&proximity_key, &clean_proximity).await?;
        }
        Ok(())
    }

    async fn reset_to_new_game(&self, game: &Game) -> Result<(), JobError> {

        // clear out all individual results 
        for i in 0..game.time.as_ref().unwrap().round {
            let result_key = format!("result:{}:{}", game.id, i);
            self.redis.clear_key(&result_key).await?;
        }

        // clear the state of all players
        self.clear_player_state().await?;

        // remove all final players
        let game_player_index = format!("game:{}:player_index", game.id);
        self.redis.clear_index(&game_player_index).await?;

        // create a new game
        self.create_game().await?;

        Ok(())
    }

    async fn check_end_game_condition(&self, game: &Game) -> Result<WinResult, JobError> {
        let result_key = format!("result:{}:{}", game.id, game.time.as_ref().unwrap().round);
        let result: RoundResult = self.redis.get_key(&result_key).await?;
        let game_player_index = format!("game:{}:player_index", game.id);
        let players: Vec<Player> = self.redis.get_index(&game_player_index).await?;

        // all tasks were completed
        if result.round_type == GameStatus::Tasks {
            let tasks: Vec<Task> = self.redis.get_index("game:tasks").await.map_err(|e| JobError::RedisError)?;

            // we disable this for games of 2 players to allow for a limited setup demo 
            if tasks.len() == result.tasks_completed.as_ref().unwrap().len() && !game.demo_play {
                
                // find all the saboteurs
                for player in players {
                    // clean them in this very MESSY way ;__;
                    if *player.role.as_ref().unwrap() == Role::Bugged {
                        let player_key = format!("player:{}", player.id);
                        let _ = self.redis.remove_from_index(&game_player_index, &player_key).await?;

                        let clean_player = Player {
                            id: player.id.clone(),
                            mobile_unit_id: player.mobile_unit_id.clone(),
                            display_name: player.display_name.clone(),
                            secret_key: None,
                            last_round_action: None,
                            eliminated: None,
                            proximity: None,
                            role: None,
                            used_action: None,
                        };
                        let _ = self.redis.set_key(&player_key, &clean_player).await?;

                        let proximity_key = format!("player:{}:proximity", player.id);
                        let clean_proximity = PlayerProximity {
                            location: None,
                            nearby_buildings: None,
                            nearby_players: None,
                            immune: None
                        };
                        let _ = self.redis.set_key(&proximity_key, &clean_proximity).await?;
                    }
                }
                return Ok(WinResult::Perfection);
            }
        }

        
        let eliminated_ids: HashSet<String> = result
            .eliminated
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|elimination| elimination.player.id.clone())
            .collect();

        let remaining_players: Vec<&Player> = players
            .iter()
            .filter(|&p| !eliminated_ids.contains(&p.id))
            .collect();

        // NEW win condition, more than 50% of players are bugs
        let number_of_bugs = remaining_players.iter().fold(0, |acc, e| {
            if *e.role.as_ref().unwrap() == Role::Bugged {
                acc + 1
            } else {
                acc
            }
        });

        if number_of_bugs as f64 >= (remaining_players.len() as f64 * 0.5) && !game.demo_play {
            return Ok(WinResult::Thuggery);
        }

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

        if no_saboteurs_remain && no_villagers_remain {
            return Ok(WinResult::Armageddon);
        }

        if no_saboteurs_remain {
            return Ok(WinResult::Democracy);
        }

        if no_villagers_remain {
            return Ok(WinResult::Thuggery);
        }

        Ok(WinResult::Null)
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
                let all_tasks_in = self.check_all_tasks_in(&game).await?;
                if all_tasks_in && time.timer > 0 {
                    let mut ngame = game.clone();
                    ngame.time = Some(Time {
                        round: time.round,
                        timer: 0,
                    });
                    let _ = self.redis.set_key("game", &ngame).await?;
                    return Ok(());
                }

                if time.timer <= 0 {
                    let new_game = self.set_task_result(&game).await?;
                    let is_end = self.check_end_game_condition(&new_game).await?;
                    self.reset_round(&new_game).await?;
                    if is_end != WinResult::Null {
                        let next_game = Game {
                            id: game.id.clone(),
                            status: GameStatus::End,
                            corrupted_players: new_game.corrupted_players,
                            demo_play: game.demo_play,
                            time: Some(Time {
                                timer: 30,
                                round: time.round 
                            }),
                            win_result: Some(is_end)
                        };
                        let _ = self.redis.set_key("game", &next_game).await?;
                    } else {
                        let next_game = Game {
                            id: game.id,
                            status: GameStatus::Vote,
                            corrupted_players: new_game.corrupted_players,
                            demo_play: game.demo_play,
                            time: Some(Time {
                                timer: 90,
                                round: time.round + 1
                            }),
                            win_result: None
                        };
                        let _ = self.redis.set_key("game", &next_game).await?;
                    }
                }
                Ok(())
            }
            // THIS TaskResult round is now obselete, TODO: delete
            GameStatus::TaskResult => {
                // if the game is in task result phase, we move game into vote phase at the right time
                // unless the game is over, then we move to the end phase
                let time = game.time.as_ref().unwrap();

                if time.timer == 0 {
                    // CHECK END CONDITIONS
                    let is_end = self.check_end_game_condition(&game).await?;
                    self.reset_round(&game).await?;
                    if is_end != WinResult::Null {
                        let next_game = Game {
                            id: game.id.clone(),
                            status: GameStatus::End,
                            demo_play: game.demo_play,
                            corrupted_players: game.corrupted_players,
                            time: Some(Time {
                                timer: 30,
                                round: time.round 
                            }),
                            win_result: Some(is_end)
                        };
                        let _ = self.redis.set_key("game", &next_game).await?;
                    } else {
                        self.reset_round(&game).await?;
                        let next_game = Game {
                            id: game.id.clone(),
                            status: GameStatus::Vote,
                            demo_play: game.demo_play,
                            corrupted_players: game.corrupted_players,
                            time: Some(Time {
                                timer: 90,
                                round: time.round + 1
                            }),
                            win_result: None
                        };
                        let _ = self.redis.set_key("game", &next_game).await?;
                    }
                }
                Ok(())
            }
            GameStatus::Vote => {
                // if the game is in the vote phase, we move the game into the vote result phase at the right time
                let time = game.time.as_ref().unwrap();
                let all_votes_in = self.check_all_votes_in(&game).await?;
                if all_votes_in && time.timer > 0 {
                    let mut ngame = game.clone();
                    ngame.time = Some(Time {
                        round: time.round,
                        timer: 0,
                    });
                    let _ = self.redis.set_key("game", &ngame).await?;
                    return Ok(());
                }

                if time.timer <= 0 {
                    let new_game = self.set_vote_result(&game).await?;
                    let next_game = Game {
                        id: game.id,
                        status: GameStatus::VoteResult,
                        corrupted_players: new_game.corrupted_players,
                        demo_play: game.demo_play,
                        time: Some(Time {
                            timer: 30,
                            round: time.round
                        }),
                        win_result: None
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
                    if is_end != WinResult::Null {
                        let next_game = Game {
                            id: game.id.clone(),
                            status: GameStatus::End,
                            demo_play: game.demo_play,
                            corrupted_players: game.corrupted_players,
                            time: Some(Time {
                                timer: 30,
                                round: time.round
                            }),
                            win_result: Some(is_end)
                        };
                        let _ = self.redis.set_key("game", &next_game).await?;
                    } else {
                        let next_game = Game {
                            id: game.id.clone(),
                            status: GameStatus::Tasks,
                            demo_play: game.demo_play,
                            corrupted_players: game.corrupted_players,
                            time: Some(Time {
                                timer: 180,
                                round: time.round + 1
                            }),
                            win_result: None
                        };
                        self.redis.set_key("game", &next_game).await?;
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
