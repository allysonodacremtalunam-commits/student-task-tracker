use crate::models::{NewTask, TaskPriority};

// ENUM: every way that submitting a new task can fail.
#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    EmptyTitle,
    EmptyDescription,
    EmptySubject,
    EmptyDueDate,
    InvalidPriority,
}

impl ValidationError {
    pub fn message(&self) -> &'static str {
        match self {
            ValidationError::EmptyTitle => "Title cannot be empty.",
            ValidationError::EmptyDescription => "Description cannot be empty.",
            ValidationError::EmptySubject => "Subject cannot be empty.",
            ValidationError::EmptyDueDate => "Due date and time cannot be empty.",
            ValidationError::InvalidPriority => "Priority must be Low, Medium, or High.",
        }
    }
}

// RESULT: validation either succeeds and gives us back a proper TaskPriority,
// or it fails with a specific ValidationError explaining why.
// FUNCTIONS: validation lives in its own function/module instead of inside
// the HTTP handler, so it can be tested on its own (see tests below).
pub fn validate_new_task(task: &NewTask) -> Result<TaskPriority, ValidationError> {
    if task.title.trim().is_empty() {
        return Err(ValidationError::EmptyTitle);
    }
    if task.description.trim().is_empty() {
        return Err(ValidationError::EmptyDescription);
    }
    if task.subject.trim().is_empty() {
        return Err(ValidationError::EmptySubject);
    }
    if task.due_at.trim().is_empty() {
        return Err(ValidationError::EmptyDueDate);
    }

    // OPTION -> RESULT: from_str returns an Option; `ok_or` turns the missing
    // case (None) into our own ValidationError.
    TaskPriority::from_str(task.priority.trim()).ok_or(ValidationError::InvalidPriority)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_task(title: &str, description: &str, subject: &str, priority: &str, due_at: &str) -> NewTask {
        NewTask {
            title: title.to_string(),
            description: description.to_string(),
            subject: subject.to_string(),
            priority: priority.to_string(),
            due_at: due_at.to_string(),
        }
    }

    #[test]
    fn valid_task_passes_validation() {
        let task = sample_task("Finish Rust project", "Build the tracker", "Rust", "High", "2026-08-20T14:30");
        assert!(validate_new_task(&task).is_ok());
    }

    #[test]
    fn empty_title_fails_validation() {
        let task = sample_task("", "desc", "Rust", "Low", "2026-08-20T14:30");
        assert_eq!(validate_new_task(&task), Err(ValidationError::EmptyTitle));
    }

    #[test]
    fn empty_description_fails_validation() {
        let task = sample_task("Title", "   ", "Rust", "Low", "2026-08-20T14:30");
        assert_eq!(validate_new_task(&task), Err(ValidationError::EmptyDescription));
    }

    #[test]
    fn empty_subject_fails_validation() {
        let task = sample_task("Title", "desc", "", "Low", "2026-08-20T14:30");
        assert_eq!(validate_new_task(&task), Err(ValidationError::EmptySubject));
    }

    #[test]
    fn empty_due_date_fails_validation() {
        let task = sample_task("Title", "desc", "Rust", "Low", "");
        assert_eq!(validate_new_task(&task), Err(ValidationError::EmptyDueDate));
    }

    #[test]
    fn invalid_priority_fails_validation() {
        let task = sample_task("Title", "desc", "Rust", "Urgent", "2026-08-20T14:30");
        assert_eq!(validate_new_task(&task), Err(ValidationError::InvalidPriority));
    }
}
