use crate::store::Store;
use crate::types::new_law::*;
#[allow(unused_imports)]
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use indexmap::IndexMap;
use tracing::{info, instrument};


pub async fn get_one_law(
    cate: String,
    num: String,
    map: Arc<IndexMap<String, NewLaws>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    let num = percent_decode_str(&num).decode_utf8_lossy();
    info!("獲取{cate}第{num}條");
    let x = format!("{}-{}", cate, num);
    let laws =
        map.get(&cate.to_string()).ok_or(LawError::NOThisChapter).map_err(|_| warp::reject::custom(handle_errors::Error::QuestionNotFound))?;
    if let Some(l) = laws.lines.iter().find(|&law| law.id == x) {
        Ok(warp::reply::json(&l))
    } else {
        Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}

pub async fn get_all_lawList(
    cate: String,
    map: Arc<IndexMap<String, NewLaws>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    let mut laws =
        map.get(&cate.to_string()).ok_or(LawError::NOThisChapter)
            .map_err(|_| warp::reject::custom(handle_errors::Error::QuestionNotFound))?.to_owned();
    laws.lines.sort_by(|a, b| {
        to_f32(a.num.clone())
            .partial_cmp(&to_f32(b.num.clone()))
            .unwrap()
    });
    match laws.lawList_create() {
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
    map: Arc<IndexMap<String, NewLaws>>,
    chapter: Chapter,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut laws =
        map.get(&chapter.chapter1.to_string()).ok_or(LawError::NOThisChapter)
            .map_err(|_| warp::reject::custom(handle_errors::Error::QuestionNotFound))?.to_owned();
    laws.lines.sort_by(|a, b| {
        to_f32(a.num.clone())
            .partial_cmp(&to_f32(b.num.clone()))
            .unwrap()
    });
    let s = laws.lawList_by_chapter(chapter.chapter1, chapter.chapter2);
    if s.is_ok() {
        Ok(warp::reply::json(&s.unwrap()))
    } else {
        Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}

pub async fn get_all_chapter(
    chapter: String,
    map: Arc<IndexMap<String, NewLaws>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let chapter = percent_decode_str(&chapter).decode_utf8_lossy();
    let mut laws =
        map.get(&chapter.to_string()).ok_or(LawError::NOThisChapter)
            .map_err(|_| warp::reject::custom(handle_errors::Error::QuestionNotFound))?.to_owned();
    laws.lines.sort_by(|a, b| {
        to_f32(a.num.clone())
            .partial_cmp(&to_f32(b.num.clone()))
            .unwrap()
    });
    let s = laws.get_chapterUlList();
    if s.is_ok() {
        Ok(warp::reply::json(&s.unwrap()))
    } else {
        Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}

fn to_f32(s: String) -> f32 {
    if s.contains("-") {
        let (big, small) = s.split_once("-").unwrap();
        let big_number: f32 = big.parse().unwrap();
        let small_number: f32 = small.parse().unwrap();
        big_number + small_number * 0.1
    } else {
        s.parse().unwrap()
    }
}


