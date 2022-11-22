use clap::Parser;
use serde::Deserialize;
use std::io::BufReader;
use std::fs::File;
mod ver_check;

#[macro_use]
extern crate prettytable;
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
    dependencies: serde_json::Value,
    #[serde(alias = "devDependencies")]
    dev_dependencies: serde_json::Value,
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
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let package: Package = serde_json::from_reader(reader)?;
    let deps_list = package
        .dependencies
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<Vec<_>>();
    if package.name.is_some() {
        println!("📦 Package name: {}", package.name.unwrap());
    }

    if package.version.is_some() {
        println!("Version: {}", package.version.unwrap());
    }

    println!(
        "Found {} dependencies",
        package.dependencies.as_object().unwrap().len()
    );
    println!(
        "Found {} devDependencies",
        package.dev_dependencies.as_object().unwrap().len()
    );
    ver_check::dependencies_version_check(deps_list).expect("failed to check versions");
    println!("Time taken: {:?}s", start_time.elapsed().as_secs());
    Ok(())
}
