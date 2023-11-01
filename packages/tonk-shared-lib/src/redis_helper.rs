use std::error::Error;
use redis::{AsyncCommands, RedisResult, aio::Connection, RedisError};
use bincode::{Decode, Encode, error};
use tokio::sync::Mutex;
use crate::{deserialize_struct, serialize_struct};
use std::env;

pub async fn get_connection() -> RedisResult<Connection> {
    let redis_url = env::var("REDIS_URL").unwrap();
    let client = redis::Client::open(redis_url)?;
    client.get_async_connection().await
}

pub struct RedisHelper {
    con: Mutex<Connection>
}

#[derive(Debug)]
pub enum RedisHelperError {
    MissingKey, Deserialization, Serialization, RedisError, Unknown 
} 


impl std::error::Error for RedisHelperError {
    fn description(&self) -> &str {
        match self {
            RedisHelperError::MissingKey => "Error: object is missing from the state",
            RedisHelperError::Deserialization => "Error: deserialization error",
            RedisHelperError::Serialization => "Error: serialization error",
            RedisHelperError::RedisError => "Error: redis error",
            RedisHelperError::Unknown => "Error: unknown error",
        }
    }
}

impl std::fmt::Display for RedisHelperError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RedisHelperError::MissingKey => write!(f, "Error: object is missing from the state"),
            RedisHelperError::Deserialization => write!(f, "Error: deserialization error"),
            RedisHelperError::Serialization => write!(f, "Error: serialization error"),
            RedisHelperError::RedisError => write!(f, "Error: redis error"),
            RedisHelperError::Unknown => write!(f, "Error: unknown error")
        }
    }
}

impl From<error::DecodeError> for RedisHelperError {
    fn from(err: error::DecodeError) -> RedisHelperError {
        RedisHelperError::Deserialization
    }
}
impl From<error::EncodeError> for RedisHelperError {
    fn from(err: error::EncodeError) -> RedisHelperError {
        RedisHelperError::Serialization
    }
}
impl From<RedisError> for RedisHelperError {
    fn from(err: RedisError) -> RedisHelperError {
        //maybe log the error?
        RedisHelperError::RedisError
    }
}


impl RedisHelper {
    pub async fn init() -> Result<Self, RedisHelperError> {
        let con = get_connection().await?;
        Ok(Self { con: Mutex::new(con) })
    }

    pub async fn get_key<T: Decode>(&self, key: &str) -> Result<T, RedisHelperError> {
        let mut con_guard = self.con.lock().await;
        let exists: bool = con_guard.exists(key).await?;
        if !exists {
            return Err(RedisHelperError::MissingKey);
        }
        let result: Vec<u8> = con_guard.get(key).await?;
        let deserialized = deserialize_struct(&result)?;
        Ok(deserialized)
    }
    pub async fn get_key_test(&self, key: &str) -> Result<String, RedisHelperError> {
        let mut con_guard = self.con.lock().await;
        let exists: bool = con_guard.exists(key).await?;
        if !exists {
            return Err(RedisHelperError::MissingKey);
        }
        let result: String = con_guard.get(key).await?;
        Ok(result)
    }

    pub async fn set_key<T: Encode>(&self, key: &str, obj: &T) -> Result<(), RedisHelperError> {
        let mut con_guard = self.con.lock().await;
        let vec = serialize_struct(obj)?;
        let _ = con_guard.set(key, vec).await?;
        Ok(())
    }

    pub async fn clear_key(&self, key: &str) -> Result<(), RedisHelperError> {
        let mut con_guard = self.con.lock().await;
        let _ = con_guard.del(key).await?;
        Ok(())
    }

    pub async fn add_to_index(&self, index: &str, key: &str) -> Result<(), RedisHelperError> {
        let mut con_guard = self.con.lock().await;
        let _ = con_guard.sadd(index, key).await?;
        Ok(())
    }

    pub async fn remove_from_index(&self, index: &str, key: &str) -> Result<(), RedisHelperError> {
        let mut con_guard = self.con.lock().await;
        let _ = con_guard.srem(index, key).await?;
        Ok(())
    }

    pub async fn get_index_keys(&self, index: &str) -> Result<Vec<String>, RedisHelperError> {
        let mut con_guard = self.con.lock().await;
        let members: Vec<String> = con_guard.smembers(index).await?;
        Ok(members)
    }

    pub async fn get_index<T: Decode>(&self, index: &str) -> Result<Vec<T>, RedisHelperError> {
        let mut con_guard = self.con.lock().await;
        let members: Vec<String> = con_guard.smembers(index).await?;
        let mut deserialized_members: Vec<T> = Vec::new();
        for member_key in members {
            let member_bytes: Vec<u8> = con_guard.get(member_key).await?;
            let member: T = deserialize_struct(&member_bytes)?;
            deserialized_members.push(member);
        }
        Ok(deserialized_members)
    }

    pub async fn clear_index(&self, index: &str) -> Result<(), RedisHelperError> {
        let mut con_guard = self.con.lock().await;
        let _ = con_guard.del(index).await?;
        Ok(())
    }

}

