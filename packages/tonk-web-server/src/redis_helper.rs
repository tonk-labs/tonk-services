use std::error::Error;
use redis::{AsyncCommands, RedisResult, aio::Connection, RedisError};
use bincode::{Decode, Encode};
use tonk_shared_lib::{deserialize_struct, serialize_struct};
use futures::stream::StreamExt;

async fn get_connection() -> RedisResult<Connection> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    client.get_async_connection().await
}

pub struct RedisHelper {
    con: Connection
}

impl RedisHelper {
    pub async fn init() -> Result<Self, RedisError> {
        let con = get_connection().await?;
        Ok(Self {
            con: con
        })
    }

    pub async fn get_key<T: Decode>(&mut self, key: &str) -> Result<T, Box<dyn Error>> {
        let result: Vec<u8> = self.con.get(key).await?;
        let deserialized = deserialize_struct(&result)?;
        Ok(deserialized)
    }

    pub async fn set_key<T: Encode>(&mut self, key: &str, obj: &T) -> Result<(), Box<dyn Error>> {
        let vec = serialize_struct(obj)?;
        let _ = self.con.set(key, vec).await?;
        Ok(())
    }

    pub async fn set_index(&mut self, index: &str, key: &str) -> Result<(), redis::RedisError> {
        self.con.sadd(index, key).await
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

