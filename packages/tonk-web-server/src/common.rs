use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerQuery {
    pub privateKey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Building {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    pub message: String,
}