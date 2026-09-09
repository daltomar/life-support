# CLAUDE.md

Project guidance for Claude Code. Read this before making changes.

## What this is

A media website written in Rust with **Axum**. It serves:

- **MP3 files** — streamable in the browser (not just downloadable). Streaming is the priority; downloads may come later.
- **Images**
- **Text / pages**

The site runs behind **Nginx** (reverse proxy, TLS termination) on a Linux
server, following the same deployment pattern as our other self-hosted sites.

## Stack & key decisions

- **Web framework:** Axum. Do not introduce Actix, Rocket, or Warp.
- **Async runtime:** Tokio.
- **Static/media serving:** `tower-http` `ServeDir` for images and static
  assets. MP3 streaming must support **HTTP byte-range requests** so the
  browser can seek without re-downloading the whole file. `ServeDir` handles
  ranges; if a custom handler is written for audio, it MUST implement
  `Range`/`206 Partial Content` correctly.
- **Templating:** Askama (compile-time checked). Prefer it over runtime engines.
- **Audio player:** custom UI built on the native HTML `<audio>` element via
  the HTMLMediaElement JS API — NOT the raw `controls` bar, and no heavy JS
  framework. Plain HTML/CSS/JS in templates or static files. The backend does
  not care how the player looks; its job is serving pages and streaming bytes.
- **Storage:** filesystem directory for media at first. Do NOT add a database
  unless a task explicitly calls for metadata/search/access-control.
- **Access:** MP3s are public and streamable initially. No auth layer yet —
  don't add one unprompted.

## Architecture notes

- Rust app listens on a local port; Nginx proxies to it and terminates TLS.
- Large media may eventually be served directly by Nginx (offloading range
  requests). Keep media paths configurable so this split is easy later.
- Keep the audio-streaming path lean — avoid buffering whole files into memory;
  stream from disk.

## Conventions

- Keep handlers thin; put logic in separate modules.
- Configuration (bind address, media directory, etc.) via env vars or a config
  file — never hardcode paths or ports.
- Return proper status codes: `404` for missing media, `206` for ranged audio,
  `416` for unsatisfiable ranges.
- Prefer `&str`/`Path` over allocating where reasonable; this is a small,
  fast-serving app.

## Commands

<!-- Fill in / confirm these as the project takes shape -->

- Build: `cargo build`
- Run (dev): `cargo run`
- Release build: `cargo build --release`
- Lint: `cargo clippy --all-targets`
- Format: `cargo fmt`
- Test: `cargo test`

## Workflow (required)

- **Branch per feature:** create a new branch for each new feature; never work
  directly on the main branch. Each feature has its own branch.
- **Code review:** all code must be reviewed by the `rust-code-analyser` agent
  before it lands.
- **Only after review approval**, commit and push the code.
- **Do NOT create pull requests.** Commit and push directly to the feature
  branch.
- **Only commit code that builds.** Run `cargo build` and confirm it succeeds
  before committing — never commit a broken build.

## When making changes

- Don't add dependencies without a clear reason; note why in the commit.
- If you touch the audio-serving path, verify range requests still work
  (`curl -H "Range: bytes=0-1023" ...` should return `206`).
- Match existing module structure and naming rather than inventing new patterns.
