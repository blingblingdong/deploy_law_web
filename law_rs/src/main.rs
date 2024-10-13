#[allow(unused_imports)]
use std::error::Error;
use csv::Writer;
use serde::Deserialize;
use law_rs::{law, new_pool, Laws};

#[allow(non_camel_case_types)]
#[derive(Debug, serde::Deserialize, Clone)]
pub struct New_Law {
    pub id: String,
    pub num: String,
    #[serde(deserialize_with = "deserialize_line")]
    pub line: Vec<String>,
    pub href: String,
    pub chapter: String,
}

fn deserialize_line<'de, D>(deserializer: D) -> anyhow::Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.split('/').map(|s| s.to_string()).collect())
}

impl New_Law {
    pub fn new(l: law) -> Self {
        let vec: Vec<_> = l.chapter.split("/").collect();
        let id = format!("{}-{}",vec.first().unwrap().to_string(), l.num);
        New_Law {
            id,
            num: l.num,
            line: l.line,
            href: l.href,
            chapter: l.chapter
        }
    }
}

pub fn new_write_law(path: String, vec: Vec<New_Law>) -> anyhow::Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;
    wtr.write_record(&["id", "num", "line", "href", "chapter"])?;

    for law in vec {
        wtr.write_record(&[law.id, law.num, law.line.join("/"), law.href, law.chapter])?;
    }
    println!("寫入成功");
    wtr.flush()?;
    Ok(())
}

#[tokio::main]
async fn main()  {
    let db_url = "postgres://dbuser:12345678@localhost:5432/law";
    let store = Laws::from_pool(&db_url).await.unwrap();
    let map = store.categories(0);
    let x = map.keys().filter(|k| *k!= "").count();
    println!("{x}");
}
