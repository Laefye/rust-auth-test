use std::{fmt::Debug, sync::Arc};

use axum::{routing::{get, post}, Router};
use repository::MemoryRepository;
use tokio::sync::Mutex;

mod user;
mod repository;
mod post;
mod token;
mod auth;

type ThreadSafeRepository<R> = Arc<Mutex<R>>;

#[derive(Debug, Clone)]
pub struct App {
    repository: ThreadSafeRepository<MemoryRepository>,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/account", post(auth::create_account))
        .route("/account/login", post(auth::login_account))
        .route("/account/me", get(auth::account_get_self))
        .with_state(App{repository: Arc::new(Mutex::new(MemoryRepository::new()))});

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
