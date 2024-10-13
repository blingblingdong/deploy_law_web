use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use redis::AsyncCommands;
use redis::RedisError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    pub user_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub exp: DateTime<Utc>,
    pub user_name: String,
    pub nbf: DateTime<Utc>
}

pub struct Redis_Database {
    pub connection: redis::aio::Connection
}

impl Redis_Database {
    pub async fn new(url: &str) -> Result<Self, RedisError> {
        let client = redis::Client::open(url)?;
        match client.get_async_connection().await {
            Ok(connection) => Ok(Redis_Database{ connection}),
            Err(e) => {
                eprintln!("Redis連接失敗!{}", e);
                Err(e)
            }
        }
    }
}

/*
pub async fn redis_connection() -> redis::aio::Connection {
    let client = redis::Client::open("redis://:11131028@localhost:6379").unwrap();
    client.get_async_connection().await.unwrap()
}
*/
