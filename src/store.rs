use argon2::Config;
use log::error;
use rand::Rng;
use sqlx::{PgPool, Row};
use sqlx::postgres::{PgPoolOptions, PgRow};
use crate::types::account::Account;
use crate::types::file::{File, Files};
use crate::types::record::{LawRecord, LawRecords};
use crate::types::directory::Directory;

#[derive(Clone)]
pub struct Store {
    pub connection: PgPool, //設定一個連接池
}

impl Store {
    pub async fn new(db_url: &str) -> Self {
        let db_pool = match PgPoolOptions::new()
            .max_connections(5) // 最多可以同時連接5個
            .connect(db_url).await {
            Ok(pool) => pool,
            Err(e) => panic!("無法連接上池：{e}"),
        };
        Store {
            connection: db_pool
        }
    }



    pub async fn add_file(&self, file: File) -> Result<File, handle_errors::Error> {
        match sqlx::query(
            "INSERT INTO file (id, content, css, user_name, directory, file_name)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, content, css, user_name, directory, file_name"
        ).bind(file.id)
            .bind(file.content)
            .bind(file.css)
            .bind(file.user_name)
            .bind(file.directory)
            .bind(file.file_name)
            .map(|row: PgRow| File {
                id: row.get("id"),
                content: row.get("content"),
                css: row.get("css"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                file_name: row.get("file_name")
            })
            .fetch_one(&self.connection)
            .await{
            Ok(file) => Ok(file),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }

    pub async fn get_file_user(&self, user_name: &str, directory: &str) -> Result<Files, handle_errors::Error>{
        match sqlx::query("SELECT * from file
        WHERE user_name = $1 AND directory = $2")
            .bind(user_name)
            .bind(directory)
            .map(|row: PgRow| File {
                id: row.get("id"),
                content: row.get("content"),
                css: row.get("css"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                file_name: row.get("file_name"),
            })
            .fetch_all(&self.connection)
            .await{
            Ok(file) => Ok(Files{vec_files: file}),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }

    pub async fn get_file(&self, id: String) -> Result<File, handle_errors::Error>{
        match sqlx::query("SELECT id, content, css, user_name, directory, file_name
        FROM file
        WHERE id = $1;")
            .bind(id)
            .map(|row: PgRow| File {
                id: row.get("id"),
                content: row.get("content"),
                css: row.get("css"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                file_name: row.get("file_name"),
            })
            .fetch_one(&self.connection)
            .await{
            Ok(file) => Ok(file),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }

    pub async fn update_content(&self, id:String, content: String) -> Result<File, handle_errors::Error> {
        match sqlx::query(
            "UPDATE file
            SET content = $1
            WHERE id = $2
            RETURNING id, content, css, user_name, directory, file_name;"
        )
            .bind(content)
            .bind(id)
            .map(|row: PgRow| File {
                id: row.get("id"),
                content: row.get("content"),
                css: row.get("css"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                file_name: row.get("file_name"),
            })
            .fetch_one(&self.connection)
            .await{
            Ok(file) => Ok(file),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }

    pub async fn update_css(&self, id:String, css: String) -> Result<File, handle_errors::Error> {
        match sqlx::query(
            "UPDATE file
            SET css = $1
            WHERE id = $2
            RETURNING id, content, css, user_name, directory, file_name;"
        )
            .bind(css)
            .bind(id)
            .map(|row: PgRow| File {
                id: row.get("id"),
                content: row.get("content"),
                css: row.get("css"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                file_name: row.get("file_name"),
            })
            .fetch_one(&self.connection)
            .await{
            Ok(file) => Ok(file),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }

    pub async fn delete_file(&self, id: String) -> Result<File, handle_errors::Error> {
        match sqlx::query(
            "DELETE FROM file
            Where id = $1
            RETURNING id, content, css, user_name, directory, file_name;"
        )
            .bind(id)
            .map(|row: PgRow| File {
                id: row.get("id"),
                content: row.get("content"),
                css: row.get("css"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                file_name: row.get("file_name")
            })
            .fetch_one(&self.connection)
            .await {
            Ok(file) => Ok(file),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }

    pub async fn get_all_records(&self) -> Result<LawRecords, handle_errors::Error>{
        match sqlx::query("SELECT * from records")
            .map(|row: PgRow| LawRecord {
                id: row.get("id"),
                chapter: row.get("chapter"),
                num: row.get("num"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                note: row.get("note"),
            })
            .fetch_all(&self.connection)
            .await{
            Ok(records) => Ok(LawRecords{vec_record: records}),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }

    pub async fn get_by_user(&self, user_name: &str, directory: &str) -> Result<LawRecords, handle_errors::Error>{
        match sqlx::query("SELECT * from records
        WHERE user_name = $1 AND directory = $2")
            .bind(user_name)
            .bind(directory)
            .map(|row: PgRow| LawRecord {
                id: row.get("id"),
                chapter: row.get("chapter"),
                num: row.get("num"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                note: row.get("note"),
            })
            .fetch_all(&self.connection)
            .await{
            Ok(records) => Ok(LawRecords{vec_record: records}),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }

    pub async fn add_directory(&self, directory: Directory) -> Result<Directory, handle_errors::Error> {
        match sqlx::query(
            "INSERT INTO directory (id, user_name, directory, public, description)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_name, directory, public, description"
        ).bind(directory.id)
            .bind(directory.user_name)
            .bind(directory.directory)
            .bind(directory.public)
            .bind(directory.description)
            .map(|row: PgRow| Directory {
                id: row.get("id"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                public: row.get("public"),
                description: row.get("description"),
            })
            .fetch_one(&self.connection)
            .await{
            Ok(directory) => Ok(directory),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }

    pub async fn get_directory(self, id: &str) -> Result<Directory, handle_errors::Error>{
        match sqlx::query("SELECT * from directory
        WHERE id = $1")
            .bind(id)
            .map(|row: PgRow| Directory {
                id: row.get("id"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                public: row.get("public"),
                description: row.get("description"),
            })
            .fetch_one(&self.connection)
            .await{
            Ok(directory) => Ok(directory),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }

    pub async fn get_directory_user(self, user_name: &str) -> Result<Vec<Directory>, handle_errors::Error>{
        match sqlx::query("SELECT * from directory
        WHERE user_name = $1")
            .bind(user_name)
            .map(|row: PgRow| Directory {
                id: row.get("id"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                public: row.get("public"),
                description: row.get("description"),
            })
            .fetch_all(&self.connection)
            .await{
            Ok(directory) => Ok(directory),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }



    pub async fn add_records(&self, record:LawRecord) -> Result<LawRecord, handle_errors::Error> {
        match sqlx::query(
            "INSERT INTO records (id, chapter, num, user_name, directory, note)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, chapter, num, user_name, directory, note"
        ).bind(record.id)
            .bind(record.chapter)
            .bind(record.num)
            .bind(record.user_name)
            .bind(record.directory)
            .bind(record.note)
            .map(|row: PgRow| LawRecord {
                id: row.get("id"),
                chapter: row.get("chapter"),
                num: row.get("num"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                note: row.get("note"),
            })
            .fetch_one(&self.connection)
            .await{
            Ok(record) => Ok(record),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }

    pub async fn update_note(&self, id:String, note: String) -> Result<LawRecord, handle_errors::Error> {
        match sqlx::query(
            "UPDATE records
            SET note = $1
            WHERE id = $2
            RETURNING id, chapter, num, user_name, directory, note;"
        )
            .bind(note)
            .bind(id)
            .map(|row: PgRow| LawRecord {
                id: row.get("id"),
                chapter: row.get("chapter"),
                num: row.get("num"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                note: row.get("note"),
            })
            .fetch_one(&self.connection)
            .await{
            Ok(record) => Ok(record),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }

    pub async fn delete_by_dir(&self, dir: &str) -> Result<LawRecords, handle_errors::Error> {
        match sqlx::query(
            "DELETE FROM records
            Where directory = $1;"
        )
            .bind(dir)
            .map(|row: PgRow| LawRecord {
                id: row.get("id"),
                chapter: row.get("chapter"),
                num: row.get("num"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                note: row.get("note"),
            })
            .fetch_all(&self.connection)
            .await {
            Ok(records) => Ok(LawRecords{vec_record: records}),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }

    pub async fn add_account(self, account: Account) -> Result<bool, handle_errors::Error> {

        match sqlx::query(
            "INSERT INTO accounts (user_name, email, password)
            VALUES ($1, $2, $3)"
        ).bind(account.user_name)
            .bind(account.email)
            .bind(account.password)
            .execute(&self.connection)
            .await {
            Ok(_) => Ok(true),
            Err(error) => {
                tracing::event!(
                    tracing::Level::ERROR,
                    code = error
                        .as_database_error()
                        .unwrap()
                        .code()
                        .unwrap()
                        .parse::<i32>()
                        .unwrap(),
                    db_message =
                        error.as_database_error().unwrap().message(),
                    constraint = error
                        .as_database_error()
                        .unwrap()
                        .constraint()
                        .unwrap()
                );
                Err(handle_errors::Error::DatabaseQueryError(error))
            }
        }
    }

    pub async fn get_account(self, email: String) -> Result<Account, handle_errors::Error> {
        match sqlx::query("SELECT * from accounts WHERE email = $1")
            .bind(email)
            .map(|row: PgRow| Account {
                user_name: row.get("user_name"),
                email: row.get("email"),
                password: row.get("password"),
            })
            .fetch_one(&self.connection)
            .await{
            Ok(account) => Ok(account),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e))
        }
    }
}

