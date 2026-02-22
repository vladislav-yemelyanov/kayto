mod generators;
#[path = "parser/parser.rs"]
mod parser;
mod spec;
#[cfg(test)]
mod test_fixtures;

use generators::Generator;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use rootcause::Report;
use tokio;

/// Supported code generation targets selected via CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Lang {
    Ts,
    Dart,
}

/// CLI arguments for loading a spec and writing generated artifacts.
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long, value_enum)]
    lang: Lang,
    #[arg(short, long)]
    input: String,
    #[arg(short, long)]
    output: PathBuf,
}

/// Prints a short request summary and optional verbose payload when debug mode is enabled.
fn log_parsed_requests(requests: &[parser::Request]) {
    println!("parsed requests: {}", requests.len());

    if env::var_os("KAYTO_DEBUG").is_none() {
        return;
    }

    if requests.is_empty() {
        println!();
        return;
    }

    println!();
    println!("parsed request details:");
    for req in requests {
        println!("{:#?}", req);
    }
    println!();
}

/// Groups and prints parser diagnostics by HTTP method and path.
fn log_issues(issues: &[parser::ParseIssue]) {
    if issues.is_empty() {
        return;
    }

    println!("diagnostics: {}", issues.len());
    println!();

    let mut grouped: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();

    for issue in issues {
        let method = issue.method.as_deref().unwrap_or("-").to_uppercase();
        let path = issue.path.as_deref().unwrap_or("-").to_string();
        let code_prefix = match issue.code {
            Some(code) => format!(":{code}"),
            None => String::new(),
        };

        let problem = match issue.status.as_deref() {
            Some(status) => format!(
                "[{}:{}{}] {} (status={})",
                issue.kind_str(),
                issue.stage,
                code_prefix,
                issue.detail,
                status
            ),
            None => format!(
                "[{}:{}{}] {}",
                issue.kind_str(),
                issue.stage,
                code_prefix,
                issue.detail
            ),
        };

        grouped.entry((method, path)).or_default().push(problem);
    }

    for ((method, path), problems) in grouped {
        println!("issue {} {}:", method, path);

        for problem in problems {
            println!("    problem: {}", problem);
        }

        println!();
    }

    log_unknown_summary(issues);
}

/// Prints aggregated summary for schemas that were mapped to `unknown`.
fn log_unknown_summary(issues: &[parser::ParseIssue]) {
    let mut by_code: BTreeMap<&str, usize> = BTreeMap::new();
    for issue in issues {
        let Some(code) = issue.code else {
            continue;
        };

        if !code.starts_with("unknown_") {
            continue;
        }

        *by_code.entry(code).or_insert(0) += 1;
    }

    if by_code.is_empty() {
        return;
    }

    let total: usize = by_code.values().sum();
    println!("unknown schema mappings: {total}");
    for (code, count) in by_code {
        println!("    {code}: {count}");
    }
    println!();
}

/// Entry point: fetches the OpenAPI document, builds IR, runs generators, and prints diagnostics.
#[tokio::main]
async fn main() -> Result<(), Report> {
    let cli = Cli::parse();
    let lang = cli.lang;
    let input = cli.input;
    let output = cli.output;

    let text = reqwest::get(&input).await?.text().await?;
    let openapi: spec::OpenAPI = serde_json::from_str(&text)?;

    let parsed =
        parser::parse(&openapi).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    match lang {
        Lang::Ts => {
            let generator = generators::ts::TsGenerator;
            generator.generate(&parsed.requests, &output)?;
        }
        Lang::Dart => {
            let generator = generators::dart::DartGenerator;
            generator.generate(&parsed.requests, &output)?;
        }
    }

    log_parsed_requests(&parsed.requests);
    log_issues(&parsed.issues);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies CLI parses all required arguments.
    #[test]
    fn cli_parses_all_required_arguments() {
        let cli = Cli::try_parse_from([
            "kayto",
            "--lang",
            "ts",
            "--input",
            "https://example.com/openapi.json",
            "--output",
            "generated/schema.ts",
        ])
        .expect("cli args should parse");

        assert_eq!(cli.lang, Lang::Ts);
        assert_eq!(cli.input, "https://example.com/openapi.json");
        assert_eq!(cli.output, PathBuf::from("generated/schema.ts"));
    }

    /// Verifies CLI parses Dart language and custom output path.
    #[test]
    fn cli_parses_dart_lang_and_custom_output() {
        let cli = Cli::try_parse_from([
            "kayto",
            "--lang",
            "dart",
            "--input",
            "https://example.com/openapi.json",
            "--output",
            "out/schema.dart",
        ])
        .expect("cli args should parse");

        assert_eq!(cli.lang, Lang::Dart);
        assert_eq!(cli.output, PathBuf::from("out/schema.dart"));
    }

    /// Verifies CLI rejects invocation when required arguments are missing.
    #[test]
    fn cli_rejects_missing_required_arguments() {
        let err = Cli::try_parse_from(["kayto", "--lang", "dart"])
            .expect_err("input and output are required");
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }
}
