// simple gentle web crawler in Rust

use std::sync::Arc;
use tokio::sync::{Mutex};
use reqwest;
use robotparser::parser::parse_robots_txt;
use robotparser::service::RobotsTxtService;
use tokio;
use url::Url;
use std::fs;

struct Graph<N, E> {
    nodes: Vec<N>,
    edges: Vec<(usize, usize, E)>,
}

impl<N: std::fmt::Debug + std::fmt::Display, E: std::fmt::Debug + std::fmt::Display> Graph<N, E> {
    fn new() -> Self {
        Graph {
            nodes: vec![],
            edges: vec![],
        }
    }

    fn visualize(&self) {
        for (i, node) in self.nodes.iter().enumerate() {
            println!("Node {}: {:?}", i, node);
        }
        for (i, edge) in self.edges.iter().enumerate() {
            println!("Edge {}: {:?} -> {:?} ({:?})", i, self.nodes[edge.0], self.nodes[edge.1], edge.2);
        }
    }

    fn save_as_obsidian_canvas(&self) {
        //format: json
        //example:
        /*  {
                "nodes":[
                    {"id":"17fdc69cfe5d7a2a","x":-120,"y":20,"width":250,"height":60,"type":"text","text":"t1"},
                    {"id":"a72bdecb85a39c1a","x":-460,"y":-180,"width":250,"height":60,"type":"text","text":"t2"}
                ],
                "edges":[
                    {"id":"aba1e187008070f3","fromNode":"17fdc69cfe5d7a2a","fromSide":"left","toNode":"a72bdecb85a39c1a","toSide":"right","label":"label"}
                ]
            }*/

        let mut canvas = String::new();
        canvas.push_str("{\n");
        canvas.push_str("\t\"nodes\":[\n");
        for (i, node) in self.nodes.iter().enumerate() {
            canvas.push_str(&format!("\t\t{{\"id\":\"{}\",\"x\":{},\"y\":{},\"width\":250,\"height\":60,\"type\":\"text\",\"text\":\"{}\"}}", i, i * 100, i * 100, node));
            if i < self.nodes.len() - 1 {
                canvas.push_str(",\n");
            }
        }
        canvas.push_str("\n\t],\n");
        canvas.push_str("\t\"edges\":[\n");
        for (i, edge) in self.edges.iter().enumerate() {
            canvas.push_str(&format!("\t\t{{\"id\":\"{}\",\"fromNode\":\"{}\",\"fromSide\":\"left\",\"toNode\":\"{}\",\"toSide\":\"right\",\"label\":\"{}\"}}", i, edge.0, edge.1, edge.2));
            if i < self.edges.len() - 1 {
                canvas.push_str(",\n");
            }
        }
        canvas.push_str("\n\t]\n");
        canvas.push_str("}");
        println!("{}", canvas);
        //save as graph.canvas
        fs::write("graph.canvas", canvas).expect("Unable to write file");

    }

    fn save_as_obsidian_vault(&self) {
        //format: markdown directory
        //in each file, write the node and its edges as markdown links to the other files
        //example:
        /*  # t1
            - [[t2]]: label
            # t2
            - [[t1]]: label
        */
        //each node is a file

        fs::create_dir("obsidian").expect("Unable to create directory");

        for (i, node) in self.nodes.iter().enumerate() {
            let mut file = String::new();
            let node = node.to_string().replace("/", "_").replace(":", "_");
            file.push_str(&format!("# {}\n", node));
            for edge in self.edges.iter() {
                if edge.0 == i {
                    file.push_str(&format!("- [[{}]]: {}\n", self.nodes[edge.1], edge.2));
                }
            }
            println!("File: {}, {}", node, file);
            fs::write(format!("obsidian/{}.md", node), file).expect("Unable to write file");
        }

    }

}

const START_URLS: [&str; 11] = [
    "https://rust-lang.org",
    "https://rust-lang.org",
    "https://rust-lang.org",
    "https://rust-lang.org",
    "https://rust-lang.org",
    "https://rust-lang.org",
    "https://rust-lang.org",
    "https://rust-lang.org",
    "https://rust-lang.org",
    "https://rust-lang.org",
    "https://rust-lang.org"
];
const USER_AGENT: &str = "RustCrawler/0.1";

#[tokio::main]
async fn main() {
    let visited = Arc::new(Mutex::new(START_URLS.iter().map(|s| s.to_string()).collect::<Vec<String>>()));
    let to_visit = Arc::new(Mutex::new(START_URLS.iter().map(|s| s.to_string()).collect::<Vec<String>>()));
    let graph = Arc::new(Mutex::new(Graph::<String, String>::new()));
    {
        let mut graph = graph.lock().await;
        graph.nodes.push("https://rust-lang.org".to_string());
    }
    let client = reqwest::Client::new();
    let max_workers = 10;
    let mut workers = vec![];

    for id in 0..max_workers {
        let client = client.clone();
        let visited = visited.clone();
        let to_visit = to_visit.clone();
        let graph = graph.clone();

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
                let mut graph = graph.lock().await;
                for link in &links {
                    if !visited.contains(&link) {
                        visited.push(link.clone());
                        to_visit.push(link.clone());
                    }

                    if graph.nodes.contains(&link) {
                        continue;
                    }

                    let node_id = graph.nodes.len();

                    let parent_id = graph.nodes.iter().rposition(|r| { r == &url}).unwrap();

                    if graph.edges.contains(&(parent_id.clone(), node_id.clone(), "link".to_string())) {
                        continue;
                    }
                    graph.nodes.push(link.clone());
                    graph.edges.push((parent_id, node_id, "link".to_string()));
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
    graph.lock().await.save_as_obsidian_canvas();
    graph.lock().await.save_as_obsidian_vault();

}
fn find_links(body: &str, url: &str) -> Vec<String> {
    let mut links = vec![];
    let base_url = Url::parse(url).unwrap();
    let robots_txt = "User-agent: *\nDisallow: /search";
    let robots_txt = parse_robots_txt(base_url.origin(), &robots_txt).get_result();

    for cap in regex::Regex::new(r#"<a[^>]*\s*href="([^"]*)"[^>]*>"#).unwrap().captures_iter(body) {
        let mut link = cap.get(1).unwrap().as_str().trim();

        //remove query parameters
        if let Some(pos) = link.find('?') {
            link = &link[..pos];
        }

        // Remove fragment identifiers
        if let Some(pos) = link.find('#') {
            link = &link[..pos];
        }

        // Normalize link
        let link = if link.starts_with('/') {
            base_url.join(link).unwrap().to_string()
        } else {
            link.to_string()
        };

        // Check for absolute URLs
        let url_link = if link.starts_with("http") {
            match Url::parse(&link) {
                Ok(url) => url,
                Err(_) => continue,
            }
        } else {
            match base_url.join(&link) {
                Ok(url) => url,
                Err(_) => continue,
            }
        };

        // Filter out links based on robots.txt
        if robots_txt.can_fetch(USER_AGENT, &url_link) {
            links.push(url_link.to_string());
        }
    }
    links
}



