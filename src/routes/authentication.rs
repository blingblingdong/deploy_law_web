use std::future;
use argon2::Config;
use chrono::{Duration, Utc};
use paseto::v1::local_paseto;
use rand::Rng;
use tracing::info;
use warp::{http::StatusCode, Filter};
use crate::store::Store;
use crate::types::account::{Account, Session};
use redis::AsyncCommands;

/*
pub async fn redis_connection() -> redis::aio::Connection {
    let client = redis::Client::open("redis://:11131028@localhost:6379").unwrap();
    client.get_async_connection().await.unwrap()
}
*/


pub fn hash_password(password: &[u8]) -> String {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config).unwrap()
}

pub async fn register(store: Store, account: Account) -> Result<impl warp::Reply, warp::Rejection> {
    let password = hash_password(account.password.as_bytes());

    let account = Account {
        user_name: account.user_name,
        email: account.email,
        password
    };

    match store.add_account(account.clone()).await {
        Ok(_) => {
            info!("成功新增帳號：{}",account.user_name);
            Ok(warp::reply::with_status("Account Added", StatusCode::OK))
        },
        Err(e) => Err(warp::reject::custom(e))
    }
}

/*
簡要流程：
1.需要一個Account結構的json，並用user_name來比對數據庫中有沒有相同的user_name
    1.1.如果有，用veryify_password函數來比對password是否正確
        1.1.1 如果正確，將封裝user_name的令牌，令牌是以paseto形式
        1.1.2 若否，則回傳WrongPassword
    1.2 若否，則回傳ArgonLibraryError
*/

pub async fn login(store: Store, login: Account) -> Result<impl warp::Reply, warp::Rejection> {
    match store.get_account(login.user_name).await {
        Ok(account) => match verify_password(
            &account.password,
            login.password.as_bytes()
        ) {
            Ok(verified) => {
                if verified {
                    let token = issue_token(account.user_name.clone());

                    // let mut conn = redis_connection().await;

                    // 儲存令牌到 Redis，設置 24 小時的過期時間
                    // let _: () = conn.set_ex(token.clone(), account.user_name.clone(), 86400).await.unwrap();

                    Ok(warp::reply::json(&token))
                } else {
                    Err(warp::reject::custom(handle_errors::Error::WrongPassword))
                }
            }
            Err(e) => Err(warp::reject::custom(
                handle_errors::Error::ArgonLibraryError(e),
            ))
        },
        Err(e) => Err(warp::reject::custom(e)),
    }
}

fn verify_password(
    hash: &str, // 數據庫中的密碼
    password: &[u8] // 登入流程中的密碼
) -> Result<bool, argon2::Error> {
    // argon2 crate接受字串與數據庫中的哈希值是否相同
    argon2::verify_encoded(hash, password)
}

fn issue_token(
    user_name: String
) -> String {
    let current_date_time = Utc::now();
    let dt = current_date_time + chrono::Duration::days(1);

    paseto::tokens::PasetoBuilder::new()
        .set_encryption_key(
            &Vec::from("RANDOM WORDS WINTER MACINTOSH PC".as_bytes())
        )
        .set_expiration(&dt)
        .set_not_before(&Utc::now())
        .set_claim("user_name", serde_json::json!(user_name))
        .build()
        .expect("建立令牌失敗")
}

pub fn verify_token(token: String) -> Result<Session, handle_errors::Error> {
    let token = paseto::tokens::validate_local_token(
        &token,
        None,// footer
        &"RANDOM WORDS WINTER MACINTOSH PC".as_bytes(), //key
        &paseto::tokens::TimeBackend::Chrono, //backend
    ).map_err(|_| handle_errors::Error::CannotDecryptToken)?;

    serde_json::from_value::<Session>(token).map_err(|_| {
        handle_errors::Error::CannotDecryptToken
    })
}

pub fn auth() -> impl Filter<Extract = (Session,), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization").and_then(|token: String| {
        let token = match verify_token(token) {
            Ok(t) => t,
            Err(_) => return future::ready(Err(warp::reject::reject())),
        };

        future::ready(Ok(token))
    })
}