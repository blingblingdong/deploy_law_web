use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{PgPool, Row};
use crate::types::record::LawRecord;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct File {
    pub id: String,
    pub content: String,
    pub css: String,
    pub user_name: String,
    pub directory: String,
    pub file_name: String
}

#[derive(Debug, Clone)]
pub struct Files {
    pub vec_files: Vec<File>
}


