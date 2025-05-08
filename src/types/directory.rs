use serde::{Deserialize, Serialize};



#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Directory {
    pub id: String, //(user_name + directory),
    pub user_name: String,
    pub directory: String,
    pub public: bool,
    pub description: String,
}

