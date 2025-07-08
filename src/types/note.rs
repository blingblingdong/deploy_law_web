use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Note {
    pub id: String,
    pub content: Option<serde_json::Value>,
    pub footer: Option<String>,
    pub user_name: String,
    pub directory: String,
    pub file_name: String,
    pub public: bool,
}
