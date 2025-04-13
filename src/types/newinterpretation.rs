use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
struct NewInterpretation {
    pub id: String,
    pub no: String,
    pub name: String,
    pub date: String,
    pub reason: Option<String>,
    pub content: Option<String>,
    pub related_law: Option<String>,
    pub source: String,
}

impl NewInterpretation {
pub async fn add_to_pool(self, pool: &PgPool)  {
    let uuid = Uuid::new_v4().to_string();

    match sqlx::query(
        "INSERT INTO interpretations (id, no, name, date, reason, content, related_law, source)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
        .bind(uuid)
        .bind(self.no)
        .bind(self.name)
        .bind(self.date)
        .bind(self.reason)
        .bind(self.content)
        .bind(self.related_law)
        .bind(self.source)
        .execute(pool)
        .await
    {
        Ok(_) => println!("Insert successful"),
        Err(e) => eprintln!("Insert failed: {}", e),
    }

    }
}