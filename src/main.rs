mod parser;
mod spec;

use std::collections::BTreeMap;
use std::fs;
use std::io;

use rootcause::Report;
use tokio;

fn log_parsed_requests(requests: &[parser::Request]) {
    eprintln!("parsed requests: {}", requests.len());
    eprintln!();

    if requests.is_empty() {
        return;
    }

    eprintln!("parsed request details:");

    for req in requests {
        eprintln!("{:#?}", req);
    }

    eprintln!();
}

fn log_issues(issues: &[parser::ParseIssue]) {
    if issues.is_empty() {
        return;
    }

    eprintln!("parse issues: {}", issues.len());
    eprintln!();

    let mut grouped: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();

    for issue in issues {
        let method = issue.method.as_deref().unwrap_or("-").to_uppercase();
        let path = issue.path.as_deref().unwrap_or("-").to_string();

        let problem = match issue.status.as_deref() {
            Some(status) => format!("[{}] {} (status={})", issue.stage, issue.detail, status),
            None => format!("[{}] {}", issue.stage, issue.detail),
        };

        grouped.entry((method, path)).or_default().push(problem);
    }

    for ((method, path), problems) in grouped {
        eprintln!("issue {} {}:", method, path);

        for problem in problems {
            eprintln!("    problem: {}", problem);
        }

        eprintln!();
    }
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    // let text = reqwest::get("https://petstore.swagger.io/v2/swagger.json")
    //     .await?
    //     .text()
    //     .await?;

    let f = fs::read("./api_example.json")?;

    // let lines: Vec<&str> = text.lines().collect();
    // for (i, line) in lines.iter().enumerate() {
    //     println!("{:6} | {}", i + 1, line);
    // }

    let openapi: spec::OpenAPI = serde_json::from_slice(&f)?;

    let parsed = parser::parse(&openapi).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    log_parsed_requests(&parsed.requests);
    log_issues(&parsed.issues);

    Ok(())
}
