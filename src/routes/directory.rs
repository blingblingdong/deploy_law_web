use percent_encoding::percent_decode_str;
use reqwest::StatusCode;
use tracing::info;
use crate::store::Store;
use crate::types::account::Session;
use crate::types::directory::Directory;

pub async fn get_dir_by_user(user_name: String, stroe: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let mut s = String::new();
    let user_name = percent_decode_str(&user_name).decode_utf8_lossy();
    let dirs = stroe.get_directory_user(&user_name.to_owned()).await?;
    dirs.iter()
        .map(|k| {format!("<li class='the-dir'><a>{}<a></li>", k.directory)})
        .for_each(|str| {
                s.push_str(&str);
        });
    Ok(warp::reply::html(s))
}

pub async fn add_dir(store: Store, directory: Directory) -> Result<impl warp::Reply, warp::Rejection> {
    match store.add_directory(directory).await {
        Ok(dir) => {
            info!("成功新增：{}",dir.id);
            Ok(warp::reply::with_status("Directory added", StatusCode::OK))
        },
        Err(e) => Err(warp::reject::custom(e))
    }
}

pub async fn get_dir_for_pop(user_name: String, stroe: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let mut s = String::new();
    let user_name = percent_decode_str(&user_name).decode_utf8_lossy();
    println!("{user_name}");
    let dirs = stroe.get_directory_user(&user_name.to_owned()).await?;
    dirs.iter()
        .map(|k| {format!("<div class='option'><input type='checkbox' id='option-{}'>
                            <label for='option-{}'>{}</label></div>", k.directory, k.directory, k.directory)})
        .for_each(|str| {
            println!("{str}");
            s.push_str(&str);
        });
    Ok(warp::reply::html(s))
}

