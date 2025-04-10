use crate::types::new_law::*;
#[allow(unused_imports)]
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{info, instrument};




pub async fn get_one_law(
    cate: String,
    num: String,
    laws: Arc<NewLaws>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    let num = percent_decode_str(&num).decode_utf8_lossy();
    info!("獲取{cate}第{num}條");
    let x = format!("{}-{}", cate, num);
    if let Some(l) = laws.lines.iter().find(|&law| law.id == x) {
        Ok(warp::reply::json(&l))
    } else {
        Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}



pub async fn get_all_lawList(
    cate: String,
    laws: Arc<NewLaws>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    match laws.lawList_create(cate.to_string()) {
        Ok(n) => Ok(warp::reply::json(&n)),
        _ => Err(warp::reject::custom(handle_errors::Error::QuestionNotFound)),
    }
}




#[derive(Deserialize, Serialize)]
pub struct Chapter {
    chapter1: String,
    chapter2: String,
}


pub async fn get_lawList_by_chapter(
    laws: Arc<NewLaws>,
    chapter: Chapter,
) -> Result<impl warp::Reply, warp::Rejection> {
    let s = laws.lawList_by_chapter(chapter.chapter1, chapter.chapter2);
    if s.is_ok() {
        Ok(warp::reply::json(&s.unwrap()))
    } else {
        Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}

pub async fn get_all_chapter(
    chapter: String,
    laws: Arc<NewLaws>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let chapter = percent_decode_str(&chapter).decode_utf8_lossy();
    let s = laws.get_chapterUlList(chapter.to_string());
    if s.is_ok() {
        Ok(warp::reply::json(&s.unwrap()))
    } else {
        Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}


