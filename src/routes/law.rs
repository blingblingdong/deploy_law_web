use crate::store::Store;
use law_rs::{law, Laws};
#[allow(unused_imports)]
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{info, instrument};

pub async fn get_table(
    cate: String,
    num: String,
    laws: Arc<Laws>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    let num = percent_decode_str(&num).decode_utf8_lossy();
    info!("獲取{cate}第{num}條");
    let x = format!("{}-{}", cate, num);
    if let Some(l) = laws.lines.iter().find(|&law| law.id == x) {
        Ok(warp::reply::html(l.law_block_result()))
    } else {
        Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}

#[derive(Deserialize, Serialize)]
pub struct OneLaw {
    id: String,
    href: String,
    chapter: Vec<String>,
    num: String,
    lines: Vec<String>,
}

fn indent(c: char) -> bool {
    let mut set = HashSet::new();
    set.extend([
        '一', '二', '三', '四', '五', '六', '七', '八', '九', '十', '第',
    ]);
    set.contains(&c)
}

pub async fn get_format_lines(
    cate: String,
    num: String,
    laws: Arc<Laws>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    let num = percent_decode_str(&num).decode_utf8_lossy();
    info!("獲取{cate}第{num}條");
    let x = format!("{}-{}", cate, num);
    if let Some(l) = laws.lines.iter().find(|&law| law.id == x) {
        let mut s2 = String::new();
        l.line.iter().for_each(|x| {
            s2.push_str(x);
            s2.push_str("/");
        });
        let line: String = s2
            .split(|c| c == '：' || c == '/')
            .filter(|(s)| !s.is_empty())
            .map(|(s)| {
                if s.starts_with(indent) {
                    format!("<div class='law-indent'>{s}</div>")
                } else {
                    format!("<li class='law-block-line'>{s}</li>")
                }
            })
            .collect();
        let lines = format!("<ul class='law-block-lines'>{}</div>", line);
        Ok(warp::reply::html(lines))
    } else {
        Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}

pub async fn get_one_law(
    cate: String,
    num: String,
    laws: Arc<Laws>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    let num = percent_decode_str(&num).decode_utf8_lossy();
    info!("獲取{cate}第{num}條");
    let x = format!("{}-{}", cate, num);
    if let Some(l) = laws.lines.iter().find(|&law| law.id == x) {
        let chapter: Vec<String> = l.chapter.split("/").map(|s| s.to_string()).collect();
        let lines: Vec<String> = l
            .line
            .clone()
            .into_iter()
            .map(|line| {
                if (line.starts_with(indent)) {
                    format!(" {}", line)
                } else {
                    line
                }
            })
            .collect();
        Ok(warp::reply::json(&OneLaw {
            id: l.id.clone(),
            chapter: chapter,
            num: l.num.clone(),
            href: l.href.clone(),
            lines,
        }))
    } else {
        Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}

pub async fn get_all_lines(
    cate: String,
    laws: Arc<Laws>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    match laws.all_in_html(cate.to_string()) {
        Ok(n) => Ok(warp::reply::html(n)),
        _ => Err(warp::reject::custom(handle_errors::Error::QuestionNotFound)),
    }
}

pub async fn get_all_lawList(
    cate: String,
    laws: Arc<Laws>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    match laws.lawList_create(cate.to_string()) {
        Ok(n) => Ok(warp::reply::json(&n)),
        _ => Err(warp::reject::custom(handle_errors::Error::QuestionNotFound)),
    }
}

pub async fn get_all_chapters(laws: Arc<Laws>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut s = String::new();
    for key in laws.categories(0).keys().filter(|&chapter| chapter != "") {
        let format_key = format!("<option value='{}'>", key);
        s.push_str(&format_key);
    }
    Ok(warp::reply::html(s))
}

pub async fn get_input_chapter(
    cate1: String,
    laws: Arc<Laws>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let cate1 = percent_decode_str(&cate1).decode_utf8_lossy();
    let mut buffer = String::new();
    let cate = cate1.to_string();
    if let Some(laws) = laws.categories(0).get(&cate) {
        let _ = laws.chapter_inputs_html("".to_string(), 1, &mut buffer);
    }

    if buffer.is_empty() {
        return Err(warp::reject::custom(handle_errors::Error::QuestionNotFound));
    }
    Ok(warp::reply::html(buffer))
}

pub async fn get_search_chapters(
    cate: String,
    laws: Arc<Laws>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let cate = percent_decode_str(&cate).decode_utf8_lossy();
    let n = laws.search_in_html_chapter(cate.to_string());
    if n.is_ok() {
        Ok(warp::reply::html(n.unwrap()))
    } else {
        Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}

#[derive(Deserialize, Serialize)]
pub struct Chapter {
    chapter1: String,
    chapter2: String,
}

pub async fn get_lines_by_chapter(
    laws: Arc<Laws>,
    chapter: Chapter,
) -> Result<impl warp::Reply, warp::Rejection> {
    let s = laws.chapter_lines_in_html(chapter.chapter1, chapter.chapter2);
    if s.is_ok() {
        Ok(warp::reply::html(s.unwrap()))
    } else {
        Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}

pub async fn get_lawList_by_chapter(
    laws: Arc<Laws>,
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
    laws: Arc<Laws>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let chapter = percent_decode_str(&chapter).decode_utf8_lossy();
    let s = laws.get_chapterUlList(chapter.to_string());
    if s.is_ok() {
        Ok(warp::reply::json(&s.unwrap()))
    } else {
        Err(warp::reject::custom(handle_errors::Error::QuestionNotFound))
    }
}

pub async fn get_laws_by_text(
    chapter: String,
    text: String,
    laws: Arc<Laws>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let chapter = percent_decode_str(&chapter).decode_utf8_lossy();
    let text = percent_decode_str(&text).decode_utf8_lossy();
    match laws.find_by_text(chapter.to_string(), text.to_string()) {
        Ok(law_text) => {
            let mut buffer = String::new();
            for law in law_text.lines {
                buffer.push_str(&law.law_block());
            }
            Ok(warp::reply::html(buffer))
        }
        _ => Err(warp::reject::custom(handle_errors::Error::QuestionNotFound)),
    }
}
