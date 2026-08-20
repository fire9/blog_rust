use chrono::NaiveDate;
use pulldown_cmark::{html, Options, Parser};
use rss::{ChannelBuilder, GuidBuilder, ItemBuilder};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Post {
    pub slug: String,
    pub title: String,
    pub date: NaiveDate,
    pub summary: String,
    pub html: String,
}

#[derive(Debug, Clone)]
pub struct Blog {
    posts: Vec<Post>,
}

impl Blog {
    pub fn load_from_dir(dir: &Path) -> Result<Self, BlogError> {
        let mut posts = Vec::new();
        let entries = fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }

            let slug = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or_else(|| BlogError::Parse(format!("无效文件名: {}", path.display())))?
                .to_string();
            let raw = fs::read_to_string(&path)?;
            posts.push(parse_markdown_post(&slug, &raw)?);
        }

        posts.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.title.cmp(&b.title)));
        Ok(Self { posts })
    }

    pub fn posts(&self) -> &[Post] {
        &self.posts
    }

    pub fn post_by_slug(&self, slug: &str) -> Option<&Post> {
        self.posts.iter().find(|post| post.slug == slug)
    }

    pub fn to_rss(&self, base_url: &str) -> String {
        let items = self
            .posts
            .iter()
            .map(|post| {
                let link = format!("{}/posts/{}", base_url.trim_end_matches('/'), post.slug);
                ItemBuilder::default()
                    .title(post.title.clone())
                    .link(link.clone())
                    .description(post.summary.clone())
                    .guid(GuidBuilder::default().value(link).permalink(true).build())
                    .pub_date(post.date.format("%Y-%m-%d 00:00:00 GMT").to_string())
                    .build()
            })
            .collect::<Vec<_>>();

        ChannelBuilder::default()
            .title("fire9 的技术博客")
            .link(base_url.trim_end_matches('/'))
            .description("记录 Rust / 系统设计 / 工程实践")
            .items(items)
            .build()
            .to_string()
    }
}

#[derive(Debug)]
pub enum BlogError {
    Io(std::io::Error),
    Parse(String),
}

impl Display for BlogError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BlogError::Io(err) => write!(f, "IO 错误: {err}"),
            BlogError::Parse(msg) => write!(f, "解析错误: {msg}"),
        }
    }
}

impl std::error::Error for BlogError {}

impl From<std::io::Error> for BlogError {
    fn from(value: std::io::Error) -> Self {
        BlogError::Io(value)
    }
}

#[derive(Debug)]
struct FrontMatter {
    title: String,
    date: String,
    summary: String,
}

pub fn parse_markdown_post(slug: &str, raw: &str) -> Result<Post, BlogError> {
    let (front_matter_raw, markdown) = split_front_matter(raw)?;
    let front_matter = parse_front_matter(front_matter_raw)?;

    let date = NaiveDate::parse_from_str(&front_matter.date, "%Y-%m-%d")
        .map_err(|err| BlogError::Parse(format!("日期格式错误，应为 YYYY-MM-DD: {err}")))?;
    let markdown = markdown.trim().to_string();

    Ok(Post {
        slug: slug.to_string(),
        title: front_matter.title,
        date,
        summary: front_matter.summary,
        html: render_markdown_to_html(&markdown),
    })
}

pub fn render_markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn split_front_matter(raw: &str) -> Result<(&str, &str), BlogError> {
    let trimmed = raw.trim_start_matches('\u{feff}');
    if !trimmed.starts_with("---\n") {
        return Err(BlogError::Parse(
            "文章缺少 front matter（应以 --- 开头）".to_string(),
        ));
    }

    let remain = &trimmed[4..];
    let marker = "\n---\n";
    let split_idx = remain
        .find(marker)
        .ok_or_else(|| BlogError::Parse("front matter 结束标记缺失（---）".to_string()))?;

    let front_matter = &remain[..split_idx];
    let markdown = &remain[split_idx + marker.len()..];
    Ok((front_matter, markdown))
}

fn parse_front_matter(input: &str) -> Result<FrontMatter, BlogError> {
    let mut title = None::<String>;
    let mut date = None::<String>;
    let mut summary = None::<String>;

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value_raw) = line.split_once('=').ok_or_else(|| {
            BlogError::Parse(format!(
                "front matter 行格式无效，应为 key = \"value\": {line}"
            ))
        })?;
        let key = key.trim();
        let value_raw = value_raw.trim();
        if !value_raw.starts_with('"') || !value_raw.ends_with('"') || value_raw.len() < 2 {
            return Err(BlogError::Parse(format!(
                "front matter 值必须是双引号字符串: {line}"
            )));
        }
        let value = value_raw[1..value_raw.len() - 1].to_string();

        match key {
            "title" => title = Some(value),
            "date" => date = Some(value),
            "summary" => summary = Some(value),
            _ => {}
        }
    }

    Ok(FrontMatter {
        title: title.ok_or_else(|| BlogError::Parse("front matter 缺少 title".to_string()))?,
        date: date.ok_or_else(|| BlogError::Parse("front matter 缺少 date".to_string()))?,
        summary: summary
            .ok_or_else(|| BlogError::Parse("front matter 缺少 summary".to_string()))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn sample_markdown(title: &str, date: &str, summary: &str, body: &str) -> String {
        format!(
            "---\ntitle = \"{}\"\ndate = \"{}\"\nsummary = \"{}\"\n---\n\n{}\n",
            title, date, summary, body
        )
    }

    fn unique_temp_posts_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("系统时间异常")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("blog_rust_posts_{nonce}"));
        fs::create_dir_all(&dir).expect("无法创建临时目录");
        dir
    }

    #[test]
    fn parse_markdown_post_successfully() {
        let raw = sample_markdown(
            "Rust trait 对象实践",
            "2026-08-20",
            "一篇关于 trait object 的经验总结",
            "# 标题\n\n这是一段 **Markdown**。",
        );

        let post = parse_markdown_post("trait-object", &raw).expect("解析失败");
        assert_eq!(post.slug, "trait-object");
        assert_eq!(post.title, "Rust trait 对象实践");
        assert_eq!(post.date.format("%Y-%m-%d").to_string(), "2026-08-20");
        assert!(post.html.contains("<strong>Markdown</strong>"));
    }

    #[test]
    fn load_from_dir_and_sort_by_date_desc() {
        let dir = unique_temp_posts_dir();
        let old_post = sample_markdown("旧文章", "2026-08-10", "旧摘要", "内容 A");
        let new_post = sample_markdown("新文章", "2026-08-19", "新摘要", "内容 B");

        fs::write(dir.join("old.md"), old_post).expect("写入 old 失败");
        fs::write(dir.join("new.md"), new_post).expect("写入 new 失败");

        let blog = Blog::load_from_dir(&dir).expect("加载文章失败");
        assert_eq!(blog.posts().len(), 2);
        assert_eq!(blog.posts()[0].slug, "new");
        assert_eq!(blog.posts()[1].slug, "old");

        fs::remove_dir_all(&dir).expect("清理临时目录失败");
    }

    #[test]
    fn rss_contains_post_links() {
        let raw = sample_markdown("标题", "2026-08-20", "摘要", "内容");
        let post = parse_markdown_post("hello", &raw).expect("解析失败");
        let blog = Blog { posts: vec![post] };

        let rss = blog.to_rss("http://127.0.0.1:3000");
        assert!(rss.contains("<title>fire9 的技术博客</title>"));
        assert!(rss.contains("http://127.0.0.1:3000/posts/hello"));
    }
}
