use bincode::{config, Decode, Encode};
use serde::{Deserialize, Serialize};

pub mod redis_helper;

#[derive(Serialize, Deserialize, Encode, Decode, Clone, PartialEq, Debug)]
pub enum GameStatus {
    Null, Lobby, Tasks, TaskResult, Vote, VoteResult, End
}

#[derive(Serialize, Deserialize, Encode, Decode, Eq, Hash, PartialEq, Clone, Debug)]
pub enum Role {
    Normal, Bugged
}

#[derive(Serialize, Deserialize, Encode, Decode, Eq, Hash, PartialEq, Clone, Debug)]
pub struct Location(pub String, pub String, pub String, pub String);

#[derive(Serialize, Deserialize, Encode, Decode, Eq, Hash, PartialEq, Clone, Debug)]
pub struct Player {
    pub id: String,
    pub mobile_unit_id: Option<String>,
    pub display_name: Option<String>,
    pub nearby_players: Option<Vec<Player>>,
    pub nearby_buildings: Option<Vec<Building>>,
    pub secret_key: Option<String>,
    pub location: Option<Location>,
    pub role: Option<Role>,
    pub used_action: Option<bool>
}

#[derive(Serialize, Deserialize, Encode, Decode, Hash, Eq, PartialEq, Clone, Debug)]
pub struct Building {
    pub id: String,
    pub location: Option<Location>,
    pub task_message: String,
    pub is_tower: bool
}

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Clone, Debug)]
pub struct Game {
    pub id: String,
    pub status: GameStatus,
    pub time: Option<Time>,
}

#[derive(Encode, Decode, Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Task {
    pub assignee: Option<Player>,
    pub destination: Option<Building>,
    pub round: u32,
    pub complete: bool
}

#[derive(Encode, Decode, Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Action {
    pub poison_target: Player,
    pub interrupted_task: bool,
    pub round: u32
}

#[derive(Serialize, Deserialize, Encode, Decode, Eq, PartialEq, Clone, Debug)]
pub struct Vote {
    pub candidate: Player,
}

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Clone, Debug)]
pub struct RoundResult {
    pub round_type: GameStatus,
    pub eliminated: Option<Vec<Player>>,
    pub tasks_completed: Option<Vec<Task>>
}

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Clone, Debug)]
pub struct Time {
    pub round: u32,
    pub timer: u32
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