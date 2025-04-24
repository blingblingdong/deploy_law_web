use crate::routes::record::NoteUpdate;
use crate::store::Store;
use crate::types::account::Redis_Database;
use crate::types::file::File;
use crate::types::note::Note;
use crate::types::record;
use bytes::BufMut;
use futures::{StreamExt, TryStreamExt};
use handle_errors::Error;
use lol_html::element;
use lol_html::{html_content::ContentType, HtmlRewriter, Settings};
use note::{Block, InlineNode};
#[allow(unused_imports)]
use percent_encoding::percent_decode_str;
use pulldown_cmark::{html, Options, Parser};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use select::document::Document;
use select::predicate::Predicate;
use select::predicate::{Class, Name};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::from_value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;
use tokio::fs::File as OtherFile;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, instrument};
use uuid::Uuid;
use warp::http::Response;
use warp::http::StatusCode;

pub async fn add_note(
    store: Store,
    note: crate::types::note::Note,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.add_note(note).await {
        Ok(note) => {
            info!("成功新增：{}", note.id);
            Ok(warp::reply::json(&note))
        }
        Err(e) => Err(warp::reject::custom(e)),
    }
}

use flate2::read::GzDecoder;
use flate2::{write::GzEncoder, Compression};
use std::io::Read;
use std::io::Write;

pub async fn get_gzip_json<T: for<'de> serde::Deserialize<'de>>(
    redis: &mut ConnectionManager,
    key: &str,
) -> redis::RedisResult<T> {
    let compressed: Vec<u8> = redis.get(key).await?;

    let mut decoder = GzDecoder::new(compressed.as_slice());
    let mut json_str = String::new();
    decoder.read_to_string(&mut json_str)?;

    let value = serde_json::from_str(&json_str).unwrap();
    Ok(value)
}

pub async fn set_gzip_json<T: serde::Serialize>(
    redis: &mut ConnectionManager,
    key: &str,
    value: &T,
) -> redis::RedisResult<()> {
    // 序列化為 JSON
    let json = serde_json::to_string(value).unwrap();
    println!("原本的 length: {} bytes", json.len(),);

    // GZIP 壓縮
    let start = Instant::now();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(json.as_bytes())?;
    let compressed = encoder.finish()?;
    println!("壓縮耗時{:?}", start.elapsed());
    println!("壓縮後的 length: {} bytes", compressed.len());

    // 寫入 Redis（注意傳的是 binary）
    redis.set(key, compressed).await
}

pub async fn get_content(
    id: String,
    store: Store,
    mut redis: ConnectionManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    let redisResult: Result<Vec<Block>, redis::RedisError> = get_gzip_json(&mut redis, &id).await;
    match redisResult {
        Ok(block) => {
            let parts: Vec<&str> = id.split("-").collect();
            let writerName = parts[0];
            let dirNmae = parts[1];
            let noteName = parts[2];
            let note = Note {
                id: id.to_string(),
                directory: dirNmae.to_string(),
                user_name: writerName.to_string(),
                footer: None,
                content: Some(serde_json::to_value(&block).unwrap()),
                file_name: noteName.to_string(),
                public: true
            };
            return Ok(warp::reply::json(&note));
        }
        Err(_) => {
            println!("not in redis");
            match store.get_note(id.to_string()).await {
                Ok(note) => {
                    println!("成功獲取：{}", note.id);
                    Ok(warp::reply::json(&note))
                }
                Err(e) => Err(warp::reject::custom(e)),
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateContent {
    content: String,
}

fn gzip_string(data: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(data.as_bytes()).unwrap();
    encoder.finish().unwrap()
}





use redis::pipe;
use tracing_subscriber::fmt::format;

pub async fn update_content(
    id: String,
    mut redis: ConnectionManager,
    content: UpdateContent,
) -> Result<impl warp::Reply, warp::Rejection> {

    let id = percent_decode_str(&id).decode_utf8_lossy();
    let content = update_nav(content.content);
    let blocks = note::parse_note(&content);
    let json = serde_json::to_string(&blocks).unwrap();

    let _ = redis.sadd("noteIdSet", &id).await.map_err(|e| warp::reject::custom(handle_errors::Error::CacheError(e)))?;
    let compressed = gzip_string(&json);
    let _ = redis.set(&id, &compressed).await.map_err(|e| warp::reject::custom(handle_errors::Error::CacheError(e)))?;

    let parts: Vec<&str> = id.split("-").collect();
    let writerName = parts[0];
    let dirNmae = parts[1];
    let noteName = parts[2];

    let note = Note {
        id: id.to_string(),
        directory: dirNmae.to_string(),
        user_name: writerName.to_string(),
        footer: None,
        content: Some(serde_json::to_value(blocks).unwrap()),
        file_name: noteName.to_string(),
        public: true
    };

    Ok(warp::reply::json(&note))
}

pub async fn update_name(
    id: String,
    newname: String,
    store: Store,
    mut redis: ConnectionManager,
) -> Result<impl warp::Reply, warp::Rejection> {

    let id = percent_decode_str(&id).decode_utf8_lossy();
    let newname = percent_decode_str(&newname).decode_utf8_lossy();

    let parts: Vec<&str> = id.split("-").collect();
    let writerName = parts.get(0).unwrap_or(&"no");
    let dirName = parts.get(1).unwrap_or(&"no");

    match store.get_note_name_by_dir(writerName, dirName).await {
        Ok(names) => {
            if names.contains(&newname.to_string()) {
                // 先暫時忽略錯誤類型，反正如果有重名，則回傳拒絕
                return Err(warp::reject::custom(handle_errors::Error::CannotDecryptToken))
            }
        },
        Err(e) => {
            // 如果根本不到資料夾或用戶，都會在這步被退回
            // e是handle_errors::Error::DatabaseQueryError(e)
            return Err(warp::reject::custom(e))
        }
    }


    // 1.先查看有沒有在redis裡
    // 2.如果有：
    // 2.1先獲取block，並將該銷毀
    // 2.2將block與新名字、id包裹成Note{}，傳進資料庫
    // 2.3將資料回傳前端
    // 3.如果沒有
    // 3.1直接往資料夾更新
    // 3.2將資料回傳
    let redisResult: Result<Vec<Block>, redis::RedisError> = get_gzip_json(&mut redis, &id).await;
    match redisResult {
        Ok(block) => {
            // 2.1銷毀
            redis.del(id.clone()).await.map_err(|e| warp::reject::custom(handle_errors::Error::CacheError(e)))?;

            // 2.2.1更新block
            let oldnote = store.update_the_note(serde_json::to_value(&block).unwrap(), id.to_string()).await?;
            let newid = format!("{}-{}-{newname}", oldnote.user_name, oldnote.directory);
            //2.2.2更新筆記名
            let note = store.update_note_name(id.to_string(), newname.to_string(), newid).await?;

            Ok(warp::reply::json(&note))
        }
        Err(_) => {
            info!("not in redis");
            //3.1更新筆記名
            let newid = format!("{writerName}-{dirName}-{newname}");
            let note = store.update_note_name(id.to_string(), newname.to_string(), newid).await?;
            Ok(warp::reply::json(&note))
        }
    }

}

pub async fn update_state(
    id: String,
    state: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {

    let id = percent_decode_str(&id).decode_utf8_lossy();
    let state = percent_decode_str(&state).decode_utf8_lossy().to_string();
    let public: bool;

    if(&state == "true") {
        public = true;
    } else if (&state == "false") {
        public = false;
    } else {
        return Err(warp::reject::custom(handle_errors::Error::CannotDecryptToken))
    }

    match store.update_note_state(id.to_string(), public).await {
        Ok(id) => {
            Ok(warp::reply::Response::new(id.into()))
        },
        Err(e) => {
            Err(warp::reject::custom(e))
        },
    }

}


pub async fn get_every_note(store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let notes = store.get_every_note().await?;
    Ok(warp::reply::json(&notes))
}

pub async fn get_note_list(
    user_name: String,
    dir: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut s = String::new();
    let user_name = percent_decode_str(&user_name).decode_utf8_lossy();
    let dir = percent_decode_str(&dir).decode_utf8_lossy();
    let notes = store
        .get_note_user(&user_name.to_owned(), &dir.to_owned())
        .await?;
    let mut buffer = Vec::new();
    for note in notes {
        buffer.push(note.file_name)
    }
    Ok(warp::reply::json(&buffer))
}

/*
pub async fn get_pdf(
    user_name: String,
    dir: String,
    file_name: String,
    store: Store, // 假設你已經有 Store 型別，它有 get_file 方法
) -> Result<impl warp::Reply, warp::Rejection> {
    // 解碼參數
    let user_name = percent_decode_str(&user_name).decode_utf8_lossy();
    let dir = percent_decode_str(&dir).decode_utf8_lossy();
    let file_name = percent_decode_str(&file_name).decode_utf8_lossy();
    let id = format!("{}-{}-{}", user_name, dir, file_name);

    // 從存儲取得檔案
    let note = store
        .get_note(id)
        .await
        .map_err(|e| warp::reject::custom(e))?;

    // 讀取 CSS 檔案
    let mut css_file = OtherFile::open("new_record.css").await.expect("can't open");
    let mut css = String::new();
    css_file
        .read_to_string(&mut css)
        .await
        .map_err(|e| handle_errors::Error::StdFileErroor(e))?;

    // 組合 HTML 內容
    let format_html = format!(
        "
    <html>
        <head><style>{}</style><meta charset='UTF-8'></head>
        <body style='border:0; padding: 20px;'>
            <div id='public-file-word-area-second'>
                <div id='public-folder-title-bar'>
                    <h1 id='public-folder-file-title'>{}</h1>
                    <div id='writer'>write by : {}</div>
                    <div id='folder'>From：{}</div>
                </div>
                <div id='content-table-area'>
                    <h3>content table</h3>
                    <ul id='content-table'>{}</ul>
                </div>
                <div id='public-folder-ck' class='ck-content ck-editor__editable ck'>{}</div>
            </div>
        </body>
    </html>
    ",
        css, file.file_name, file.user_name, file.directory, file.css, file.content
    );

    let tmp_path = "/tmp/test.html";

    // 寫入 HTML 內容到 /tmp/test.html
    fs::write(tmp_path, format_html.clone()).map_err(|e| {
        eprintln!("Error writing temporary file: {:?}", e);
        handle_errors::Error::TokenNotFound
    })?;

    fs::write("test.html", format_html.clone()).unwrap();

    // 確認檔案存在
    if !Path::new(tmp_path).exists() {
        eprintln!("Temporary file not found: {}", tmp_path);
        return Err(warp::reject::custom(handle_errors::Error::TokenNotFound));
    }

    let wkhtmltopdf_url = std::env::var("WKHTMLTOPDF_URL").unwrap();
    // 使用 wkhtmltopdf 轉換 HTML 為 PDF
    let output = Command::new(wkhtmltopdf_url)
        .arg("--enable-local-file-access")
        .arg("--margin-top")
        .arg("0")
        .arg("--margin-bottom")
        .arg("0")
        .arg("--margin-left")
        .arg("0")
        .arg("--margin-right")
        .arg("0")
        .arg(tmp_path) // 使用臨時檔案作為輸入
        .arg("-") // 輸出到標準輸出
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| {
            eprintln!("Error spawning wkhtmltopdf: {:?}", e);
            handle_errors::Error::TokenNotFound
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("wkhtmltopdf failed: {}", stderr);
        return Err(warp::reject::custom(handle_errors::Error::TokenNotFound));
    }

    if let Err(e) = fs::remove_file(tmp_path) {
        eprintln!("Warning: Failed to remove temporary file: {:?}", e);
    }

    // 返回 PDF 檔案作為回應
    Ok(Response::builder()
        .header("Content-Type", "application/pdf")
        .body(output.stdout)
        .expect("output failed"))
}

 */

pub async fn delete_file(id: String, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    let res = match store.delete_note(&id.to_string()).await {
        Ok(note) => {
            info!("成功刪除筆記：{}", note.id);
            note
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

pub fn update_nav(file_content: String) -> String {
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
    contents
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct H2Nav {
    id: String,
    text: String,
    children: Vec<H3Nav>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct H3Nav {
    id: String,
    text: String,
}

pub async fn get_note_nav(id: String, store: Store, mut redis: ConnectionManager,) -> Result<impl warp::Reply, warp::Rejection> {
    let mut h2NavVec = Vec::new();
    let id = percent_decode_str(&id).decode_utf8_lossy();
    let mut blocks: Vec<Block>;

    let redisResult: Result<Vec<Block>, redis::RedisError> = get_gzip_json(&mut redis, &id).await;
    match redisResult {
        Ok(block) => {
            blocks = block;
        }
        Err(_) => {
            info!("not in redis");
            match store.get_note(id.to_string()).await {
                Ok(note) => {
                    blocks = from_value(note.content.into()).unwrap();
                }
                Err(e) => return Err(warp::reject::custom(e)),
            }
        }
    }

    let mut buffer = H2Nav {
        id: "".to_string(),
        text: "".to_string(),
        children: Vec::new(),
    };

    for block in blocks {
        match block {
            Block::H2 {
                attributes,
                children,
            } => {
                let mut id = "".to_string();
                if let Some(id2) = attributes.unwrap().id {
                    id = id2;
                }
                let mut vec = Vec::new();
                for child in children {
                    match child {
                        InlineNode::Text { text, attributes } => {
                            vec.push(text);
                        }
                        _ => {}
                    }
                }
                h2NavVec.push(buffer.clone());
                buffer.children = Vec::new();
                buffer.id = id.to_string();
                buffer.text = vec.join("").clone();
            }
            Block::H3 {
                attributes,
                children,
            } => {
                let mut id = "".to_string();
                if let Some(id2) = attributes.unwrap().id {
                    id = id2;
                }
                let mut vec = Vec::new();
                for child in children {
                    match child {
                        InlineNode::Text { text, attributes } => {
                            vec.push(text);
                        }
                        _ => {}
                    }
                }
                let h3nav = H3Nav {
                    id: id.to_string(),
                    text: vec.join("").clone(),
                };
                buffer.children.push(h3nav);
            }
            _ => {}
        }
    }
    h2NavVec.push(buffer.clone());
    Ok(warp::reply::json(&h2NavVec))
}
