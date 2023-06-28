use clap::Parser;
use indicatif::{HumanDuration, MultiProgress};
use owo_colors::OwoColorize;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

mod checker;
mod utils;

use checker::check_version;

#[derive(Parser)]
#[clap(version = "1.0", author = "Timmy")]
struct Cli {
    #[clap(short, long)]
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: Option<String>,
    version: Option<String>,
    dependencies: Option<serde_json::Value>,
    #[serde(alias = "devDependencies")]
    dev_dependencies: Option<serde_json::Value>,
}
const DEFAULT_PATH: &str = "package.json";
fn main() {
    let args = Cli::parse();
    let mut file_path = String::new();
    if file_path.is_empty() {
        file_path = DEFAULT_PATH.to_string();
    } else {
        file_path = args.path.unwrap();
    }
    read_file(file_path).expect("Error reading file");
}

fn read_file(file_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    let binding = std::env::current_dir()?;
    let root_name = binding.file_name().unwrap().to_str().unwrap();

    println!(
        "{} {} Resolving package.json in {}",
        "[1/3]".bold().dimmed(),
        "📦",
        root_name.blue().bold()
    );

    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);
    let package: Package = serde_json::from_reader(reader)?;
    let mut deps_list = Vec::new();

    let deps = package.dependencies.unwrap();

    for (key, value) in deps.as_object().unwrap() {
        deps_list.push((key.to_string(), value.to_string()));
    }

    if package.name.is_some() {
        println!("Package name: {}", package.name.unwrap());
    }

    if package.version.is_some() {
        println!("Version: {}", package.version.unwrap());
    }

    if deps_list.len() > 0 {
        println!("Found {} dependencies", deps_list.len());
    }

    if package.dev_dependencies.is_some() {
        println!(
            "Found {} dev dependencies",
            package.dev_dependencies.unwrap().as_object().unwrap().len()
        );
    }

    println!(
        "{} {}Fetching latest versions...",
        "[2/3]".bold().dimmed(),
        "🔍"
    );

    let m = MultiProgress::new();

    let result_table = check_version(deps_list, &m)?;

    println!("{} Done in {}", "✅", HumanDuration(start_time.elapsed()));
    println!("{}", result_table);
    Ok(())
}
