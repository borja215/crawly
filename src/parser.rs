use scraper::{Html, Selector};
use url::{ParseError, Url};

const SELECTOR: &str = "a";
const HREF_ATTR: &str = "href";

pub fn parse_html_for_links(base: &Url, html: &str) -> Vec<Url> {
    let mut res = vec![];
    let fragment = Html::parse_fragment(html);

    match Selector::parse(SELECTOR) {
        Ok(selector) => {
            for element in fragment.select(&selector) {
                if let Some(href) = element.value().attr(HREF_ATTR) {
                    match build_url(base, href) {
                        Ok(url) => res.push(url),
                        Err(e) => println!("Failed to parse URL: {}", e),
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to parse HTML: {}", e);
        }
    }
    res
}

fn build_url(base: &Url, href: &str) -> Result<Url, ParseError> {
    match Url::parse(href) {
        Ok(url) => Ok(url),
        Err(ParseError::RelativeUrlWithoutBase) => Ok(base.join(href)?),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Url {
        Url::parse("https://example.com/some/page").unwrap()
    }

    // --- build_url ---

    #[test]
    fn build_url_absolute() {
        let url = build_url(&base(), "https://other.com/path").unwrap();
        assert_eq!(url.as_str(), "https://other.com/path");
    }

    #[test]
    fn build_url_root_relative() {
        let url = build_url(&base(), "/about").unwrap();
        assert_eq!(url.as_str(), "https://example.com/about");
    }

    #[test]
    fn build_url_relative_path() {
        let url = build_url(&base(), "../contact").unwrap();
        assert_eq!(url.as_str(), "https://example.com/contact");
    }

    #[test]
    fn build_url_fragment() {
        let url = build_url(&base(), "#section").unwrap();
        assert_eq!(url.as_str(), "https://example.com/some/page#section");
    }

    #[test]
    fn build_url_query() {
        let url = build_url(&base(), "?q=rust").unwrap();
        assert_eq!(url.as_str(), "https://example.com/some/page?q=rust");
    }

    #[test]
    fn build_url_invalid_returns_err() {
        let result = build_url(&base(), "http://bad url");
        assert!(result.is_err());
    }

    // --- parse_html_for_links ---

    #[test]
    fn parse_returns_empty_for_no_anchors() {
        let html = "<div><p>No links here</p></div>";
        let urls = parse_html_for_links(&base(), html);
        assert!(urls.is_empty());
    }

    #[test]
    fn parse_returns_empty_for_anchor_without_href() {
        let html = r#"<a name="top">Anchor</a>"#;
        let urls = parse_html_for_links(&base(), html);
        assert!(urls.is_empty());
    }

    #[test]
    fn parse_single_absolute_link() {
        let html = r#"<a href="https://other.com">Link</a>"#;
        let urls = parse_html_for_links(&base(), html);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].as_str(), "https://other.com/");
    }

    #[test]
    fn parse_single_relative_link() {
        let html = r#"<a href="/about">About</a>"#;
        let urls = parse_html_for_links(&base(), html);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].as_str(), "https://example.com/about");
    }

    #[test]
    fn parse_multiple_links() {
        let html = r#"
            <a href="/about">About</a>
            <a href="/contact">Contact</a>
            <a href="https://other.com">External</a>
        "#;
        let urls = parse_html_for_links(&base(), html);
        assert_eq!(urls.len(), 3);
    }

    #[test]
    fn parse_skips_invalid_hrefs() {
        let html = r#"
            <a href="http://bad url">Bad</a>
            <a href="/good">Good</a>
        "#;
        let urls = parse_html_for_links(&base(), html);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].as_str(), "https://example.com/good");
    }

    #[test]
    fn parse_empty_html() {
        let urls = parse_html_for_links(&base(), "");
        assert!(urls.is_empty());
    }

    #[test]
    fn parse_deduplicates_links() {
        let html = r#"
            <a href="/about">About 1</a>
            <a href="/about">About 2</a>
        "#;
        // Current impl does NOT deduplicate — this documents that behaviour.
        // Change assert to `== 1` if you add deduplication later.
        let urls = parse_html_for_links(&base(), html);
        assert_eq!(urls.len(), 2);
    }
}
