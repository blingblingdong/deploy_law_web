use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct File {
    pub id: String,
    pub content: String,
    pub css: String,
    pub user_name: String,
    pub directory: String,
    pub file_name: String,
    pub content_nav: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Files {
    pub vec_files: Vec<File>,
}
