use std::collections::HashMap;
use std::ops::{RangeBounds, Index};
use log::*;
use redis::{Commands, ToRedisArgs, RedisResult};
use reqwest;
use gql_client;
use serde::{Deserialize,Serialize};
use std::env;

use tonk_shared_lib::{self, PlayerProximity};
use tonk_shared_lib::redis_helper::*;
use super::error::JobError;

#[derive(Deserialize, Debug)]
pub struct Data {
    game: Game
}

#[derive(Deserialize, Debug)]
pub struct Game {
    id: String,
    state: State,
}


#[derive(Deserialize, Debug)]
pub struct State {
    nodes: Vec<Node>
}

#[derive(Deserialize, Debug)]
pub struct Node {
    id: String,
    player: Player,
    location: Location
}

#[derive(Deserialize, Debug)]
pub struct Player {
    id: String,
    addr: String,
}

#[derive(Deserialize, Debug)]
pub struct Location {
    id: String,
    tile: Tile
}

#[derive(Deserialize, Debug)]
pub struct Tile {
    id: String,
    coords: Vec<String>
}


pub struct SyncGraph {
    client: reqwest::Client,
    redis: RedisHelper
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BuildingVars {
    gameID: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerVars {
    gameID: String,
    ids: Vec<String>
}

pub const DS_PLAYER_QUERY: &str = r#"query DSPlayers($gameID: ID!, $ids: [String!]!) {
    game(id: $gameID){    
      id
      state {
        nodes(match: {kinds: "MobileUnit", ids: $ids}) {
          id
          player: node(match: { kinds: "Player" }) {
            ...SelectedPlayer
          }
          location: edge(match: { kinds: "Tile", via: { rel: "Location", key: 1 } }) {
            ...Location
          }
        }
      }
    }
  }
  
  fragment Location on Edge {
      id
      tile: node {
          id
          coords: keys
      }
  }
  
  fragment SelectedPlayer on Node {
      ...WorldPlayer
  }
  
  fragment WorldPlayer on Node {
      id
      addr: key
  }"#;

pub const DS_BUILDING_QUERY: &str = r#"query DSBuildings($gameID: ID!) {
    game(id: $gameID) {
        id
        name
        state {
            nodes(match: {kinds: "Tile"}) {
                coords: keys
                building: node(match: { kinds: "Building", via: { rel: "Location", dir: IN } }) {
                    id
                        kind: node(match: { kinds: "BuildingKind", via: { rel: "Is" } }) {
                            ...BuildingKind
                        }
                }
            }
        }
    }
}

fragment BuildingKind on Node {
    id
    name: annotation(name: "name") {
        value
    }
    description: annotation(name: "description") {
        value
    }
    model: annotation(name: "model") {
        value
    }
}
"#;

#[derive(Debug)]
struct Cube {
    q: i32,
    r: i32,
    s: i32,
}

fn hex_twos_complement_to_i32(hex: &str) -> i32 {
    if let Ok(val) = u16::from_str_radix(hex.replace("0x", "").as_str(), 16) {
        if (val & 0x8000) != 0 {
            -((!(val - 1)) as i32)
        } else {
            val as i32
        }
    } else {
        i32::MAX
    }
}


impl Cube {
    fn new(loc: &tonk_shared_lib::Location) -> Self {
        Self {
            q: hex_twos_complement_to_i32(&loc.1),
            r: hex_twos_complement_to_i32(&loc.2),
            s: hex_twos_complement_to_i32(&loc.3),
        }
    }
    fn add(&self, other: &Cube) -> Cube {
        Cube {
            q: self.q + other.q,
            r: self.r + other.r,
            s: self.s + other.s,
        }
    }
    fn subtract(&self, other: &Cube) -> Cube {
        Cube {
            q: self.q - other.q,
            r: self.r - other.r,
            s: self.s - other.s,
        }
    }

    fn distance(&self, other: &Cube) -> i32 {
        let vec = self.subtract(other);
        (vec.q.abs() + vec.r.abs() + vec.s.abs()) / 2
    }
}

const CUBE_DIRECTION_VECTORS: [Cube; 6] = [
    Cube { q: 1, r: 0, s: -1 },
    Cube { q: 1, r: -1, s: 0 },
    Cube { q: 0, r: -1, s: 1 },
    Cube { q: -1, r: 0, s: 1 },
    Cube { q: -1, r: 1, s: 0 },
    Cube { q: 0, r: 1, s: -1 },
];

async fn get_test_data(vars: &PlayerVars, redis: &RedisHelper) -> Result<Option<Data>, RedisHelperError> {
    let redis_keys: Vec<String> = vars.ids.iter().map(|e|  {
        format!("locations:{}", e)
    }).collect();
    let mut nodes: Vec<Node> = Vec::new();
    for rkey in redis_keys {
        let result = redis.get_key_test(&rkey).await?;
        let node: Node = serde_json::from_str(&result).map_err(|_| RedisHelperError::Unknown)?;
        nodes.push(node);
    }
    Ok(Some(Data {
        game: Game {
            id: vars.gameID.clone(),
            state: State {
                nodes
            }
        }
    }))
}


impl SyncGraph {
    pub fn new(redis: RedisHelper) -> Self {
        Self {
            redis,
            client: reqwest::Client::new()
        }
    }

    fn update_locations_player(&self, data: &Data, players: &Vec<tonk_shared_lib::Player>) -> HashMap<String, tonk_shared_lib::Location> {
        let mut player_locations: HashMap<String, tonk_shared_lib::Location> = HashMap::new();
        for entry in &data.game.state.nodes {
            if let Some(player) = players.iter().find(|p| {
                if let Some(id) = &p.mobile_unit_id {
                    *id == entry.id
                } else {
                    false
                }
            }) {
                let location_coords = tonk_shared_lib::Location(
                    entry.location.tile.coords[0].to_string(),
                    entry.location.tile.coords[1].to_string(),
                    entry.location.tile.coords[2].to_string(),
                    entry.location.tile.coords[3].to_string(),
                );
                player_locations.insert(player.id.clone(), location_coords.clone());
            } 
        }
        // println!("{:?}", player_locations);
        player_locations
    }

    async fn calculate_distance(&self, 
        players: &Vec<tonk_shared_lib::Player>, 
        player_locations: &HashMap<String, tonk_shared_lib::Location>
    ) -> Result<HashMap<String, tonk_shared_lib::PlayerProximity>, JobError> {

        let building_index = format!("building:index");
        let buildings: Vec<tonk_shared_lib::Building> = self.redis.get_index(&building_index).await?;
        let mut player_proximities: HashMap<String, tonk_shared_lib::PlayerProximity> = HashMap::new();
        for i in 0..players.len() {
            let mut nearby_buildings: Vec<tonk_shared_lib::Building> = Vec::new();
            let mut immune = Some(false);
            let location_unwrapped = player_locations.get(&players[i].id);
            if location_unwrapped.is_none() {
                println!("no player id in player_locations for: {}", &players[i].id.clone());
                return Err(JobError::Unknown)
            }
            let location = location_unwrapped.unwrap();
            let player_cube_coord = Cube::new(location);
            // players[i].immune = Some(false);
            for j in 0..buildings.len() {
                let buildings_cube_coord = Cube::new(buildings[j].location.as_ref().unwrap());
                let distance = player_cube_coord.distance(&buildings_cube_coord);
                if distance < 2 {
                    nearby_buildings.push(tonk_shared_lib::Building { 
                        id: buildings[j].id.clone(), 
                        readable_id: buildings[j].readable_id.clone(),
                        location: buildings[j].location.clone(), 
                        is_tower: buildings[j].is_tower,
                        task_message: "".to_string(),
                    });
                }
                if distance < 4 && buildings[j].is_tower {
                    immune = Some(true);
                } 
            }
            player_proximities.insert(players[i].id.clone(), tonk_shared_lib::PlayerProximity {
                nearby_buildings: Some(nearby_buildings.clone()),
                nearby_players: None,
                immune: immune.clone(),
                location: None,
            });
        }

        for i in 0..players.len() {
            let location = player_locations.get(&players[i].id).unwrap();
            let player_cube_coord = Cube::new(location);
            let mut nearby_players: Vec<tonk_shared_lib::Player> = Vec::new();
            let mut show_role = false;
            if players[i].role.is_some() && *players[i].role.as_ref().unwrap() == tonk_shared_lib::Role::Bugged {
                show_role = true;
            }
            for j in 0..players.len() {
                let other_location = player_locations.get(&players[j].id).unwrap();
                let other_cube_coord = Cube::new(other_location);
                let distance = player_cube_coord.distance(&other_cube_coord);
                // let is_another_bug = players[j].role.as_ref().unwrap_or(&tonk_shared_lib::Role::Bugged).clone() == tonk_shared_lib::Role::Bugged;
                if distance < 3 && j != i {
                    let mut role = None;
                    if show_role {
                        role = players[j].role.clone()
                    } 
                    let j_proximal = player_proximities.get(&players[j].id).unwrap();
                    nearby_players.push(tonk_shared_lib::Player {
                        id: players[j].id.clone(),
                        mobile_unit_id: players[j].mobile_unit_id.clone(),
                        display_name: players[j].display_name.clone(),
                        used_action: None,
                        last_round_action: None,
                        proximity: Some(PlayerProximity {
                            nearby_buildings: None,
                            nearby_players: None,
                            immune: j_proximal.immune.clone(),
                            location: None,
                        }),
                        secret_key: None,
                        role,
                        eliminated: None
                    });
                }
            }
            let prev_values = player_proximities.get(&players[i].id.clone()).unwrap();
            let location = player_locations.get(&players[i].id).unwrap();
            player_proximities.insert(players[i].id.clone(), tonk_shared_lib::PlayerProximity {
                nearby_buildings: prev_values.nearby_buildings.clone(),
                nearby_players: Some(nearby_players),
                immune: prev_values.immune.clone(),
                location: Some(location.clone())
            });
        }
        Ok(player_proximities)
    }

    pub async fn run(&self) -> Result<(), JobError> {
        let game: tonk_shared_lib::Game = self.redis.get_key("game").await?;
        if game.status == tonk_shared_lib::GameStatus::End {
            return Ok(());
        }
        let game_index = format!("game:{}:player_index", game.id);
        let mut game_players: Vec<tonk_shared_lib::Player> = self.redis.get_index(&game_index).await?;
        // let mut reg_players: Vec<tonk_shared_lib::Player> = self.redis.get_index("player:index").await?;
        // print!("{:?}", reg_players);
        let ids: Vec<String> = game_players.iter_mut().map(|p| p.mobile_unit_id.clone().unwrap_or("".to_string()) ).collect();
        // println!("{:?}", ids);
        if ids.len() == 0 {
            // println!("{:?}", "skipping location update, no players in the game");
            return Ok(());
        }

        let endpoint = env::var("DS_ENDPOINT").unwrap();
        let client = gql_client::Client::new(endpoint);
        let vars = PlayerVars {
            gameID: "DOWNSTREAM".to_string(),
            ids,
        };
        let result: Result<Option<Data>, gql_client::GraphQLError> = client.query_with_vars::<Data, PlayerVars>(DS_PLAYER_QUERY, vars).await;
        if result.is_err() {
            // println!("{:?}", result.as_ref().err().unwrap());
            return Ok(());
        } else {
            // println!("{:?}", result.as_ref().unwrap());
        }

        let round = game.time.as_ref().unwrap().round;

        if let Some(data) = result.unwrap() {
            let player_locations = self.update_locations_player(&data, &game_players);
            let player_proximities = self.calculate_distance(&game_players, &player_locations).await?;
            for player in game_players {
                // let player_key = format!("player:{}", player.id);
                // println!("immunity for {:?}:{:?}", player.display_name, player.immune);
                // SUPER hacky, but we're just going to do this for now to get the job done.
                // if player.last_round_action.is_some() && *player.last_round_action.as_ref().unwrap() < round {
                //     player.used_action = Some(false);
                // }
                // let _: () = self.redis.set_key(&player_key, &player).await?;
                let proximity = player_proximities.get(&player.id).unwrap();
                let proximity_key = format!("player:{}:proximity", player.id);
                let _: () = self.redis.set_key(&proximity_key, &proximity).await?;
            }
            Ok(())
        } else {
            Ok(())
        }
    } 

    pub async fn mock_run(&self) -> Result<(), JobError> {
        let game: tonk_shared_lib::Game = self.redis.get_key("game").await?;
        if game.status == tonk_shared_lib::GameStatus::End {
            return Ok(());
        }
        let game_index = format!("game:{}:player_index", game.id);
        let mut game_players: Vec<tonk_shared_lib::Player> = self.redis.get_index(&game_index).await?;
        // let mut reg_players: Vec<tonk_shared_lib::Player> = self.redis.get_index("player:index").await?;
        // print!("{:?}", reg_players);
        let ids: Vec<String> = game_players.iter_mut().map(|p| p.mobile_unit_id.clone().unwrap_or("".to_string()) ).collect();
        // println!("{:?}", ids);
        if ids.len() == 0 {
            // println!("{:?}", "skipping location update, no players in the game");
            return Ok(());
        }

        let endpoint = env::var("DS_ENDPOINT").unwrap();
        let vars = PlayerVars {
            gameID: "DOWNSTREAM".to_string(),
            ids,
        };
        let result: Result<Option<Data>, RedisHelperError> = get_test_data(&vars, &self.redis).await;

        let round = game.time.as_ref().unwrap().round;

        if let Some(data) = result.unwrap() {
            let player_locations = self.update_locations_player(&data, &game_players);
            let player_proximities = self.calculate_distance(&game_players, &player_locations).await?;
            for player in game_players {
                // let player_key = format!("player:{}", player.id);
                // println!("immunity for {:?}:{:?}", player.display_name, player.immune);
                // SUPER hacky, but we're just going to do this for now to get the job done.
                // if player.last_round_action.is_some() && *player.last_round_action.as_ref().unwrap() < round {
                //     player.used_action = Some(false);
                // }
                // let _: () = self.redis.set_key(&player_key, &player).await?;
                let proximity = player_proximities.get(&player.id).unwrap();
                let proximity_key = format!("player:{}:proximity", player.id);
                let _: () = self.redis.set_key(&proximity_key, &proximity).await?;
            }
            Ok(())
        } else {
            Ok(())
        }
    } 
}