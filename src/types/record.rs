#[allow(unused_imports)]
use std::string::String;
use std::sync::Arc;
use law_rs::*;
use sqlx::postgres::{PgPoolOptions, PgPool, PgRow};
use sqlx::{Row};
use indexmap::{IndexMap};

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LawRecord {
    pub id: String,// primary key(user+directory+chapter+num)
    pub chapter: String,
    pub num: String,
    pub user_name: String,
    pub directory: String,
    pub note: String,
}






impl LawRecord {
    pub fn new(chapter: String, num: String, user_name: String, directory: String) -> Self {
        let id = format!("{}-{}-{}-{}", user_name, directory, chapter, num);
        LawRecord {id, chapter, num, user_name, directory, note: "新增筆記".to_string()}
    }
}


#[derive(Debug, Clone)]
pub struct LawRecords {
    pub vec_record: Vec<LawRecord>
}

impl LawRecords {

    pub fn categorize_by_dir(&self)  -> Result<IndexMap<String, Vec<LawRecord>>, handle_errors::Error> {
        let mut map = IndexMap::new();
        for r in self.vec_record.iter() {
            map.entry(r.directory.clone()).or_insert_with(Vec::new).push(r.clone());
        }
        Ok(map)
    }

    pub fn get_by_dir(&self, dir: String) -> Result<LawRecords, handle_errors::Error> {
        let map = self.categorize_by_dir()?;
        let records = map.get(&dir).unwrap();
        Ok(LawRecords { vec_record: records.clone() })
    }

    pub async fn show_records(&self) -> String {
        let res = self.vec_record.clone();
        let mut table = String::new();
        table.push_str("<h2>查詢記錄</h2>");
        table.push_str("<ul>");
        for law in res.iter() {
            table.push_str(&format!(
                "<li class='record-button'>{}-{}</li>",
                law.chapter, law.num
            ));
        }
        table.push_str("</ul>");
        table
    }

    pub  fn get_laws(&self, laws: Arc<Laws>) -> Vec<(law, String)> {
        let res = self.vec_record.clone();
        let res: Vec<LawRecord> = res.iter().filter(|&x| x.chapter != "創建").map(|x| x.clone()).collect();
        let map = laws.categories(0);
        let mut new_vec = Vec::new();
        for r in res.iter() {
            let chapter = &r.chapter;
            let num = &r.num;
            if let Some(l) = map.get(chapter) {
                if let Some(law) = l.clone().lines.into_iter().find(|law| law.num == *num) {
                    new_vec.push((law,r.note.clone()));
                }
            }
        }
        new_vec
    }
}



