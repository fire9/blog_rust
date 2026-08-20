use crate::models::Post;

const STYLE: &str = r#"
:root {
  --bg: #0f172a;
  --panel: #1e293b;
  --text: #e2e8f0;
  --muted: #94a3b8;
  --accent: #38bdf8;
  --accent-strong: #0ea5e9;
  --border: #334155;
}
* { box-sizing: border-box; }
body {
  margin: 0;
  font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
  background: var(--bg);
  color: var(--text);
  line-height: 1.6;
}
.site-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--border);
  background: var(--panel);
}
.brand {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--accent);
  text-decoration: none;
  letter-spacing: 0.02em;
}
.container { max-width: 760px; margin: 0 auto; padding: 2rem 1.5rem; }
.btn {
  display: inline-block;
  background: var(--accent-strong);
  color: #0b1120;
  font-weight: 600;
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  text-decoration: none;
  border: none;
  cursor: pointer;
  font-size: 0.95rem;
}
.btn:hover { background: var(--accent); }
.post-list { list-style: none; padding: 0; margin: 0; display: grid; gap: 1rem; }
.post-card {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 0.75rem;
  padding: 1.25rem 1.5rem;
}
.post-card h2 { margin: 0 0 0.25rem; font-size: 1.2rem; }
.post-card a { color: var(--text); text-decoration: none; }
.post-card a:hover { color: var(--accent); }
.meta { color: var(--muted); font-size: 0.85rem; }
.excerpt { color: var(--muted); margin: 0.5rem 0 0; }
.post-body { white-space: pre-wrap; margin-top: 1rem; }
.field { display: flex; flex-direction: column; gap: 0.35rem; margin-bottom: 1rem; }
.field label { font-weight: 600; }
.field input, .field textarea {
  background: #0b1120;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  padding: 0.65rem 0.8rem;
  color: var(--text);
  font-size: 1rem;
  font-family: inherit;
}
.field textarea { min-height: 180px; resize: vertical; }
.error { background: #7f1d1d; color: #fecaca; padding: 0.75rem 1rem; border-radius: 0.5rem; margin-bottom: 1rem; }
.back { color: var(--muted); text-decoration: none; font-size: 0.9rem; }
.back:hover { color: var(--accent); }
.site-footer { text-align: center; color: var(--muted); padding: 2rem 1rem; font-size: 0.85rem; }
h1 { margin-top: 0; }
"#;

fn escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn excerpt(body: &str) -> String {
    let trimmed: String = body.chars().take(140).collect();
    if body.chars().count() > 140 {
        format!("{trimmed}…")
    } else {
        trimmed
    }
}

fn layout(title: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} · blog_rust</title>
<style>{style}</style>
</head>
<body>
<header class="site-header">
  <a class="brand" href="/">blog_rust</a>
  <nav><a class="btn" href="/new">New Post</a></nav>
</header>
<main class="container">
{content}
</main>
<footer class="site-footer">Powered by Rust · axum · SQLite</footer>
</body>
</html>"#,
        title = escape(title),
        style = STYLE,
        content = content,
    )
}

pub fn index_page(posts: &[Post]) -> String {
    let content = if posts.is_empty() {
        "<h1>No posts yet</h1><p class=\"meta\">Be the first to publish something.</p>".to_string()
    } else {
        let mut items = String::new();
        for p in posts {
            items.push_str(&format!(
                r#"<li class="post-card">
  <h2><a href="/posts/{id}">{title}</a></h2>
  <div class="meta">Published {created_at}</div>
  <p class="excerpt">{excerpt}</p>
</li>"#,
                id = p.id,
                title = escape(&p.title),
                created_at = escape(&p.created_at),
                excerpt = escape(&excerpt(&p.body)),
            ));
        }
        format!("<h1>Latest posts</h1><ul class=\"post-list\">{items}</ul>")
    };
    layout("Home", &content)
}

pub fn post_page(post: &Post) -> String {
    let content = format!(
        r#"<a class="back" href="/">← All posts</a>
<h1>{title}</h1>
<div class="meta">Published {created_at}</div>
<div class="post-body">{body}</div>"#,
        title = escape(&post.title),
        created_at = escape(&post.created_at),
        body = escape(&post.body),
    );
    layout(&post.title, &content)
}

pub fn new_post_page() -> String {
    new_post_form(None)
}

pub fn new_post_error_page(message: &str) -> String {
    new_post_form(Some(message))
}

fn new_post_form(error: Option<&str>) -> String {
    let error_html = match error {
        Some(msg) => format!("<div class=\"error\">{}</div>", escape(msg)),
        None => String::new(),
    };
    let content = format!(
        r#"<a class="back" href="/">← All posts</a>
<h1>New post</h1>
{error_html}
<form method="post" action="/posts">
  <div class="field">
    <label for="title">Title</label>
    <input id="title" name="title" type="text" placeholder="A great title" required>
  </div>
  <div class="field">
    <label for="body">Body</label>
    <textarea id="body" name="body" placeholder="Write something worth reading…" required></textarea>
  </div>
  <button class="btn" type="submit">Publish</button>
</form>"#,
        error_html = error_html,
    );
    layout("New post", &content)
}

pub fn not_found_page() -> String {
    layout(
        "Not found",
        "<h1>Post not found</h1><p class=\"meta\">That post does not exist.</p><a class=\"back\" href=\"/\">← All posts</a>",
    )
}
