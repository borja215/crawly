use crate::parser::parse_html_for_links;
use reqwest::Client;
use std::collections::{HashMap, HashSet, VecDeque};
use futures::future::join_all;
use url::Url;

pub struct Crawler {
    initial_url: Url,
    cached_urls: HashSet<Url>,
    queued_urls: VecDeque<Url>,
    site_map: HashMap<String, Vec<String>>,
    client: Client,
}

impl Crawler {
    pub(crate) fn new(initial_url: Url, client: Client) -> Self {
        let queued_urls = VecDeque::from([initial_url.clone()]);

        let mut cached_urls = HashSet::new();
        cached_urls.insert(initial_url.clone());

        let site_map = HashMap::new();

        Crawler {
            initial_url: initial_url.clone(),
            cached_urls,
            queued_urls,
            site_map,
            client,
        }
    }

    pub async fn crawl(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while !self.queued_urls.is_empty() {
            let urls: Vec<Url> = self.queued_urls.drain(..).collect();

            let futures: Vec<_> = urls.into_iter().map(async |url| (url.clone(), self.process_page(&url).await)).collect();

            let results: Vec<(Url, Vec<Url>)> = join_all(futures).await;

            for pair in results {
                for child in &pair.1[..] {
                    if self.cached_urls.contains(child) {
                        continue;
                    }
                    self.cached_urls.insert(child.clone());
                    self.queued_urls.push_back(child.clone());
                }

                self.site_map.insert(pair.0.to_string(), pair.1.into_iter().map(|url| url.to_string()).collect());
            }
        }

        let json = serde_json::to_string(&self.site_map)?;
        std::fs::write("sitemap.json", &json)?;
        Ok(())
    }

    async fn process_page(&self, url: &Url) -> Vec<Url>{
        let resp = self.fetch_page(url).await;

        let urls = parse_html_for_links(&self.initial_url, &resp);

        urls
            .into_iter()
            .filter(|found_url| found_url.domain() == self.initial_url.domain())
            .collect::<Vec<Url>>()
    }

    async fn fetch_page(&self, url:&Url) -> String {
        let empty_response = "".to_string();
        match self.client.get(url.as_str()).send().await {
            Ok(response) => match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    println!("Error encountered converting response to text {} {}", e, url);
                    empty_response
                }
            },
            Err(e) => {
                println!("Error encountered fetching page: {}", e);
                empty_response
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    // --- new() ---

    #[test]
    fn new_initializes_with_url_queued_and_cached() {
        let url = Url::parse("https://example.com").unwrap();
        let client = Client::new();
        let crawler = Crawler::new(url.clone(), client);

        assert_eq!(crawler.queued_urls, vec![url.clone()]);
        assert!(crawler.cached_urls.contains(&url));
        assert_eq!(crawler.initial_url, url);
    }

    // --- crawl() ---

    #[tokio::test]
    async fn crawl_visits_initial_url() {
        let mut server = Server::new_async().await;
        let url = Url::parse(&server.url()).unwrap();

        let mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_body("<html></html>")
            .create_async()
            .await;

        let client = Client::new();
        let mut crawler = Crawler::new(url, client);
        crawler.crawl().await.unwrap();

        mock.assert_async().await; // verifies the request was actually made
    }

    #[tokio::test]
    async fn crawl_follows_same_domain_links() {
        let mut server = Server::new_async().await;
        let url = Url::parse(&server.url()).unwrap();

        let index_mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_body(r#"<a href="/about">About</a>"#)
            .create_async()
            .await;

        let about_mock = server
            .mock("GET", "/about")
            .with_status(200)
            .with_body("<html></html>")
            .create_async()
            .await;

        let client = Client::new();
        let mut crawler = Crawler::new(url, client);
        crawler.crawl().await.unwrap();

        index_mock.assert_async().await;
        about_mock.assert_async().await;
    }

    #[tokio::test]
    async fn crawl_does_not_follow_external_links() {
        let mut server = Server::new_async().await;
        let url = Url::parse(&server.url()).unwrap();

        server
            .mock("GET", "/")
            .with_status(200)
            .with_body(r#"<a href="https://external.com/page">External</a>"#)
            .create_async()
            .await;

        // If the crawler incorrectly follows the external link, it will
        // make a real network request and likely fail or timeout.
        // A cleaner alternative is to assert crawled_urls length (see below).
        let client = Client::new();
        let mut crawler = Crawler::new(url, client);
        crawler.crawl().await.unwrap();

        assert_eq!(crawler.cached_urls.len(), 1);
    }

    #[tokio::test]
    async fn crawl_does_not_revisit_cached_urls() {
        let mut server = Server::new_async().await;
        let url = Url::parse(&server.url()).unwrap();

        // Both pages link back to each other — without caching this would loop forever
        let index_mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_body(r#"<a href="/about">About</a>"#)
            .create_async()
            .await;

        let about_mock = server
            .mock("GET", "/about")
            .with_status(200)
            .with_body(r#"<a href="/">Home</a>"#)
            .create_async()
            .await;

        let client = Client::new();
        let mut crawler = Crawler::new(url, client);
        crawler.crawl().await.unwrap();

        // Each page visited exactly once
        index_mock.assert_async().await;
        about_mock.assert_async().await;
    }

    #[tokio::test]
    async fn crawl_handles_empty_page() {
        let mut server = Server::new_async().await;
        let url = Url::parse(&server.url()).unwrap();

        server
            .mock("GET", "/")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let client = Client::new();
        let mut crawler = Crawler::new(url, client);
        let result = crawler.crawl().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn crawl_continues_on_connection_failure() {
        // Point at a port nothing is listening on
        let url = Url::parse("http://127.0.0.1:1").unwrap();
        let client = Client::new();
        let mut crawler = Crawler::new(url, client);
        let result = crawler.crawl().await;

        assert!(result.is_ok());
    }
}
