use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Local;
use sqlx::SqlitePool;
use std::collections::HashMap;

use crate::{
    database,
    errors::AppError,
    models::{NewTask, Task, TaskQuery, TaskStats},
    validation,
};

// ASYNC/AWAIT: this is an Axum handler, so it is async. `State(pool)` borrows
// the shared database pool that was registered in routes.rs.
pub async fn list_tasks(
    State(pool): State<SqlitePool>,
    Query(query): Query<TaskQuery>,
) -> Result<Json<Vec<Task>>, AppError> {
    let all_tasks = database::get_all_tasks(&pool).await?;
    Ok(Json(filter_tasks(all_tasks, &query)))
}

// FUNCTIONS + ITERATORS: pulled out of the handler above so it is a plain,
// synchronous function we can unit test without a database (see tests below).
// Uses `filter` three times, one per optional condition (search/status/priority).
pub fn filter_tasks(tasks: Vec<Task>, query: &TaskQuery) -> Vec<Task> {
    tasks
        .into_iter()
        .filter(|task| match &query.search {
            Some(term) if !term.trim().is_empty() => {
                let term = term.to_lowercase();
                task.title.to_lowercase().contains(&term)
                    || task.subject.to_lowercase().contains(&term)
            }
            _ => true,
        })
        .filter(|task| match &query.status {
            Some(status) if !status.is_empty() && status != "All" => task.status == *status,
            _ => true,
        })
        .filter(|task| match &query.priority {
            Some(priority) if !priority.is_empty() && priority != "All" => {
                task.priority == *priority
            }
            _ => true,
        })
        .collect()
}

// FUNCTIONS + ITERATORS: keeps only completed tasks and orders them so the
// most recently completed task appears first — this is the "History" list.
pub fn history_tasks(tasks: Vec<Task>) -> Vec<Task> {
    let mut completed: Vec<Task> = tasks.into_iter().filter(|t| t.is_completed()).collect();
    completed.sort_by(|a, b| b.completed_at.cmp(&a.completed_at));
    completed
}

pub async fn get_history(State(pool): State<SqlitePool>) -> Result<Json<Vec<Task>>, AppError> {
    let all_tasks = database::get_all_tasks(&pool).await?;
    Ok(Json(history_tasks(all_tasks)))
}

pub async fn create_task(
    State(pool): State<SqlitePool>,
    Json(new_task): Json<NewTask>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    // RESULT: validation can fail, and `?` immediately returns the error
    // (converted into AppError::Validation) if it does.
    let priority = validation::validate_new_task(&new_task)
        .map_err(|e| AppError::Validation(e.message().to_string()))?;

    let created_at = Local::now().format("%Y-%m-%d %H:%M").to_string();

    let id = database::insert_task(
        &pool,
        new_task.title.trim(),
        new_task.description.trim(),
        new_task.subject.trim(),
        priority.as_str(),
        &created_at,
        new_task.due_at.trim(),
    )
    .await?;

    let task = database::get_task_by_id(&pool, id)
        .await?
        .ok_or_else(|| AppError::Database("Task was saved but could not be reloaded.".into()))?;

    Ok((StatusCode::CREATED, Json(task)))
}

pub async fn get_task(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Task>, AppError> {
    // OPTION -> RESULT: turn "no task with this id" into a proper 404
    // instead of an empty/blank response.
    match database::get_task_by_id(&pool, id).await? {
        Some(task) => Ok(Json(task)),
        None => Err(AppError::NotFound(format!("Task {} was not found.", id))),
    }
}

pub async fn complete_task(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Task>, AppError> {
    let existing = database::get_task_by_id(&pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} was not found.", id)))?;

    // Avoid pointlessly re-writing the row (and overwriting completed_at)
    // if the task is already completed.
    if existing.is_completed() {
        return Ok(Json(existing));
    }

    let completed_at = Local::now().format("%Y-%m-%d %H:%M").to_string();
    database::mark_task_completed(&pool, id, &completed_at).await?;

    let updated = database::get_task_by_id(&pool, id)
        .await?
        .ok_or_else(|| AppError::Database("Task disappeared after update.".into()))?;

    Ok(Json(updated))
}

pub async fn remove_task(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let rows_deleted = database::delete_task(&pool, id).await?;

    if rows_deleted == 0 {
        return Err(AppError::NotFound(format!("Task {} was not found.", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_stats(State(pool): State<SqlitePool>) -> Result<Json<TaskStats>, AppError> {
    let tasks = database::get_all_tasks(&pool).await?;
    Ok(Json(build_stats(&tasks)))
}

// FUNCTIONS + ITERATORS + HASHMAP: another pure helper, easy to unit test.
// Uses `iter().filter(...).count()` for the simple counts, and a HashMap to
// count tasks per subject with the `entry(...).or_insert(0)` pattern.
pub fn build_stats(tasks: &[Task]) -> TaskStats {
    let total = tasks.len();
    let pending = tasks.iter().filter(|t| t.status == "Pending").count();
    let completed = tasks.iter().filter(|t| t.status == "Completed").count();
    let high_priority = tasks.iter().filter(|t| t.priority == "High").count();

    let mut by_subject: HashMap<String, usize> = HashMap::new();
    for task in tasks {
        // BORROWING: `&task.subject` is borrowed here; entry() clones the
        // key only when it actually needs to insert a new one.
        *by_subject.entry(task.subject.clone()).or_insert(0) += 1;
    }

    TaskStats {
        total,
        pending,
        completed,
        high_priority,
        by_subject,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tasks() -> Vec<Task> {
        vec![
            Task {
                id: 1,
                title: "Finish Rust project".to_string(),
                description: "Build the tracker".to_string(),
                subject: "Rust".to_string(),
                status: "Pending".to_string(),
                priority: "High".to_string(),
                created_at: "2026-01-01 09:00".to_string(),
                due_at: "2026-01-10T23:59".to_string(),
                completed_at: None,
            },
            Task {
                id: 2,
                title: "Database assignment".to_string(),
                description: "Design the schema".to_string(),
                subject: "Database".to_string(),
                status: "Completed".to_string(),
                priority: "Medium".to_string(),
                created_at: "2026-01-02 10:00".to_string(),
                due_at: "2026-01-04T23:59".to_string(),
                completed_at: Some("2026-01-03 08:00".to_string()),
            },
            Task {
                id: 3,
                title: "Read chapter 4".to_string(),
                description: "Programming fundamentals".to_string(),
                subject: "Rust".to_string(),
                status: "Completed".to_string(),
                priority: "Low".to_string(),
                created_at: "2026-01-03 11:00".to_string(),
                due_at: "2026-01-05T23:59".to_string(),
                completed_at: Some("2026-01-05 09:00".to_string()),
            },
        ]
    }

    #[test]
    fn search_matches_title_or_subject_case_insensitively() {
        let query = TaskQuery {
            search: Some("rust".to_string()),
            status: None,
            priority: None,
        };

        let result = filter_tasks(sample_tasks(), &query);

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|t| t.subject == "Rust"));
    }

    #[test]
    fn status_and_priority_filters_combine_with_search() {
        let query = TaskQuery {
            search: None,
            status: Some("Pending".to_string()),
            priority: Some("High".to_string()),
        };

        let result = filter_tasks(sample_tasks(), &query);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, 1);
    }

    #[test]
    fn no_filters_returns_every_task() {
        let query = TaskQuery {
            search: None,
            status: None,
            priority: None,
        };

        let result = filter_tasks(sample_tasks(), &query);

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn stats_are_calculated_correctly() {
        let stats = build_stats(&sample_tasks());

        assert_eq!(stats.total, 3);
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.completed, 2);
        assert_eq!(stats.high_priority, 1);
        assert_eq!(stats.by_subject.get("Rust"), Some(&2));
        assert_eq!(stats.by_subject.get("Database"), Some(&1));
    }

    #[test]
    fn history_only_contains_completed_tasks_newest_first() {
        let history = history_tasks(sample_tasks());

        assert_eq!(history.len(), 2);
        // Task 3 was completed on 2026-01-05, task 2 on 2026-01-03, so 3
        // should come first.
        assert_eq!(history[0].id, 3);
        assert_eq!(history[1].id, 2);
    }
}
