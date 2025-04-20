use futures::future::join_all;
use otherlawresource::scrapeNewInterpretation;
use rayon::prelude::*;
use select::document::Document;
use select::predicate::Name;
use select::predicate::{And, Attr, Class, Or};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};

pub async fn new_pool(url: &str) -> PgPool {
    let db_pool = match PgPoolOptions::new().max_connections(50).connect(url).await {
        Ok(pool) => pool,
        Err(e) => panic!("sss {}", e),
    };
    db_pool
}

#[tokio::test]
async fn get_one_inter() -> Result<(), Box<dyn Error>> {
    let pool = new_pool("postgresql://postgres:IoNTPUpeBHZMjpfpbdHDfIKzzbSQCIEm@autorack.proxy.rlwy.net:10488/railway").await;
    let newinter = otherlawresource::get_newinterpretations(&pool).await;
    std::fs::write("law.json", serde_json::to_string_pretty(&newinter).unwrap());
    Ok(())
}

#[tokio::test]
async fn get_one_law() -> Result<(), Box<dyn Error>> {
    let pool = new_pool("postgresql://postgres:IoNTPUpeBHZMjpfpbdHDfIKzzbSQCIEm@autorack.proxy.rlwy.net:10488/railway").await;
    let href = "https://law.moj.gov.tw/LawClass/LawAll.aspx?pcode=D0070119".to_string();
    let semaphore = Arc::new(Semaphore::new(50));
    let pool = Arc::new(pool);

    let html = reqwest::get(href).await.unwrap().text().await.unwrap();

    let information = otherlawresource::scrape_lawinformation(html.clone()).unwrap();
    information.add_to_pool(&pool).await;

    let vec = otherlawresource::scrape_new_law(information.name, html)
        .await
        .unwrap();

    let mut handles = Vec::new();

    for v in vec.lines {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pool = pool.clone();

        let handle = tokio::spawn(async move {
            println!("{}", v.num);
            v.add_to_pool(&pool).await;
            drop(permit);
        });

        handles.push(handle);
    }

    join_all(handles).await;
    Ok(())
}

#[tokio::test]
async fn update_newinter() -> Result<(), Box<dyn Error>> {
    let pool = new_pool("postgresql://postgres:IoNTPUpeBHZMjpfpbdHDfIKzzbSQCIEm@autorack.proxy.rlwy.net:10488/railway").await;
    let vec = vec![
        "一、", "二、", "三、", "四、", "五、", "六、", "七、", "八、", "九、", "十、",
    ];
    let exist_law = otherlawresource::get_all_information(&pool).await;
    let mut law_vec = exist_law
        .iter()
        .map(|x| x.name.clone())
        .filter(|x| !x.is_empty())
        .collect::<Vec<String>>();
    law_vec.push("刑法".to_string());

    let semaphore = Arc::new(Semaphore::new(50));
    let pool = Arc::new(pool);
    let mut handles = Vec::new();

    for inter in otherlawresource::get_newinterpretations(&pool).await {
        println!("{}", inter.no);
        let (content, reason) = captruex(&inter.content.clone().unwrap());

        let mut buffer = Vec::new();
        let content = replace_the_text(&content);

        if !content.contains("一、") {
            buffer.push(content.clone());
            println!("lll");
        } else {
            for i in 0..vec.len() - 1 {
                let pattern = format!(r"(?s){}(?P<content>.+?){}", vec[i], vec[i + 1]);
                let re = regex::Regex::new(&pattern).unwrap();
                if let Some(caps) = re.captures(content.clone().replace(" ", "").as_str()) {
                    if let Some(p) = caps.name("content") {
                        buffer.push(format!("{}{}", vec[i], p.as_str()));
                    }
                }
            }

            let pattern = format!(r"(?s){}(?P<content>.+)", vec[buffer.len()],);
            let re = regex::Regex::new(&pattern).unwrap();
            if let Some(caps) = re.captures(content.clone().replace(" ", "").as_str()) {
                if let Some(p) = caps.name("content") {
                    buffer.push(format!("{}{}", vec[buffer.len()], p.as_str()));
                }
            }
        }

        let maincontent = buffer;
        let reflaws = findinglaw(
            replace_the_text(inter.content.clone().unwrap().as_str()),
            law_vec.clone(),
        );

        let realnew = otherlawresource::NewInter {
            id: inter.id,
            casename: inter.name,
            name: inter.no,
            date: inter.date,
            casesummary: inter.reason,
            maincontent,
            reason: replace_the_text(&reason),
            related_law: inter.related_law,
            source: inter.source,
            year: inter.year,
            number: inter.number,
            reflaws,
        };
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pool = pool.clone();

        let handle = tokio::spawn(async move {
            realnew.add_to_pool(&pool).await;
            drop(permit);
        });

        handles.push(handle);
    }
    join_all(handles).await;
    Ok(())
}

// 爬取多個法條
#[tokio::test]
async fn get_multiple_law() -> Result<(), Box<dyn Error>> {
    let pool = new_pool("postgresql://postgres:IoNTPUpeBHZMjpfpbdHDfIKzzbSQCIEm@autorack.proxy.rlwy.net:10488/railway").await;
    let href = "https://law.moj.gov.tw/Law/LawSearchLaw.aspx?TY=04007016&mo=1".to_string();
    let mut vec = get_law_href(href).await;
    let exist_law = otherlawresource::get_all_information(&pool).await;
    for law in exist_law {
        vec.retain(|x| *x != format!("https://law.moj.gov.tw/LawClass/{}", law.originalid));
    }

    let semaphore = Arc::new(Semaphore::new(50));
    let pool = Arc::new(pool);

    for law_href in vec {
        let html = reqwest::get(law_href).await.unwrap().text().await.unwrap();

        let information = otherlawresource::scrape_lawinformation(html.clone()).unwrap();
        information.add_to_pool(&pool).await;

        let vec = otherlawresource::scrape_new_law(information.name, html)
            .await
            .unwrap();

        let mut handles = Vec::new();

        for v in vec.lines {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let pool = pool.clone();

            let handle = tokio::spawn(async move {
                println!("{}", v.num);
                v.add_to_pool(&pool).await;
                drop(permit);
            });

            handles.push(handle);
        }

        join_all(handles).await;
    }
    Ok(())
}

use std::panic;

fn chinese_digits_to_number(ch: &str) -> Option<u32> {
    let digit_map = [
        ('零', '0'),
        ('○', '0'),
        ('一', '1'),
        ('二', '2'),
        ('三', '3'),
        ('四', '4'),
        ('五', '5'),
        ('六', '6'),
        ('七', '7'),
        ('八', '8'),
        ('九', '9'),
    ];

    let mut result = String::new();
    for c in ch.chars() {
        match digit_map.iter().find(|(k, _)| *k == c) {
            Some((_, v)) => result.push(*v),
            None => return None, // 非合法中文數字，直接跳過
        }
    }
    result.parse().ok()
}

fn convert_chinese_law_numbers2(text: &str) -> String {
    let re = regex::Regex::new(r"(?s)第([一二三四五六七八九十○零]+)號").unwrap();

    let try_convert = panic::catch_unwind(|| {
        re.replace_all(text, |caps: &regex::Captures| {
            let chinese = &caps[1];

            match chinese_digits_to_number(chinese) {
                Some(val) => format!("第{}號", val),
                None => caps[0].to_string(), // 轉換失敗就保持原樣
            }
        })
        .to_string()
    });

    // 如果整體有 panic，就還原 text；否則回傳轉換後結果
    try_convert.unwrap_or_else(|_| text.to_string())
}

fn convert_chinese_law_numbers(text: &str) -> String {
    let re = regex::Regex::new(
        r"(?s)第([一二三四五六七八九十百千萬〇○Ｏ０壹貳參肆伍陸柒捌玖拾佰仟萬億兆]+)(條|項|款)",
    )
    .unwrap();

    let try_convert = panic::catch_unwind(|| {
        re.replace_all(text, |caps: &regex::Captures| {
            let chinese = &caps[1];
            let suffix = &caps[2];

            match chinese_number::parse_chinese_number_to_u64(
                chinese_number::ChineseNumberCountMethod::Low,
                chinese,
            ) {
                Ok(val) => format!("第{}{}", val, suffix),
                Err(_) => caps[0].to_string(), // 轉換失敗就保持原樣
            }
        })
        .to_string()
    });

    // 如果整體有 panic，就還原 text；否則回傳轉換後結果
    try_convert.unwrap_or_else(|_| text.to_string())
}
#[test]
fn tr() {
    let sample = "本院釋字第二四三號、第三八二號、第四三○號、第四六二號、第六五三號解釋參照"
        .to_string()
        .replace(" ", "");

    let result = convert_chinese_law_numbers2(&sample);
    let result = findinginter(result);
    println!("{:#?}", result); // 第14條第1項、第653號、第117條
}

#[tokio::test]
async fn update_oldinter2() -> Result<(), Box<dyn Error>> {
    let pool = new_pool("postgresql://postgres:IoNTPUpeBHZMjpfpbdHDfIKzzbSQCIEm@autorack.proxy.rlwy.net:10488/railway").await;

    let semaphore = Arc::new(Semaphore::new(50));
    let pool = Arc::new(pool);
    let mut handles = Vec::new();
    let mut exist_law = otherlawresource::get_all_information(&pool).await;
    exist_law.push(otherlawresource::Lawinformation {
        name: "刑法".to_string(),
        originalid: ".".to_string(),
        update_date: "".to_string(),
        release_date: "".to_string(),
    });
    exist_law.push(otherlawresource::Lawinformation {
        name: "憲法".to_string(),
        originalid: ".".to_string(),
        update_date: "".to_string(),
        release_date: "".to_string(),
    });

    for inter in otherlawresource::get_all_oldinterpretation(&pool).await {
        let reason = inter.reasoning.clone().unwrap_or("".to_string());
        let content = inter.content.clone().unwrap_or("".to_string());
        let data = format!("{content}{reason}");

        let matched: Vec<_> = exist_law
            .clone()
            .par_iter()
            .filter_map(|inform| {
                if data.contains(&inform.name) {
                    Some(inform.name.clone())
                } else {
                    None
                }
            })
            .filter(|x| !x.is_empty())
            .collect();
        let mut law_vec = exist_law
            .iter()
            .map(|x| x.name.clone())
            .filter(|x| !x.is_empty())
            .collect::<Vec<String>>();
        law_vec.push("刑法".to_string());
        let reflawid = findinglaw(data.clone(), law_vec);

        let refinter = findinginter(data.clone());

        let realnew = otherlawresource::OldInterpretation {
            id: inter.id,
            date: inter.date,
            reasoning: inter.reasoning,
            content: inter.content,
            trouble: inter.trouble,
            related_law: inter.related_law,
            source: inter.source,
            reflaws: Some(matched),
            reflawid: Some(reflawid),
            refinter: Some(refinter),
        };

        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pool = pool.clone();

        let handle = tokio::spawn(async move {
            realnew.update(&pool).await;
            drop(permit);
        });

        handles.push(handle);
    }
    join_all(handles).await;
    Ok(())
}

#[tokio::test]
async fn update_oldinter() -> Result<(), Box<dyn Error>> {
    let pool = new_pool("postgresql://postgres:IoNTPUpeBHZMjpfpbdHDfIKzzbSQCIEm@autorack.proxy.rlwy.net:10488/railway").await;

    let semaphore = Arc::new(Semaphore::new(50));
    let pool = Arc::new(pool);
    let mut handles = Vec::new();
    let mut exist_law = otherlawresource::get_all_information(&pool).await;
    exist_law.push(otherlawresource::Lawinformation {
        name: "刑法".to_string(),
        originalid: ".".to_string(),
        update_date: "".to_string(),
        release_date: "".to_string(),
    });
    exist_law.push(otherlawresource::Lawinformation {
        name: "憲法".to_string(),
        originalid: ".".to_string(),
        update_date: "".to_string(),
        release_date: "".to_string(),
    });

    for inter in otherlawresource::get_all_oldinterpretation(&pool).await {
        let reason = inter.reasoning.unwrap_or("".to_string());
        /*
        let re = fancy_regex::Regex::new(r"(?<!。)\n").unwrap();
        let reasonfromat = re.replace_all(&reason, "").to_string().replace(" ", "");
        */

        let content = inter.content.unwrap_or("".to_string());
        /*
        let re = fancy_regex::Regex::new(r"(?<!。)\n").unwrap();
        let contentformat = re.replace_all(&content, "").to_string().replace(" ", "");
        */

        /*
        let data = format!(
            "{}{}",
            inter.content.clone().unwrap_or("".to_string()),
            inter.reasoning.clone().unwrap_or("".to_string())
        )
        .replace(" ", "");
        */
        // let c = captruey(&inter.content.unwrap_or("".to_string()));
        // let reasoning = convert_chinese_law_numbers2(reasonfromat.as_str());

        let contenting = content.replace("中華民國中華民國", "中華民國");
        let reasoning = reason.replace("中華民國中華民國", "中華民國");
        let data = format!("{contenting}{reasoning}");

        let matched: Vec<_> = exist_law
            .clone()
            .par_iter()
            .filter_map(|inform| {
                if data.contains(&inform.name) {
                    Some(inform.name.clone())
                } else {
                    None
                }
            })
            .filter(|x| !x.is_empty())
            .collect();

        let realnew = otherlawresource::OldInterpretation {
            id: inter.id,
            date: inter.date,
            reasoning: Some(reasoning),
            content: Some(contenting),
            trouble: inter.trouble,
            related_law: inter.related_law,
            source: inter.source,
            reflaws: Some(matched),
            refinter: inter.refinter,
            reflawid: inter.reflawid,
        };

        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pool = pool.clone();

        let handle = tokio::spawn(async move {
            realnew.update(&pool).await;
            drop(permit);
        });

        handles.push(handle);
    }
    join_all(handles).await;
    Ok(())
}

fn replace_the_text(text: &str) -> String {
    // 1.將文內的\n替換
    let re = fancy_regex::Regex::new(r"(?<!。)\n").unwrap();
    let textfromat1 = re.replace_all(text, "").to_string().replace(" ", "");

    // 2.將刑法法替換為中華民國刑法
    let textformat2 = textfromat1
        .replace("刑法第", "中華民國刑法第")
        .replace("中華民國中華民國", "中華民國");
    textformat2
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = new_pool("postgresql://postgres:IoNTPUpeBHZMjpfpbdHDfIKzzbSQCIEm@autorack.proxy.rlwy.net:10488/railway").await;
    let semaphore = Arc::new(Semaphore::new(50));
    let pool = Arc::new(pool);
    let mut handles = Vec::new();

    for num in 807..808 {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pool = pool.clone();

        let handle = tokio::spawn(async move {
            let mut inter = otherlawresource::scrapeOldInterpretation(num.to_string()).await;

            // 1.將content擷取我要的內容
            let rawcontet = captruey(&inter.content.unwrap_or("".to_string()));

            // 2.替換標籤
            let formatcontent = replace_the_text(&rawcontet);
            let formatreason = replace_the_text(&inter.reasoning.unwrap_or("".to_string()));

            //3.將法條文字替換
            let formatcontent2 = convert_chinese_law_numbers(&formatcontent);
            let formatreason2 = convert_chinese_law_numbers(&formatreason);

            //4.將釋字文字替換
            let formatcontent3 = convert_chinese_law_numbers2(&formatcontent2);
            let formatreason3 = convert_chinese_law_numbers2(&formatreason2);

            inter.reasoning = Some(formatreason3);
            inter.content = Some(formatcontent3);

            inter.add_to_pool(&pool).await;

            drop(permit);
        });

        handles.push(handle);
    }

    join_all(handles).await;

    /*
    for x in 1..7 {
        let read_file_path = format!("憲判字導覽頁/{x}.html");
        let mut file = File::open(read_file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let doc = Document::from(contents.as_str());

        for td in doc.find(Name("td")) {
            if let Some(a) = td.find(Name("a")).next() {
                let href = a.attr("href").unwrap();
                let new_href = format!("https://mojlaw.moj.gov.tw/{href}");
                let newinter = scrapeNewInterpretation(1, new_href).await;
                newinter.add_to_pool(&pool).await;
            }
        }
    }
    */

    // let mut buffer = Vec::new();

    /*

    let doc = Document::from(contents.as_str());
    for tr in doc.find(Name("tr")) {
        let num = tr.find(Name("th")).next().unwrap().text();
        let trouble = tr.find(Name("td")).last().expect("lll").text();
        buffer.push((num, trouble));
    }

    let missing_ids: Vec<String> = vec!["572", "641", "642", "646", "651", "675"]
        .into_iter()
        .map(String::from)
        .collect();

    let buffer_map: HashMap<String, (String, String)> = buffer
        .into_iter()
        .map(|(num, s)| (num.clone(), (num, s)))
        .collect();

    for id in missing_ids {
        if let Some((num, s)) = buffer_map.get(&id) {
            let inter = otherlawresource::scrapeOldInterpretation(num.clone(), s.clone()).await;
            inter.add_to_pool(&pool).await;
        }
    }*/

    /*
    let read_file_path = format!("民庭決議導覽頁/2.html");
    let mut file = File::open(read_file_path).expect("ppp");
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    */

    // 解析 HTML 內容
    // let pool = new_pool("postgresql://postgres:IoNTPUpeBHZMjpfpbdHDfIKzzbSQCIEm@autorack.proxy.rlwy.net:10488/railway").await;
    //
    //
    //

    /*

    let href = "https://law.moj.gov.tw/Law/LawSearchLaw.aspx?TY=04007005&mo=1".to_string();
    let mut vec = get_law_href(href).await;
    let data = tokio::fs::read_to_string("test.txt").await.unwrap();
    let exist_law = otherlawresource::get_all_information(&pool).await;
    let arc_exist_law = exist_law;

    let matched: Vec<_> = arc_exist_law
        .par_iter()
        .filter_map(|inform| {
            if data.contains(&inform.name) {
                Some(inform.clone())
            } else {
                None
            }
        })
        .collect();
    println!("{:#?}", matched);
    */

    // println!("{:#?}", matched);

    /*

    println!("{:#?}", exist_law);
    for law in exist_law {
        vec.retain(|x| *x != format!("https://law.moj.gov.tw/LawClass/{}", law.originalid));
    }

    for law_href in vec {
        let html = reqwest::get(law_href).await.unwrap().text().await.unwrap();

        let information = otherlawresource::scrape_lawinformation(html.clone()).unwrap();
        information.add_to_pool(&pool).await;

        let vec = otherlawresource::scrape_new_law(information.name, html)
            .await
            .unwrap();

        let mut handles = Vec::new();

        for v in vec.lines {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let pool = pool.clone();

            let handle = tokio::spawn(async move {
                println!("{}", v.num);
                v.add_to_pool(&pool).await;
                drop(permit);
            });

            handles.push(handle);
        }

        join_all(handles).await;
    }
    */

    /*

    let mut map = HashMap::new();

    if let Some(node) = doc.find(And(Class("list"), Name("div"))).next() {
        for a in node.find(Name("a")) {
            let title = a.attr("title").unwrap();
            let href = a.attr("href").unwrap();
            let new_href = format!("'{}',", href);
            map.insert(title.to_string(), new_href);
            //println!("{}", new_href);
        }
    }

    let mut res_buffer = Vec::new();
    for x in 1..38 {
        let res = otherlawresource::scrapeResolution(x.to_string());
        res_buffer.push(res);
    }

    let semaphore = Arc::new(Semaphore::new(10));
    let pool = Arc::new(pool);
    let mut handles = Vec::new();

    for mut res in res_buffer {
        let href = map.get(&res.name).unwrap();
        res.source = href.clone();
        let t = otherlawresource::rename(res.name);
        let (year, time) = otherlawresource::extract_year_and_time(&t).unwrap();
        res.name = t;
        res.year = year;
        res.time = time;

        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pool = pool.clone();

        let handle = tokio::spawn(async move {
            res.add_to_pool(&pool).await;
            drop(permit);
        });

        handles.push(handle);
    }
    */

    /*
    for num in 400..411 {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pool = pool.clone();
        let num_clone = num;

        sleep(Duration::from_millis(200)).await;

        let handle = tokio::spawn(async move {
            let pres = otherlawresource::scrape_precedent(num_clone.clone()).await;
            println!("{num_clone}");
            for pre in pres {
                pre.add_to_pool(&pool).await;
            }
            drop(permit);
        });

        handles.push(handle);
    }
    */

    Ok(())
}

async fn get_law_href(href: String) -> Vec<String> {
    let mut buffer = Vec::new();
    let html = reqwest::get(href.clone())
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let document = Document::from(html.as_str());
    if let Some(table) = document
        .find(And(Class("table-hover"), Name("table")))
        .next()
    {
        for td in table.find(And(Name("td"), (Attr("id", ())))) {
            let id = td.attr("id").unwrap();
            buffer.push(format!(
                "https://law.moj.gov.tw/LawClass/LawAll.aspx?pcode={}",
                id
            ));
        }
    }
    buffer
}

#[test]
fn lol() {
    let vec = vec![
        "一、", "二、", "三、", "四、", "五、", "六、", "七、", "八、", "九、", "十、",
    ];

    let s = "一、日治時期為人民所有，嗣因逾土地總登記期限，未登記為人民所有，致登記為國有且持續至今之土地，在人民基於該土地所有人地位，請求國家塗銷登記時，無民法消滅時效規定之適用。最高法院70年台上字第311號民事判例關於「……系爭土地如尚未依吾國法令登記為被上訴人所有，而登記為國有後，迄今已經過15年，被上訴人請求塗銷此項國有登記，上訴人既有時效完成拒絕給付之抗辯，被上訴人之請求，自屬無從准許。」部分，不符憲法第15條保障人民財產權之意旨。,二、其餘聲請不受理。三、哈哈笨蛋".to_string();

    let mut buffer = Vec::new();

    for i in 0..vec.len() - 1 {
        let pattern = format!(
            r"(?s){}(?P<content>.+?){}",
            regex::escape(vec[i]),
            regex::escape(vec[i + 1])
        );
        let re = regex::Regex::new(&pattern).unwrap();
        if let Some(caps) = re.captures(&s) {
            if let Some(p) = caps.name("content") {
                buffer.push(format!("{}{}", vec[i], p.as_str()));
            }
        }
    }

    let pattern = format!(r"(?s){}(?P<content>.+)", regex::escape(vec[buffer.len()]),);
    let re = regex::Regex::new(&pattern).unwrap();
    if let Some(caps) = re.captures(&s) {
        if let Some(p) = caps.name("content") {
            buffer.push(format!("{}{}", vec[buffer.len()], p.as_str()));
        }
    }

    println!("{:#?}", buffer);

    /*
    s.split("主  文").enumerate().for_each(|(i, name)| {
        println!("{i}：{name}");
    });
    */
}

fn findinglaw(data: String, law_vec: Vec<String>) -> Vec<String> {
    let buffer = Mutex::new(Vec::new());
    law_vec.par_iter().for_each(|x| {
        let pattern = format!(r"(?s){}第(?P<num>\d+)條(之(?P<num2>\d+)?)?", x);
        let re = regex::Regex::new(&pattern).unwrap();

        for caps in re.captures_iter(data.as_str()) {
            let num = caps.name("num").unwrap().as_str();
            let num2 = caps.name("num2").map(|m| m.as_str()).unwrap_or("");
            let lawname;
            if num2.is_empty() {
                if x == "刑法" {
                    lawname = format!("中華民國刑法-{num}");
                } else {
                    lawname = format!("{x}-{num}");
                }
            } else {
                if x == "刑法" {
                    lawname = format!("中華民國刑法-{num}");
                } else {
                    lawname = format!("{x}-{num}");
                }
            }
            let mut buf = buffer.lock().unwrap();
            buf.push(lawname)
        }
    });
    buffer.lock().unwrap().to_vec()
}

fn findinginter(data: String) -> Vec<String> {
    let mut buffer = Vec::new();
    let pattern = format!(r"(?s)第(?P<num>\d+)號");
    let re = regex::Regex::new(&pattern).unwrap();

    for caps in re.captures_iter(data.as_str()) {
        if let Some(num) = caps.name("num") {
            buffer.push(num.as_str().to_string());
        }
    }
    buffer
}

fn captruex(content: &str) -> (String, String) {
    let re = regex::Regex::new(
        r"(?s)\s{1,}主\s{1,}文(?P<maincontent>.+?)\s{1,}理\s{1,}由(?P<reason>.+)",
    )
    .unwrap();
    let caps = re.captures(content).unwrap();
    let main_content = caps.name("maincontent").unwrap().as_str();
    let reason = caps.name("reason").unwrap().as_str();
    (main_content.to_string(), reason.to_string())
}

fn captruey(content: &str) -> String {
    let re = regex::Regex::new(r"(?s)(?P<maincontent>.+?)大法官會議主席").unwrap();
    if let Some(caps) = re.captures(content) {
        let main_content = caps.name("maincontent").unwrap().as_str();
        main_content.to_string()
    } else {
        content.to_string()
    }
}
