#[allow(unused_imports)]
use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};
use anyhow::Result;
use indexmap::{IndexMap};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPoolOptions, PgPool, PgRow};
use sqlx::{Row};

#[derive(Clone)]
pub struct Laws {
    pub lines: Vec<crate::law>
}

impl crate::Laws {
    pub fn new() -> Self {
        crate::Laws {
            lines: Vec::new()
        }
    }

    pub fn from_csv(path:String) -> Self{
        let mut vec = Vec::new();
        let file = File::open(path).expect("打開不能");
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            // Notice that we need to provide a type hint for automatic
            // deserialization.
            let record: crate::law = result.unwrap();
            vec.push(record);
        }

        crate::Laws {
            lines:vec
        }

    }

    pub fn filter_by_cate(&self, cate: String, num: String) -> String {
        let map = self.categories(0);
        if let Some(laws) = map.get(&cate) {
            let x: Vec<law>= laws.clone().lines.into_iter().filter(|x| x.num == num).collect();
            if !x.is_empty() {
                let name = &x.get(0).unwrap().num;
                let chapter:Vec<&str> = x.get(0).unwrap().chapter.split("/").collect();
                let l: String = x.get(0).unwrap().line.iter().enumerate()
                    .map(|(i, s)| format!("{}:{s}\n",i+1)).collect();
                format!("{}：{name}\n{l}", chapter.first().unwrap())
            } else {
                format!("查無項目，考慮下列類似選項")
            }
        } else {
            format!("沒有這個項目，考慮下列類似選項")
        }
    }

    pub fn filter_by_cate2(&self, cate: String, num: String) -> String {
        let map = self.categories(0);
        if let Some(laws) = map.get(&cate) {
            let x: Vec<law>= laws.clone().lines.into_iter().filter(|x| x.num == num).collect();
            if !x.is_empty() {
                let name = &x.get(0).unwrap().num;
                let chapter:Vec<&str> = x.get(0).unwrap().chapter.split("/").collect();
                let l: String = x.get(0).unwrap().line.iter().enumerate()
                    .map(|(i, s)| format!("<h3>{}:{s}</h3><br>",i+1)).collect();
                format!("<h2>{}第{name}條：</h2><br>{l}", chapter.first().unwrap())
            } else {
                format!("<h2>查無項目，考慮下列類似選項</h2>")
            }
        } else {
            format!("<h2>沒有這個項目，考慮下列類似選項</h2>")
        }
    }

    pub fn categories(&self, index:usize) -> IndexMap<String, crate::Laws> {
        let mut map = IndexMap::new();
        for law in &self.lines {
            let name_vec = law.chapter.split('/').collect::<Vec<&str>>();
            if name_vec.len() > index {
                let name = name_vec.get(index).unwrap().to_string();
                map.entry(name).or_insert_with(crate::Laws::new).lines.push(law.clone());
            }
        }
        map
    }

    pub fn search_in_html_chapter(&self, chapter: String) -> String {
        let binding = self.categories(0);
        let l = binding.get(&chapter).unwrap();
        let chapter_num = self.lines.first().unwrap().chapter.split("/").count();
        let mut html_text = String::new();
        l.print_all_chapter_html(1, chapter_num, &mut html_text);
        html_text
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

    pub fn chapter_lines_in_html(&self, chapter1:String, num: String, chapter2: String) ->  String{
        let binding = self.categories(0);
        println!("{chapter1}");
        let l = binding.get(&chapter1).expect("找無");
        let mut html_text = String::new();
        let binding2 = l.categories(num.parse().unwrap());
        let l2 = binding2.get(&chapter2).unwrap();
        let chapter_num = l2.lines.first().unwrap().chapter.split("/").count();
        l2.print_all_html(num.parse().unwrap(), chapter_num, &mut html_text);
        html_text
    }


    pub fn all_in_html(&self, chapter:String) -> String {
        let binding = self.categories(0);
        let l = binding.get(&chapter).unwrap();
        let chapter_num = self.lines.first().unwrap().chapter.split("/").count();
        let mut html_text = String::new();
        l.print_all_html(0, chapter_num, &mut html_text);
        html_text
    }

    pub fn print_all_html(&self, level: usize, max_level: usize, html_text: &mut String) {
        if level == max_level {
            for l in &self.lines {
                html_text.push_str(&l.law_block());
            }
        }else {
            let map = self.categories(level);
            for (s, l) in map {
                let s = format!("<div 'in-chapter'><h2>{}</h2></div>", s);
                html_text.push_str(&s);
                l.print_all_html(level + 1, max_level, html_text);
            }
        }
    }

    pub fn table_of_cate(&self) {
        let chapter_num = self.lines.first().unwrap().chapter.split("/").count();
        self.print_categories(0, chapter_num);
    }

    fn print_categories(&self, level: usize, max_level: usize) {
        if level == max_level {
            let _chapter = self.lines.first().unwrap().chapter.split("/").last().unwrap();
            for l in &self.lines {
                println!("{}", l.num);
            }
        }else {
            let map = self.categories(level);
            for (s, l) in map {
                print!("{}", "--".repeat(level));
                println!("{s}");
                l.print_categories(level + 1, max_level);
            }
        }
    }

    pub async fn from_pool(db_url: &str) -> Result<Self, sqlx::Error> {
        let mut attempts = 0;
        let max_attempts = 5;

        let db_pool = match PgPoolOptions::new()
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
            .map(|row: PgRow| law{
                id: row.get("id"),
                num: row.get("num"),
                line: row.get("line"),
                href: row.get("href"),
                chapter: row.get("chapter")
            })
            .fetch_all(&db_pool)
            .await{
                Ok(lines) => Ok(Laws{lines}),
                Err(_e) => Err(sqlx::Error::WorkerCrashed)
        }
    }


    pub fn view(&self) {
        println!("本章節總共有：{}", self.lines.len());
        println!("first element is:{:?}", self.lines.first());
    }

}

pub fn group(map:IndexMap<String, Laws>) -> Laws {
    let key = map.iter().last().unwrap();
    let mut num : usize;
    for (i, law) in key.1.lines.first().unwrap().chapter.split("/").enumerate() {
        if key.0 == law {
            num = i;
            println!("group by {}{}", num, law)
        }
    }
    let mut l = Laws::new();
    for x in map.into_iter(){
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
    Ok(s.split('/').map(|s| s.to_string()).collect())
}


impl crate::law {
    pub fn new(num: String, line:Vec<String> , href: String, chapter: String) -> Self {
        let vec: Vec<_> = chapter.split("/").collect();
        let id = format!("{}-{}",vec.first().unwrap().to_string(), num);
        crate::law {
            id,
            num,
            line,
            href,
            chapter
        }
    }

    pub fn format_chapter(&self) -> String {
        let chapter: Vec<&str> = self.chapter.split("/").collect();
        let c = chapter.first().unwrap();
        format!("{}第{}條", c, self.num)
    }

    pub fn law_block(&self) -> String {
        let mut s = String::new();
        s.push_str("<div class='container'><div class='box1'>");
        s.push_str("<div class='law-content'>");
        let chapter = format!("<div class='law-chapter'>{}</div>", self.format_chapter());
        s.push_str(&chapter);
        let line: String = self.line.iter().enumerate().filter(|(_, s)| !s.is_empty())
            .map(|(i, s)| format!("<div class='law-line'>{}:{s}</div>",i+1)).collect();
        let lines = format!("<div class='law-lines'>{}</div>", line);
        s.push_str(&lines);
        s.push_str("</div></div>");
        let add_but = format!("<div class='box3'><button class='add-law' id='add-{}'>新增至</button></div>", self.id);
        s.push_str(&add_but);
        s.push_str("</div>");
        s
    }

    pub fn law_block_result(&self) -> String {
        let mut s = String::new();
        s.push_str("<div class='box1'>");
        s.push_str("<div class='law-content'>");
        let chapter = format!("<div class='law-chapter'>{}</div>", self.format_chapter());
        s.push_str(&chapter);
        let line: String = self.line.iter().enumerate().filter(|(_, s)| !s.is_empty())
            .map(|(i, s)| format!("<div class='law-line'>{}:{s}</div>",i+1)).collect();
        let lines = format!("<div class='law-lines'>{}</div>", line);
        s.push_str(&lines);
        s.push_str("</div></div>");
        let add_but = format!("<div class='box3'><button class='add-law' id='add-{}'>新增至</button></div>", self.id);
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
        let line: String = self.line.iter().enumerate().filter(|(_, s)| !s.is_empty())
            .map(|(i, s)| format!("<div class='card-law-line'>{}:{s}</div>",i+1)).collect();
        s.push_str("<div class='card-law-note' id='card-law-note-{}' style='display: none;'>筆記</div>");
        let lines = format!("<div class='card-law-lines' id='card-law-lines-{}'>{}</div>",self.id, line);
        s.push_str(&lines);
        s.push_str("</div>");
        let delete_but = format!("<div class='card-tools'><button class='delete-law' id='delete-{}'></button><button class='toggle-note-law' id='toggle-note-{}'></button></div>", self.id, self.id);
        s.push_str(&delete_but);
        s.push_str("</div>");
        let note = format!("<div class='card-law-note' id='card-law-note-{}' style='display: none;'>", self.id);
        s.push_str(note.as_str());
        s.push_str("<div class='note-title'>筆記</div>");
        let note2 = format!("<div class='law-note-area' id='law-note-area-{}'>{}</div>", self.id, notepoo);
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



    pub async fn add_to_pool(&self, pool: &PgPool) {
        match sqlx::query(
            "INSERT INTO law (id, num, line, href, chapter) VALUES ($1, $2, $3, $4, $5)"
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

pub async fn new_pool() -> PgPool {
    let db_pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect("").await {
        Ok(pool) => pool,
        Err(e) => panic!("sss {}", e),
    };
    db_pool
}

