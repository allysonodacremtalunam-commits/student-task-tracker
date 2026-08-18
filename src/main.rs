// MODULES: the application is split into small files instead of one giant
// main.rs. Each `mod` line below loads one file from src/.
mod database;
mod errors;
mod handlers;
mod models;
mod routes;
mod validation;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // "sqlite://tasks.db" creates/opens tasks.db right inside the project
    // folder. Because it is a real file (not ":memory:"), tasks survive a
    // server restart.
    let database_url = "sqlite://tasks.db";

    let pool = database::init_db(database_url)
        .await
        .expect("Failed to set up the database");

    let app = routes::create_router(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Student Task Tracker running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to 127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
