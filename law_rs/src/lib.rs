use anyhow::Result;
use csv::{Reader, Writer};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;
use std::alloc::Layout;
use std::collections::HashSet;
#[allow(unused_imports)]
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::fs::File;
// use sqlx::sqlite::{SqlitePool, SqlitePoolOptions,SqliteRow};

#[derive(Debug)]
pub enum LawError {
    NOThisChapter,
    CsvReadingError(Box<dyn Error>),
    SQLError(sqlx::Error),
}

impl fmt::Display for LawError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LawError::NOThisChapter => write!(f, "Chapter not found"),
            LawError::CsvReadingError(ref err) => write!(f, "CSV reading error: {}", err),
            LawError::SQLError(ref err) => write!(f, "SQL error: {}", err),
        }
    }
}

impl Error for LawError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            LawError::NOThisChapter => None,
            LawError::CsvReadingError(ref err) => Some(err.as_ref()),
            LawError::SQLError(ref err) => Some(err),
        }
    }
}

#[derive(Clone)]
pub struct Laws {
    pub lines: Vec<crate::law>,
}

impl crate::Laws {
    pub fn new() -> Self {
        crate::Laws { lines: Vec::new() }
    }

    /*
    pub async fn from_lite_pool(db_url: &str) -> Result<Self, sqlx::Error> {
        let mut attempts = 0;
        let max_attempts = 5;

        let db_pool = match SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url).await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Failed to connect to the database: {}. Retry in 5 seconds...", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                attempts += 1;
                return Err(sqlx::Error::WorkerCrashed)
            },
        };
        match sqlx::query("SELECT * FROM law
        ORDER BY created_at ASC;")
            .map(|row: SqliteRow|       {
                let line: String = row.get("line");
                let lines = line.split('/').map(String::from).collect::<Vec<String>>();
                law {
                    id: row.get("id"),
                    num: row.get("num"),
                    line: lines,
                    href: row.get("href"),
                    chapter: row.get("chapter")
                }
            })
            .fetch_all(&db_pool)
            .await {
            Ok(lines) => Ok(Laws { lines }),
            Err(_e) => Err(sqlx::Error::WorkerCrashed)
        }
    }
    **/
    /*
        pub async fn populate_sqlite_with_laws(&self, sqlite_path: &str) -> Result<(), sqlx::Error> {
            let pool = SqlitePool::connect(sqlite_path).await?;
            for law in &self.lines {
                sqlx::query("INSERT INTO law (id, num, line, href, chapter) VALUES (?, ?, ?, ?, ?)")
                    .bind(&law.id)
                    .bind(&law.num)
                    .bind(&law.line.join("/")) // 假設 line 是 Vec<String>
                    .bind(&law.href)
                    .bind(&law.chapter)
                    .execute(&pool)
                    .await?;
            }
            Ok(())
        }
    */

    pub fn from_csv(path: String) -> Result<Laws, LawError> {
        let mut vec = Vec::new();
        let file = File::open(path).map_err(|e| LawError::CsvReadingError(e.into()))?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: crate::law = result.map_err(|e| LawError::CsvReadingError(e.into()))?;
            vec.push(record);
        }

        let laws = crate::Laws { lines: vec };
        Ok(laws)
    }

    pub fn find_by_text(&self, chapter: String, text: String) -> Result<Self, LawError> {
        match self.categories(0).get(&chapter) {
            Some(laws) => {
                let mut l = Laws::new();
                for law in laws.lines.clone() {
                    law.line
                        .iter()
                        .filter(|law| law.contains(&text))
                        .for_each(|x| l.lines.push(law.clone()));
                }
                Ok(l)
            }
            _ => Err(LawError::NOThisChapter),
        }
    }

    // 用來數總共分為幾個章節
    pub fn count_chapter(&self) -> usize {
        let mut number: Vec<usize> = Vec::new();
        for law in self.lines.clone() {
            let count = law.chapter.split("/").count();
            number.push(count);
        }
        *number.iter().max().unwrap()
    }

    pub fn categories(&self, index: usize) -> IndexMap<String, crate::Laws> {
        let mut map = IndexMap::new();
        for law in &self.lines {
            let name_vec = law.chapter.split('/').collect::<Vec<&str>>();
            if name_vec.len() > index {
                let name = name_vec.get(index).unwrap().to_string();
                map.entry(name)
                    .or_insert_with(crate::Laws::new)
                    .lines
                    .push(law.clone());
            }
        }
        map
    }

    // 打印出html格式的章節
    pub fn search_in_html_chapter(&self, chapter: String) -> Result<String, LawError> {
        let binding = self.categories(0);
        let l = binding.get(&chapter).ok_or(LawError::NOThisChapter)?;
        let chapter_num = self.count_chapter();
        let mut html_text = String::new();
        l.print_all_chapter_html(1, chapter_num, &mut html_text);
        Ok(html_text)
    }

    // 打印出html格式的章節選擇
    pub fn chapter_inputs_html(&self, father: String, level: usize, buffer: &mut String) {
        let map = self.categories(level);
        for (name, laws) in &map {
            let max = laws.count_chapter();
            if level == 1 {
                println!("{name}");
                let s = format!("<option value='{}'>", name);
                buffer.push_str(&s);
                if max > level + 1 {
                    laws.chapter_inputs_html(name.clone(), level + 1, buffer);
                }
            } else {
                let father_and_child = format!("{father}/{name}");
                println!("{father_and_child}");
                let s = format!("<option value='{}'>", father_and_child);
                buffer.push_str(&s);
                if max > level + 1 {
                    laws.chapter_inputs_html(father_and_child, level + 1, buffer);
                }
            }
        }
    }

    pub fn print_all_chapter_html(&self, level: usize, max_level: usize, html_text: &mut String) {
        let map = self.categories(level);

        for (s, l) in &map {
            // 只在 level 為 1 的時候加入外層 <ul>
            if level == 1 {
                html_text.push_str(&format!("<ul class='chapter-ul-{}'>", level));
            }

            // <li> 標籤
            html_text.push_str(&format!("<li class='chapter-li-{}'><a>{}</a>", level, s));

            // 只有在還有子項時才遞歸繼續產生 <ul> 結構
            if level < max_level - 1 {
                html_text.push_str(&format!("<ul class='chapter-ul-{}'>", level + 1));
                l.print_all_chapter_html(level + 1, max_level, html_text);
                html_text.push_str("</ul>");
            }

            html_text.push_str("</li>"); // 關閉 <li>

            // 在 level == 1 時關閉外層 <ul>
            if level == 1 {
                html_text.push_str("</ul>");
            }
        }
    }

    pub fn chapter_lines_in_html(
        &self,
        chapter1: String,
        chapter2: String,
    ) -> Result<String, LawError> {
        let mut html_text = String::new();
        let mut max_level: usize;
        let num = chapter2.split("/").count();
        match self.find_by_chapter(chapter1, chapter2) {
            Ok(laws) => {
                max_level = laws.count_chapter() - 1;
                laws.print_all_html(num, &mut html_text);
                Ok(html_text)
            }
            Err(e) => Err(e),
        }
    }

    pub fn all_in_html(&self, chapter: String) -> Result<String, LawError> {
        let binding = self.categories(0);
        let l = binding.get(&chapter).ok_or(LawError::NOThisChapter)?;
        let chapter_num = l.count_chapter();
        let mut html_text = String::new();
        l.print_all_html(0, &mut html_text);
        Ok(html_text)
    }

    pub fn print_all_html(&self, level: usize, html_text: &mut String) {
        let map1 = self.categories(level);
        map1.iter().for_each(|(name, laws)| {
            if laws.count_chapter() - 1 > level {
                let chapter = format!("<div class='in-chapter'><h3>{}</h3></div>", name);
                html_text.push_str(&chapter);
                laws.print_all_html(level + 1, html_text);
            } else {
                let chapter = format!("<div class='in-chapter'><h3>{}</h3></div>", name);
                html_text.push_str(&chapter);
                laws.lines
                    .iter()
                    .map(|law| law.law_block().clone())
                    .for_each(|law_block| html_text.push_str(&law_block))
            }
        })
    }

    pub fn find_by_chapter(&self, chapter1: String, chapter2: String) -> Result<Laws, LawError> {
        let tp = format!("{chapter1}/{chapter2}");
        match self.categories(0).get(&chapter1) {
            Some(laws) => {
                let mut l = Laws::new();
                let laws = laws
                    .lines
                    .iter()
                    .filter(|&law| law.chapter.contains(&tp))
                    .for_each(|law| l.lines.push(law.clone()));
                Ok(l)
            }
            _ => Err(LawError::NOThisChapter),
        }
    }

    pub async fn from_pool(db_url: &str) -> Result<Self, sqlx::Error> {
        let mut attempts = 0;
        let max_attempts = 5;

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
            "SELECT * FROM law
        ORDER BY created_at ASC;",
        )
        .map(|row: PgRow| law {
            id: row.get("id"),
            num: row.get("num"),
            line: row.get("line"),
            href: row.get("href"),
            chapter: row.get("chapter"),
        })
        .fetch_all(&db_pool)
        .await
        {
            Ok(lines) => Ok(Laws { lines }),
            Err(_e) => Err(sqlx::Error::WorkerCrashed),
        }
    }

    pub fn view(&self) {
        println!("本章節總共有：{}", self.lines.len());
        println!("first element is:{:?}", self.lines.first());
    }

    pub async fn update_line(&self, pool: &PgPool) -> std::result::Result<(), sqlx::Error> {
        let futures = self
            .lines
            .iter()
            .map(|law| {
                sqlx::query("UPDATE law SET line = $1 WHERE id = $2")
                    .bind(&law.line)
                    .bind(&law.id)
                    .execute(pool)
            })
            .collect::<Vec<_>>();

        for result in futures {
            match result.await {
                Ok(_) => println!("成功更新"),
                Err(e) => eprintln!("更新失敗: {e}"),
            }
        }

        Ok(())
    }
}

pub fn group(map: IndexMap<String, Laws>) -> Laws {
    let key = map.iter().last().unwrap();
    let mut num: usize;
    for (i, law) in key.1.lines.first().unwrap().chapter.split("/").enumerate() {
        if key.0 == law {
            num = i;
            println!("group by {}{}", num, law)
        }
    }
    let mut l = Laws::new();
    for x in map.into_iter() {
        l.lines.extend(x.1.lines);
    }

    l
}

#[allow(non_camel_case_types)]
#[derive(Debug, serde::Deserialize, Clone, Serialize)]
pub struct law {
    pub id: String,
    pub num: String,
    #[serde(deserialize_with = "deserialize_line")]
    pub line: Vec<String>,
    pub href: String,
    pub chapter: String,
}

fn deserialize_line<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.split(|c| c == '/').map(|s| s.to_string()).collect())
}

impl crate::law {
    pub fn new(num: String, line: Vec<String>, href: String, chapter: String) -> Self {
        let vec: Vec<_> = chapter.split("/").collect();
        let id = format!("{}-{}", vec.first().unwrap().to_string(), num);
        crate::law {
            id,
            num,
            line,
            href,
            chapter,
        }
    }

    pub fn format_chapter(&self) -> String {
        let chapter: Vec<&str> = self.chapter.split("/").collect();
        let c = chapter.first().unwrap();
        format!("{}第{}條", c, self.num)
    }

    fn indent(c: char) -> bool {
        let mut set = HashSet::new();
        set.extend([
            '一', '二', '三', '四', '五', '六', '七', '八', '九', '十', '第',
        ]);
        set.contains(&c)
    }

    pub fn law_block(&self) -> String {
        let chapter: Vec<&str> = self.chapter.split("/").collect();
        let c = chapter.first().unwrap();
        let mut s = String::new();
        let mut s2 = String::new();
        self.line.iter().for_each(|x| {
            s2.push_str(x);
            s2.push_str("/");
        });
        let line: String = s2
            .split(|c| c == '：' || c == '/')
            .filter(|(s)| !s.is_empty())
            .map(|(s)| {
                if s.starts_with(Self::indent) {
                    format!("<div class='law-indent'>{s}</div>")
                } else {
                    format!("<li class='law-block-line'>{s}</li>")
                }
            })
            .collect();
        let lines = format!("<ul class='law-block-lines'>{}</div>", line);
        let r = format!("<div class='law-content-area'>
                <div class='top-search-law-title' style='display: flex'>
                    <p>第{}條<br>章節：{}</p>
                    <div><div class='top-law_search-add-area'><button class='add-law normal-button' id='add-{}-{}'>新增至</button></div></div>
                </div>
                {}
            </div>",self.num, self.chapter,c, self.num, lines);
        s.push_str(&r);
        s
    }

    pub fn law_block_result(&self) -> String {
        let mut s = String::new();
        s.push_str("<div class='box1'>");
        s.push_str("<div class='law-content'>");
        let chapter = format!("<div class='law-chapter'>{}</div>", self.format_chapter());
        s.push_str(&chapter);
        let line: String = self
            .line
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.is_empty())
            .map(|(i, s)| format!("<div class='law-line'>{}:{s}</div>", i + 1))
            .collect();
        let lines = format!("<div class='law-lines'>{}</div>", line);
        s.push_str(&lines);
        s.push_str("</div></div>");
        let add_but = format!(
            "<div class='box3'><button class='add-law' id='add-{}'>新增至</button></div>",
            self.id
        );
        s.push_str(&add_but);
        s
    }

    pub fn law_block_delete(&self, notepoo: String) -> String {
        let mut s = String::new();
        let chapter: Vec<&str> = self.chapter.split("/").collect();
        let c = chapter.first().unwrap();
        s.push_str("<div class='law-card'>");
        s.push_str("<div class='law-card-up'>");
        s.push_str("<div class='card-law-content'>");
        let chapter = format!("<div class='card-law-chapter'><div class='title'>{}</div><div class='num'>第{}條</div></div>", c, self.num);
        s.push_str(&chapter);
        let line: String = self
            .line
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.is_empty())
            .map(|(i, s)| format!("<div class='card-law-line'>{}:{s}</div>", i + 1))
            .collect();
        s.push_str(
            "<div class='card-law-note' id='card-law-note-{}' style='display: none;'>筆記</div>",
        );
        let lines = format!(
            "<div class='card-law-lines' id='card-law-lines-{}'>{}</div>",
            self.id, line
        );
        s.push_str(&lines);
        s.push_str("</div>");
        let delete_but = format!("<div class='card-tools'><button class='delete-law' id='delete-{}'></button><button class='toggle-note-law' id='toggle-note-{}'></button></div>", self.id, self.id);
        s.push_str(&delete_but);
        s.push_str("</div>");
        let note = format!(
            "<div class='card-law-note' id='card-law-note-{}' style='display: none;'>",
            self.id
        );
        s.push_str(note.as_str());
        s.push_str("<div class='note-title'>筆記</div>");
        let note2 = format!(
            "<div class='law-note-area' id='law-note-area-{}'>{}</div>",
            self.id, notepoo
        );
        s.push_str(note2.as_str());
        let note_but = format!("<div class='note-tools'><button class='note-edit-btn' id='note-edit-btn-{}'></button><button class='note-hide-btn' id='note-hide-btn-{}'></button></div>", self.id, self.id);
        s.push_str(note_but.as_str());
        s.push_str("</div>");
        s.push_str("</div>");
        s
    }

    pub fn update_chapter(&mut self, chapter: String) {
        self.chapter = chapter;
    }

    pub fn upadate_line(&mut self, vec: Vec<String>) {
        self.line = vec;
    }

    /*
        pub async fn add_to_lite_pool(&self, pool: &SqlitePool) {
            match sqlx::query(
                "INSERT INTO law (id, num, line, href, chapter) VALUES ($1, $2, $3, $4, $5)"
            )
                .bind(self.id.clone())
                .bind(self.num.clone())
                .bind(self.line.join("/").clone())
                .bind(self.href.clone())
                .bind(self.chapter.clone())
                .execute(pool)
                .await
            {
                Ok(_) => println!("Insert successful"),
                Err(e) => eprintln!("Insert failed: {}", e),
            }
        }
    */

    pub async fn add_to_pool(&self, pool: &PgPool) {
        match sqlx::query(
            "INSERT INTO law (id, num, line, href, chapter) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(self.id.clone())
        .bind(self.num.clone())
        .bind(self.line.clone())
        .bind(self.href.clone())
        .bind(self.chapter.clone())
        .execute(pool)
        .await
        {
            Ok(_) => println!("Insert successful"),
            Err(e) => eprintln!("Insert failed: {}", e),
        }
    }
}

pub fn write_law(path: String, vec: Vec<crate::law>) -> std::result::Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;
    wtr.write_record(&["id", "num", "line", "href", "chapter"])?;

    for law in vec {
        wtr.write_record(&[law.id, law.num, law.line.join("/"), law.href, law.chapter])?;
    }
    println!("寫入成功");
    wtr.flush()?;
    Ok(())
}

pub async fn new_pool(url: &str) -> PgPool {
    let db_pool = match PgPoolOptions::new().max_connections(10).connect(url).await {
        Ok(pool) => pool,
        Err(e) => panic!("sss {}", e),
    };
    db_pool
}

/*
pub async fn new_lite_pool(url: &str) -> SqlitePool {
    let db_pool = match SqlitePoolOptions::new()
        .max_connections(5)
        .connect(url).await {
        Ok(pool) => pool,
        Err(e) => panic!("sss {}", e),
    };
    db_pool
}
*/
