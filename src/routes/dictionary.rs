use crate::store::Store;
use crate::types::dictionary::{Dictionary, VocabItem, VocabItemLaw};
use percent_encoding::percent_decode_str;

pub async fn get_dictionary(
    dictionary_id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let dictionary_id = percent_decode_str(&dictionary_id).decode_utf8_lossy();
    let dic = store.get_dictionary(dictionary_id.as_ref()).await?;
    Ok(warp::reply::json(&dic))
}

pub async fn get_dictionary_by_user(
    user_name: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_name = percent_decode_str(&user_name).decode_utf8_lossy();
    let dic = store.get_dictionary_by_user(user_name.as_ref()).await?;
    Ok(warp::reply::json(&dic))
}

pub async fn add_dictionary(
    dictionary_name: String,
    user_name: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let name = percent_decode_str(&dictionary_name)
        .decode_utf8_lossy()
        .to_string();
    let user_name = percent_decode_str(&user_name)
        .decode_utf8_lossy()
        .to_string();
    let id = uuid::Uuid::new_v4().to_string();
    let res = store.add_dictionary(&user_name, &name, &id).await?;
    Ok(warp::reply::json(&res))
}

pub async fn delete_dictionary(
    dictionary_id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&dictionary_id).decode_utf8_lossy();
    let res = store.delete_dictionary(id.as_ref()).await?;
    Ok(warp::reply::json(&res))
}

pub async fn delete_vocabitem(
    vocabitem_id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&vocabitem_id).decode_utf8_lossy();
    let res = store.delete_vocabitem(id.as_ref()).await?;
    Ok(warp::reply::json(&res))
}

pub async fn add_vocabitem(
    store: Store,
    item: VocabItem,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = uuid::Uuid::new_v4().to_string();
    let mut newitem = item;
    newitem.id = id;
    let res = store.add_vocabitem(newitem).await?;
    Ok(warp::reply::json(&res))
}

pub async fn update_vocabitem(
    store: Store,
    item: VocabItem,
) -> Result<impl warp::Reply, warp::Rejection> {
    let res = store.update_vocabitem(item).await?;
    Ok(warp::reply::json(&res))
}

pub async fn get_vocabitems_by_dictionary(
    dictionary_id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&dictionary_id).decode_utf8_lossy();
    let items = store.get_vocabitem_dictionary(id.as_ref()).await?;
    Ok(warp::reply::json(&items))
}

pub async fn get_vocabitems_by_term(
    term: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let term = percent_decode_str(&term).decode_utf8_lossy();
    let real_term = format!("%{}%", term);
    let items = store.get_vocabitem_term(&real_term).await?;
    Ok(warp::reply::json(&items))
}

pub async fn get_vocabitems_by_definition(
    keyword: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let keyword = percent_decode_str(&keyword).decode_utf8_lossy();
    let real_keyword = format!("%{}%", keyword);
    let items = store.get_vocabitem_def(&real_keyword).await?;
    Ok(warp::reply::json(&items))
}

pub async fn get_vocabitems_by_user(
    user_name: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_name = percent_decode_str(&user_name).decode_utf8_lossy();
    let items = store.get_vocabitem_user(user_name.as_ref()).await?;
    Ok(warp::reply::json(&items))
}

pub async fn add_vocabitem_law(
    store: Store,
    mapping: VocabItemLaw,
) -> Result<impl warp::Reply, warp::Rejection> {
    let res = store.add_vocabitem_law(mapping).await?;
    Ok(warp::reply::json(&res))
}

pub async fn get_vocabitems_by_law(
    lawid: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let lawid = percent_decode_str(&lawid).decode_utf8_lossy();
    let items = store.get_vocabitems_by_law_id(lawid.as_ref()).await?;
    Ok(warp::reply::json(&items))
}

pub async fn get_lawid_by_vocabitem(
    itemid: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let itemid = percent_decode_str(&itemid).decode_utf8_lossy();
    let items = store.get_law_ids_by_vocabitem(itemid.as_ref()).await?;
    Ok(warp::reply::json(&items))
}

pub async fn get_laws_by_vocabitem(
    itemid: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let itemid = percent_decode_str(&itemid).decode_utf8_lossy();
    let items = store.get_laws_by_vocabitem(itemid.as_ref()).await?;
    Ok(warp::reply::json(&items))
}

pub async fn delete_vocabitemlaw(
    itemid: String,
    lawid: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let itemid = percent_decode_str(&itemid).decode_utf8_lossy();
    let lawid = percent_decode_str(&lawid).decode_utf8_lossy();
    store
        .delete_vocabitem_law(itemid.as_ref(), lawid.as_ref())
        .await?;
    Ok(warp::reply::with_status(
        "刪除成功",
        warp::http::StatusCode::OK,
    ))
}
