use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// STRUCT: represents one row from the "tasks" table.
// `sqlx::FromRow` lets sqlx build this struct directly from a database row.
// `Serialize` lets axum turn it into JSON for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub subject: String,
    pub status: String,
    pub priority: String,
    pub created_at: String,
    // When the assignment needs to be passed/submitted, e.g. "2026-08-20T14:30"
    pub due_at: String,
    // OPTION: only has a value once the task is marked Completed.
    pub completed_at: Option<String>,
}

impl Task {
    // IMPL + MATCH: a small helper method on Task.
    // We store status as a String in the database, but this method gives us
    // a safe, readable way to check it instead of comparing strings everywhere.
    pub fn is_completed(&self) -> bool {
        matches!(self.status.as_str(), "Completed")
    }
}

// ENUM: restricts task status to only two valid values instead of any String.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Completed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "Pending",
            TaskStatus::Completed => "Completed",
        }
    }

    // OPTION: returns None if the given text is not a valid status,
    // instead of crashing or guessing.
    pub fn from_str(value: &str) -> Option<TaskStatus> {
        match value {
            "Pending" => Some(TaskStatus::Pending),
            "Completed" => Some(TaskStatus::Completed),
            _ => None,
        }
    }
}

// ENUM: restricts task priority to only three valid values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Low => "Low",
            TaskPriority::Medium => "Medium",
            TaskPriority::High => "High",
        }
    }

    pub fn from_str(value: &str) -> Option<TaskPriority> {
        match value {
            "Low" => Some(TaskPriority::Low),
            "Medium" => Some(TaskPriority::Medium),
            "High" => Some(TaskPriority::High),
            _ => None,
        }
    }
}

// Data coming in from the "Add Task" form (sent as JSON from app.js).
#[derive(Debug, Deserialize)]
pub struct NewTask {
    pub title: String,
    pub description: String,
    pub subject: String,
    pub priority: String,
    pub due_at: String,
}

// Search/filter options coming from the URL query string, e.g.
// /api/tasks?search=rust&status=Pending&priority=High
// Every field is Option because the user does not have to provide all of them.
#[derive(Debug, Deserialize)]
pub struct TaskQuery {
    pub search: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

// Dashboard statistics sent to the frontend as JSON.
#[derive(Debug, Serialize)]
pub struct TaskStats {
    pub total: usize,
    pub pending: usize,
    pub completed: usize,
    pub high_priority: usize,
    // HASHMAP: counts how many tasks exist per subject, e.g. {"Rust": 3, "Math": 1}
    pub by_subject: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_status_strings_parse_correctly() {
        assert_eq!(TaskStatus::from_str("Pending"), Some(TaskStatus::Pending));
        assert_eq!(TaskStatus::from_str("Completed"), Some(TaskStatus::Completed));
    }

    #[test]
    fn invalid_status_string_returns_none() {
        assert_eq!(TaskStatus::from_str("Cancelled"), None);
    }

    #[test]
    fn valid_priority_strings_parse_correctly() {
        assert_eq!(TaskPriority::from_str("Low"), Some(TaskPriority::Low));
        assert_eq!(TaskPriority::from_str("High"), Some(TaskPriority::High));
    }

    #[test]
    fn invalid_priority_string_returns_none() {
        assert_eq!(TaskPriority::from_str("Urgent"), None);
    }

    #[test]
    fn task_transitions_from_pending_to_completed() {
        let mut task = Task {
            id: 1,
            title: "Finish Rust project".to_string(),
            description: "Build the task tracker".to_string(),
            subject: "Rust".to_string(),
            status: "Pending".to_string(),
            priority: "High".to_string(),
            created_at: "2026-01-01 09:00".to_string(),
            due_at: "2026-01-05T23:59".to_string(),
            completed_at: None,
        };

        assert!(!task.is_completed());

        task.status = "Completed".to_string();
        task.completed_at = Some("2026-01-02 08:00".to_string());

        assert!(task.is_completed());
        assert!(task.completed_at.is_some());
    }
}
