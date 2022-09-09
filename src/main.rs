use clap::Parser;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
mod ver_check;
#[macro_use] extern crate prettytable;
#[derive(Parser)]
#[clap(author, about, version)]
struct Cli {
    #[clap(short, long)]
    check_lastest: bool,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: Option<String>,
    version: Option<String>,
    dependencies: serde_json::Value,
    #[serde(alias = "devDependencies")]
    dev_dependencies: serde_json::Value,
}

fn main() {
    let args = Cli::parse();
    println!("🪄 Checking for updates...");
    read_file(args.check_lastest).expect("Error reading file");
}

fn read_file(check: bool) -> std::io::Result<()> {
    println!("🚀 Found package.json file");
    let file = File::open("package.json")?;
    let reader = BufReader::new(file);
    let mut deps_list = Vec::new();

    let package: Package = serde_json::from_reader(reader).expect("error parsing json");

    if package.name.is_some() {
        println!("📦 Package name: {}", package.name.unwrap());
    }
    println!("📦 Package name: not found");

    if package.version.is_some() {
        println!("Version: {}", package.version.unwrap());
    } else {
        println!("\n");
    }

    println!(
        "Found {} dependencies",
        package.dependencies.as_object().unwrap().len()
    );
    for (key, value) in package.dependencies.as_object().unwrap().iter() {
        println!("{}: {}", key, value);
        deps_list.push((key.to_string(), value.to_string()));
    }
    println!(
        "Found {} devDependencies",
        package.dev_dependencies.as_object().unwrap().len()
    );
    for (key, value) in package.dev_dependencies.as_object().unwrap().iter() {
        println!("{}: {}", key, value);
        deps_list.push((key.to_string(), value.to_string()));
    }

    if check {
        ver_check::dependencies_version_check(deps_list).expect("failed to check versions");
    }

    Ok(())
}

