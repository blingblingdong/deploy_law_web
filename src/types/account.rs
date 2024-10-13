use chrono::prelude::*;
use serde::{Serialize, Deserialize};

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