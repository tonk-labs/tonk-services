use bincode::{config, Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Debug)]
pub enum GameStatus {
    Null, Lobby, Tasks, Vote, End
}

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Debug)]
pub struct Location(pub String, pub String, pub String, pub String);

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Debug)]
pub struct Player {
    pub id: String,
    pub is_near_player: Option<Vec<Player>>,
    pub is_near_building: Option<Vec<Building>>,
    pub display_name: Option<String>,
    pub secret_key: Option<String>,
    pub location: Option<Location>
}

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Debug)]
pub struct Building {
    pub id: String,
    pub location: Option<Location>,
    pub is_tower: bool
}

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Debug)]
pub struct Game {
    pub id: String,
    pub status: GameStatus,
    pub time: Option<Time>,
    pub players: Option<Vec<Player>>
}

#[derive(Encode, Decode, Serialize, Deserialize, PartialEq, Debug)]
pub struct Task {
    pub assignee: Player,
    pub destination: Building ,
    pub round: u32,
}

#[derive(Encode, Decode, Serialize, Deserialize, PartialEq, Debug)]
pub struct Action {
    pub poison_target: Player,
    pub round: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Debug)]
pub struct Vote {
    pub voter: Player,
    pub candidate: Player
}

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Debug)]
pub struct Time {
    pub round: u32,
    pub timer: u32
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct GameState {
    pub id: String,
    pub players: Vec<Player>,
    pub bugged_players: Vec<Player>,
    pub current_votes: Vec<Vote>,
    pub current_tasks: Vec<Task>
}

pub fn serialize_struct<T: Encode>(obj: &T) -> Result<Vec<u8>, bincode::error::EncodeError> {
    let config = config::standard();
    bincode::encode_to_vec(obj, config)
}

pub fn deserialize_struct<T: Decode>(vec: &Vec<u8>) -> Result<T, bincode::error::DecodeError> {
    let config = config::standard();
    let (decoded, _): (T, usize) = bincode::decode_from_slice(vec, config)?;
    Ok(decoded)
}