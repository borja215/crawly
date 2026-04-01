pub mod crawler;
pub mod parser;

use crate::crawler::Crawler;
use reqwest::Client;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let initial_url = Url::parse("https://crawler-test.com/");
    match initial_url {
        Ok(url) => {
            let mut crawler = Crawler::new(url, Client::new());
            crawler.crawl().await?;
            Ok(())
        }
        _ => Err("Invalid url")?,
    }
}
