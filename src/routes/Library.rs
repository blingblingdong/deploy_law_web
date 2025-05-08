use percent_encoding::percent_decode_str;
use reqwest::StatusCode;
use tracing::info;
use crate::store::Store;
use crate::types::file::File;
use crate::types::Library::{Library, LibraryItem};

pub async fn add_library(user_name: String, library_name: String, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let library = Library{
        id: uuid::Uuid::new_v4().to_string(),
        library_name: percent_decode_str(&library_name).decode_utf8_lossy().to_string(),
        user_name: percent_decode_str(&user_name).decode_utf8_lossy().to_string(),
        public: false,
    };
    match store.add_library(library).await {
        Ok(library) => {
            info!("成功新增：{}", library.id);
            Ok(warp::reply::with_status("file added", StatusCode::OK))
        }
        Err(e) => Err(warp::reject::custom(e)),
    }
}

pub async fn add_library_item(library_id: String, item_id: String, item_type: String, item_name: String, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let item = LibraryItem {
        id: uuid::Uuid::new_v4().to_string(),
        item_library: library_id,
        item_id: percent_decode_str(&item_id).decode_utf8_lossy().to_string(),
        item_type: percent_decode_str(&item_type).decode_utf8_lossy().to_string(),
        item_name: percent_decode_str(&item_name).decode_utf8_lossy().to_string(),
        order: 0,
    };
    match store.add_library_item(item).await {
        Ok(library) => {
            info!("成功新增：{}", library.id);
            Ok(warp::reply::with_status("file added", StatusCode::OK))
        }
        Err(e) => Err(warp::reject::custom(e)),
    }
}

pub async fn get_library_item(library_id: String, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let library_id = percent_decode_str(&library_id).decode_utf8_lossy();
    match store.get_item_by_library(&library_id).await {
        Ok(items) => {
            Ok(warp::reply::json(&items))
        }
        Err(e) => Err(warp::reject::custom(e)),
    }
}

pub async fn get_library_by_user(username: String, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let username = percent_decode_str(&username).decode_utf8_lossy();
    match store.get_library_user(&username).await {
        Ok(library) => {
            Ok(warp::reply::json(&library))
        }
        Err(e) => Err(warp::reject::custom(e)),
    }
}