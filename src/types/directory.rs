use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::store::Store;
use crate::types::record;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Directory {
    pub id: String, //(user_name + directory),
    pub user_name: String,
    pub directory: String,
    pub public: bool,
    pub description: String,
}

