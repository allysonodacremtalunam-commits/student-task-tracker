use axum::routing::{get, post};
use axum::Router;
use sqlx::SqlitePool;
use tower_http::services::ServeDir;

use crate::handlers;

// MODULES: keeping route wiring in its own file (separate from handlers.rs
// and main.rs) makes it easy to see every endpoint the app exposes at a glance.
pub fn create_router(pool: SqlitePool) -> Router {
    Router::new()
        .route(
            "/api/tasks",
            get(handlers::list_tasks).post(handlers::create_task),
        )
        // A literal path segment like "history" is matched before the
        // dynamic ":id" segment below, so this never gets treated as an id.
        .route("/api/tasks/history", get(handlers::get_history))
        .route(
            "/api/tasks/:id",
            get(handlers::get_task).delete(handlers::remove_task),
        )
        .route("/api/tasks/:id/complete", post(handlers::complete_task))
        .route("/api/stats", get(handlers::get_stats))
        // Anything that is not one of the /api/... routes above falls back
        // to serving files from the static/ folder (index.html, css, js).
        .fallback_service(ServeDir::new("static"))
        .with_state(pool)
}
