use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AvoidItem {
    pub id: i64,
    pub title: String,
    pub category: String,
    pub reason: String,
    pub alternative: Option<String>,
    pub severity: String,
    pub created_at: Option<String>, // Armazenado como string ISO8601 no SQLite
}

#[derive(Debug, Deserialize)]
pub struct CreateItemInput {
    pub title: String,
    pub category: String,
    pub reason: String,
    pub alternative: Option<String>,
    pub severity: String,
}

/// Busca todos os itens cadastrados no banco de dados, ordenados pelos mais recentes.
pub async fn get_all_items(pool: &SqlitePool) -> Result<Vec<AvoidItem>, sqlx::Error> {
    sqlx::query_as::<_, AvoidItem>(
        "SELECT id, title, category, reason, alternative, severity, strftime('%Y-%m-%d %H:%M:%S', created_at) as created_at 
         FROM avoid_items 
         ORDER BY id DESC"
    )
    .fetch_all(pool)
    .await
}

/// Cria um novo item no banco de dados SQLite e retorna o item recém-criado.
pub async fn create_item(
    pool: &SqlitePool,
    input: CreateItemInput,
) -> Result<AvoidItem, sqlx::Error> {
    // Insere o novo registro
    let id = sqlx::query(
        "INSERT INTO avoid_items (title, category, reason, alternative, severity) 
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&input.title)
    .bind(&input.category)
    .bind(&input.reason)
    .bind(&input.alternative)
    .bind(&input.severity)
    .execute(pool)
    .await?
    .last_insert_rowid();

    // Busca e retorna o item recém-inserido
    sqlx::query_as::<_, AvoidItem>(
        "SELECT id, title, category, reason, alternative, severity, strftime('%Y-%m-%d %H:%M:%S', created_at) as created_at 
         FROM avoid_items 
         WHERE id = ?"
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

/// Deleta um item do banco de dados pelo seu ID.
pub async fn delete_item(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM avoid_items WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
