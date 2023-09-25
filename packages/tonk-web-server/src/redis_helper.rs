use std::error::Error;
use redis::{AsyncCommands, RedisResult, aio::Connection, RedisError};
use bincode::{Decode, Encode};
use tonk_shared_lib::{deserialize_struct, serialize_struct};

async fn get_connection() -> RedisResult<Connection> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    client.get_async_connection().await
}

pub struct RedisHelper {
    con: Connection
}

#[derive(Debug)]
pub enum RedisHelperError {
    MissingKey
} 

impl std::error::Error for RedisHelperError {
    fn description(&self) -> &str {
        match self {
            RedisHelperError::MissingKey => "Error: object is missing from the state",
        }
    }
}

impl std::fmt::Display for RedisHelperError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RedisHelperError::MissingKey => write!(f, "Error: object is missing from the state")
        }
    }
}

impl RedisHelper {
    pub async fn init() -> Result<Self, RedisError> {
        let con = get_connection().await?;
        Ok(Self {
            con: con
        })
    }

    pub async fn get_key<T: Decode>(&mut self, key: &str) -> Result<T, Box<dyn Error>> {
        let exists: bool = self.con.exists(key).await?;
        if !exists {
            return Err(Box::new(RedisHelperError::MissingKey));
        }
        let result: Vec<u8> = self.con.get(key).await?;
        let deserialized = deserialize_struct(&result)?;
        Ok(deserialized)
    }

    pub async fn set_key<T: Encode>(&mut self, key: &str, obj: &T) -> Result<(), Box<dyn Error>> {
        let vec = serialize_struct(obj)?;
        let _ = self.con.set(key, vec).await?;
        Ok(())
    }

    pub async fn set_index(&mut self, index: &str, key: &str) -> Result<(), Box<dyn Error>> {
        let _ = self.con.sadd(index, key).await?;
        Ok(())
    }

    pub async fn get_index<T: Decode>(&mut self, index: &str) -> Result<Vec<T>, Box<dyn Error>> {
        let members: Vec<String> = self.con.smembers(index).await?;
        let mut deserialized_members: Vec<T> = Vec::new();
        let mut connection = get_connection().await?;
        for member_key in members {
            let member_bytes: Vec<u8> = connection.get(member_key).await?;
            let member: T = deserialize_struct(&member_bytes)?;
            deserialized_members.push(member);
        }
        Ok(deserialized_members)
    }

}

