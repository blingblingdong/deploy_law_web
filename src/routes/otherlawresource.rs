use crate::store::Store;
use indexmap::IndexMap;
use otherlawresource::{OldInterpretation, OtherSourceList, Precedent, Resolution};
use percent_encoding::percent_decode_str;
use std::sync::Arc;

pub async fn get_newinter(store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    match store.get_newinterpretations().await {
        Ok(n) => Ok(warp::reply::json(&n)),
        _ => Err(warp::reject::custom(handle_errors::Error::QuestionNotFound)),
    }
}

pub async fn get_newinter_list(
    vec: Arc<Vec<otherlawresource::NewInter>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let list: Vec<_> = vec
        .iter()
        .map(|item| {
            let name = format!("{}憲判{}", item.year, item.number);
            otherlawresource::OtherSourceList {
                id: item.id.clone(),
                name,
                sourcetype: "newinterpretation".to_string(),
            }
        })
        .collect();

    Ok(warp::reply::json(&list))
}

pub async fn get_newinter_by_id(
    id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    match store.get_newinterpretation_by_id(id.to_string()).await {
        Ok(data) => Ok(warp::reply::json(&data)),
        Err(_) => Err(warp::reject::custom(handle_errors::Error::QuestionNotFound)),
    }
}

pub async fn get_oldinter(store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    match store.get_all_oldinterpretation().await {
        Ok(data) => Ok(warp::reply::json(&data)),
        _ => Err(warp::reject::custom(handle_errors::Error::QuestionNotFound)),
    }
}

pub async fn get_oldinter_list(
    vec: Arc<Vec<OldInterpretation>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let list: Vec<_> = vec
        .iter()
        .map(|item| OtherSourceList {
            id: item.id.clone(),
            name: format!("釋字{}", item.id.clone()),
            sourcetype: "oldinterpretation".to_string(),
        })
        .collect();

    Ok(warp::reply::json(&list))
}

pub async fn get_oldinter_by_id(
    id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    match store.get_oldinter_by_id(id.to_string()).await {
        Ok(data) => Ok(warp::reply::json(&data)),
        Err(_) => Err(warp::reject::custom(handle_errors::Error::QuestionNotFound)),
    }
}

pub async fn get_precedent_list(
    vec: Arc<Vec<Precedent>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let list: Vec<_> = vec
        .iter()
        .map(|item| OtherSourceList {
            id: item.id.clone(),
            name: item.name.clone(),
            sourcetype: "precedent".to_string(),
        })
        .collect();

    Ok(warp::reply::json(&list))
}

// GET /precedent/{id}
pub async fn get_precedent_by_id(
    id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    match store.get_precedent_by_id(id.to_string()).await {
        Ok(p) => Ok(warp::reply::json(&p)),
        Err(_) => Err(warp::reject::custom(handle_errors::Error::QuestionNotFound)),
    }
}

pub async fn get_precedents(store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    match store.get_all_precedents().await {
        Ok(data) => Ok(warp::reply::json(&data)),
        _ => Err(warp::reject::custom(handle_errors::Error::QuestionNotFound)),
    }
}

pub async fn get_resolutions(store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    match store.get_all_resolution().await {
        Ok(data) => Ok(warp::reply::json(&data)),
        _ => Err(warp::reject::custom(handle_errors::Error::QuestionNotFound)),
    }
}

pub async fn get_resolution_list(
    vec: Arc<Vec<Resolution>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let list: Vec<_> = vec
        .iter()
        .map(|item| OtherSourceList {
            id: item.id.clone(),
            name: item.name.clone(),
            sourcetype: "resolution".to_string(),
        })
        .collect();

    Ok(warp::reply::json(&list))
}

pub async fn get_resolution_by_id(
    id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = percent_decode_str(&id).decode_utf8_lossy();
    match store.get_resolution_by_id(id.to_string()).await {
        Ok(data) => Ok(warp::reply::json(&data)),
        Err(_) => Err(warp::reject::custom(handle_errors::Error::QuestionNotFound)),
    }
}

pub async fn get_all_lawname_list(
    map: Arc<IndexMap<String, new_law::NewLaws>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut buffer = Vec::new();
    for name in map.keys() {
        buffer.push(OtherSourceList {
            id: name.clone(),
            name: name.clone(),
            sourcetype: "lawname".to_string(),
        })
    }
    Ok(warp::reply::json(&buffer))
}

pub async fn get_note_list_user(store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    match store.get_notelist_user().await {
        Ok(list) => Ok(warp::reply::json(&list)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

pub async fn get_folder_list_user(store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    match store.get_every_folder().await {
        Ok(list) => Ok(warp::reply::json(&list)),
        Err(_) => Err(warp::reject::custom(handle_errors::Error::QuestionNotFound)),
    }
}
