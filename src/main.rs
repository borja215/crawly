pub mod crawler;
pub mod parser;

use crate::crawler::Crawler;
use reqwest::Client;
use url::Url;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    url: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let initial_url = Url::parse(args.url.as_str());
    
    match initial_url {
        Ok(url) => {
            let mut crawler = Crawler::new(url, Client::new());
            crawler.crawl().await?;
            Ok(())
        }
        _ => Err("Invalid url")?,
    }
}
