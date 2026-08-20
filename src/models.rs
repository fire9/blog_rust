use serde::Serialize;

/// A single blog post.
#[derive(Debug, Clone, Serialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub created_at: String,
}
