use rusqlite::{params, Connection, Result};

use crate::models::Post;

/// Open (or create) the SQLite database at `path`.
pub fn open(path: &str) -> Result<Connection> {
    Connection::open(path)
}

/// Create the schema if it does not already exist. Safe to call repeatedly.
pub fn init(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            title      TEXT NOT NULL,
            body       TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;
    Ok(())
}

/// Insert a starter post the first time the blog is opened with an empty table.
pub fn seed_if_empty(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM posts", [], |r| r.get(0))?;
    if count == 0 {
        create_post(
            conn,
            "Welcome to blog_rust",
            "This blog is powered by Rust, axum, and SQLite.\n\nClick \"New Post\" to publish your first entry.",
        )?;
    }
    Ok(())
}

/// Insert a new post and return its id.
pub fn create_post(conn: &Connection, title: &str, body: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO posts (title, body) VALUES (?1, ?2)",
        params![title, body],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Return all posts, newest first.
pub fn list_posts(conn: &Connection) -> Result<Vec<Post>> {
    let mut stmt =
        conn.prepare("SELECT id, title, body, created_at FROM posts ORDER BY id DESC")?;
    let rows = stmt.query_map([], row_to_post)?;
    rows.collect()
}

/// Return a single post by id, if it exists.
pub fn get_post(conn: &Connection, id: i64) -> Result<Option<Post>> {
    let mut stmt = conn.prepare("SELECT id, title, body, created_at FROM posts WHERE id = ?1")?;
    let mut rows = stmt.query_map(params![id], row_to_post)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

fn row_to_post(row: &rusqlite::Row<'_>) -> Result<Post> {
    Ok(Post {
        id: row.get(0)?,
        title: row.get(1)?,
        body: row.get(2)?,
        created_at: row.get(3)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init(&conn).unwrap();
        conn
    }

    #[test]
    fn create_and_get_post() {
        let conn = memory_db();
        let id = create_post(&conn, "Hello", "World").unwrap();
        let post = get_post(&conn, id).unwrap().expect("post exists");
        assert_eq!(post.title, "Hello");
        assert_eq!(post.body, "World");
    }

    #[test]
    fn list_returns_newest_first() {
        let conn = memory_db();
        create_post(&conn, "first", "a").unwrap();
        create_post(&conn, "second", "b").unwrap();
        let posts = list_posts(&conn).unwrap();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].title, "second");
    }

    #[test]
    fn seed_only_runs_once() {
        let conn = memory_db();
        seed_if_empty(&conn).unwrap();
        seed_if_empty(&conn).unwrap();
        assert_eq!(list_posts(&conn).unwrap().len(), 1);
    }

    #[test]
    fn missing_post_returns_none() {
        let conn = memory_db();
        assert!(get_post(&conn, 999).unwrap().is_none());
    }
}
