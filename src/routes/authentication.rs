use std::future;
use argon2::Config;
use chrono::{Duration, Utc};
use paseto::v1::local_paseto;
use rand::Rng;
use redis::AsyncCommands;
use tracing::info;
use warp::{http::StatusCode, Filter};
use crate::store::Store;
use crate::types::account::{Account, Redis_Database, Session};



// 以姓名確認token是否存在
pub async fn are_you_in_redis(you: String) -> Result<impl warp::Reply, warp::Rejection> {
    let redis_url= std::env::var("REDIS_PUBLIC_URL").unwrap();
    let mut redis_database = Redis_Database::new(&redis_url).await
        .map_err(|e| warp::reject::custom(handle_errors::Error::CacheError(e)))?;
    let exists_or_not: Result<Option<String>, redis::RedisError> = redis_database.connection.get(&you).await;

    match exists_or_not {
        Ok(Some(token)) => {
            Ok(warp::reply::json(&token))
        },
        Ok(None) => {
            // 如果没有找到令牌，返回自定义错误
            Err(warp::reject::custom(handle_errors::Error::TokenNotFound))
        },
        Err(e) => {
            // 如果操作中出现了错误，例如连接问题
            eprintln!("Redis error: {}", e);  // 记录错误到 stderr 或日志系统
            Err(warp::reject::custom(handle_errors::Error::CacheError(e)))
        }
    }
}


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
                    let token = issue_token(account.user_name.clone()).await.unwrap();
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

async fn issue_token(
    user_name: String
) -> Result<String, handle_errors::Error> {
    let current_date_time = Utc::now();
    let dt = current_date_time + chrono::Duration::days(1);
    let paseto_key = std::env::var("PASETO_KEY").unwrap();

    let token = paseto::tokens::PasetoBuilder::new()
        .set_encryption_key(
            &Vec::from(paseto_key.as_bytes())
        )
        .set_expiration(&dt)
        .set_not_before(&Utc::now())
        .set_claim("user_name", serde_json::json!(user_name))
        .build()
        .expect("建立令牌失敗");

    let redis_url= std::env::var("REDIS_PUBLIC_URL").unwrap();
    let mut redis_database = Redis_Database::new(&redis_url).await
        .map_err(|e| handle_errors::Error::CacheError(e))?;
    let _: () = redis_database.connection.set_ex(user_name, token.clone(), 86400).await
        .map_err(|e| handle_errors::Error::CacheError(e))?;
    Ok(token)
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