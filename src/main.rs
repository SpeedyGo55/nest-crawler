// simple gentle web crawler in Rust

use std::sync::Arc;
use tokio::sync::{Mutex};
use reqwest;
use robotparser::parser::parse_robots_txt;
use robotparser::service::RobotsTxtService;
use tokio;
use url::Url;

const START_URLS: [&str; 20] = [
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/",
    "https://www.rust-lang.org/"
];
const USER_AGENT: &str = "RustCrawler/0.1";

#[tokio::main]
async fn main() {
    let visited = Arc::new(Mutex::new(START_URLS.iter().map(|s| s.to_string()).collect::<Vec<String>>()));
    let to_visit = Arc::new(Mutex::new(START_URLS.iter().map(|s| s.to_string()).collect::<Vec<String>>()));
    let client = reqwest::Client::new();
    let max_workers = 10;
    let mut workers = vec![];

    for id in 0..max_workers {
        let client = client.clone();
        let visited = visited.clone();
        let to_visit = to_visit.clone();
        let worker = tokio::spawn(async move {
            let mut finished = false;
            while !finished {
                let url = {
                    let mut to_visit = to_visit.lock().await;
                    if to_visit.is_empty() {
                        finished = true;
                        continue;
                    }
                    to_visit.pop().unwrap()
                };
                println!("Worker {}: Visiting: {}", &id, &url);
                let body = client.get(&url).send().await.unwrap().text().await.unwrap();
                let links = find_links(&body, &url);
                let mut visited = visited.lock().await;
                let mut to_visit = to_visit.lock().await;
                for link in links {
                    if !visited.contains(&link) {
                        visited.push(link.clone());
                        to_visit.push(link);
                    }
                }
                drop(visited);
                drop(to_visit);
            }
        });
        workers.push(worker);
    }

    for worker in workers {
        worker.await.unwrap();
    }

}

fn find_links(body: &str, url: &str) -> Vec<String> {
    let mut links = vec![];
    let base_url = Url::parse(url).unwrap();
    let robots_txt = "User-agent: *\nDisallow: /search";
    let robots_txt = parse_robots_txt(base_url.origin(), &robots_txt).get_result();

    for cap in regex::Regex::new(r#"<a[^>]*\s*href="([^"]*)"[^>]*>"#).unwrap().captures_iter(body) {
        let mut link = cap.get(1).unwrap().as_str();
        if link.contains("#") {
            link = link.split("#").collect::<Vec<&str>>()[0];
        }
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


