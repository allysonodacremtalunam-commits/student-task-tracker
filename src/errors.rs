use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

// ENUM: every kind of error a handler can produce. Grouping them here means
// handlers return `Result<_, AppError>` and use `?` instead of `.unwrap()`,
// so a database problem or a bad id never causes a panic/crash.
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Validation(String),
    Database(String),
}

#[derive(Serialize)]
struct ErrorBody {
    message: String,
}

// Turns any AppError into a proper HTTP response with a JSON body like
// { "message": "..." }, which the frontend reads and shows to the user.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(ErrorBody { message })).into_response()
    }
}

// Lets us use `?` on any sqlx database call inside a handler: sqlx::Error is
// automatically converted into AppError::Database.
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}
