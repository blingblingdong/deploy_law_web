#[allow(unused_imports)]
use percent_encoding::percent_decode_str;
use serde::Deserialize;
use warp::http::StatusCode;
use law_rs::Laws;
use crate::types::record;
use crate::store::Store;
use tracing::{instrument, info};
use crate::types::account::Session;

#[derive(Deserialize)]
pub struct NoteUpdate {
    note: String,
}

pub async fn add_record(store: Store, law_record: record::LawRecord) -> Result<impl warp::Reply, warp::Rejection> {
    match store.add_records(law_record).await {
        Ok(record) => {
            info!("成功新增：{}",record.id);
            Ok(warp::reply::with_status("Records added", StatusCode::OK))
        },
        Err(e) => Err(warp::reject::custom(e))
    }
}

pub async fn update_note(id: String, session: Session, store: Store, note: NoteUpdate) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    let vec = id.split("-").collect::<Vec<&str>>();
    let user_name = vec.first().unwrap();
    if user_name == &session.user_name {
        let res = match store.update_note(id.to_string(), note.note).await {
            Ok(record) => {
                info!("成功更新筆記：{}",record.id);
                record
            },
            Err(e) => return Err(warp::reject::custom(e))
        };
        Ok(warp::reply::json(&res))
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }
}



pub async fn get_records_to_laws(user_name: String, directory: String, stroe: Store, laws: Laws) -> Result<impl warp::Reply, warp::Rejection> {
    let mut s = String::new();
    let user_name = percent_decode_str(&user_name).decode_utf8_lossy();
    let directory = percent_decode_str(&directory).decode_utf8_lossy();
    let res =stroe.get_by_user(&user_name.to_owned(), &directory.to_owned()).await?;
    if res.vec_record.len() == 1 {
        s.push_str("<h2>尚無加入任何法條</h2>");
    } else {
        for (law, note) in res.get_laws(laws) {
            let block = law.law_block_delete(note);
            s.push_str(&block);
        }
    }
    // 製作一個新增的card
    s.push_str("<div class='law-card'>");
    s.push_str("<div class='card-law-up'>");
    s.push_str("<div class='card-law-content'>");
    s.push_str("<div class='card-law-chapter'>新增法條</div>");
    s.push_str("<div class='card-law-lines'>");
    s.push_str("<form class='card-add-form'><input list='law-name-data' id='card-form-chapter'></input><input id='card-form-num' placeholder='條目' required></input><button type='submit'>新增</button></form>");
    s.push_str("</div></div></div></div>");
    Ok(warp::reply::html(s))
}

/*
pub async fn get_dir(user_name: String,session: Session,stroe: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let mut s = String::new();
    let user_name = percent_decode_str(&user_name).decode_utf8_lossy();
    if user_name == session.user_name {
        let records = stroe.get_by_user(&user_name.to_owned()).await?;
        let map = records.categorize_by_dir()?;
        map.keys()
            .map(|k| {format!("<li class='the-dir'><a>{}<a></li>", k)})
            .for_each(|str| {
                s.push_str(&str);
            });
        Ok(warp::reply::html(s))
    }else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }

}
 */



pub async fn delete_dir_by_name(dir: String, stroe: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let dir = percent_decode_str(&dir).decode_utf8_lossy();
    println!("刪除{dir}");
    let x = stroe.delete_by_dir(&dir.to_owned()).await?;
    Ok(warp::reply::with_status("Records delete", StatusCode::OK))
}




