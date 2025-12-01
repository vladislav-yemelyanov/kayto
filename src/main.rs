mod parser;
mod spec;

use reqwest;
use rootcause::Report;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Report> {
    let text = reqwest::get("https://petstore.swagger.io/v2/swagger.json")
        .await?
        .text()
        .await?;

    // let lines: Vec<&str> = text.lines().collect();
    // for (i, line) in lines.iter().enumerate() {
    //     println!("{:6} | {}", i + 1, line);
    // }

    let openapi: spec::OpenAPI = serde_json::from_str(&text)?;
    // println!("{:?}", parsed.paths);

    parser::parse_openapi(&openapi);

    Ok(())
}
