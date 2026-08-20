mod db;
mod models;
mod templates;

use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Json, Router,
};
use rusqlite::Connection;
use serde::Deserialize;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Connection>>,
}

#[derive(Debug, Deserialize)]
struct NewPost {
    title: String,
    body: String,
}

#[tokio::main]
async fn main() {
    let db_path = std::env::var("BLOG_DB_PATH").unwrap_or_else(|_| "blog.db".to_string());
    let conn = db::open(&db_path).expect("failed to open database");
    db::init(&conn).expect("failed to initialize database");
    db::seed_if_empty(&conn).expect("failed to seed database");

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/new", get(new_form))
        .route("/posts", post(create_post))
        .route("/posts/:id", get(show_post))
        .route("/health", get(health))
        .route("/api/posts", get(api_list).post(api_create))
        .with_state(state);

    let addr = std::env::var("BLOG_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {addr}: {e}"));
    println!("blog_rust listening on http://{addr}");
    axum::serve(listener, app).await.expect("server error");
}

async fn health() -> &'static str {
    "ok"
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let conn = state.db.lock().unwrap();
    let posts = db::list_posts(&conn).unwrap_or_default();
    Html(templates::index_page(&posts))
}

async fn new_form() -> Html<String> {
    Html(templates::new_post_page())
}

async fn show_post(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();
    match db::get_post(&conn, id) {
        Ok(Some(post)) => Html(templates::post_page(&post)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Html(templates::not_found_page())).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error").into_response(),
    }
}

async fn create_post(
    State(state): State<AppState>,
    Form(input): Form<NewPost>,
) -> impl IntoResponse {
    let title = input.title.trim();
    let body = input.body.trim();
    if title.is_empty() || body.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Html(templates::new_post_error_page(
                "Title and body are both required.",
            )),
        )
            .into_response();
    }
    let conn = state.db.lock().unwrap();
    match db::create_post(&conn, title, body) {
        Ok(id) => Redirect::to(&format!("/posts/{id}")).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error").into_response(),
    }
}

async fn api_list(State(state): State<AppState>) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();
    let posts = db::list_posts(&conn).unwrap_or_default();
    Json(posts)
}

async fn api_create(
    State(state): State<AppState>,
    Json(input): Json<NewPost>,
) -> impl IntoResponse {
    let title = input.title.trim();
    let body = input.body.trim();
    if title.is_empty() || body.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "title and body are required" })),
        )
            .into_response();
    }
    let conn = state.db.lock().unwrap();
    match db::create_post(&conn, title, body) {
        Ok(id) => {
            let post = db::get_post(&conn, id).ok().flatten();
            (StatusCode::CREATED, Json(serde_json::json!(post))).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "database error" })),
        )
            .into_response(),
    }
}
