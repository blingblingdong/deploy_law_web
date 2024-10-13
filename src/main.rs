pub mod types;
pub mod routes;
mod store;

#[allow(unused_imports)]
use handle_errors::return_error;
use warp::{http::Method, Filter};
use law_rs::Laws;
use tracing_subscriber::fmt::format::FmtSpan;
use crate::routes::file::{delete_file, get_content_markdown, insert_content, update_content};
use crate::routes::law::get_on_law;
use crate::routes::record::{get_dir_for_pop, update_note};
use crate::store::Store;
use config::Config;
use std::env;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct Args {
    log_level: String,
    database_host: String,
    database_port: u16,
    database_name:String,
    port: u16,
    database_username: String,
    database_password: String,
}

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    let config = Config::builder()
        .add_source(config::File::with_name("setup"))
        .build()
        .unwrap();

    let config = config
        .try_deserialize::<Args>()
        .unwrap();

    let log_filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| {
            format!("handle_errors={},rust_web_dev={},warp={}",
            config.log_level, config.log_level, config.log_level)
        });

    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();



    let db_url = format!("postgres://{}:{}@{}:{}/{}",config.database_username,config.database_password, config.database_host, config.database_port, config.database_name);
    println!("{}", db_url);
    let store = store::Store::new(&db_url).await;
    let store_filter = warp::any().map(move || store.clone());
    let law = Laws::from_pool(&db_url).await
        .map_err(|e| handle_errors::Error::DatabaseQueryError(e))?;

    let law_filter = warp::any().map(move || law.clone());

    let redis_url = "redis://default:YezaUpCuecITVAKhENlObxOddVrcGRHH@autorack.proxy.rlwy.net:33895".to_string();
    let redis_filter = warp::any().map(move || redis_url.clone());



    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Content-Type", "Authorization"])
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_dir = warp::get()
        .and(warp::path("all_dir"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and_then(routes::record::get_dir);

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
        .and_then(routes::record::get_dir_for_pop);

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
            )})
        );

    let get_all_lines = warp::get()
        .and(warp::path("questions"))
        .and(warp::path("all_lines"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(law_filter.clone())
        .and_then(routes::law::get_all_lines);

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

    let get_lines_by_chapter = warp::get()
        .and(warp::path("lines_by_chapter"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(law_filter.clone())
        .and_then(routes::law::get_lines_by_chapter);

    let update_note = warp::put()
        .and(warp::path("update_note"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::record::update_note);

    let add_file = warp::post()
        .and(warp::path("file"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::file::add_file);

    let update_content = warp::put()
        .and(warp::path("file"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::file::update_content);

    let update_css = warp::put()
        .and(warp::path("css"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::file::update_css);

    let get_content_markdown = warp::get()
        .and(warp::path("file_markdown"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::file::get_content_markdown);

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
        .and(law_filter.clone())
        .and_then(routes::law::get_on_law);

    let delete_file = warp::delete()
        .and(warp::path("file"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::file::delete_file);

    let insert_content = warp::post()
        .and(warp::path("law_block"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and_then(routes::file::insert_content);

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
        .and(redis_filter.clone())
        .and_then(routes::authentication::login);

    let are_you_in_redis = warp::post()
        .and(warp::path("find_token_in_redis"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(redis_filter.clone())
        .and_then(routes::authentication::are_you_in_redis);


    // let static_files = warp::fs::dir("static");


    // 新增靜態文件路由

    let routes = get_all_lines
        .or(login)
        .or(are_you_in_redis)
        .or(update_css)
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
        .or(delete_dir_by_name)
        .or(get_content_html)
        .or(update_note)
        .or(add_file)
        .or(update_content)
        .or(delete_file)
        .with(warp::trace::request())// 提供靜態文件
        .with(cors)
        .recover(return_error);

    warp::serve(routes)
        .run(([0, 0, 0, 0], config.port))
        .await;


    Ok(())
}
