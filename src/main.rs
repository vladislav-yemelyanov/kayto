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
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

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

/// Lightweight terminal spinner to indicate progress for long-running stages.
struct Spinner {
    done: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    fn start(message: impl Into<String>) -> Self {
        let message = message.into();
        let done = Arc::new(AtomicBool::new(false));
        let done_for_thread = Arc::clone(&done);

        let handle = thread::spawn(move || {
            let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut idx = 0usize;
            while !done_for_thread.load(Ordering::Relaxed) {
                eprint!("\r{} {}", frames[idx % frames.len()], message);
                let _ = io::stderr().flush();
                idx += 1;
                thread::sleep(Duration::from_millis(100));
            }
        });

        Self {
            done,
            handle: Some(handle),
        }
    }

    fn finish(mut self, message: impl AsRef<str>) {
        self.done.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        eprintln!("\r{}\x1b[K", message.as_ref());
    }
}

fn format_duration(duration: Duration) -> String {
    if duration.as_millis() < 1000 {
        format!("{} ms", duration.as_millis())
    } else {
        format!("{:.2} s", duration.as_secs_f64())
    }
}

fn print_summary(
    requests_count: usize,
    issues_count: usize,
    fetch_duration: Duration,
    parse_duration: Duration,
    generate_duration: Duration,
    total_duration: Duration,
) {
    println!();
    println!("Summary");
    println!();
    println!("✅ Requests parsed: {requests_count}");
    println!("📝 Diagnostics: {issues_count}");
    println!("🌐 Fetch: {}", format_duration(fetch_duration));
    println!("🔍 Parse: {}", format_duration(parse_duration));
    println!("⚙️ Generate: {}", format_duration(generate_duration));
    println!("⏱️ Total: {}", format_duration(total_duration));
}

/// Prints a short request summary and optional verbose payload when debug mode is enabled.
#[allow(dead_code)]
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
    let total_start = Instant::now();
    let cli = Cli::parse();
    let lang = cli.lang;
    let input = cli.input;
    let output = cli.output;

    let lang_name = match lang {
        Lang::Ts => "TypeScript",
        Lang::Dart => "Dart",
    };
    println!("🚀 Starting generation");
    println!("🎯 Target: {lang_name}");
    println!("🌐 Input: {input}");
    println!("📦 Output: {}", output.display());
    println!();

    let fetch_spinner = Spinner::start("Fetching OpenAPI spec...");
    let fetch_start = Instant::now();
    let text = reqwest::get(&input).await?.text().await?;
    let fetch_duration = fetch_start.elapsed();
    fetch_spinner.finish(format!(
        "✅ Fetched OpenAPI spec in {}",
        format_duration(fetch_duration)
    ));

    let parse_spinner = Spinner::start("Parsing OpenAPI...");
    let parse_start = Instant::now();
    let openapi: spec::OpenAPI = serde_json::from_str(&text)?;
    let parsed =
        parser::parse(&openapi).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    let parse_duration = parse_start.elapsed();
    parse_spinner.finish(format!(
        "✅ Parsed OpenAPI in {}",
        format_duration(parse_duration)
    ));

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let generate_spinner = Spinner::start(format!(
        "Generating {} schema...",
        match lang {
            Lang::Ts => "TypeScript",
            Lang::Dart => "Dart",
        }
    ));
    let generate_start = Instant::now();
    match lang {
        Lang::Ts => {
            let generator = generators::ts::TsGenerator;
            generator.generate(&parsed.requests, &parsed.models, &output)?;
        }
        Lang::Dart => {
            let generator = generators::dart::DartGenerator;
            generator.generate(&parsed.requests, &parsed.models, &output)?;
        }
    }
    let generate_duration = generate_start.elapsed();
    generate_spinner.finish(format!(
        "✅ Generated schema in {}",
        format_duration(generate_duration)
    ));

    // log_parsed_requests(&parsed.requests);
    log_issues(&parsed.issues);
    print_summary(
        parsed.requests.len(),
        parsed.issues.len(),
        fetch_duration,
        parse_duration,
        generate_duration,
        total_start.elapsed(),
    );

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
