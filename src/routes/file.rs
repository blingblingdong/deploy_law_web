#[allow(unused_imports)]
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use warp::http::StatusCode;
use tracing::{instrument, info};
use bytes::BufMut;
use crate::types::file::{File};
use crate::types::record;
use pulldown_cmark::{html, Options, Parser};
use uuid::Uuid;
use warp::hyper::client;
use crate::routes::record::NoteUpdate;
use crate::store::Store;
use futures::{StreamExt, TryStreamExt};


pub async fn add_file(store: Store, file: File) -> Result<impl warp::Reply, warp::Rejection> {
    match store.add_file(file).await {
        Ok(file) => {
            info!("成功新增：{}", file.id);
            Ok(warp::reply::with_status("file added", StatusCode::OK))
        },
        Err(e) => Err(warp::reject::custom(e))
    }
}

#[derive(Deserialize, Serialize)]
pub struct Image {
    name: String,
    bucket: String,
    generation: String,
    metageneration: String,
    contentType: String,
    timeCreated: String,
    updated: String,
    storageClass: String,
    size: String,
    md5Hash: String,
    contentEncoding: String,
    contentDisposition: String,
    crc32c: String,
    etag: String,
    downloadTokens: String,

}

#[derive(Deserialize, Serialize)]
pub struct ImageUrl {
    url: String
}

use warp::hyper::body::Bytes;
use warp::multipart;
use warp::multipart::FormData;

pub async fn upload_image(user_name: String, directory: String, form: FormData) -> Result<impl warp::Reply, warp::Rejection> {
    // 解碼名稱與目錄
    let user_name = percent_decode_str(&user_name).decode_utf8_lossy();
    let directory = percent_decode_str(&directory).decode_utf8_lossy();
    let file_name = format!("{}.jpg", Uuid::new_v4());
    let url = format!(
        "https://firebasestorage.googleapis.com/v0/b/rust-law-web-frdata.appspot.com/o?name={}/{}/{}",
        user_name, directory, file_name
    );

    let mut value = Vec::new();
    let mut parts= form.into_stream();
    if let Some(Ok(p)) =parts.next().await {
         value = p
            .stream()
            .try_fold(Vec::new(), |mut vec, data| {
                vec.put(data);
                async move { Ok(vec) }
            })
            .await
            .map_err(|e| {
                eprintln!("reading file error: {}", e);
                warp::reject::reject()
            })?;
    }


    // 發送請求
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .body(value.clone()) // 使用 multipart 表單上傳
        .send()
        .await
        .map_err(|_| warp::reject())?; // 處理錯誤

    if response.status().is_success() {
        // 解析 Firebase 回應 JSON
        let image_info: Image = response.json().await.map_err(|_| warp::reject())?;

        // 生成下載 URL
        let download_url = format!(
            "https://firebasestorage.googleapis.com/v0/b/{}/o/{}%2F{}%2F{}?alt=media&token={}",
            image_info.bucket, user_name, directory, file_name, image_info.downloadTokens
        );

        // 回傳下載 URL
        Ok(warp::reply::json(&ImageUrl { url: download_url }))
    } else {
        println!("圖片上傳失敗: {}", response.status());
        Err(warp::reject::custom(handle_errors::Error::TokenNotFound))
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
            let json_file = File { id: file.id, content: html_output, css: file.css, user_name: file.user_name, directory: file.directory, file_name: file.file_name};
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

pub async fn get_file_list(user_name: String, dir: String, stroe: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let mut s = String::new();
    let user_name = percent_decode_str(&user_name).decode_utf8_lossy();
    let dir = percent_decode_str(&dir).decode_utf8_lossy();
    let files = stroe.get_file_user(&user_name.to_owned(), &dir.to_owned()).await?;
    files.vec_files.iter()
        .map(|file| {format!("<li class='the-file'><a>{}<a></li>", file.file_name)})
        .for_each(|str| {
            s.push_str(&str);
        });
    Ok(warp::reply::html(s))
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