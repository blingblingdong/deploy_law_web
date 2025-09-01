#[allow(non_snake_case)]
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;
use std::error::Error;
use std::io::BufRead;

#[derive(Debug)]
pub enum LawError {
    NOThisChapter,
    CsvReadingError(Box<dyn Error>),
    SQLError(sqlx::Error),
}

#[derive(Deserialize, Serialize, Clone, Debug, sqlx::FromRow)]
pub struct Line {
    pub line_type: String,
    pub content: String,
}

/*

fn format_lines(node: select::node::Node) -> Vec<Line> {
    let mut vec = Vec::new();
    for line in node.find(Or(Class("line-0000"), Class("line-0004"))) {
        let content = line.text();
        let line_type;
        if line.is(Class("line-0000")) {
            line_type = "normal".to_string();
        } else {
            line_type = "indent".to_string();
        }
        vec.push(Line { line_type, content })
    }
    vec
}
*/

#[derive(Deserialize, Serialize, Clone, Debug, sqlx::FromRow)]
pub struct NewLaw {
    pub id: String,
    pub href: String,
    pub chapter: Vec<String>,
    pub num: String,
    pub lines: Vec<Line>,
}

impl NewLaw {
    pub async fn add_to_pool(&self, pool: &PgPool) {
        let json_lines = serde_json::to_value(&self.lines).unwrap();
        match sqlx::query(
            "INSERT INTO newlaw (id, num, lines, href, chapter)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (id) DO UPDATE
             SET num = EXCLUDED.num,
                 lines = EXCLUDED.lines,
                 href = EXCLUDED.href,
                 chapter = EXCLUDED.chapter",
        )
        .bind(&self.id)
        .bind(&self.num)
        .bind(&json_lines)
        .bind(&self.href)
        .bind(&self.chapter)
        .execute(pool)
        .await
        {
            Ok(_) => println!("✅ Upserted: {}", self.id),
            Err(e) => eprintln!("❌ Failed [{}]: {}", self.id, e),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NewLaws {
    pub lines: Vec<NewLaw>,
}

#[derive(Debug, serde::Deserialize, Clone, Serialize)]
pub struct LawList {
    pub chapter: Vec<String>,
    pub laws: Vec<NewLaw>,
}

#[derive(Debug, serde::Deserialize, Clone, Serialize)]
pub struct ChapterUl {
    pub chapter: String,
    pub level: usize,
    #[allow(non_snake_case)]
    pub childChapters: Vec<ChapterUl>,
}

impl NewLaws {
    pub fn new() -> Self {
        NewLaws { lines: Vec::new() }
    }

    pub fn count_chapter(&self) -> usize {
        let mut number: Vec<usize> = Vec::new();
        for law in self.lines.clone() {
            let count = law.chapter.iter().len();
            number.push(count);
        }
        *number.iter().max().unwrap()
    }

    pub fn categories(&self, index: usize) -> IndexMap<String, NewLaws> {
        let mut map = IndexMap::new();
        for law in &self.lines {
            if law.chapter.len() > index {
                let name = law.chapter.get(index).unwrap().to_string();
                map.entry(name)
                    .or_insert_with(NewLaws::new)
                    .lines
                    .push(law.clone());
            }
        }
        map
    }

    #[allow(non_snake_case)]
    pub fn get_chapterUlList(&self) -> Result<Vec<ChapterUl>, LawError> {
        let x = self.chapter_ul_list_create(1);
        Ok(x)
    }

    pub fn chapter_ul_list_create(&self, level: usize) -> Vec<ChapterUl> {
        let mut list = Vec::new();
        let map = self.categories(level); // 假設這回傳一個 IndexMap
        for (chapter_name, laws) in &map {
            let mut chapter_ul = ChapterUl {
                chapter: chapter_name.clone(),
                level,
                childChapters: Vec::new(),
            };
            let max = laws.count_chapter();
            println!("{max}");
            if level < max - 1 {
                // 遞迴建立子章節列表
                chapter_ul.childChapters = laws.chapter_ul_list_create(level + 1);
            }
            list.push(chapter_ul);
        }
        list
    }

    #[allow(non_snake_case)]
    pub fn lawList_by_chapter(
        &self,
        chapter1: String,
        chapter2: String,
    ) -> Result<Vec<LawList>, LawError> {
        let mut list: Vec<LawList> = Vec::new();
        let mut buffer: Vec<String> = Vec::new();
        let mut max_level: usize;
        let num = chapter2.split("/").count();
        match self.find_by_chapter(chapter1, chapter2) {
            Ok(laws) => {
                max_level = laws.count_chapter() - 1;
                laws.lawList_push(num, &mut list, &mut buffer);
                Ok(list)
            }
            Err(e) => Err(e),
        }
    }

    #[allow(non_snake_case)]
    pub fn lawList_create(&self) -> Result<Vec<LawList>, LawError> {
        let mut list: Vec<LawList> = Vec::new();
        let mut buffer: Vec<String> = Vec::new();
        self.lawList_push(0, &mut list, &mut buffer);
        Ok(list)
    }

    #[allow(non_snake_case)]
    pub fn lawList_push(&self, level: usize, list: &mut Vec<LawList>, buffer: &mut Vec<String>) {
        let map1 = self.categories(level);
        map1.iter().for_each(|(name, laws)| {
            if laws.count_chapter() - 1 > level {
                buffer.push(name.clone());
                laws.lawList_push(level + 1, list, buffer);
            } else {
                let mut chapter = Vec::new();
                if buffer.len() > 0 {
                    buffer.iter().for_each(|s| chapter.push(s.clone()));
                    buffer.clear();
                }
                chapter.push(name.clone());
                list.push(LawList {
                    chapter: chapter,
                    laws: laws.lines.clone(),
                })
            }
        })
    }

    pub fn find_by_chapter(&self, chapter1: String, chapter2: String) -> Result<NewLaws, LawError> {
        let tp = format!("{chapter1}/{chapter2}");
        match self.categories(0).get(&chapter1) {
            Some(laws) => {
                let mut l: Vec<NewLaw> = Vec::new();
                let _ = laws
                    .lines
                    .iter()
                    .filter(|&law| law.chapter.contains(&tp))
                    .for_each(|law| l.push(law.clone()));
                Ok(NewLaws { lines: l })
            }
            _ => Err(LawError::NOThisChapter),
        }
    }

    pub async fn from_pool(db_url: &str) -> Result<Self, sqlx::Error> {
        let mut attempts = 0;

        let db_pool = match PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await
        {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!(
                    "Failed to connect to the database: {}. Retry in 5 seconds...",
                    e
                );
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                attempts += 1;
                return Err(sqlx::Error::WorkerCrashed);
            }
        };
        match sqlx::query(
            "SELECT * FROM newlaw
        ORDER BY created_at ASC;",
        )
        .map(|row: PgRow| {
            let lines_json: serde_json::Value = row.get("lines");
            let lines: Vec<Line> = serde_json::from_value(lines_json).unwrap();
            NewLaw {
                id: row.get("id"),
                num: row.get("num"),
                lines,
                href: row.get("href"),
                chapter: row.get("chapter"),
            }
        })
        .fetch_all(&db_pool)
        .await
        {
            Ok(lines) => Ok(NewLaws { lines }),
            Err(_e) => Err(sqlx::Error::WorkerCrashed),
        }
    }
}
