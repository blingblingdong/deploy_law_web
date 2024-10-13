#[allow(unused_imports)]
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use warp::http::StatusCode;
use tracing::{instrument, info};
use crate::types::file::{File};
use crate::types::record;
use pulldown_cmark::{html, Options, Parser};
use crate::routes::record::NoteUpdate;
use crate::store::Store;


pub async fn add_file(store: Store, file: File) -> Result<impl warp::Reply, warp::Rejection> {
    match store.add_file(file).await {
        Ok(file) => {
            info!("成功新增：{}", file.id);
            Ok(warp::reply::with_status("file added", StatusCode::OK))
        },
        Err(e) => Err(warp::reject::custom(e))
    }
}

pub async fn get_content_markdown(id: String, stroe: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    match stroe.get_file(id.to_string()).await {
        Ok(file) => {
            info!("成功獲取：{}", file.id);
            Ok(warp::reply::json(&file))
        },
        Err(e) => Err(warp::reject::custom(e))
    }
}

pub async fn get_content_html(id: String, stroe: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    match stroe.get_file(id.to_string()).await {
        Ok(file) => {
            info!("成功獲取：{}", file.id);
            let parser = Parser::new_ext(&file.content, Options::all());
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);
            let json_file = File { id: file.id, content: html_output, css: file.css, user_name: file.user_name, directory: file.directory};
            Ok(warp::reply::json(&json_file))
        },
        Err(e) => Err(warp::reject::custom(e))
    }
}

#[derive(Deserialize, Serialize)]
pub struct UpdateContent {
    content: String,
}

pub async fn update_content(id: String, stroe: Store, contnet: UpdateContent) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    let res = match stroe.update_content(id.to_string(), contnet.content).await {
        Ok(file) => {
            info!("成功更新筆記：{}",file.id);
            file
        },
        Err(e) => return Err(warp::reject::custom(e))
    };
    Ok(warp::reply::json(&res))
}

#[derive(Deserialize, Serialize)]
pub struct UpdateCss{
    css: String,
}

pub async fn update_css(id: String, stroe: Store, css: crate::routes::file::UpdateCss) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    let res = match stroe.update_css(id.to_string(), css.css).await {
        Ok(file) => {
            info!("成功更新css：{}",file.id);
            file
        },
        Err(e) => return Err(warp::reject::custom(e))
    };
    Ok(warp::reply::json(&res))
}





pub async fn delete_file(id: String, stroe: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    let res = match stroe.delete_file(id.to_string()).await {
        Ok(file) => {
            info!("成功刪除筆記：{}",file.id);
            file
        },
        Err(e) => return Err(warp::reject::custom(e))
    };
    Ok(warp::reply::json(&res))
}

#[derive(Deserialize, Serialize)]
pub struct LawBlock {
    old_content: String,
    new_content: String
}

pub async fn insert_content(law_block: LawBlock) -> Result<impl warp::Reply, warp::Rejection> {
    let new_content = law_block.old_content.replace("law-card-insertion-place", law_block.new_content.as_str());
    info!("get嗨嗨嗨");
    Ok(warp::reply::html(new_content))
}