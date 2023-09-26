use tonk_shared_lib::redis_helper::*;

#[derive(Debug)]
pub enum JobError {
    RedisError, ClientError, SerializationError, Unknown
} 


impl std::error::Error for JobError {
    fn description(&self) -> &str {
        match self {
            JobError::SerializationError => "Error: serialization error",
            JobError::RedisError => "Error: redis error",
            JobError::ClientError => "Error: client error",
            JobError::Unknown =>  "Error: unknown error"
        }
    }
}

impl std::fmt::Display for JobError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            JobError::SerializationError => write!(f, "Error: serialization error"),
            JobError::ClientError => write!(f, "Error: client error"),
            JobError::RedisError => write!(f, "Error: redis error"),
            JobError::Unknown => write!(f, "Error: unknown error")
        }
    }
}

impl From<RedisHelperError> for JobError {
    fn from(err: RedisHelperError) -> JobError {
        match err {
            RedisHelperError::Deserialization => {
                JobError::SerializationError
            }
            RedisHelperError::Serialization => {
                JobError::SerializationError
            }
            _ => {
                JobError::Unknown
            }
        }
    }
}
