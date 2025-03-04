use crate::routes::record::NoteUpdate;
use crate::store::Store;
use crate::types::file::File;
use crate::types::record;
use bytes::BufMut;
use futures::{StreamExt, TryStreamExt};
use lol_html::element;
use lol_html::{html_content::ContentType, HtmlRewriter, Settings};
#[allow(unused_imports)]
use percent_encoding::percent_decode_str;
use pulldown_cmark::{html, Options, Parser};
use select::document::Document;
use select::predicate::Predicate;
use select::predicate::{Class, Name};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use tracing::{info, instrument};
use uuid::Uuid;
use warp::http::StatusCode;
use warp::hyper::client;

pub async fn add_file(store: Store, file: File) -> Result<impl warp::Reply, warp::Rejection> {
    match store.add_file(file).await {
        Ok(file) => {
            info!("成功新增：{}", file.id);
            Ok(warp::reply::with_status("file added", StatusCode::OK))
        }
        Err(e) => Err(warp::reject::custom(e)),
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
    url: String,
}

use warp::hyper::body::Bytes;
use warp::multipart;
use warp::multipart::FormData;

pub async fn upload_image(
    user_name: String,
    directory: String,
    form: FormData,
) -> Result<impl warp::Reply, warp::Rejection> {
    // 解碼名稱與目錄
    let user_name = percent_decode_str(&user_name).decode_utf8_lossy();
    let directory = percent_decode_str(&directory).decode_utf8_lossy();
    let file_name = format!("{}.jpg", Uuid::new_v4());
    let url = format!(
        "https://firebasestorage.googleapis.com/v0/b/rust-law-web-frdata.appspot.com/o?name={}/{}/{}",
        user_name, directory, file_name
    );

    let mut value = Vec::new();
    let mut parts = form.into_stream();
    if let Some(Ok(p)) = parts.next().await {
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
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build().unwrap();

    let response = client
        .post(&url)
        .body(value.clone()) // 使用 multipart 表單上傳
        .send()
        .await
        .map_err(|e| warp::reject::custom(handle_errors::Error::ExternalAPIError(e)))?; // 處理錯誤

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
        println!("圖片上傳失2: {}", response.status());
        Err(warp::reject::custom(handle_errors::Error::TokenNotFound))
    }
}
pub async fn get_content_markdown(
    id: String,
    stroe: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    match stroe.get_file(id.to_string()).await {
        Ok(file) => {
            info!("成功獲取：{}", file.id);
            Ok(warp::reply::json(&file))
        }
        Err(e) => Err(warp::reject::custom(e)),
    }
}

pub async fn get_content_html(
    id: String,
    stroe: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    match stroe.get_file(id.to_string()).await {
        Ok(file) => {
            info!("成功獲取：{}", file.id);
            let parser = Parser::new_ext(&file.content, Options::all());
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);
            let json_file = File {
                id: file.id,
                content: html_output,
                css: file.css,
                user_name: file.user_name,
                directory: file.directory,
                file_name: file.file_name,
                content_nav: file.content_nav,
            };
            Ok(warp::reply::json(&json_file))
        }
        Err(e) => Err(warp::reject::custom(e)),
    }
}

#[derive(Deserialize, Serialize)]
pub struct UpdateContent {
    content: String,
}

pub async fn update_file_name(
    id: String,
    file_name: String,
    store: Store
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    let file_name = percent_decode_str(&file_name).decode_utf8_lossy();
    let old_id = id.clone();
    let id_vec: Vec<&str> = id.split("-").collect();
    let new_id = format!("{}-{}-{}", id_vec[0], id_vec[1], file_name.clone());
    let res = match store
        .update_file_name(
            old_id.to_string(),
            file_name.to_string(),
            new_id,
        )
        .await
    {
        Ok(file) => {
            info!("成功更新筆記：{}", file.id);
            file
        }
        Err(e) => return Err(warp::reject::custom(e)),
    };
    Ok(warp::reply::json(&res))
}

pub async fn update_content(
    id: String,
    stroe: Store,
    contnet: UpdateContent,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    let vec = update_nav(contnet.content);
    let res = match stroe
        .update_content_and_css(
            id.to_string(),
            vec[0].clone(),
            vec[1].clone(),
            vec[2].clone(),
        )
        .await
    {
        Ok(file) => {
            info!("成功更新筆記：{}", file.id);
            file
        }
        Err(e) => return Err(warp::reject::custom(e)),
    };
    Ok(warp::reply::json(&res))
}

pub async fn get_file_list(
    user_name: String,
    dir: String,
    stroe: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut s = String::new();
    let user_name = percent_decode_str(&user_name).decode_utf8_lossy();
    let dir = percent_decode_str(&dir).decode_utf8_lossy();
    let files = stroe
        .get_file_user(&user_name.to_owned(), &dir.to_owned())
        .await?;
    files
        .vec_files
        .iter()
        .map(|file| format!("<li class='the-file'><a>{}<a></li>", file.file_name))
        .for_each(|str| {
            s.push_str(&str);
        });
    Ok(warp::reply::html(s))
}

pub async fn get_file_list2(
    user_name: String,
    dir: String,
    stroe: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_name = percent_decode_str(&user_name).decode_utf8_lossy();
    let dir = percent_decode_str(&dir).decode_utf8_lossy();
    let files = stroe
        .get_file_user(&user_name.to_owned(), &dir.to_owned())
        .await?;
    let mut vec:Vec<String> = Vec::new();
    files
        .vec_files
        .iter()
        .for_each(|files| {
            vec.push(files.file_name.clone());
        });
    Ok(warp::reply::json(&vec))
}


pub async fn delete_file(id: String, stroe: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    let res = match stroe.delete_file(id.to_string()).await {
        Ok(file) => {
            info!("成功刪除筆記：{}", file.id);
            file
        }
        Err(e) => return Err(warp::reject::custom(e)),
    };
    Ok(warp::reply::json(&res))
}

#[derive(Deserialize, Serialize)]
pub struct LawBlock {
    old_content: String,
    new_content: String,
}

pub async fn insert_content(law_block: LawBlock) -> Result<impl warp::Reply, warp::Rejection> {
    let new_content = law_block
        .old_content
        .replace("law-card-insertion-place", law_block.new_content.as_str());
    info!("get嗨嗨嗨");
    Ok(warp::reply::html(new_content))
}

pub fn update_nav(file_content: String) -> Vec<String> {
    let mut output = Vec::new();
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!("h2, h3", |el| {
                if !el.has_attribute("id") {
                    let id = Uuid::new_v4().to_string();
                    el.set_attribute("id", &id);
                }
                Ok(())
            })],
            ..Settings::default()
        },
        |chunk: &[u8]| output.extend_from_slice(chunk),
    );

    rewriter.write(file_content.as_bytes()).unwrap();
    rewriter.end().unwrap();
    let contents = String::from_utf8(output).unwrap();

    //找出所有h2、h3
    let mut heading_vec = Vec::new();
    let document = Document::from(contents.as_str());
    heading_vec.extend(document.find(Name("h2").or(Name("h3"))).map(|x| {
        let id = x.attr("id").unwrap_or("no");
        let name = x.name().unwrap_or("h2");
        format!(
            "<li><a class='content-table-{}' href='#{}'>{}</a></li>",
            name,
            id,
            x.text()
        )
    }));

    // 找出用到的法律
    let useLaw = findUseLaw(&contents);
    vec![contents, heading_vec.join(""), useLaw]
}

fn findUseLaw(file_content: &str) -> String {
    let document = Document::from(file_content);
    let mut lawhash = LawHash {
        inner: HashMap::new(),
    };
    document
        .find(Class("law-block-chapter-num"))
        .for_each(|node| {
            let mut chapter: String = String::new();
            let mut num: String = String::new();
            if let Some(n) = node.find(Class("law-block-chapter")).next() {
                chapter = n.text();
            };
            if let Some(n) = node.find(Class("law-block-num")).next() {
                num = n.text();
            };
            if !num.is_empty() && !chapter.is_empty() {
                lawhash.insert(chapter, num);
            }
        });

    let s = lawhash.format();
    if s.is_empty() {
        "目前沒有使用任何法條".to_string()
    } else {
        s
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct usinglaw {
    chapter: String,
    num: String,
}

impl usinglaw {
    pub fn new(chapter: String, num: String) -> Self {
        usinglaw { chapter, num }
    }
}

struct LawHash {
    inner: HashMap<String, HashSet<usinglaw>>,
}

impl LawHash {
    pub fn format(self) -> String {
        let mut buffer = String::new();
        for (key, set) in self.inner {
            let ul = format!("<ul class='using-law-chapter'>{}", key);
            buffer.push_str(&ul);
            set.iter().for_each(|law| {
                let li = format!("<li>{}</li>", law.num.clone());
                buffer.push_str(&li);
            });
            buffer.push_str("</ul>")
        }
        buffer
    }

    pub fn insert(&mut self, chapter: String, num: String) {
        self.inner
            .entry(chapter.clone())
            .or_default()
            .insert(usinglaw::new(chapter, num));
    }
}
