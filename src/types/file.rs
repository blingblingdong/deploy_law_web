use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{PgPool, Row};


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct File {
    pub id: String,
    pub content: String,
    pub css: String,
    pub user_name: String,
    pub directory: String,
}



