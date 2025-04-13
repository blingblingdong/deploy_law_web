#![recursion_limit = "512"]
pub mod routes;
mod store;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};
pub mod types;
use config::Config;
#[allow(unused_imports)]
use handle_errors::return_error;
use law_rs::Laws;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{http::Method, Filter};

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct Args {
    log_level: String,
    port: u16,
}

#[macro_export]
macro_rules! trace_async {
    ($label:expr, $expr:expr) => {
        $crate::utils::trace::trace_async($label, $expr).await
    };
}

/*
#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    let store = store::Store::new("postgresql://postgres:IoNTPUpeBHZMjpfpbdHDfIKzzbSQCIEm@autorack.proxy.rlwy.net:10488/railway").await;
    let files = store
        .get_every_file()
        .await?;
    for file in files.vec_files {
        let content = note::parse_note(&file.content);
        let x = store.update_the_note(serde_json::json!(content), file.id).await.unwrap();
        println!("{}", x.id);
    }
    Ok(())
}
*/

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    let config = Config::builder()
        .add_source(config::File::with_name("setup"))
        .build()
        .unwrap();

    let config = config.try_deserialize::<Args>().unwrap();

    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "handle_errors={},rust_web_dev={},warp={}",
            config.log_level, config.log_level, config.log_level
        )
    });

    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // 初始化
    dotenv::dotenv().ok();

    if let Err(_) = std::env::var("DATABASE_PUBLIC_URL") {
        panic!("找不到資料庫");
    }

    if let Err(_) = std::env::var("REDIS_PUBLIC_URL") {
        panic!("找不到Redis");
    }

    if let Err(_) = std::env::var("PASETO_KEY") {
        panic!("找不到Redis");
    }

    let db_url = std::env::var("DATABASE_PUBLIC_URL").unwrap();
    println!("{}", db_url);
    let store = store::Store::new(&db_url).await;


    let mut new_inters = store.clone().get_newinterpretations().await?;
    new_inters.sort_by(|a, b|{
        (a.year, a.number).cmp(&(b.year, b.number))
    });
    let new_inter_shared = Arc::new(new_inters);
    let new_inters_filter = warp::any().map(move || new_inter_shared.clone());

    let mut old_inters = store.clone().get_all_oldinterpretation().await?;
    old_inters.sort_by(|a, b|{
        let a_num = a.id.parse().unwrap_or(0);
        let b_num = b.id.parse().unwrap_or(0);
        a_num.cmp(&b_num)
    });
    let old_inter_shared = Arc::new(old_inters);
    let old_inters_filter = warp::any().map(move || old_inter_shared.clone());

    let mut resolutions = store.clone().get_all_resolution().await?;
    resolutions.sort_by(|a, b|{
        (a.year, a.time).cmp(&(b.year, b.time))
    });
    let resolution_shared = Arc::new(resolutions);
    let resolution_filter = warp::any().map(move || resolution_shared.clone());

    let mut precedents = store.clone().get_all_precedents().await?;
    precedents.sort_by(|a, b|{
        (a.year, a.num).cmp(&(b.year, b.num))
    });
    precedents.reverse();
    let precedents_shared = Arc::new(precedents);
    let pecedent_filter = warp::any().map(move || precedents_shared.clone());


    let store_filter = warp::any().map(move || store.clone());
    let law = Laws::from_pool(&db_url)
        .await
        .map_err(|e| handle_errors::Error::DatabaseQueryError(e))?;
    let laws_shared = Arc::new(law);
    let law_filter = warp::any().map(move || laws_shared.clone());

    let new_law = crate::types::new_law::NewLaws::from_pool(&db_url)
        .await
        .map_err(|e| handle_errors::Error::DatabaseQueryError(e))?;
    let new_laws_shared = Arc::new(new_law.categories(0));
    let new_law_filter = warp::any().map(move || new_laws_shared.clone());

    // 建立redis資料庫聯繫
    let redis_url = std::env::var("REDIS_PUBLIC_URL").unwrap_or("redis://127.0.0.1/".to_string());
    println!("{}", redis_url);
    let client = Client::open(redis_url).unwrap();
    let manager = ConnectionManager::new(client).await.unwrap();
    let redis_filter = warp::any().map(move || manager.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Content-Type", "Authorization"])
        .allow_methods(&[
            Method::PUT,
            Method::DELETE,
            Method::GET,
            Method::POST,
            Method::OPTIONS,
        ]);

    let get_dir = warp::get()
        .and(warp::path("all_dir"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::directory::get_dir_by_user);

    let get_file_list = warp::get()
        .and(warp::path("file_list"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(store_filter.clone())
        .and_then(routes::file::get_file_list);

    let get_note_list = warp::get()
        .and(warp::path("note_list"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(store_filter.clone())
        .and_then(routes::note::get_note_list);

    let update_note = warp::put()
        .and(warp::path("note"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(redis_filter.clone())
        .and(warp::body::json())
        .and_then(routes::note::update_content);

    let get_note = warp::get()
        .and(warp::path("note"))
        .and(warp::path::param::<String>())
        .and(store_filter.clone())
        .and(redis_filter.clone())
        .and_then(routes::note::get_content);

    let add_note = warp::post()
        .and(warp::path("note"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::note::add_note);

    let get_note_nav = warp::get()
        .and(warp::path("note_nav"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::note::get_note_nav);

    let get_every_files = warp::get()
        .and(warp::path("every_file"))
        .and(store_filter.clone())
        .and_then(routes::file::get_every_files);

    let get_every_notes = warp::get()
        .and(warp::path("every_notes"))
        .and(store_filter.clone())
        .and_then(routes::note::get_every_note);

    let get_file_list2 = warp::get()
        .and(warp::path("file_list2"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(store_filter.clone())
        .and_then(routes::file::get_file_list2);

    let delete_dir_by_name = warp::delete()
        .and(warp::path("delete_dir_by_name"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::record::delete_dir_by_name);

    let get_dir_for_pop = warp::get()
        .and(warp::path("dir_for_pop"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::directory::get_dir_for_pop);

    let get_dir_pub = warp::get()
        .and(warp::path("pub_dir"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::directory::get_pub_dir);

    let get_dir_gallery = warp::get()
        .and(warp::path("gallery"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::directory::get_gallery_dir);

    let get_table = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(law_filter.clone())
        .and_then(routes::law::get_table)
        .with(warp::trace(|info| {
            tracing::info_span!(
                "get_questions request",
                method = %info.method(),
                path = %info.path(),
                id = %uuid::Uuid::new_v4(),
            )
        }));

    let get_all_lines = warp::get()
        .and(warp::path("questions"))
        .and(warp::path("all_lines"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(law_filter.clone())
        .and_then(routes::law::get_all_lines);

    let get_all_chapter = warp::get()
        .and(warp::path("allChapter"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(new_law_filter.clone())
        .and_then(routes::new_law::get_all_chapter);

    let get_all_lawList = warp::get()
        .and(warp::path("all_lawList"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(new_law_filter.clone())
        .and_then(routes::new_law::get_all_lawList);

    let get_laws_by_text = warp::get()
        .and(warp::path("laws_by_text"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(law_filter.clone())
        .and_then(routes::law::get_laws_by_text);

    let get_search_chapters = warp::get()
        .and(warp::path("search"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(law_filter.clone())
        .and_then(routes::law::get_search_chapters);

    let get_all_chapters = warp::get()
        .and(warp::path("all_chapters"))
        .and(warp::path::end())
        .and(law_filter.clone())
        .and_then(routes::law::get_all_chapters);

    let add_record = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::record::add_record);

    let get_records_to_laws = warp::get()
        .and(warp::path("records_to_laws"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(law_filter.clone())
        .and_then(routes::record::get_records_to_laws);

    let get_lines_by_chapter = warp::post()
        .and(warp::path("lines_by_chapter"))
        .and(warp::path::end())
        .and(law_filter.clone())
        .and(warp::body::json())
        .and_then(routes::law::get_lines_by_chapter);

    let get_lawList_by_chapter = warp::post()
        .and(warp::path("lawList_by_chapter"))
        .and(warp::path::end())
        .and(new_law_filter.clone())
        .and(warp::body::json())
        .and_then(routes::new_law::get_lawList_by_chapter);

    let get_input_chapter = warp::get()
        .and(warp::path("input_chapter"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(law_filter.clone())
        .and_then(routes::law::get_input_chapter);

    let add_file = warp::post()
        .and(warp::path("file"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::file::add_file);

    let add_dir = warp::post()
        .and(warp::path("dir"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::directory::add_dir);

    let update_content = warp::put()
        .and(warp::path("file"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::file::update_content);

    let update_dir = warp::put()
        .and(warp::path("dir"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::directory::updtae_dir);

    let update_file_name = warp::put()
        .and(warp::path("file_name"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::file::update_file_name);

    let get_content_markdown = warp::get()
        .and(warp::path("file_markdown"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::file::get_content_markdown);

    let get_dir_information = warp::get()
        .and(warp::path("dir_information"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::directory::get_dir_information);

    let get_content_html = warp::get()
        .and(warp::path("file_html"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::file::get_content_html);

    let get_one_law = warp::get()
        .and(warp::path("one_law"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(new_law_filter.clone())
        .and_then(routes::new_law::get_one_law);

    let get_law_lines = warp::get()
        .and(warp::path("law_lines"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(law_filter.clone())
        .and_then(routes::law::get_format_lines);

    let delete_file = warp::delete()
        .and(warp::path("file"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::file::delete_file);

    let get_pdf = warp::get()
        .and(warp::path("pdf"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::file::get_pdf);

    let insert_content = warp::post()
        .and(warp::path("law_block"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and_then(routes::file::insert_content);

    let upload_image = warp::post()
        .and(warp::path("upload_image"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::multipart::form())
        .and_then(routes::file::upload_image);

    let image = warp::post()
        .and(warp::path("image"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::multipart::form())
        .and_then(routes::file::upload_image);

    let registration = warp::post()
        .and(warp::path("registration"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::register);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::login);

    let get_newinters = warp::get()
        .and(warp::path("inter"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::otherlawresource::get_newinter);

    let get_oldinters = warp::get()
        .and(warp::path("oldinter"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::otherlawresource::get_oldinter);

    let get_precedents = warp::get()
        .and(warp::path("precedent"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::otherlawresource::get_precedents);

    let get_precedent_list = warp::get()
        .and(warp::path("precedentlist"))
        .and(warp::path::end())
        .and(pecedent_filter.clone())
        .and_then(routes::otherlawresource::get_precedent_list);

    let get_precedent_by_id = warp::get()
        .and(warp::path!("precedent" / String)) // 動態取得 id
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::otherlawresource::get_precedent_by_id);

    let get_resolutions = warp::get()
        .and(warp::path("resolution"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::otherlawresource::get_resolutions);

    let get_newinter_by_id = warp::get()
        .and(warp::path!("newinterpretation" / String))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::otherlawresource::get_newinter_by_id);

    let get_oldinter_by_id = warp::get()
        .and(warp::path!("oldinterpretation" / String))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::otherlawresource::get_oldinter_by_id);

    let get_resolution_by_id = warp::get()
        .and(warp::path!("resolution" / String))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::otherlawresource::get_resolution_by_id);

    let get_oldinter_list = warp::get()
        .and(warp::path!("oldinterpretationlist"))
        .and(warp::path::end())
        .and(old_inters_filter.clone())
        .and_then(routes::otherlawresource::get_oldinter_list);

    let get_lawname_list = warp::get()
        .and(warp::path!("lawnamelist"))
        .and(warp::path::end())
        .and(new_law_filter.clone())
        .and_then(routes::otherlawresource::get_all_lawname_list);

    let get_newinter_list = warp::get()
        .and(warp::path!("newinterpretationlist"))
        .and(warp::path::end())
        .and(new_inters_filter.clone())
        .and_then(routes::otherlawresource::get_newinter_list);

    let get_resolution_list = warp::get()
        .and(warp::path!("resolutionlist"))
        .and(warp::path::end())
        .and(resolution_filter.clone())
        .and_then(routes::otherlawresource::get_resolution_list);




    let are_you_in_redis = warp::post()
        .and(warp::path("find_token_in_redis"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(routes::authentication::are_you_in_redis);

    // 3. 建立路由：GET /get?key=xxx
    let get_route = warp::path("get")
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(redis_filter.clone())
        .and_then(handle_get);

    // 4. 路由：POST /set?key=xxx&value=yyy
    let set_route = warp::path("set")
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(redis_filter.clone())
        .and_then(handle_set);

    // let static_files = warp::fs::dir("static");

    // 新增靜態文件路由

    let routes = get_all_lines
        .or(get_lawname_list)
        .or(get_precedent_by_id)
        .or(get_newinter_by_id)
        .or(get_oldinter_by_id)
        .or(get_resolution_by_id)
        .or(get_oldinter_list)
        .or(get_precedent_list)
        .or(get_resolution_list)
        .or(get_newinter_list)
        .or(get_newinters)
        .or(get_oldinters)
        .or(get_precedents)
        .or(get_resolutions)
        .or(set_route)
        .or(get_route)
        .or(get_every_notes)
        .or(get_note_nav)
        .or(get_note_list)
        .or(add_note)
        .or(get_note)
        .or(update_note)
        .or(get_input_chapter)
        .or(update_file_name)
        .or(image)
        .or(get_all_lawList)
        .or(update_dir)
        .or(get_dir_information)
        .or(upload_image)
        .or(login)
        .or(add_dir)
        .or(get_law_lines)
        .or(get_dir_pub)
        .or(get_pdf)
        .or(are_you_in_redis)
        .or(get_file_list)
        .or(insert_content)
        .or(add_record)
        .or(get_table)
        .or(registration)
        .or(get_dir)
        .or(get_one_law)
        .or(get_content_markdown)
        .or(get_search_chapters)
        .or(get_all_chapters)
        .or(get_records_to_laws)
        .or(get_lines_by_chapter)
        .or(get_dir_for_pop)
        .or(get_lawList_by_chapter)
        .or(delete_dir_by_name)
        .or(get_content_html)
        .or(add_file)
        .or(update_content)
        .or(delete_file)
        .or(get_laws_by_text)
        .or(get_file_list2)
        .or(get_every_files)
        .or(get_all_chapter)
        .or(get_dir_gallery)
        .with(warp::trace::request()) // 提供靜態文件
        .with(cors)
        .recover(return_error);
    warp::serve(routes).run(([0, 0, 0, 0], config.port)).await;

    /*
    tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(250));

        loop {
            interval.tick().await;
            let redis_url= std::env::var("REDIS_PUBLIC_URL").unwrap();
            let mut redis_database = Redis_Database::new(&redis_url).await
                .map_err(|e| warp::reject::custom(handle_errors::Error::CacheError(e)))?;
            let exists_or_not: Result<Option<String>, redis::RedisError> = redis_database.connection.get(&).await;
            let x =


        }
    });
    */

    Ok(())
}

// GET handler
async fn handle_get(
    params: std::collections::HashMap<String, String>,
    mut redis: ConnectionManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    let key = params.get("key").cloned().unwrap_or_default();
    let result: redis::RedisResult<String> = redis.get(&key).await;

    Ok(match result {
        Ok(val) => format!("Value: {}", val),
        Err(_) => "Key not found".to_string(),
    })
}

// SET handler
async fn handle_set(
    params: std::collections::HashMap<String, String>,
    mut redis: ConnectionManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    let key = params.get("key").cloned().unwrap_or_default();
    let value = params.get("value").cloned().unwrap_or_default();

    let _: redis::RedisResult<()> = redis.set(&key, &value).await;
    Ok(format!("Saved key={} value={}", key, value))
}
