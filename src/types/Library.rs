use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Library {
    pub id: String, //uuid
    pub library_name: String,
    pub user_name: String,
    pub public: bool,
}

#[derive(Serialize, Deserialize)]
pub struct LibraryItem {
    pub id: String,// uuid
    pub item_library: String, // 作為索引
    pub item_type: String,
    pub item_id: String,
    pub item_name: String,
    pub order: i16, // 預設為1，favorite為99
}