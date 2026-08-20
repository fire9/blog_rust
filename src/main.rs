mod blog;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use blog::{Blog, BlogError};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    blog: Arc<Blog>,
}

#[derive(Debug, Clone)]
struct PostListItem {
    slug: String,
    title: String,
    date: String,
    summary: String,
}

#[derive(Debug, Clone)]
struct PostDetailView {
    title: String,
    date: String,
    summary: String,
    html: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    posts: Vec<PostListItem>,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    post: PostDetailView,
}

#[tokio::main]
async fn main() -> Result<(), BlogError> {
    let blog = Blog::load_from_dir(std::path::Path::new("posts"))?;
    let app_state = AppState {
        blog: Arc::new(blog),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/posts/{slug}", get(post_detail))
        .route("/rss.xml", get(rss))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .map_err(|err| BlogError::Parse(format!("无法绑定 127.0.0.1:3000: {err}")))?;

    println!("Blog running at http://127.0.0.1:3000");
    axum::serve(listener, app)
        .await
        .map_err(|err| BlogError::Parse(format!("服务启动失败: {err}")))?;
    Ok(())
}

async fn index(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let posts = state
        .blog
        .posts()
        .iter()
        .map(|post| PostListItem {
            slug: post.slug.clone(),
            title: post.title.clone(),
            date: post.date.format("%Y-%m-%d").to_string(),
            summary: post.summary.clone(),
        })
        .collect::<Vec<_>>();

    let html = IndexTemplate { posts }
        .render()
        .map_err(|err| AppError::render(err.to_string()))?;
    Ok(Html(html))
}

async fn post_detail(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let post = state
        .blog
        .post_by_slug(&slug)
        .ok_or_else(|| AppError::not_found(format!("文章不存在: {slug}")))?;
    let html = PostTemplate {
        post: PostDetailView {
            title: post.title.clone(),
            date: post.date.format("%Y-%m-%d").to_string(),
            summary: post.summary.clone(),
            html: post.html.clone(),
        },
    }
    .render()
    .map_err(|err| AppError::render(err.to_string()))?;

    Ok(Html(html))
}

async fn rss(State(state): State<AppState>) -> impl IntoResponse {
    let xml = state.blog.to_rss("http://127.0.0.1:3000");
    (
        StatusCode::OK,
        [("content-type", "application/rss+xml; charset=utf-8")],
        xml,
    )
}

struct AppError {
    status: StatusCode,
    message: String,
}

impl AppError {
    fn not_found(message: String) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message,
        }
    }

    fn render(message: String) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"/><title>错误</title></head><body><h1>{}</h1></body></html>",
            self.message
        );
        (self.status, Html(body)).into_response()
    }
}
