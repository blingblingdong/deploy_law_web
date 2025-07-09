#![recursion_limit = "512"]
pub mod routes;
mod store;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client, RedisError, RedisResult};
pub mod types;
use crate::routes::note::get_gzip_json;
use config::Config;
#[allow(unused_imports)]
use handle_errors::return_error;
use note::Block;
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
    let dirlist = store.clone().get_directory_pub().await.unwrap();
    for dir in dirlist {
        let notelist = store.get_note_user(&dir.user_name, &dir.directory).await.unwrap()
            .iter().map(|note| note.file_name.clone()).collect::<Vec<String>>();
        let _ = store.clone().update_note_order(dir.id, notelist).await.unwrap();
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

    // 建立redis資料庫聯繫
    let redis_url = std::env::var("REDIS_PUBLIC_URL").unwrap_or("redis://127.0.0.1/".to_string());
    println!("{}", redis_url);
    let client = Client::open(redis_url).unwrap();
    let manager = ConnectionManager::new(client).await.unwrap();

    let mut new_inters = store.clone().get_newinterpretations().await?;
    new_inters.sort_by(|a, b| (a.year, a.number).cmp(&(b.year, b.number)));
    let new_inter_shared = Arc::new(new_inters);
    let new_inters_filter = warp::any().map(move || new_inter_shared.clone());

    let mut old_inters = store.clone().get_all_oldinterpretation().await?;
    old_inters.sort_by(|a, b| {
        let a_num = a.id.parse().unwrap_or(0);
        let b_num = b.id.parse().unwrap_or(0);
        a_num.cmp(&b_num)
    });
    old_inters.reverse();
    let old_inter_shared = Arc::new(old_inters);
    let old_inters_filter = warp::any().map(move || old_inter_shared.clone());

    let mut resolutions = store.clone().get_all_resolution().await?;
    resolutions.sort_by(|a, b| (a.year, a.time).cmp(&(b.year, b.time)));
    let resolution_shared = Arc::new(resolutions);
    let resolution_filter = warp::any().map(move || resolution_shared.clone());

    let mut precedents = store.clone().get_all_precedents().await?;
    precedents.sort_by(|a, b| (a.year, a.num).cmp(&(b.year, b.num)));
    precedents.reverse();
    let precedents_shared = Arc::new(precedents);
    let pecedent_filter = warp::any().map(move || precedents_shared.clone());

    let new_law = new_law::NewLaws::from_pool(&db_url)
        .await
        .map_err(|e| handle_errors::Error::DatabaseQueryError(e))?;
    let new_laws_shared = Arc::new(new_law.categories(0));
    let new_law_filter = warp::any().map(move || new_laws_shared.clone());

    let store_filter = warp::any().map(move || store.clone());
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

    let update_note_order = warp::put()
        .and(warp::path("note_order"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::directory::update_note_order);

    let update_note = warp::put()
        .and(warp::path("note"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(redis_filter.clone())
        .and(warp::body::json())
        .and_then(routes::note::update_content);

    let update_note_state = warp::get()
        .and(warp::path("note_state"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::note::update_state);

    let update_note_name = warp::get()
        .and(warp::path("note_name"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(redis_filter.clone())
        .and_then(routes::note::update_name);

    let clean_redis = warp::get()
        .and(warp::path("redis_clean"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(redis_filter.clone())
        .and_then(routes::note::clean_redis);

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

    let delete_note = warp::delete()
        .and(warp::path("note"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::note::delete_note);

    let get_note_nav = warp::get()
        .and(warp::path("note_nav"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(redis_filter.clone())
        .and_then(routes::note::get_note_nav);

    let get_library = warp::get()
        .and(warp::path("library"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::Library::get_library_by_user);

    let get_library_item = warp::get()
        .and(warp::path("library_item"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::Library::get_library_item);

    let add_library_item = warp::post()
        .and(warp::path("library_item"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::Library::add_library_item);

    let add_library = warp::post()
        .and(warp::path("library"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::Library::add_library);

    let get_history_law = warp::get()
        .and(warp::path("historylaw"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::new_law::get_history_law);

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

    let get_note_date = warp::get()
        .and(warp::path("date"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::note::get_note_date);

    let update_note_date = warp::post()
        .and(warp::path("date"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::note::update_note_date);

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

    let get_all_chapters = warp::get()
        .and(warp::path("all_chapters"))
        .and(warp::path::end())
        .and(new_law_filter.clone())
        .and_then(routes::new_law::get_all_chapters);

    let get_lawList_by_chapter = warp::post()
        .and(warp::path("lawList_by_chapter"))
        .and(warp::path::end())
        .and(new_law_filter.clone())
        .and(warp::body::json())
        .and_then(routes::new_law::get_lawList_by_chapter);

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

    let delete_file = warp::delete()
        .and(warp::path("file"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::file::delete_file);

    let delete_folder = warp::delete()
        .and(warp::path("folder"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::directory::delete_dir);

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

    let get_note_order = warp::get()
        .and(warp::path!("note_order" / String))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::directory::get_note_order);

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

    let get_note_list_user = warp::get()
        .and(warp::path("notelist"))
        .and(store_filter.clone())
        .and_then(routes::otherlawresource::get_note_list_user);

    let get_folder_list = warp::get()
        .and(warp::path("folderlist"))
        .and(store_filter.clone())
        .and_then(routes::otherlawresource::get_folder_list_user);

    let are_you_in_redis = warp::post()
        .and(warp::path("find_token_in_redis"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(routes::authentication::are_you_in_redis);

    // 3. 建立路由：GET /get?key=xxx

    // let static_files = warp::fs::dir("static");

    // 新增靜態文件路由

    let routes = clean_redis
        .or(get_library_item)
        .or(get_folder_list)
        .or(get_note_order)
        .or(get_library)
        .or(add_library)
        .or(add_library_item)
        .or(get_history_law)
        .or(update_note_date)
        .or(get_note_date)
        .or(delete_note)
        .or(update_note_state)
        .or(update_note_name)
        .or(get_note_list_user)
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
        .or(get_every_notes)
        .or(get_note_nav)
        .or(get_note_list)
        .or(add_note)
        .or(get_note)
        .or(update_note)
        .or(update_file_name)
        .or(image)
        .or(get_all_lawList)
        .or(update_dir)
        .or(get_dir_information)
        .or(upload_image)
        .or(login)
        .or(add_dir)
        .or(get_dir_pub)
        .or(get_pdf)
        .or(are_you_in_redis)
        .or(get_file_list)
        .or(insert_content)
        .or(registration)
        .or(get_dir)
        .or(get_one_law)
        .or(get_content_markdown)
        .or(get_all_chapters)
        .or(get_dir_for_pop)
        .or(get_lawList_by_chapter)
        .or(get_content_html)
        .or(add_file)
        .or(update_content)
        .or(delete_file)
        .or(delete_folder)
        .or(get_file_list2)
        .or(get_every_files)
        .or(get_all_chapter)
        .or(get_dir_gallery)
        .or(update_note_order)
        .with(warp::trace::request()) // 提供靜態文件
        .with(cors)
        .recover(return_error);
    warp::serve(routes).run(([0, 0, 0, 0], config.port)).await;

    Ok(())
}
