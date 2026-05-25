use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AvoidItem {
    pub id: i64,
    pub title: String,
    pub category: String,
    pub reason: Option<String>,
    pub alternative: Option<String>,
    pub severity: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateItemInput {
    pub title: String,
    pub category: String,
    pub reason: String,
    pub alternative: Option<String>,
    pub severity: String,
}

#[derive(Debug, Clone)]
pub struct NewAvoidItem {
    pub title: String,
    pub category: String,
    pub reason: Option<String>,
    pub alternative: Option<String>,
    pub severity: String,
}

impl TryFrom<CreateItemInput> for NewAvoidItem {
    type Error = &'static str;

    fn try_from(value: CreateItemInput) -> Result<Self, Self::Error> {
        let title = value.title.trim().to_string();
        let category = value.category.trim().to_string();
        let reason = to_optional(value.reason);
        let alternative = value.alternative.and_then(to_optional);
        let severity = value.severity.trim().to_string();

        if title.is_empty() {
            return Err("Titulo e obrigatorio.");
        }
        if category.is_empty() {
            return Err("Categoria e obrigatoria.");
        }
        if !matches!(severity.as_str(), "low" | "medium" | "high") {
            return Err("Severidade invalida.");
        }

        Ok(Self {
            title,
            category,
            reason,
            alternative,
            severity,
        })
    }
}

fn to_optional(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub async fn init(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("PRAGMA journal_mode = WAL;").execute(pool).await?;
    sqlx::query("PRAGMA foreign_keys = ON;").execute(pool).await?;
    sqlx::query("PRAGMA busy_timeout = 5000;")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_items(pool: &SqlitePool) -> Result<Vec<AvoidItem>, sqlx::Error> {
    sqlx::query_as::<_, AvoidItem>(
        "SELECT
            id,
            title,
            category,
            NULLIF(reason, '') AS reason,
            alternative,
            severity,
            strftime('%Y-%m-%d %H:%M', created_at) AS created_at
         FROM avoid_items
         ORDER BY id DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_item(pool: &SqlitePool, input: NewAvoidItem) -> Result<AvoidItem, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let id = sqlx::query(
        "INSERT INTO avoid_items (title, category, reason, alternative, severity)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&input.title)
    .bind(&input.category)
    .bind(input.reason.clone().unwrap_or_default())
    .bind(&input.alternative)
    .bind(&input.severity)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    let item = sqlx::query_as::<_, AvoidItem>(
        "SELECT
            id,
            title,
            category,
            NULLIF(reason, '') AS reason,
            alternative,
            severity,
            strftime('%Y-%m-%d %H:%M', created_at) AS created_at
         FROM avoid_items
         WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(item)
}

pub async fn remove_item(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM avoid_items WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_categories(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT category
         FROM avoid_items
         WHERE TRIM(category) <> ''
         ORDER BY category COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await
}

pub async fn search_items_by_title(pool: &SqlitePool, query: &str) -> Result<Vec<AvoidItem>, sqlx::Error> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return list_items(pool).await;
    }

    let pattern = format!("%{}%", trimmed.to_lowercase());
    sqlx::query_as::<_, AvoidItem>(
        "SELECT
            id,
            title,
            category,
            NULLIF(reason, '') AS reason,
            alternative,
            severity,
            strftime('%Y-%m-%d %H:%M', created_at) AS created_at
         FROM avoid_items
         WHERE LOWER(title) LIKE ?
         ORDER BY id DESC",
    )
    .bind(pattern)
    .fetch_all(pool)
    .await
}
