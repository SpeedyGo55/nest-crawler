// simple gentle web crawler in Rust

use reqwest;
use robotparser::parser::parse_robots_txt;
use robotparser::service::RobotsTxtService;
use tokio;
use url::Url;

const START_URL: &str = "https://www.rust-lang.org/";
const USER_AGENT: &str = "RustCrawler/0.1";

#[tokio::main]
async fn main() {
    let mut visited = vec![START_URL.to_string()];
    let mut to_visit = vec![START_URL.to_string()];

    while let Some(url) = to_visit.pop() {
        println!("Visiting: {}", url);

        let response = reqwest::get(&url).await.unwrap();
        let body = response.text().await.unwrap();

        for link in find_links(&body, &url) {
            if !visited.contains(&link) {
                visited.push(link.clone());
                to_visit.push(link);
            }
        }
    }
}

fn find_links(body: &str, url: &str) -> Vec<String> {
    let mut links = vec![];
    let base_url = Url::parse(url).unwrap();
    let robots_txt = "User-agent: *\nDisallow: /search";
    let robots_txt = parse_robots_txt(base_url.origin(), &robots_txt).get_result();

    for cap in regex::Regex::new(r#"<a[^>]*\s*href="([^"]*)"[^>]*>"#).unwrap().captures_iter(body) {
        let link = cap.get(1).unwrap().as_str();
        if link.starts_with("http") {
            let url_link = Url::parse(&link).unwrap();
            if robots_txt.can_fetch("*", &url_link) {
                links.push(link.to_string());
            }
            continue;
        } else {
            let link = base_url.join(&link).unwrap();
            let url_link = Url::parse((&link).as_ref()).unwrap();
            if robots_txt.can_fetch(USER_AGENT, &url_link) {
                links.push(link.to_string());
            }
            continue;
        }

    }
    links
}


