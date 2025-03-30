#[allow(unused_imports)]
use std::error::Error;
use csv::Writer;
use serde::Deserialize;
use law_rs::{law, Laws};


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
    let law = Laws::from_pool("postgresql://postgres:IoNTPUpeBHZMjpfpbdHDfIKzzbSQCIEm@autorack.proxy.rlwy.net:10488/railway")
        .await.unwrap();
    let x = law.lawList_create("民法".to_string()).unwrap();
    println!("{:?}", x);

}
