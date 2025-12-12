mod logger;
mod parser;
mod spec;

use std::fs;

use rootcause::Report;
use tokio;

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

    let mut p = parser::Parser::new(openapi);
    p.parse();

    Ok(())
}
