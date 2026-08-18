use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

use crate::models::Task;

// Connects to the SQLite file (creating it if it does not exist yet) and
// makes sure the "tasks" table is present. Called once when the server starts.
//
// RESULT: this can fail (bad path, disk error, etc.), so it returns a
// Result instead of panicking. main.rs decides what to do if it fails.
pub async fn init_db(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tasks (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            title         TEXT NOT NULL,
            description   TEXT NOT NULL,
            subject       TEXT NOT NULL,
            status        TEXT NOT NULL,
            priority      TEXT NOT NULL,
            created_at    TEXT NOT NULL,
            due_at        TEXT NOT NULL DEFAULT '',
            completed_at  TEXT
        )",
    )
    .execute(&pool)
    .await?;

    // Lightweight "migration" for anyone upgrading from an older version of
    // this project whose tasks.db file was created before due_at/completed_at
    // existed. Adding a column that is already there returns an error, which
    // we simply ignore here — a brand new database will already have both
    // columns from CREATE TABLE above, so these two calls do nothing on it.
    let _ = sqlx::query("ALTER TABLE tasks ADD COLUMN due_at TEXT NOT NULL DEFAULT ''")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE tasks ADD COLUMN completed_at TEXT")
        .execute(&pool)
        .await;

    Ok(pool)
}

// Inserts a new task with status "Pending" and returns its new id.
// All values are passed as bound parameters (the `?` placeholders), never
// concatenated into the SQL string, so user input cannot break the query.
#[allow(clippy::too_many_arguments)]
pub async fn insert_task(
    pool: &SqlitePool,
    title: &str,
    description: &str,
    subject: &str,
    priority: &str,
    created_at: &str,
    due_at: &str,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO tasks (title, description, subject, status, priority, created_at, due_at)
         VALUES (?, ?, ?, 'Pending', ?, ?, ?)",
    )
    .bind(title)
    .bind(description)
    .bind(subject)
    .bind(priority)
    .bind(created_at)
    .bind(due_at)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

// VECTOR: returns every task as a Vec<Task>. Filtering/searching/history is
// done in Rust (see handlers.rs) using iterator methods.
pub async fn get_all_tasks(pool: &SqlitePool) -> Result<Vec<Task>, sqlx::Error> {
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT id, title, description, subject, status, priority, created_at, due_at, completed_at
         FROM tasks ORDER BY id DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

// OPTION: fetch_optional gives us None automatically when no row matches,
// instead of us having to check "if rows.is_empty()" by hand.
pub async fn get_task_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Task>, sqlx::Error> {
    let task = sqlx::query_as::<_, Task>(
        "SELECT id, title, description, subject, status, priority, created_at, due_at, completed_at
         FROM tasks WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(task)
}

// Marks a task Completed and records when that happened, so the History
// view can show "Completed On" for it.
pub async fn mark_task_completed(
    pool: &SqlitePool,
    id: i64,
    completed_at: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("UPDATE tasks SET status = 'Completed', completed_at = ? WHERE id = ?")
        .bind(completed_at)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

pub async fn delete_task(pool: &SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
