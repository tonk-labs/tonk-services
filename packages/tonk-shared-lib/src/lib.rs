use bincode::{config, Decode, Encode};
use serde::{Deserialize, Serialize};

pub mod redis_helper;

#[derive(Serialize, Deserialize, Encode, Decode, Clone, PartialEq, Debug)]
pub enum GameStatus {
    Null, Lobby, Tasks, TaskResult, Vote, VoteResult, End
}

#[derive(Serialize, Deserialize, Encode, Decode, Clone, PartialEq, Debug)]
pub enum WinResult {
    Thuggery, Democracy, Perfection, Armageddon, Null
}

#[derive(Serialize, Deserialize, Encode, Decode, Eq, Hash, PartialEq, Clone, Debug)]
pub enum Role {
    Normal, Bugged
}

#[derive(Serialize, Deserialize, Encode, Decode, Eq, Hash, PartialEq, Clone, Debug)]
pub struct Location(pub String, pub String, pub String, pub String);

#[derive(Serialize, Deserialize, Encode, Decode, Eq, Hash, PartialEq, Clone, Debug)]
pub enum ActionStatus {
    Unused, ReturnToTower, NextDepot, TaskComplete, Voted, 
}

#[derive(Serialize, Deserialize, Encode, Decode, Eq, Hash, PartialEq, Clone, Debug)]
pub struct Player {
    pub id: String,
    pub mobile_unit_id: Option<String>,
    pub display_name: Option<String>,
    pub secret_key: Option<String>,
    pub role: Option<Role>,
    pub used_action: Option<ActionStatus>,
    pub last_round_action: Option<u32>,
    pub eliminated: Option<bool>,
    pub proximity: Option<PlayerProximity>
}


#[derive(Serialize, Deserialize, Encode, Decode, Eq, Hash, PartialEq, Clone, Debug)]
pub struct PlayerProximity {
    pub nearby_players: Option<Vec<Player>>,
    pub nearby_buildings: Option<Vec<Building>>,
    pub immune: Option<bool>,
    pub location: Option<Location>,
}

#[derive(Serialize, Deserialize, Encode, Decode, Hash, Eq, PartialEq, Clone, Debug)]
pub struct Building {
    pub id: String,
    pub readable_id: String,
    pub location: Option<Location>,
    pub task_message: String,
    pub is_tower: bool
}

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Clone, Debug)]
pub struct Game {
    pub id: String,
    pub status: GameStatus,
    pub time: Option<Time>,
    pub win_result: Option<WinResult>,
    pub corrupted_players: Option<Vec<Player>>,
    pub demo_play: bool,
}

#[derive(Encode, Decode, Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Task {
    pub assignee: Option<Player>,
    pub destination: Option<Building>,
    pub round: u32,
    pub dropped_off: bool,
    pub complete: bool
}

#[derive(Encode, Decode, Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Action {
    pub poison_target: Player,
    pub interrupted_task: bool,
    pub confirmed: bool,
    pub round: u32
}

#[derive(Serialize, Deserialize, Encode, Decode, Eq, PartialEq, Clone, Debug)]
pub struct Vote {
    pub candidate: Player,
}

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Clone, Debug)]
pub enum EliminationReason {
    BuggedOut, VotedOut, Inaction 
}

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Clone, Debug)]
pub struct Elimination {
    pub player: Player,
    pub reason: EliminationReason
}

#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Clone, Debug)]
pub struct RoundResult {
    pub round_type: GameStatus,
    pub eliminated: Option<Vec<Elimination>>,
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