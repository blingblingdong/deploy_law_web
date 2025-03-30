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
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::{Command, Stdio};
use handle_errors::Error;
use note::{Block, InlineNode};
use serde_json::from_value;
use tokio::fs::File as OtherFile;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, instrument};
use uuid::Uuid;
use warp::http::Response;
use warp::http::StatusCode;
use crate::types::note::Note;

pub async fn add_note(store: Store, note: crate::types::note::Note) -> Result<impl warp::Reply, warp::Rejection> {
    match store.add_note(note).await {
        Ok(note) => {
            info!("成功新增：{}", note.id);
            Ok(warp::reply::with_status("note added", StatusCode::OK))
        }
        Err(e) => Err(warp::reject::custom(e)),
    }
}


pub async fn get_content(
    id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    match store.get_note(id.to_string()).await {
        Ok(note) => {
            info!("成功獲取：{}", note.id);
            Ok(warp::reply::json(&note))
        }
        Err(e) => Err(warp::reject::custom(e)),
    }
}

#[derive(Deserialize, Serialize)]
pub struct UpdateContent {
    content: String,
}


pub async fn update_content(
    id: String,
    store: Store,
    content: UpdateContent,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    let content = update_nav(content.content);
    let jsonContent = note::parse_note(&content);
    let res = match store
        .update_the_note(serde_json::json!(jsonContent), id.to_string())
        .await
    {
        Ok(note) => {
            info!("成功更新筆記：{}", note.id);
            note
        }
        Err(e) => return Err(warp::reject::custom(e)),
    };
    Ok(warp::reply::json(&res))
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
pub struct H2Nav{
    id: String,
    text: String,
    children: Vec<H3Nav>,
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct H3Nav{
    id: String,
    text: String,
}



pub async fn get_note_nav(id: String, store: Store)-> Result<impl warp::Reply, warp::Rejection> {
    let mut h2NavVec = Vec::new();
    let id = percent_decode_str(&id).decode_utf8_lossy();
    let note = store.get_note(id.to_string()).await?;
    let mut buffer = H2Nav{id: "".to_string(), text: "".to_string(), children: Vec::new()};
    if let Some(content) = note.content {
        let blocks: Vec<Block> = from_value(content).unwrap();
        for block in blocks {
            match block {
                Block::H2 { attributes, children} => {
                    let mut id = "".to_string();
                    if let Some(id2) = attributes.unwrap().id {
                        id = id2;
                    }
                    let mut vec = Vec::new();
                    for child in children {
                        match child {
                            InlineNode::Text { text, attributes} => {
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
                Block::H3 { attributes, children } => {
                    let mut id = "".to_string();
                    if let Some(id2) = attributes.unwrap().id {
                        id = id2;
                    }
                    let mut vec = Vec::new();
                    for child in children {
                        match child {
                            InlineNode::Text { text, attributes} => {
                                vec.push(text);
                            }
                            _ => {}
                        }
                    }
                    let h3nav = H3Nav {id: id.to_string(), text: vec.join("").clone()};
                    buffer.children.push(h3nav);
                }
                _ => {}
            }
        }

    }

    /*
    let document = Document::from(content.content.as_str());
    let mut buffer = H2Nav{id: "".to_string(), text: "".to_string(), children: Vec::new()};
    document.find(Name("h2").or(Name("h3"))).for_each(|x| {
        let id = x.attr("id").unwrap_or("no");
        let name = x.name().unwrap_or("h2");
        let text = x.text();
        if name == "h2"   {
            h2NavVec.push(buffer.clone());
            buffer.children = Vec::new();
            buffer.id = id.to_string();
            buffer.text = text.clone();

        } else if name == "h3" {
            let h3nav = H3Nav {id: id.to_string(), text};
            buffer.children.push(h3nav);
        }
    });

     */
    h2NavVec.push(buffer.clone());
    println!("{:#?}", &h2NavVec);
    Ok(warp::reply::json(&h2NavVec))
}