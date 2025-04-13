use reqwest::get;
use select::document::Document;
use select::predicate::{And, Attr, Class, Or};
use select::predicate::{Name, Predicate};
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewInterpretation {
    pub id: String,
    pub no: String,
    pub name: String,
    pub date: String,
    pub reason: Option<String>,
    pub content: Option<String>,
    pub related_law: Option<String>,
    pub source: String,
    pub year: i16,
    pub number: i16,
}

impl NewInterpretation {
    pub async fn add_to_pool(self, pool: &PgPool) {
        let uuid = Uuid::new_v4().to_string();

        match sqlx::query(
            "INSERT INTO newinterpretations (id, no, name, date, reason, content, related_law, source, year, number)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(uuid)
        .bind(self.no)
        .bind(self.name)
        .bind(self.date)
        .bind(self.reason)
        .bind(self.content)
        .bind(self.related_law)
        .bind(self.source)
        .bind(self.year)
        .bind(self.number)
        .execute(pool)
        .await
        {
            Ok(_) => println!("Insert successful"),
            Err(e) => eprintln!("Insert failed: {}", e),
        }
    }
}

pub async fn scrapeNewInterpretation(num: usize, href: String) -> NewInterpretation {
    let html = get(href.clone()).await.unwrap().text().await.unwrap();
    let doc = Document::from(html.as_str());
    let mut inter = NewInterpretation {
        id: num.to_string(),
        no: "".to_string(),
        name: "".to_string(),
        date: "".to_string(),
        related_law: None,
        reason: None,
        content: None,
        source: href,
        year: 0,
        number: 0,
    };

    for pre in doc.find(Name("pre")) {
        if (pre.text().starts_with("憲法法庭判決")) {
            inter.content = Some(pre.text());
        }
    }

    for tr in doc.find(Name("tr")) {
        if let Some(th) = tr.find(Name("th")).next() {
            if th.text() == "裁判字號：" {
                let res = tr.find(Name("td")).next().unwrap().text();
                let re = Regex::new(r"憲法法庭 (?P<year>\d{2,4})年憲判字第 (?P<number>\d{1,4}) 號")
                    .unwrap();
                let caps = re.captures(res.trim()).unwrap();
                let year = caps.name("year").unwrap().as_str().parse().ok().unwrap();
                let number = caps.name("number").unwrap().as_str().parse().ok().unwrap();

                let no = format!("{}年憲判字第{}號", year, number);
                inter.year = year;
                inter.number = number;
                inter.no = no;
            } else if th.text() == "案　　名：" {
                let date = tr.find(Name("td")).next().unwrap().text();
                inter.name = date;
            } else if th.text() == "裁判日期：" {
                let date = tr.find(Name("td")).next().unwrap().text();
                inter.date = date;
            } else if th.text() == "相關法條：" {
                let date = tr.find(Name("td")).next().unwrap().text();
                inter.related_law = Some(date);
            } else if th.text() == "案　　由：" {
                let date = tr.find(Name("td")).next().unwrap().text();
                inter.reason = Some(date);
            }
        }
    }
    println!("{:#?}", inter);
    inter
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OldInterpretation {
    pub id: String,
    pub date: String,
    pub reasoning: Option<String>,
    pub content: Option<String>,
    pub trouble: Option<String>,
    pub related_law: Option<String>,
    pub source: String,
}

impl OldInterpretation {
    pub async fn add_to_pool(self, pool: &PgPool) {
        match sqlx::query(
            "INSERT INTO oldinterpretations (id, date, reasoning, content, trouble, related_law, source)
            VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(self.id)
        .bind(self.date)
        .bind(self.reasoning)
        .bind(self.content)
        .bind(self.trouble)
        .bind(self.related_law)
        .bind(self.source)
        .execute(pool)
        .await
        {
            Ok(_) => println!("Insert successful"),
            Err(e) => eprintln!("Insert failed: {}", e),
        }
    }
}

pub async fn scrapeOldInterpretation(num: String, trouble: String) -> OldInterpretation {
    let href = format!(
        "https://mojlaw.moj.gov.tw/LawContentExShow.aspx?id=D%2C{}&type=c&kw=",
        num.clone()
    );
    let html = get(href.clone()).await.unwrap().text().await.unwrap();
    let doc = Document::from(html.as_str());
    let mut inter = OldInterpretation {
        id: num.to_string(),
        trouble: Some(trouble),
        related_law: None,
        date: "".to_string(),
        reasoning: None,
        content: None,
        source: href.clone(),
    };
    for pre in doc.find(Name("pre")) {
        if (pre.text().starts_with("理 由 書：")) {
            inter.content = Some(pre.text());
        }
    }

    for tr in doc.find(Name("tr")) {
        if let Some(th) = tr.find(Name("th")).next() {
            if th.text() == "解釋文：" {
                let res = tr.find(Name("td")).next().unwrap().text();
                inter.reasoning = Some(res);
            } else if th.text() == "解釋日期：" {
                let date = tr.find(Name("td")).next().unwrap().text();
                inter.date = date;
            } else if th.text() == "相關法條：" {
                let date = tr.find(Name("td")).next().unwrap().text();
                inter.related_law = Some(date);
            }
        }
    }

    inter
}

use regex::Regex;

#[derive(Debug, Serialize, Deserialize)]
pub struct Precedent {
    pub id: String,
    pub name: String,    // 字號
    pub holding: String, // 要旨
    pub source: String,
    pub year: i16,
    pub num: i16,
    pub specific: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OtherSourceList {
    pub id: String,
    pub name: String,
    pub sourcetype: String,
}

pub async fn scrape_precedent(num: usize) -> Vec<Precedent> {
    let href = format!(
        "https://mojlaw.moj.gov.tw/LawResultList.aspx?id=&check=jtype&search=3&valid=3&star=&end=&number=&kw=&sort=&LawType=jtype&iPageSize=10&page={}",
        num.clone()
    );
    let html = get(href.clone()).await.unwrap().text().await.unwrap();
    let doc = Document::from(html.as_str());
    let mut buffer = Vec::new();

    for node in doc.find(Name("tr")) {
        let mut pre = Precedent {
            id: "".to_string(),
            name: "".to_string(),
            holding: "".to_string(),
            source: "".to_string(),
            year: 0,
            num: 0,
            specific: "".to_string(),
        };

        if let Some(a) = node.find(Name("a")).next() {
            let name = a.text();
            let href = a.attr("href").unwrap();
            let new_href = format!("https://mojlaw.moj.gov.tw/{}", href);
            let re =
                Regex::new(r"(?P<year>\d{2,4})年(?P<word>.+?)字第(?P<number>\d{1,4})號").unwrap();
            let caps = re.captures(&name).unwrap();
            let year = caps.name("year").unwrap().as_str().parse().ok().unwrap();
            let word = caps.name("word").unwrap().as_str().to_string();
            let number = caps.name("number").unwrap().as_str().parse().ok().unwrap();
            pre.specific = word;
            pre.num = number;
            pre.year = year;
            pre.name = name;
            pre.source = new_href;
        };

        if let Some(node) = node.find(Name("pre")).next() {
            pre.holding = node.text();
        };

        buffer.push(pre);
    }
    buffer
}

impl Precedent {
    pub async fn add_to_pool(self, pool: &PgPool) {
        let id = format!("{}-{}-{}", self.year, self.specific, self.num);
        match sqlx::query(
            "INSERT INTO precedents (id, name, holding, source, year, num, specific)
            VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(id.clone())
        .bind(self.name)
        .bind(self.holding)
        .bind(self.source)
        .bind(self.year)
        .bind(self.num)
        .bind(self.specific)
        .execute(pool)
        .await
        {
            Ok(_) => println!("Insert successful"),
            Err(e) => {
                eprintln!("Insert failed: {} ", e);
                println!("dup: {}", id);
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resolution {
    pub id: String,
    pub lawtype: String,
    pub related_law: String,
    pub name: String,
    pub content: String,
    pub source: String,
    pub year: i16,
    pub time: i16,
}

impl Resolution {
    pub async fn add_to_pool(self, pool: &PgPool) {
        let id = format!("{}-{}", self.year, self.time);
        match sqlx::query(
            "INSERT INTO resolution (id, lawtype, related_law, name, content, source, year, time)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(id.clone())
        .bind(self.lawtype)
        .bind(self.related_law)
        .bind(self.name)
        .bind(self.content)
        .bind(self.source)
        .bind(self.year)
        .bind(self.time)
        .execute(pool)
        .await
        {
            Ok(_) => println!("Insert successful"),
            Err(e) => {
                eprintln!("Insert failed: {}", e);
                println!("dup: {}", id);
            }
        }
    }
}

use std::fs::File;
use std::io::{BufRead, BufReader, Read};

pub fn scrapeResolution(num: String) -> Resolution {
    let read_file_path = format!("民庭決議2/{num}.html");
    let mut file = File::open(read_file_path).expect("ppp");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let doc = Document::from(contents.as_str());
    let mut resolution = Resolution {
        id: "".to_string(),
        lawtype: "民事".to_string(),
        related_law: "".to_string(),
        name: "".to_string(),
        content: "".to_string(),
        source: "".to_string(),
        year: 0,
        time: 0,
    };
    if let Some(node) = doc.find(And(Class("title"), Name("h2"))).next() {
        let title = node.text();
        resolution.name = title;
    };
    if let Some(node) = doc.find(And(Class("cp"), Name("section"))).next() {
        let content = node.inner_html();
        resolution.content = content;
    };
    resolution
}

pub fn extract_year_and_time(s: &str) -> Option<(i16, i16)> {
    let re = Regex::new(r"(?P<year>\d{2,4})年(?:度)?(?:[^第]+)?第(?P<time>\d{1,2})次").ok()?;
    let caps = re.captures(s)?;
    let year = caps.name("year")?.as_str().parse().ok()?;
    let time = caps.name("time")?.as_str().parse().ok()?;
    Some((year, time))
}

pub fn rename(old_name: String) -> String {
    let mut name = old_name;

    let year_map = [
        ("一○八", "108"),
        ("一〇八", "108"),
        ("一零八", "108"),
        ("一○七", "107"),
        ("一〇七", "107"),
        ("一零七", "107"),
        ("一○六", "106"),
        ("一〇六", "106"),
        ("一零六", "106"),
        ("一○五", "105"),
        ("一〇五", "105"),
        ("一零五", "105"),
        ("一○四", "104"),
        ("一〇四", "104"),
        ("一零四", "104"),
        ("一○三", "103"),
        ("一〇三", "103"),
        ("一零三", "103"),
        ("一○二", "102"),
        ("一〇二", "102"),
        ("一零二", "102"),
        ("一○一", "101"),
        ("一〇一", "101"),
        ("一零一", "101"),
        ("一○○", "100"),
        ("一〇〇", "100"),
        ("一零零", "100"),
        ("九十九", "99"),
        ("九十八", "98"),
        ("九十七", "97"),
        ("九十六", "96"),
        ("九十五", "95"),
        ("九十四", "94"),
        ("九十三", "93"),
        ("九十二", "92"),
        ("九十一", "91"),
        ("九十", "90"),
        ("八十九", "89"),
    ];

    let time_map = [
        ("第二十次", "第20次"),
        ("第十九次", "第19次"),
        ("第十八次", "第18次"),
        ("第十七次", "第17次"),
        ("第十六次", "第16次"),
        ("第十五次", "第15次"),
        ("第十四次", "第14次"),
        ("第十三次", "第13次"),
        ("第十二次", "第12次"),
        ("第十一次", "第11次"),
        ("第十次", "第10次"),
        ("第九次", "第9次"),
        ("第八次", "第8次"),
        ("第七次", "第7次"),
        ("第六次", "第6次"),
        ("第五次", "第5次"),
        ("第四次", "第4次"),
        ("第三次", "第3次"),
        ("第二次", "第2次"),
        ("第一次", "第1次"),
    ];

    for (ch, num) in year_map {
        if (name.contains(ch)) {
            name = name.replace(ch, num);
        }
    }
    for (ch, num) in time_map {
        if (name.contains(ch)) {
            name = name.replace(ch, num);
        }
    }

    name
}

use new_law::{Line, NewLaw, NewLaws};

fn format_lines(node: select::node::Node) -> Vec<new_law::Line> {
    let mut vec = Vec::new();

    for line in node.find(Or(
        Or(
            Or(
                Or(
                    Or(
                        Or(
                            Or(
                                Or(Class("line-0000"), Class("line-0001")),
                                Class("line-0002"),
                            ),
                            Class("line-0003"),
                        ),
                        Class("line-0004"),
                    ),
                    Class("line-0005"),
                ),
                Class("line-0006"),
            ),
            Class("line-0007"),
        ),
        Class("line-0008"),
    )) {
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

use chrono::{Datelike, Local};
pub fn scrape_lawinformation(html: String) -> Result<Lawinformation, Box<dyn Error>> {
    let document = Document::from(html.as_str());
    let now = Local::now();
    let date = format!("{}年{}月{}日", now.year(), now.month(), now.day());

    let mut information = Lawinformation {
        name: "".to_string(),
        originalid: "".to_string(),
        release_date: "".to_string(),
        update_date: date,
    };

    if let Some(node) = document
        .find(And(Name("a"), Attr("id", "hlLawName")))
        .next()
    {
        information.name = node.text();
        let href = node.attr("href").unwrap();
        information.originalid = href.to_string();
    }

    if let Some(tr) = document.find(Name("tr")).nth(1) {
        let th = tr
            .find(Name("th"))
            .next()
            .map_or("預設文字".to_string(), |n| n.text());
        let td = tr
            .find(Name("td"))
            .next()
            .map_or("預設文字".to_string(), |n| n.text());
        information.release_date = format!("{}：{}", th, td);
    }

    Ok(information)
}

use std::error::Error;
pub async fn scrape_new_law(title: String, html: String) -> Result<NewLaws, Box<dyn Error>> {
    let document = Document::from(html.as_str());
    let mut law_vec: Vec<NewLaw> = Vec::new();
    document.find(Class("row")).for_each(|row| {
        // 查找連結 href 和 name 屬性
        if let Some(link) = row.find(Name("a")).next() {
            let href = link.attr("href").unwrap_or("無連結").to_string();
            let num = link.attr("name").unwrap_or("無名稱").to_string();

            let lines = format_lines(row);
            let id = format!("{title}-{num}");
            let chapter = vec![title.clone()];

            law_vec.push(NewLaw {
                id,
                href,
                chapter,
                num,
                lines,
            });
        }
    });

    let mut chapter_vec = Vec::new();
    if let Some(node) = document.find(Attr("id", "hlkHD_CHAR")).next() {
        let href = node.attr("href").unwrap();
        let new_href = format!("https://law.moj.gov.tw{}", href);
        chapter_vec = scrape_chapter(new_href, title.clone()).await;
    } else {
        return Ok(NewLaws { lines: law_vec });
    }

    chapter_vec.push(Chapter {
        title: ";;".to_string(),
        num: ";;;".to_string(),
        level: 100000,
    });

    let mut count = 0;
    let mut t = String::new();

    for mut law in &mut law_vec {
        if law.num == chapter_vec.get(count).unwrap().num {
            count += 1;
            t = chapter_vec.get(count - 1).unwrap().title.clone();
        }
        let c = t.clone();
        law.chapter = c.split("/").map(|s| s.to_string()).collect();
    }

    let law_vec: Vec<_> = law_vec
        .iter()
        .filter(|x| !x.href.is_empty())
        .map(|x| x.to_owned())
        .collect();

    Ok(NewLaws { lines: law_vec })
}

use std::collections::HashMap;
pub async fn scrape_chapter(href: String, maintitle: String) -> Vec<Chapter> {
    let html = reqwest::get(href.clone())
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let doc = Document::from(html.as_str());
    let mut char_map = HashMap::new();
    char_map.insert(1, "".to_string());
    char_map.insert(2, "".to_string());
    char_map.insert(3, "".to_string());
    char_map.insert(4, "".to_string());
    fn get_title(map: &HashMap<usize, String>, num: usize, title: String) -> String {
        let mut buffer = Vec::new();
        buffer.push(title);
        for number in 1..num + 1 {
            let title = map.get(&number).unwrap();
            if !title.is_empty() {
                buffer.push(title.clone());
            };
        }
        buffer.join("/")
    }
    let mut map = HashMap::new();

    if let Some(node) = doc.find(Class("law-reg-content")).next() {
        // 尋找所有h3
        for h3 in node.find(Class("h3")) {
            let title = h3.find(Name("a")).next().unwrap().text();
            let name = h3.text();
            let re = regex::Regex::new(r"§  (?P<num>.+)").unwrap();
            let caps = re.captures(&name).unwrap();
            let try_num = caps.name("num").unwrap().as_str();
            let num = try_num.to_string();

            for level in 1..5 {
                let class = format!("char-{level}");
                let has_class = h3
                    .attr("class")
                    .unwrap_or("")
                    .split_whitespace()
                    .any(|c| c.contains(&class));

                if has_class {
                    char_map.insert(level, title.trim().to_string());
                    let title = get_title(&char_map, level, maintitle.clone());
                    map.insert(
                        num.clone(),
                        Chapter {
                            title,
                            num: num.clone(),
                            level,
                        },
                    );
                }
            }
        }
    }
    let mut vec = Vec::new();
    map.values().for_each(|x| vec.push(x.clone()));
    vec.sort_by(|a, b| {
        to_f32(a.num.clone())
            .partial_cmp(&to_f32(b.num.clone()))
            .unwrap()
    });
    vec
}

fn to_f32(s: String) -> f32 {
    if s.contains("-") {
        let (big, small) = s.split_once("-").unwrap();
        let big_number: f32 = big.parse().unwrap();
        let small_number: f32 = small.parse().unwrap();
        big_number + small_number * 0.1
    } else {
        s.parse().unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct Chapter {
    pub title: String,
    pub num: String,
    pub level: usize,
}

#[derive(Clone, Debug)]
pub struct Lawinformation {
    pub name: String,
    pub originalid: String,
    pub update_date: String,
    pub release_date: String,
}

impl Lawinformation {
    pub async fn add_to_pool(&self, pool: &PgPool) {
        let query = r#"
            INSERT INTO lawinformation (originalid, name, update_date, release_date)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (originalid) DO UPDATE
            SET name = EXCLUDED.name,
                update_date = EXCLUDED.update_date,
                release_date = EXCLUDED.release_date
        "#;

        match sqlx::query(query)
            .bind(&self.originalid)
            .bind(&self.name)
            .bind(&self.update_date)
            .bind(&self.release_date)
            .execute(pool)
            .await
        {
            Ok(_) => println!("✅ Upserted: {}", self.originalid),
            Err(e) => eprintln!("❌ Failed [{}]: {}", self.originalid, e),
        }
    }
}

use sqlx::{postgres::PgRow, Row};

pub async fn get_all_information(pool: &PgPool) -> Vec<Lawinformation> {
    let result =
        sqlx::query("SELECT name, originalid, update_date, release_date FROM lawinformation")
            .map(|row: PgRow| Lawinformation {
                name: row.get("name"),
                originalid: row.get("originalid"),
                update_date: row.get("update_date"),
                release_date: row.get("release_date"),
            })
            .fetch_all(pool)
            .await;

    match result {
        Ok(list) => list,
        Err(e) => {
            eprintln!("Database query failed: {}", e);
            Vec::new()
        }
    }
}
