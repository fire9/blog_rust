# blog_rust

一个使用 **Rust + Axum** 构建的个人技术博客系统，文章内容以 Markdown 文件存储在仓库中。

## 功能

- 首页：按日期倒序展示文章列表（标题、日期、摘要）
- 文章详情页：Markdown 渲染为 HTML
- RSS 订阅：`/rss.xml`
- 示例文章：开箱即看效果
- 基础自动化测试：覆盖文章解析、加载排序、RSS 生成

## 本地运行

```bash
cargo run
```

启动后访问：

- 博客首页：<http://127.0.0.1:3000>
- RSS：<http://127.0.0.1:3000/rss.xml>

## 运行测试

```bash
cargo test
```

## 如何新增文章

在 `posts/` 目录下新增一个 `.md` 文件即可自动发布。文件名会作为文章 slug（例如 `my-first-post.md` -> `/posts/my-first-post`）。

文章格式如下（front matter 使用 TOML）：

```md
---
title = "文章标题"
date = "2026-08-20"
summary = "一句话摘要"
---

# 正文开始

你的 Markdown 内容...
```

> `date` 格式必须为 `YYYY-MM-DD`。

## 项目结构

```text
.
├── Cargo.toml
├── posts/                 # Markdown 文章
├── src/
│   ├── blog.rs            # 文章加载、Markdown 解析、RSS 生成
│   └── main.rs            # Axum 路由与 HTTP 服务入口
└── templates/             # Askama HTML 模板
    ├── base.html
    ├── index.html
    └── post.html
```
