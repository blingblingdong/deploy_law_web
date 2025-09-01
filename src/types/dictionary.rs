use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Dictionary {
    pub id: String, //uuid
    pub name: String,
    pub user_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct VocabItem {
    pub id: String,
    pub user_name: String,
    pub term: String,
    pub definition: String,
    pub dictionary: String,
}

#[derive(Serialize, Deserialize)]
pub struct VocabItemLaw {
    pub vocabitem_id: String,
    pub law_id: String,
}
