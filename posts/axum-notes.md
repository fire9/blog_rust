---
title = "Axum 实战笔记：路由与状态管理"
date = "2026-08-18"
summary = "用最小心智负担组织 Axum 应用的几个经验。"
---

我对 Axum 的一个感受是：它把「框架魔法」降到了很低。

## 1) 路由要简单直白

把路由定义集中在 `main.rs`，对于小项目非常高效：

```rust
let app = Router::new()
    .route("/", get(index))
    .route("/posts/{slug}", get(post_detail));
```

## 2) 状态用 `State<T>`

把不可变共享数据包进 `Arc`，通过提取器拿出来即可，写起来非常顺手。

## 3) 先做好领域边界

把文章读取/解析放到 `blog` 模块后，HTTP 层基本只剩「数据转视图」和「错误映射」。
