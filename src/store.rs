use crate::types::account::Account;
use crate::types::directory::Directory;
use crate::types::file::{File, Files};
use crate::types::note::Note;
use argon2::Config;
use chrono::Utc;
use log::error;
use rand::Rng;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{PgPool, Row};
use crate::types::Library::{Library, LibraryItem};

#[derive(Clone)]
pub struct Store {
    pub connection: PgPool, //設定一個連接池
}

impl Store {
    pub async fn new(db_url: &str) -> Self {
        let db_pool = match PgPoolOptions::new()
            .max_connections(5) // 最多可以同時連接5個
            .connect(db_url)
            .await
        {
            Ok(pool) => pool,
            Err(e) => panic!("無法連接上池：{e}"),
        };
        Store {
            connection: db_pool,
        }
    }

    pub async fn add_file(&self, file: File) -> Result<File, handle_errors::Error> {
        match sqlx::query(
            "INSERT INTO file (id, content, css, user_name, directory, file_name, content_nav)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, content, css, user_name, directory, file_name, content_nav",
        )
        .bind(file.id)
        .bind(file.content)
        .bind(file.css)
        .bind(file.user_name)
        .bind(file.directory)
        .bind(file.file_name)
        .bind(file.content_nav)
        .map(|row: PgRow| File {
            id: row.get("id"),
            content: row.get("content"),
            css: row.get("css"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            file_name: row.get("file_name"),
            content_nav: row.get("content_nav"),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(file) => Ok(file),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_every_file(&self) -> Result<Files, handle_errors::Error> {
        match sqlx::query("SELECT * from file")
            .map(|row: PgRow| File {
                id: row.get("id"),
                content: row.get("content"),
                css: row.get("css"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                file_name: row.get("file_name"),
                content_nav: row.get("content_nav"),
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(file) => Ok(Files { vec_files: file }),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_every_note(&self) -> Result<Vec<Note>, handle_errors::Error> {
        match sqlx::query("SELECT * from note")
            .map(|row: PgRow| Note {
                id: row.get("id"),
                content: row.get("content"),
                footer: row.get("footer"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                file_name: row.get("file_name"),
                public: row.get("public")
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(note) => Ok(note),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_file_user(
        &self,
        user_name: &str,
        directory: &str,
    ) -> Result<Files, handle_errors::Error> {
        match sqlx::query(
            "SELECT * from file
        WHERE user_name = $1 AND directory = $2",
        )
        .bind(user_name)
        .bind(directory)
        .map(|row: PgRow| File {
            id: row.get("id"),
            content: row.get("content"),
            css: row.get("css"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            file_name: row.get("file_name"),
            content_nav: row.get("content_nav"),
        })
        .fetch_all(&self.connection)
        .await
        {
            Ok(file) => Ok(Files { vec_files: file }),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_file(&self, id: String) -> Result<File, handle_errors::Error> {
        match sqlx::query(
            "SELECT id, content, css, user_name, directory, file_name, content_nav
            FROM file 
            WHERE id = $1;",
        )
        .bind(id)
        .map(|row: PgRow| File {
            id: row.get("id"),
            content: row.get("content"),
            css: row.get("css"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            file_name: row.get("file_name"),
            content_nav: row.get("content_nav"),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(file) => Ok(file),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn update_content(
        &self,
        id: String,
        content: String,
    ) -> Result<File, handle_errors::Error> {
        match sqlx::query(
            "UPDATE file
            SET content = $1
            WHERE id = $2
            RETURNING id, content, css, user_name, directory, file_name, content_nav;",
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
            content_nav: row.get("content_nav"),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(file) => Ok(file),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn update_file_name(
        &self,
        id: String,
        file_name: String,
        new_id: String,
    ) -> Result<File, handle_errors::Error> {
        match sqlx::query(
            "UPDATE file
            SET id = $1, file_name = $2
            WHERE id = $3
            RETURNING id, content, css, user_name, directory, file_name, content_nav;",
        )
        .bind(new_id)
        .bind(file_name)
        .bind(id)
        .map(|row: PgRow| File {
            id: row.get("id"),
            content: row.get("content"),
            css: row.get("css"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            file_name: row.get("file_name"),
            content_nav: row.get("content_nav"),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(file) => Ok(file),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn update_content_and_css(
        &self,
        id: String,
        content: String,
        css: String,
        nav: String,
    ) -> Result<File, handle_errors::Error> {
        match sqlx::query(
            "UPDATE file
            SET content = $1, css = $2, content_nav = $4
            WHERE id = $3
            RETURNING id, content, css, user_name, directory, file_name, content_nav;",
        )
        .bind(content)
        .bind(css)
        .bind(id)
        .bind(nav)
        .map(|row: PgRow| File {
            id: row.get("id"),
            content: row.get("content"),
            css: row.get("css"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            file_name: row.get("file_name"),
            content_nav: row.get("content_nav"),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(file) => Ok(file),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn delete_file(&self, id: String) -> Result<File, handle_errors::Error> {
        match sqlx::query(
            "DELETE FROM file
            Where id = $1
            RETURNING id, content, css, user_name, directory, file_name, content_nav;",
        )
        .bind(id)
        .map(|row: PgRow| File {
            id: row.get("id"),
            content: row.get("content"),
            css: row.get("css"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            file_name: row.get("file_name"),
            content_nav: row.get("content_nav"),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(file) => Ok(file),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }





    pub async fn update_directory(
        &self,
        public: bool,
        description: String,
        id: String,
    ) -> Result<Directory, handle_errors::Error> {
        match sqlx::query(
            "UPDATE directory 
            SET public = $1, description = $2
            WHERE id = $3
            RETURNING id, user_name, directory, public, description",
        )
        .bind(public)
        .bind(description)
        .bind(id)
        .map(|row: PgRow| Directory {
            id: row.get("id"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            public: row.get("public"),
            description: row.get("description"),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(directory) => Ok(directory),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn update_note_name(
        &self,
        id: String,
        file_name: String,
        new_id: String,
    ) -> Result<Note, handle_errors::Error> {
        match sqlx::query(
            "UPDATE note
            SET id = $1, file_name = $2
            WHERE id = $3
             RETURNING id, user_name, directory, file_name, content, footer, public",
        )
            .bind(new_id)
            .bind(file_name)
            .bind(id)
            .map(|row: PgRow| Note {
                id: row.get("id"),
                user_name: row.get("user_name"),
                directory: row.get("directory"),
                file_name: row.get("file_name"),
                footer: row.get("footer"),
                content: row.get("content"),
                public: row.get("public")
            })
            .fetch_one(&self.connection)
            .await
        {
            Ok(note) => Ok(note),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn update_note_state(
        &self,
        id: String,
        public: bool,
    ) -> Result<String, handle_errors::Error> {
        match sqlx::query(
            "UPDATE note
            SET public = $1
            WHERE id = $2
            RETURNING id",
        )
            .bind(public)
            .bind(id)
            .map(|row: PgRow| {
                let id: String = row.get("id");
                id
            })
            .fetch_one(&self.connection)
            .await
        {
            Ok(id) => Ok(id),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_note_user(
        &self,
        user_name: &str,
        directory: &str,
    ) -> Result<Vec<Note>, handle_errors::Error> {
        match sqlx::query(
            "SELECT * from note
        WHERE user_name = $1 AND directory = $2",
        )
        .bind(user_name)
        .bind(directory)
        .map(|row: PgRow| Note {
            id: row.get("id"),
            content: row.get("content"),
            footer: row.get("footer"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            file_name: row.get("file_name"),
            public: row.get("public")
        })
        .fetch_all(&self.connection)
        .await
        {
            Ok(note) => Ok(note),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_note_name_by_dir(
        &self,
        user_name: &str,
        directory: &str,
    ) -> Result<Vec<String>, handle_errors::Error> {
        match sqlx::query(
            "SELECT file_name from note
        WHERE user_name = $1 AND directory = $2",
        )
            .bind(user_name)
            .bind(directory)
            .map(|row: PgRow| {
                let name: String = row.get("file_name");
                name
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(names) => Ok(names),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_notelist_user(
        &self,
        user_name: &str,
    ) -> Result<Vec<otherlawresource::OtherSourceList>, handle_errors::Error> {
        match sqlx::query(
            "SELECT * from note
        WHERE user_name = $1 AND public = $2",
        )
            .bind(user_name)
            .bind(true)
            .map(|row: PgRow| otherlawresource::OtherSourceList {
                id: row.get("id"),
                name: row.get("file_name"),
                sourcetype: "note".to_string(),
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(list) => Ok(list),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn update_the_note(
        &self,
        content: serde_json::Value,
        id: String,
    ) -> Result<Note, handle_errors::Error> {
        match sqlx::query(
            "UPDATE note 
            SET content = $1
            WHERE id = $2
            RETURNING id, user_name, directory, file_name, content, footer, public",
        )
        .bind(content)
        .bind(id)
        .map(|row: PgRow| Note {
            id: row.get("id"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            file_name: row.get("file_name"),
            footer: row.get("footer"),
            content: row.get("content"),
            public: row.get("public")
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(note) => Ok(note),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_note(&self, id: String) -> Result<Note, handle_errors::Error> {
        match sqlx::query(
            "SELECT id, user_name, directory, file_name, content, footer, public
            FROM note
            WHERE id = $1",
        )
        .bind(id)
        .map(|row: PgRow| Note {
            id: row.get("id"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            file_name: row.get("file_name"),
            footer: row.get("footer"),
            content: row.get("content"),
            public: row.get("public")
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(note) => Ok(note),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }



    pub async fn get_note_date(&self, id: String) -> Result<chrono::DateTime<Utc>, handle_errors::Error> {
        match sqlx::query(
            "SELECT date
            FROM note
            WHERE id = $1",
        )
            .bind(id)
            .map(|row: PgRow| {
                let date:chrono::DateTime<Utc> = row.get("date");
                date
            })
            .fetch_one(&self.connection)
            .await
        {
            Ok(date) => Ok(date),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn update_note_date(
        &self,
        id: String,
        date: chrono::DateTime<Utc>
    ) -> Result<String, handle_errors::Error> {
        match sqlx::query(
            "UPDATE note
            SET date = $1
            WHERE id = $2
            RETURNING id",
        )
            .bind(date)
            .bind(id)
            .map(|row: PgRow| {
                let id: String = row.get("id");
                id
            })
            .fetch_one(&self.connection)
            .await
        {
            Ok(id) => Ok(id),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn delete_note(&self, id: &str) -> Result<Note, handle_errors::Error> {
        match sqlx::query(
            "DELETE FROM note
            Where id = $1
            RETURNING id, user_name, directory, file_name, content, footer, public",
        )
        .bind(id)
        .map(|row: PgRow| Note {
            id: row.get("id"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            file_name: row.get("file_name"),
            footer: row.get("footer"),
            content: row.get("content"),
            public: row.get("public")
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(note) => Ok(note),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn add_note(&self, note: Note) -> Result<Note, handle_errors::Error> {
        match sqlx::query(
            "INSERT INTO note (id, user_name, directory, file_name, content, footer, public)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_name, directory, file_name, content, footer, public",
        )
        .bind(note.id)
        .bind(note.user_name)
        .bind(note.directory)
        .bind(note.file_name)
        .bind(note.content)
        .bind(note.footer)
            .bind(note.public)
        .map(|row: PgRow| Note {
            id: row.get("id"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            file_name: row.get("file_name"),
            footer: row.get("footer"),
            content: row.get("content"),
            public: row.get("public")
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(note) => Ok(note),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn add_directory(
        &self,
        directory: Directory,
    ) -> Result<Directory, handle_errors::Error> {
        match sqlx::query(
            "INSERT INTO directory (id, user_name, directory, public, description)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_name, directory, public, description",
        )
        .bind(directory.id)
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
        .await
        {
            Ok(directory) => Ok(directory),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_directory(self, id: &str) -> Result<Directory, handle_errors::Error> {
        match sqlx::query(
            "SELECT * from directory
        WHERE id = $1",
        )
        .bind(id)
        .map(|row: PgRow| Directory {
            id: row.get("id"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            public: row.get("public"),
            description: row.get("description"),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(directory) => Ok(directory),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_directory_user(
        self,
        user_name: &str,
    ) -> Result<Vec<Directory>, handle_errors::Error> {
        match sqlx::query(
            "SELECT * from directory
        WHERE user_name = $1",
        )
        .bind(user_name)
        .map(|row: PgRow| Directory {
            id: row.get("id"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            public: row.get("public"),
            description: row.get("description"),
        })
        .fetch_all(&self.connection)
        .await
        {
            Ok(directory) => Ok(directory),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_directory_pub(self) -> Result<Vec<Directory>, handle_errors::Error> {
        match sqlx::query(
            "SELECT * from directory
        WHERE public = true",
        )
        .map(|row: PgRow| Directory {
            id: row.get("id"),
            user_name: row.get("user_name"),
            directory: row.get("directory"),
            public: row.get("public"),
            description: row.get("description"),
        })
        .fetch_all(&self.connection)
        .await
        {
            Ok(directory) => Ok(directory),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn add_account(self, account: Account) -> Result<bool, handle_errors::Error> {
        match sqlx::query(
            "INSERT INTO accounts (user_name, email, password)
            VALUES ($1, $2, $3)",
        )
        .bind(account.user_name)
        .bind(account.email)
        .bind(account.password)
        .execute(&self.connection)
        .await
        {
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
                    db_message = error.as_database_error().unwrap().message(),
                    constraint = error.as_database_error().unwrap().constraint().unwrap()
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
            .await
        {
            Ok(account) => Ok(account),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_newinterpretations(
        self,
    ) -> Result<Vec<otherlawresource::NewInter>, handle_errors::Error> {
        match sqlx::query("SELECT * from newinters")
            .map(|row: PgRow| otherlawresource::NewInter {
                id: row.get("id"),
                casename: row.get("casename"),
                casesummary: row.get("casesummary"),
                maincontent: row.get("maincontent"),
                date: row.get("date"),
                reason: row.get("reason"),
                related_law: row.get("related_law"),
                source: row.get("source"),
                name: row.get("name"),
                year: row.get("year"),
                number: row.get("number"),
                reflaws: row.get("reflaws"),
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(inters) => Ok(inters),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_newinterpretations_list(
        &self,
    ) -> Result<Vec<otherlawresource::OtherSourceList>, handle_errors::Error> {
        match sqlx::query("SELECT id, name FROM newinters")
            .map(|row: PgRow| {
                let year: String = row.get("year");
                let number: String = row.get("number");
                let name =  format!("{}憲判{}", year, number);
                otherlawresource::OtherSourceList{
                id: row.get("id"),
                name,
                sourcetype: "newinterpretation".to_string(),

            }
                })
            .fetch_all(&self.connection)
            .await
        {
            Ok(list) => Ok(list),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }


        pub async fn get_newinterpretation_by_id(
            &self,
            id: String,
        ) -> Result<otherlawresource::NewInter, handle_errors::Error> {
            match sqlx::query("SELECT * FROM newinters WHERE id = $1")
                .bind(id)
                .map(|row: PgRow| otherlawresource::NewInter {
                    id: row.get("id"),
                    casename: row.get("casename"),
                    casesummary: row.get("casesummary"),
                    maincontent: row.get("maincontent"),
                    date: row.get("date"),
                    reason: row.get("reason"),
                    related_law: row.get("related_law"),
                    source: row.get("source"),
                    name: row.get("name"),
                    year: row.get("year"),
                    number: row.get("number"),
                    reflaws: row.get("reflaws"),
                })
                .fetch_one(&self.connection)
                .await
            {
                Ok(res) => Ok(res),
                Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
            }
        }



    pub async fn get_all_resolution(
        self,
    ) -> Result<Vec<otherlawresource::Resolution>, handle_errors::Error> {
        match sqlx::query("SELECT * FROM resolution")
            .map(|row: PgRow| otherlawresource::Resolution {
                id: row.get("id"),
                lawtype: row.get("lawtype"),
                related_law: row.get("related_law"),
                name: row.get("name"),
                content: row.get("content"),
                source: row.get("source"),
                year: row.get("year"),
                time: row.get("time"),
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(resolutions) => Ok(resolutions),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_resolutions_list(
        &self,
    ) -> Result<Vec<otherlawresource::OtherSourceList>, handle_errors::Error> {
        match sqlx::query("SELECT id, name FROM resolution")
            .map(|row: PgRow| {
                otherlawresource::OtherSourceList{
                    id: row.get("id"),
                    name: row.get("name"),
                    sourcetype: "resolution".to_string()
                }})
            .fetch_all(&self.connection)
            .await
        {
            Ok(list) => Ok(list),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_resolution_by_id(
        &self,
        id: String,
    ) -> Result<otherlawresource::Resolution, handle_errors::Error> {
        match sqlx::query("SELECT * FROM resolution WHERE id = $1")
            .bind(id)
            .map(|row: PgRow| otherlawresource::Resolution {
                id: row.get("id"),
                lawtype: row.get("lawtype"),
                related_law: row.get("related_law"),
                name: row.get("name"),
                content: row.get("content"),
                source: row.get("source"),
                year: row.get("year"),
                time: row.get("time"),
            })
            .fetch_one(&self.connection)
            .await
        {
            Ok(res) => Ok(res),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }



    pub async fn get_all_oldinterpretation(
        &self,
    ) -> Result<Vec<otherlawresource::OldInterpretation>, handle_errors::Error> {
        match sqlx::query("SELECT * FROM oldinters")
            .map(|row: PgRow| otherlawresource::OldInterpretation {
                id: row.get("id"),
                date: row.get("date"),
                reasoning: row.get("reasoning"),
                content: row.get("content"),
                trouble: row.get("trouble"),
                related_law: row.get("related_law"),
                source: row.get("source"),
                reflaws: row.get("reflaws"),
                reflawid: row.get("reflawid"),
                refinter: row.get("refinter"),
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(result) => Ok(result),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_oldinterpretations_list(
        &self,
    ) -> Result<Vec<otherlawresource::OtherSourceList>, handle_errors::Error> {
        match sqlx::query("SELECT id FROM oldinters")
            .map(|row: PgRow| {
                let id: String = row.get("id");
                let name = format!("釋字{}", id.clone());
                otherlawresource::OtherSourceList{
                    id: id.clone(),
                    name: name,
                    sourcetype: "oldinterpretation".to_string()
                }})
            .fetch_all(&self.connection)
            .await
        {
            Ok(list) => Ok(list),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_oldinter_by_id(
        &self,
        id: String,
    ) -> Result<otherlawresource::OldInterpretation, handle_errors::Error> {
        match sqlx::query("SELECT * FROM oldinters WHERE id = $1")
            .bind(id)
            .map(|row: PgRow| otherlawresource::OldInterpretation {
                id: row.get("id"),
                date: row.get("date"),
                reasoning: row.get("reasoning"),
                content: row.get("content"),
                trouble: row.get("trouble"),
                related_law: row.get("related_law"),
                source: row.get("source"),
                reflaws: row.get("reflaws"),
                reflawid: row.get("reflawid"),
                refinter: row.get("refinter"),
            })
            .fetch_one(&self.connection)
            .await
        {
            Ok(res) => Ok(res),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }



    pub async fn get_all_precedents(
        &self,
    ) -> Result<Vec<otherlawresource::Precedent>, handle_errors::Error> {
        match sqlx::query("SELECT * FROM precedents")
            .map(|row: PgRow| otherlawresource::Precedent {
                id: row.get("id"),
                name: row.get("name"),
                holding: row.get("holding"),
                source: row.get("source"),
                year: row.get("year"),
                num: row.get("num"),
                specific: row.get("specific"),
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(precedents) => Ok(precedents),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_precedentlist(
        &self,
    ) -> Result<Vec<otherlawresource::OtherSourceList>, handle_errors::Error> {
        match sqlx::query("SELECT id, name FROM precedents")
            .map(|row: PgRow| otherlawresource::OtherSourceList{
                id: row.get("id"),
                name: row.get("name"),
                sourcetype: "precedent".to_string()
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(list) => Ok(list),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }


    pub async fn get_precedent_by_id(
        &self,
        id: String,
    ) -> Result<otherlawresource::Precedent, handle_errors::Error> {
        match sqlx::query("SELECT * FROM precedents WHERE id = $1")
            .bind(id)
            .map(|row: PgRow| otherlawresource::Precedent {
                id: row.get("id"),
                name: row.get("name"),
                holding: row.get("holding"),
                source: row.get("source"),
                year: row.get("year"),
                num: row.get("num"),
                specific: row.get("specific"),
            })
            .fetch_one(&self.connection)
            .await
        {
            Ok(prec) => Ok(prec),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_historylaw(&self, lawid: String) -> Result<Vec<otherlawresource::HistoryLaw>, handle_errors::Error> {
        match sqlx::query("SELECT * FROM history_law WHERE lawid  = $1")
            .bind(lawid)
            .map(|row: PgRow| otherlawresource::HistoryLaw {
                id: row.get("id"),
                lawid: row.get("lawid"),
                date: row.get("date"),
                content: row.get("content"),
                no: row.get("no"),
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(list) => Ok(list),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn add_library(&self, library: Library) -> Result<Library, handle_errors::Error> {
        match sqlx::query(
            "INSERT INTO library (id, library_name, user_name, public)
         VALUES ($1, $2, $3, $4)
         RETURNING id, library_name, user_name, public"
        )
            .bind(&library.id)
            .bind(&library.library_name)
            .bind(&library.user_name)
            .bind(library.public)
            .map(|row: PgRow| Library {
                id: row.get("id"),
                library_name: row.get("library_name"),
                user_name: row.get("user_name"),
                public: row.get("public"),
            })
            .fetch_one(&self.connection)
            .await {
            Ok(library) => Ok(library),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn add_library_item(&self, item: LibraryItem) -> Result<LibraryItem, handle_errors::Error> {
        match sqlx::query(
            "INSERT INTO library_item (id, item_library, item_type, item_name, item_id, ordering)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, item_library, item_type, item_name, item_id, ordering"
        )
            .bind(&item.id)
            .bind(&item.item_library)
            .bind(&item.item_type)
            .bind(&item.item_name)
            .bind(item.item_id)
            .bind(item.order)
            .map(|row: PgRow| LibraryItem {
                id: row.get("id"),
                item_id: row.get("item_id"),
                item_library: row.get("item_library"),
                item_type: row.get("item_type"),
                item_name: row.get("item_name"),
                order: row.get("ordering"),
            })
            .fetch_one(&self.connection)
            .await {
            Ok(item) => Ok(item),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }


    pub async fn get_library_user(
        self,
        user_name: &str,
    ) -> Result<Vec<Library>, handle_errors::Error> {
        match sqlx::query(
            "SELECT * FROM library
         WHERE user_name = $1",
        )
            .bind(user_name)
            .map(|row: PgRow| Library {
                id: row.get("id"),
                library_name: row.get("library_name"),
                user_name: row.get("user_name"),
                public: row.get("public"),
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(libraries) => Ok(libraries),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }

    pub async fn get_item_by_library(
        self,
        library_id: &str,
    ) -> Result<Vec<LibraryItem>, handle_errors::Error> {
        match sqlx::query(
            "SELECT * FROM library_item
         WHERE item_library = $1
         ORDER BY ordering DESC",
        )
            .bind(library_id)
            .map(|row: PgRow| LibraryItem {
                id: row.get("id"),
                item_library: row.get("item_library"),
                item_type: row.get("item_type"),
                item_id: row.get("item_id"),
                item_name: row.get("item_name"),
                order: row.get("ordering"),
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(items) => Ok(items),
            Err(e) => Err(handle_errors::Error::DatabaseQueryError(e)),
        }
    }





}

