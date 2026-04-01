# Simple Rust Web Crawler: Crawly

A lightweight, asynchronous web crawler built with Rust. This project demonstrates how to use tokio for async runtime, reqwest for HTTP requests, and scraper for HTML parsing to recursively discover and visit links within the same domain.

### 🚀 Features

    Asynchronous Crawling: Built on tokio and reqwest for efficient, non-blocking I/O.

    Domain-Locked: Automatically stays within the same domain as the initial URL to prevent unwanted "wandering."

    Cyclic Link Protection: Uses a cache to keep track of visited URLs, ensuring the crawler never gets stuck in a loop.

    Robust URL Parsing: Handles absolute, root-relative, and relative paths correctly using the url crate.

    Comprehensive Testing: Includes unit tests and integration tests with mockito for mocking HTTP servers.

### 🛠️ Architecture

The project is divided into three main modules:

    main.rs: The entry point. Initializes the Crawler with a target URL.

    crawler.rs: Manages the crawl state, including the queue of pending URLs, the cache of visited URLs, and the fetch logic.

    parser.rs: Responsible for extracting links from HTML strings and resolving them against the base URL.

### 📥 Getting Started
#### Prerequisites

    Rust (1.60+)

    Cargo

#### Installation & Running

    Clone the repository.

    Run the crawler:
    Bash

    cargo run

    By default, it is configured to crawl https://crawler-test.com/.

#### Running Tests

The project includes a suite of tests covering URL resolution, HTML parsing, and crawler logic:
Bash

`cargo test`

### 📦 Dependencies

    tokio: Async runtime.

    reqwest: HTTP client.

    scraper: HTML parsing and CSS selectors.

    url: URL validation and manipulation.

    mockito: (Dev dependency) HTTP mocking for tests.

### ⚠️ Limitations

    Single-threaded execution: While async, it currently processes URLs sequentially from the queue.

    No Depth Limit: It will attempt to find every link on the domain unless modified.

    Respects robots.txt: Not currently implemented.