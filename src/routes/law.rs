#[allow(unused_imports)]
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use law_rs::{law, Laws};
use tracing::{instrument, info};


pub async fn get_table(cate: String, num: String, laws: Laws) -> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    let num = percent_decode_str(&num).decode_utf8_lossy();
    info!("獲取{cate}第{num}條");
    let x = format!("{}-{}", cate, num);
    if let Some(l) = laws.lines.iter().find(|&law| law.id == x){
        return Ok(warp::reply::html(l.law_block_result()))
    } else {
        return Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}

#[derive(Deserialize, Serialize)]
pub struct OneLaw {
    chapter: String,
    num: String,
    lines: Vec<String>
}

pub async fn get_on_law(cate: String, num: String, laws: Laws) -> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    let num = percent_decode_str(&num).decode_utf8_lossy();
    info!("獲取{cate}第{num}條");
    let x = format!("{}-{}", cate, num);
    if let Some(l) = laws.lines.iter().find(|&law| law.id == x){
        let one_law = OneLaw{chapter: l.chapter.clone(), num: l.num.clone(), lines: l.line.clone()};
        return Ok(warp::reply::json(&one_law))
    } else {
        return Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}

pub async fn get_all_lines(cate: String, laws: Laws) -> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    let n = laws.all_in_html(cate.to_string());
    Ok(warp::reply::html(n))
}

pub async fn get_all_chapters(laws: Laws) -> Result<impl warp::Reply, warp::Rejection> {
    let mut s = String::new();
    for key in laws.categories(0).keys().filter(|&chapter| chapter != "") {
        let format_key = format!("<option value='{}'>", key);
        s.push_str(&format_key);
    }
    Ok(warp::reply::html(s))
}

pub async fn get_input_chapter(cate1: String, laws: Laws)-> Result<impl warp::Reply, warp::Rejection> {
    let cate1 = percent_decode_str(&cate1).decode_utf8_lossy();
    let mut buffer = String::new();
    let cate = cate1.to_string();
    if let Some(laws) = laws.categories(0).get(&cate){
        let _ = laws.chapter_inputs_html("".to_string(), 1, &mut buffer);
    }
    Ok(warp::reply::html(buffer))
}

pub async fn get_search_chapters(cate: String, laws: Laws)-> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    println!("{cate}");
    info!("dddd");
    let n = laws.search_in_html_chapter(cate.to_string());
    Ok(warp::reply::html(n))
}

#[derive(Deserialize, Serialize)]
pub struct Chapter {
    chapter1: String,
    chapter2: String,
}

pub async fn get_lines_by_chapter(laws: Laws, chapter: Chapter,) -> Result<impl warp::Reply, warp::Rejection> {
    let s = laws.chapter_lines_in_html(chapter.chapter1, chapter.chapter2);
    Ok(warp::reply::html(s))
}
