use argon2::Error as ArgonError;
use redis::RedisError;
use reqwest::Error as ReqwestError;
use std::io::Error as stdIoError;
use tracing::{event, instrument, Level};
#[allow(unused_imports)]
use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden},
    http::StatusCode,
    reject::Reject,
    Rejection, Reply,
};

#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    QuestionNotFound,
    DatabaseQueryError(sqlx::Error),
    ExternalAPIError(ReqwestError),
    ArgonLibraryError(ArgonError),
    WrongPassword,
    CannotDecryptToken,
    Unauthorized,
    TokenNotFound,
    CacheError(RedisError),
    StdFileErroor(stdIoError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ParseError(ref err) => write!(f, "Cannot parse parameter: {}", err),
            Error::MissingParameters => write!(f, "Missing parameter"),
            Error::QuestionNotFound => write!(f, "Question not found"),
            Error::DatabaseQueryError(ref e) => {
                write!(f, "Query not be excuted {}", e)
            }
            Error::TokenNotFound => {
                write!(f, "沒找到token!")
            }
            Error::Unauthorized => {
                write!(f, "認證錯誤")
            }
            Error::WrongPassword => {
                write!(f, "密碼錯誤")
            }
            Error::CannotDecryptToken => {
                write!(f, "密碼錯誤")
            }
            Error::ArgonLibraryError(_) => {
                write!(f, "無法驗證帳號")
            }
            Error::CacheError(ref e) => {
                write!(f, "redis錯誤{}", e)
            }
            Error::ExternalAPIError(ref err) => {
                write!(f, "cannot execute: {}", err)
            }
            Error::StdFileErroor(ref err) => {
                write!(f, "std file 錯誤: {}", err)
            }
        }
    }
}

impl Reject for Error {}

const DUPLICATE_KET: u32 = 23505;

#[instrument]
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<Error>() {
        event!(Level::ERROR, "{}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(error) = r.find::<RedisError>() {
        event!(Level::ERROR, "{}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        event!(Level::ERROR, "{}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        event!(Level::ERROR, "{}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(crate::Error::ExternalAPIError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::StdFileErroor(e)) = r.find() {
        event!(Level::ERROR, "{}", "stdFile讀寫錯誤", e);
        Ok(warp::reply::with_status(
            "密碼錯誤".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else if let Some(crate::Error::WrongPassword) = r.find() {
        event!(Level::ERROR, "{}", "密碼錯誤");
        Ok(warp::reply::with_status(
            "密碼錯誤".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else if let Some(crate::Error::TokenNotFound) = r.find() {
        event!(Level::ERROR, "{}", "找不到token錯誤");
        Ok(warp::reply::with_status(
            "找不到token錯誤".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(crate::Error::DatabaseQueryError(e)) = r.find() {
        event!(Level::ERROR, "{}", "Database query error");

        match e {
            sqlx::Error::Database(err) => {
                if err.code().unwrap().parse::<u32>().unwrap() == DUPLICATE_KET {
                    Ok(warp::reply::with_status(
                        "已經存在相同帳號".to_string(),
                        StatusCode::UNPROCESSABLE_ENTITY,
                    ))
                } else {
                    Ok(warp::reply::with_status(
                        "無法更新資料".to_string(),
                        StatusCode::UNPROCESSABLE_ENTITY,
                    ))
                }
            }
            _ => Ok(warp::reply::with_status(
                "無法更新資料".to_string(),
                StatusCode::UNPROCESSABLE_ENTITY,
            )),
        }
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}
