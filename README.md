# Crawly — Simple Rust Web Crawler

A lightweight, asynchronous web crawler built with Rust. Crawly recursively discovers and visits links within a target domain, building a site map of every page and its outbound links.

## Features

- **Concurrent crawling** — fetches multiple pages in parallel per crawl wave using `tokio` and `futures::join_all`, rather than processing URLs one at a time.
- **Domain-locked** — stays within the same domain as the initial URL.
- **Cycle protection** — tracks visited URLs in a `HashSet` to avoid revisiting pages or getting stuck in loops.
- **Site map generation** — builds a `HashMap<String, Vec<String>>` mapping each crawled page to its discovered child links.
- **Robust URL resolution** — handles absolute, root-relative, and relative paths via the `url` crate.
- **Resilient to errors** — failed fetches and unparseable URLs are logged and skipped; the crawler always returns `Ok(())`.
- **Comprehensive tests** — unit and integration tests backed by `mockito` for HTTP mocking.

## Architecture

The project is divided into three main modules:
```
src/
├── main.rs       # Entry point — parses the initial URL and starts the crawler
├── crawler.rs    # Crawl state, concurrency logic, fetch, and site map construction
└── parser.rs     # HTML link extraction and URL resolution
```
**`crawler.rs`** owns the crawl loop. Each iteration drains the pending URL queue, fans out concurrent `process_page` futures, awaits all results, then filters and enqueues newly discovered URLs. Both `fetch_page` and `process_page` take `&self` (not `&mut self`) so they can be called concurrently — `reqwest::Client` is cheaply cloneable and safe to share across tasks.

**`parser.rs`** parses HTML with `scraper` and resolves every `<a href>` against the base URL, correctly handling absolute URLs, root-relative paths (`/about`), and relative paths (`../contact`).

## Getting Started

### Prerequisites

- Rust 1.65+
- Cargo

### Run
```bash
cargo run
```

Crawls `https://crawler-test.com/` by default. Change the URL in `main.rs` to target a different site.

### Test
```bash
cargo test
```

## Dependencies

| Crate | Purpose |
|---|---|
| `tokio` | Async runtime |
| `reqwest` | HTTP client |
| `scraper` | HTML parsing and CSS selectors |
| `url` | URL validation and resolution |
| `futures` | `join_all` for concurrent page fetching |
| `mockito` *(dev)* | HTTP mock server for integration tests |

## Limitations

- **No depth limit** — crawls the entire domain reachable from the starting URL.
- **No rate limiting** — all queued URLs in a wave are fetched concurrently with no delay.
- **No `robots.txt` support** — does not check crawl permissions.
- **In-memory only** — the site map is printed to stdout at the end and not persisted.