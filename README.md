# blog_rust

A minimal, self-contained blog web application written in Rust.

- **Web framework:** [axum](https://github.com/tokio-rs/axum)
- **Async runtime:** [tokio](https://tokio.rs)
- **Storage:** SQLite via [rusqlite](https://github.com/rusqlite/rusqlite) (bundled, no system SQLite required)
- **Rendering:** server-side HTML with a small dark theme

## Features

- List posts (newest first)
- Read an individual post
- Create a post via an HTML form
- JSON API at `/api/posts` (`GET` to list, `POST` to create)
- Health probe at `/health`

## Getting started

Prerequisites: a Rust toolchain (`rustc`/`cargo`) and a C compiler (`gcc`/`cc`) for building the bundled SQLite.

```bash
# Build everything (also warms the compile cache)
cargo build

# Run the server (defaults to 0.0.0.0:3000)
cargo run
```

Then open http://localhost:3000.

## Configuration

| Variable       | Default        | Description                       |
| -------------- | -------------- | --------------------------------- |
| `BLOG_ADDR`    | `0.0.0.0:3000` | Address the HTTP server binds to. |
| `BLOG_DB_PATH` | `blog.db`      | Path to the SQLite database file. |

## API examples

```bash
# List posts
curl -s http://localhost:3000/api/posts

# Create a post
curl -s -X POST http://localhost:3000/api/posts \
  -H 'content-type: application/json' \
  -d '{"title":"Hello","body":"My first post"}'
```

## Tests

```bash
cargo test
```

## Cloud Agent environment

The development environment is described in [`.cursor/environment.json`](.cursor/environment.json):
`cargo build` runs on setup, and the server is started in a `web` terminal with `cargo run`.
