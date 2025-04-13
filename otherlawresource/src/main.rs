use futures::future::join_all;
use otherlawresource::scrapeNewInterpretation;
use select::document::Document;
use select::predicate::Name;
use select::predicate::{And, Attr, Class, Or};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};

pub async fn new_pool(url: &str) -> PgPool {
    let db_pool = match PgPoolOptions::new().max_connections(50).connect(url).await {
        Ok(pool) => pool,
        Err(e) => panic!("sss {}", e),
    };
    db_pool
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = new_pool("postgresql://postgres:IoNTPUpeBHZMjpfpbdHDfIKzzbSQCIEm@autorack.proxy.rlwy.net:10488/railway").await;

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

    /*
    let semaphore = Arc::new(Semaphore::new(50));
    let pool = Arc::new(pool);

    let href = "https://law.moj.gov.tw/Law/LawSearchLaw.aspx?TY=04007005&mo=1".to_string();
    let mut vec = get_law_href(href).await;

    let exist_law = otherlawresource::get_all_information(&pool).await;
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
